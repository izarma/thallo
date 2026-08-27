use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};

use bevy::{
    asset::RenderAssetUsages,
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

struct VideoPlayerData {
    path: String,
    stream_index: usize,
    decoder: ffmpeg::decoder::Video,
    input_context: ffmpeg::format::context::Input,
    scaler: ffmpeg::software::scaling::Context,
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

        Ok(Self {
            path: path.to_string_lossy().to_string(),
            stream_index,
            decoder,
            input_context,
            scaler,
        })
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
pub fn spawn_video_player(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    audio_sources: &mut Assets<VideoAudioSource>,
    players: &mut VideoPlayers,
    path: &str,
    loop_video: bool,
    with_audio: bool,
) -> Option<(Entity, Handle<Image>)> {
    let data = VideoPlayerData::new(path)
        .inspect_err(|err| error!("Failed to load video {path}: {err}"))
        .ok()?;

    let audio_handle = if with_audio {
        match decode_audio(Path::new(path)) {
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
        let plane = resampled.plane::<f32>(0);
        samples.extend_from_slice(plane);
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
                    let plane = resampled.plane::<f32>(0);
                    samples.extend_from_slice(plane);
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

fn update_video_players(
    mut players: NonSendMut<VideoPlayers>,
    mut query: Query<(Entity, &mut VideoPlayer)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, mut player) in &mut query {
        let Some(data) = players.data.get_mut(&entity) else {
            continue;
        };

        if decode_and_upload_frame(data, &player.image_handle, &mut images) {
            continue;
        }

        if player.loop_video {
            // Recreate the ffmpeg context to loop cleanly.
            match VideoPlayerData::new(&data.path) {
                Ok(new_data) => {
                    *data = new_data;
                    // Decode a frame immediately so the screen never goes blank.
                    let _ = decode_and_upload_frame(data, &player.image_handle, &mut images);
                }
                Err(err) => {
                    error!("Failed to loop video {}: {err}", data.path);
                }
            }
        } else {
            // Non-looping video has reached its end. Hold on the final frame
            // and signal callers that the cutscene is complete.
            player.finished = true;
        }
    }
}

fn decode_and_upload_frame(
    data: &mut VideoPlayerData,
    image_handle: &Handle<Image>,
    images: &mut Assets<Image>,
) -> bool {
    while let Some((stream, packet)) = data.input_context.packets().next() {
        if stream.index() != data.stream_index {
            continue;
        }

        if let Err(err) = data.decoder.send_packet(&packet) {
            error!("ffmpeg send_packet error: {err}");
            continue;
        }

        let mut decoded = ffmpeg::frame::Video::empty();
        if data.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgba_frame = ffmpeg::frame::Video::empty();
            if let Err(err) = data.scaler.run(&decoded, &mut rgba_frame) {
                error!("ffmpeg scaler error: {err}");
                continue;
            }

            if let Some(image) = images.get_mut(image_handle) {
                image
                    .data
                    .as_mut()
                    .expect("video texture data should stay in the main world")
                    .copy_from_slice(rgba_frame.data(0));
            }
            return true;
        }
    }

    // Flush any buffered frames.
    match data.decoder.send_eof() {
        Err(err) if !matches!(err, ffmpeg::Error::Eof) => {
            error!("ffmpeg send_eof error: {err}");
        }
        _ => {}
    }

    let mut decoded = ffmpeg::frame::Video::empty();
    if data.decoder.receive_frame(&mut decoded).is_ok() {
        let mut rgba_frame = ffmpeg::frame::Video::empty();
        if data.scaler.run(&decoded, &mut rgba_frame).is_ok() {
            if let Some(image) = images.get_mut(image_handle) {
                image
                    .data
                    .as_mut()
                    .expect("video texture data should stay in the main world")
                    .copy_from_slice(rgba_frame.data(0));
            }
            return true;
        }
    }

    false
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
        assert!(source.samples.len() > 1000);
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
