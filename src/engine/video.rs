use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use bevy::{
    asset::{RenderAssetUsages, io::file::FileAssetReader},
    audio::{AddAudioSource, AudioPlayer, Decodable, PlaybackSettings, Source},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use ffmpeg_next as ffmpeg;

pub(super) fn plugin(app: &mut App) {
    app.init_non_send_resource::<VideoPlayers>();
    app.add_audio_source::<VideoAudioSource>();
    app.add_systems(Startup, init_ffmpeg);
    app.add_systems(
        Update,
        (update_video_players, cleanup_despawned_players).chain(),
    );
}

fn init_ffmpeg() {
    if let Err(err) = ffmpeg::init() {
        error!("Failed to initialize ffmpeg: {err}");
    }
}

/// Tag component for entities playing a video. Spawn one via [`spawn_video_player`].
#[derive(Component)]
pub struct VideoPlayer {
    pub loop_video: bool,
    /// Set to `true` once a non-looping video has played to completion.
    /// Looping videos never set this.
    pub finished: bool,
    pub(crate) image_handle: Handle<Image>,
}

/// Non-Send storage for per-entity ffmpeg state.
#[derive(Default)]
pub struct VideoPlayers {
    data: HashMap<Entity, VideoPlayerData>,
}

/// A decoded frame that is waiting for its presentation time before being shown.
struct PendingFrame {
    /// Presentation time in seconds, relative to the start of the video stream.
    when: f64,
    /// The scaled RGBA frame, ready to be copied into the GPU texture.
    frame: ffmpeg::frame::Video,
}

enum FrameDecode {
    Frame(PendingFrame),
    EndOfStream,
}

struct VideoPlayerData {
    path: String,
    stream_index: usize,
    decoder: ffmpeg::decoder::Video,
    input_context: ffmpeg::format::context::Input,
    scaler: ffmpeg::software::scaling::Context,
    /// Seconds per stream tick (e.g. 1/60 for a 60 fps video).
    time_base_secs: f64,
    /// The stream's start time in ticks (usually 0). Frame PTS are relative to
    /// this when converted to a presentation time.
    stream_start_ticks: i64,
    /// Container duration in seconds, used for progress logging.
    duration_secs: f64,
    /// Number of frames in the stream (`-1` when the container does not say).
    frame_count: i64,
    /// When the current playthrough started (wall clock). `None` until the
    /// first frame is due.
    playback_start: Option<Instant>,
    /// The next frame to show, held until its presentation time arrives.
    pending: Option<PendingFrame>,
    /// Number of times a looping video has restarted.
    loop_count: u32,
}

impl VideoPlayerData {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, ffmpeg::Error> {
        let path = path.as_ref();
        let input_context = ffmpeg::format::input(path)?;
        let input_stream = input_context
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let stream_index = input_stream.index();

        let context_decoder =
            ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
        let decoder = context_decoder.decoder().video()?;

        let scaler = ffmpeg::software::scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ffmpeg::format::Pixel::RGBA,
            decoder.width(),
            decoder.height(),
            ffmpeg::software::scaling::flag::Flags::BILINEAR,
        )?;

        // Read the stream timing metadata before `input_context` is moved into
        // the struct below.
        let time_base = input_stream.time_base();
        let time_base_secs = time_base.numerator() as f64 / time_base.denominator() as f64;
        let stream_start_ticks = input_stream.start_time().max(0);
        let frame_count = input_stream.frames();
        // `Input::duration` is in microseconds (AV_TIME_BASE units).
        let duration_secs = (input_context.duration() as f64 / 1_000_000.0).max(0.0);

        Ok(Self {
            path: path.to_string_lossy().to_string(),
            stream_index,
            decoder,
            input_context,
            scaler,
            time_base_secs,
            stream_start_ticks,
            duration_secs,
            frame_count,
            playback_start: None,
            pending: None,
            loop_count: 0,
        })
    }

    /// Convert a frame's PTS (in stream ticks) to a presentation time in
    /// seconds, relative to the start of the stream.
    fn frame_time(&self, pts: Option<i64>) -> f64 {
        match pts {
            Some(pts) => (pts as f64 - self.stream_start_ticks as f64) * self.time_base_secs,
            // Frames without a PTS are rare (typically the last flushed frames);
            // show them as soon as they are decoded.
            None => f64::NEG_INFINITY,
        }
    }
}

/// A Bevy-decodable audio source produced by decoding an MP4's audio track.
///
/// Used internally so cutscene audio can be played through the same audio
/// pipeline as the rest of the game.
#[derive(Asset, Clone, TypePath)]
pub struct VideoAudioSource {
    samples: Arc<[f32]>,
    channels: u16,
    sample_rate: u32,
}

pub struct VideoAudioDecoder {
    samples: Arc<[f32]>,
    position: usize,
    channels: u16,
    sample_rate: u32,
}

impl Iterator for VideoAudioDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples.len() {
            let sample = self.samples[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for VideoAudioDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.position))
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let samples_per_second = self.channels as f64 * self.sample_rate as f64;
        if samples_per_second == 0.0 {
            return None;
        }
        Some(Duration::from_secs_f64(
            self.samples.len() as f64 / samples_per_second,
        ))
    }
}

impl Decodable for VideoAudioSource {
    type DecoderItem = f32;
    type Decoder = VideoAudioDecoder;

    fn decoder(&self) -> Self::Decoder {
        VideoAudioDecoder {
            samples: self.samples.clone(),
            position: 0,
            channels: self.channels,
            sample_rate: self.sample_rate,
        }
    }
}

/// Spawn a video-playing entity and return its `Entity` plus the `Handle<Image>`
/// that the caller can attach to a `Sprite`, `ImageNode`, etc.
///
/// If `with_audio` is `true` and the file contains a decodable audio track, an
/// [`AudioPlayer`] is attached to the same entity so the cutscene's sound plays
/// in sync with the video.
///
/// The caller is responsible for inserting any display components and for
/// despawning the entity when playback should stop.
fn resolve_video_path(path: &str) -> PathBuf {
    FileAssetReader::get_base_path().join(path)
}

pub fn spawn_video_player(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    audio_sources: &mut Assets<VideoAudioSource>,
    players: &mut VideoPlayers,
    path: &str,
    loop_video: bool,
    with_audio: bool,
) -> Option<(Entity, Handle<Image>)> {
    let resolved_path = resolve_video_path(path);
    let data = VideoPlayerData::new(&resolved_path)
        .inspect_err(|err| error!("Failed to load video {}: {err}", resolved_path.display()))
        .ok()?;

    let audio_handle = if with_audio {
        match decode_audio(&resolved_path) {
            Ok(Some(source)) => Some(audio_sources.add(source)),
            Ok(None) => None,
            Err(err) => {
                error!("Failed to decode audio for {path}: {err}");
                None
            }
        }
    } else {
        None
    };

    let size = Extent3d {
        width: data.decoder.width(),
        height: data.decoder.height(),
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;

    let image_handle = images.add(image);

    let mut entity_commands = commands.spawn(VideoPlayer {
        loop_video,
        finished: false,
        image_handle: image_handle.clone(),
    });

    if let Some(audio_handle) = audio_handle {
        let playback = if loop_video {
            PlaybackSettings::LOOP
        } else {
            PlaybackSettings::ONCE
        };
        entity_commands.insert((AudioPlayer(audio_handle), playback));
        info!("Attached audio track to video player for {path}");
    }

    let entity = entity_commands.id();
    players.data.insert(entity, data);

    Some((entity, image_handle))
}

/// Spawn a full-screen video node that fills the viewport and despawns when the
/// `despawn_on_exit` state is exited.
///
/// This wraps [`spawn_video_player`] with the display and lifecycle components
/// that every menu cutscene needs, so callers only have to supply a path and a
/// name. Returns the spawned entity, or `None` if the video failed to load.
pub fn spawn_fullscreen_video<S: States>(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    audio_sources: &mut Assets<VideoAudioSource>,
    players: &mut VideoPlayers,
    path: &str,
    loop_video: bool,
    with_audio: bool,
    name: &str,
    despawn_on_exit: S,
) -> Option<Entity> {
    let (entity, image_handle) = spawn_video_player(
        commands,
        images,
        audio_sources,
        players,
        path,
        loop_video,
        with_audio,
    )?;

    commands.entity(entity).insert((
        Name::new(name.to_string()),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode::new(image_handle),
        DespawnOnExit(despawn_on_exit),
    ));

    Some(entity)
}

/// Returns `true` when the menu's cutscene has finished playing, or when no
/// cutscene video was spawned at all (for example because the file failed to
/// load). Callers use this to defer their UI until the video completes.
pub fn cutscene_finished(video: &Query<&VideoPlayer>) -> bool {
    video.iter().next().is_none_or(|player| player.finished)
}

fn decode_audio(path: &Path) -> Result<Option<VideoAudioSource>, ffmpeg::Error> {
    let mut input_context = ffmpeg::format::input(path)?;
    let Some(audio_stream) = input_context.streams().best(ffmpeg::media::Type::Audio) else {
        return Ok(None);
    };
    let stream_index = audio_stream.index();

    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(audio_stream.parameters())?;
    let mut decoder = context_decoder.decoder().audio()?;

    let output_format = ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed);
    let mut resampler = ffmpeg::software::resampling::Context::get(
        decoder.format(),
        decoder.channel_layout(),
        decoder.rate(),
        output_format,
        decoder.channel_layout(),
        decoder.rate(),
    )?;

    let channels = decoder.channels();
    let sample_rate = decoder.rate();
    let mut samples: Vec<f32> = Vec::new();

    let mut process_decoded_frame = |decoded: &ffmpeg::frame::Audio| -> Result<(), ffmpeg::Error> {
        let mut resampled = ffmpeg::frame::Audio::empty();
        resampler.run(decoded, &mut resampled)?;
        append_packed_f32_samples(&mut samples, &resampled, channels)?;
        Ok(())
    };

    for (stream, packet) in input_context.packets() {
        if stream.index() != stream_index {
            continue;
        }

        if let Err(err) = decoder.send_packet(&packet) {
            error!("ffmpeg audio send_packet error: {err}");
            continue;
        }

        let mut decoded = ffmpeg::frame::Audio::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            process_decoded_frame(&decoded)?;
        }
    }

    match decoder.send_eof() {
        Err(err) if !matches!(err, ffmpeg::Error::Eof) => {
            error!("ffmpeg audio send_eof error: {err}");
        }
        _ => {}
    }

    let mut decoded = ffmpeg::frame::Audio::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        process_decoded_frame(&decoded)?;
    }

    // Only flush if the resampler reports remaining delay. In some FFmpeg
    // builds calling flush with no buffered samples returns AVERROR_OUTPUT_CHANGED,
    // which we can safely ignore.
    if resampler.delay().is_some() {
        loop {
            let mut resampled = ffmpeg::frame::Audio::empty();
            match resampler.flush(&mut resampled) {
                Ok(Some(_)) => {
                    append_packed_f32_samples(&mut samples, &resampled, channels)?;
                }
                Ok(None) | Err(_) => break,
            }
        }
    }

    if samples.is_empty() {
        return Ok(None);
    }

    Ok(Some(VideoAudioSource {
        samples: samples.into(),
        channels,
        sample_rate,
    }))
}

/// Append the valid portion of an interleaved F32 audio frame.
///
/// `Audio::plane::<f32>` has a length of `frame.samples()` even for packed
/// audio, which omits all but one channel. Read the packed byte buffer directly
/// and account for every channel instead.
fn append_packed_f32_samples(
    output: &mut Vec<f32>,
    frame: &ffmpeg::frame::Audio,
    channels: u16,
) -> Result<(), ffmpeg::Error> {
    let sample_count = frame.samples() * usize::from(channels);
    let byte_count = sample_count * size_of::<f32>();
    let bytes = frame
        .data(0)
        .get(..byte_count)
        .ok_or(ffmpeg::Error::InvalidData)?;

    output.extend(
        bytes
            .chunks_exact(size_of::<f32>())
            .map(|bytes| f32::from_ne_bytes(bytes.try_into().expect("f32-sized chunk"))),
    );
    Ok(())
}

fn update_video_players(
    mut players: NonSendMut<VideoPlayers>,
    mut query: Query<(Entity, &mut VideoPlayer)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, mut player) in &mut query {
        if player.finished {
            continue;
        }
        let Some(data) = players.data.get_mut(&entity) else {
            continue;
        };

        // Playback is paced by the wall clock, not the game's frame rate: a
        // video must play N frames over their real duration regardless of how
        // fast the game runs. Without this, a 60 fps cutscene plays at the
        // game's tick rate (e.g. 2x speed on a 120 Hz display).
        let playback_start = *data.playback_start.get_or_insert_with(|| {
            if data.loop_count == 0 {
                info!(
                    "[video] {}: playback started (expected {:.2}s, {} frames @ {:.0} fps)",
                    data.path,
                    data.duration_secs,
                    data.frame_count.max(0),
                    1.0 / data.time_base_secs,
                );
            }
            Instant::now()
        });
        let elapsed = playback_start.elapsed().as_secs_f64();

        // Show the held frame once its time arrives, then keep decoding until
        // the next frame is due in the future (or the stream ends). Frames that
        // are already overdue are shown immediately, so a slow tick catches up
        // by skipping frames instead of slowing the video down.
        let mut eof = false;
        while !eof {
            match data.pending.take() {
                Some(pending) if pending.when <= elapsed => {
                    upload_frame(&player.image_handle, &mut images, &pending.frame);
                }
                Some(pending) => {
                    // Not due yet: keep holding it on screen.
                    data.pending = Some(pending);
                    break;
                }
                None => match decode_next_frame(data) {
                    FrameDecode::Frame(pending) => {
                        data.pending = Some(pending);
                    }
                    FrameDecode::EndOfStream => {
                        eof = true;
                        break;
                    }
                },
            }
        }

        if !eof || elapsed < data.duration_secs {
            continue;
        }

        if player.loop_video {
            // Recreate the ffmpeg context to loop cleanly.
            let loop_count = data.loop_count + 1;
            info!("[video] {}: looping (loop #{loop_count})", data.path);
            match VideoPlayerData::new(&data.path) {
                Ok(mut new_data) => {
                    new_data.loop_count = loop_count;
                    *data = new_data;
                    // Decode a frame immediately so the screen never goes blank.
                    if let FrameDecode::Frame(pending) = decode_next_frame(data) {
                        upload_frame(&player.image_handle, &mut images, &pending.frame);
                    }
                }
                Err(err) => {
                    error!("Failed to loop video {}: {err}", data.path);
                }
            }
        } else {
            // Non-looping video has reached its end. Hold on the final frame
            // and signal callers that the cutscene is complete.
            if let Some(start) = data.playback_start {
                let expected = data.duration_secs.max(0.001);
                let elapsed = start.elapsed().as_secs_f64();
                info!(
                    "[video] {}: finished in {elapsed:.2}s (expected {expected:.2}s, {:.2}x realtime)",
                    data.path,
                    elapsed / expected,
                );
            }
            player.finished = true;
        }
    }
}

fn decode_next_frame(data: &mut VideoPlayerData) -> FrameDecode {
    // A packet can produce more than one frame. Drain previously submitted
    // packet data before reading another packet, otherwise frames can be lost
    // or send_packet can fail with EAGAIN.
    let mut decoded = ffmpeg::frame::Video::empty();
    if data.decoder.receive_frame(&mut decoded).is_ok() {
        return scale_frame(data, &decoded);
    }

    while let Some((stream, packet)) = data.input_context.packets().next() {
        if stream.index() != data.stream_index {
            continue;
        }

        if let Err(err) = data.decoder.send_packet(&packet) {
            error!("ffmpeg send_packet error: {err}");
            continue;
        }

        if data.decoder.receive_frame(&mut decoded).is_ok() {
            return scale_frame(data, &decoded);
        }
    }

    // Flush any buffered frames.
    match data.decoder.send_eof() {
        Err(err) if !matches!(err, ffmpeg::Error::Eof) => {
            error!("ffmpeg send_eof error: {err}");
        }
        _ => {}
    }

    if data.decoder.receive_frame(&mut decoded).is_ok() {
        return scale_frame(data, &decoded);
    }

    FrameDecode::EndOfStream
}

fn scale_frame(data: &mut VideoPlayerData, decoded: &ffmpeg::frame::Video) -> FrameDecode {
    let mut rgba_frame = ffmpeg::frame::Video::empty();
    if let Err(err) = data.scaler.run(decoded, &mut rgba_frame) {
        error!("ffmpeg scaler error: {err}");
        return FrameDecode::EndOfStream;
    }

    FrameDecode::Frame(PendingFrame {
        when: data.frame_time(decoded.pts()),
        frame: rgba_frame,
    })
}

fn upload_frame(
    image_handle: &Handle<Image>,
    images: &mut Assets<Image>,
    frame: &ffmpeg::frame::Video,
) {
    if let Some(image) = images.get_mut(image_handle) {
        image
            .data
            .as_mut()
            .expect("video texture data should stay in the main world")
            .copy_from_slice(frame.data(0));
    }
}

fn cleanup_despawned_players(
    mut players: NonSendMut<VideoPlayers>,
    mut removed: RemovedComponents<VideoPlayer>,
) {
    for entity in removed.read() {
        players.data.remove(&entity);
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Once};

    use super::decode_audio;

    static INIT: Once = Once::new();

    fn init() {
        INIT.call_once(|| {
            ffmpeg_next::init().ok();
        });
    }

    #[test]
    fn bootup_mp4_has_audio() {
        init();
        let source = decode_audio(Path::new("assets/cutscenes/bootup.mp4"))
            .expect("decode_audio should not error")
            .expect("bootup.mp4 should have an audio track");
        assert_eq!(source.channels, 2);
        assert_eq!(source.sample_rate, 44100);
        let duration =
            source.samples.len() as f64 / (source.channels as f64 * source.sample_rate as f64);
        assert!((duration - 5.967).abs() < 0.02, "decoded {duration:.3}s");
    }

    #[test]
    fn bootup_mp4_timing_metadata() {
        init();
        let data = super::VideoPlayerData::new(Path::new("assets/cutscenes/bootup.mp4"))
            .expect("VideoPlayerData::new should not error");

        // 60 fps, 358 frames, ~5.97 s: the values the pacing clock relies on.
        assert!((1.0 / data.time_base_secs - 60.0).abs() < 0.001);
        assert_eq!(data.frame_count, 358);
        assert!((data.duration_secs - 5.97).abs() < 0.1);

        // PTS 0 is the first frame; PTS 30 is exactly half a second in.
        assert!(data.frame_time(Some(0)).abs() < f64::EPSILON);
        assert!((data.frame_time(Some(30)) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn thallo_ending_mp4_has_audio() {
        init();
        let source = decode_audio(Path::new("assets/cutscenes/thallo_ending.mp4"))
            .expect("decode_audio should not error")
            .expect("thallo_ending.mp4 should have an audio track");
        assert_eq!(source.channels, 2);
        assert!(source.samples.len() > 1000);
    }

    #[test]
    fn death_screen_mp4_has_audio() {
        init();
        let source = decode_audio(Path::new("assets/cutscenes/death_screen.mp4"))
            .expect("decode_audio should not error")
            .expect("death_screen.mp4 should have an audio track");
        assert_eq!(source.channels, 2);
        assert!(source.samples.len() > 1000);
    }

    #[test]
    fn disclaimer_mp4_audio_can_be_decoded() {
        // The disclaimer video is played without audio by choice, but the file
        // does contain a track; make sure decoding it does not error.
        init();
        let source = decode_audio(Path::new("assets/cutscenes/disclaimer.mp4"))
            .expect("decode_audio should not error")
            .expect("disclaimer.mp4 should have a decodable audio track");
        assert_eq!(source.channels, 2);
        assert!(source.samples.len() > 1000);
    }
}
