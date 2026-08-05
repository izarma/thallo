use std::{collections::HashMap, path::Path};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use ffmpeg_next as ffmpeg;

pub(super) fn plugin(app: &mut App) {
    app.init_non_send_resource::<VideoPlayers>();
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

/// Spawn a video-playing entity and return its `Entity` plus the `Handle<Image>`
/// that the caller can attach to a `Sprite`, `ImageNode`, etc.
///
/// The caller is responsible for inserting any display components and for
/// despawning the entity when playback should stop.
pub fn spawn_video_player(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    players: &mut VideoPlayers,
    path: &str,
    loop_video: bool,
) -> Option<(Entity, Handle<Image>)> {
    let data = VideoPlayerData::new(path)
        .inspect_err(|err| error!("Failed to load video {path}: {err}"))
        .ok()?;

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
    let entity = commands
        .spawn(VideoPlayer {
            loop_video,
            image_handle: image_handle.clone(),
        })
        .id();

    players.data.insert(entity, data);

    Some((entity, image_handle))
}

fn update_video_players(
    mut players: NonSendMut<VideoPlayers>,
    query: Query<(Entity, &VideoPlayer)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, player) in &query {
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
    if let Err(err) = data.decoder.send_eof() {
        if !matches!(err, ffmpeg::Error::Eof) {
            error!("ffmpeg send_eof error: {err}");
        }
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
