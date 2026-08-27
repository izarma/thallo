use bevy::prelude::*;

use crate::{
    engine::{
        asset_tracking::LoadResource,
        audio::{Music, music},
        screens::Screen,
        scripted_events::UnlockState,
    },
    game::beats::ApplyBeatCommand,
};

pub mod beats;
mod dialogues;
mod files;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<FileAssets>();
    app.add_systems(OnEnter(Screen::Desktop), setup_desktop_for_beat);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct FileAssets {
    #[dependency]
    pub omega: Handle<Image>,
    #[dependency]
    pub act_transition: Handle<Image>,
    #[dependency]
    pub ac1bg: Handle<AudioSource>,
    #[dependency]
    pub ac2bg: Handle<AudioSource>,
    #[dependency]
    pub tans_act: Handle<AudioSource>,
}

impl FromWorld for FileAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            omega: assets.load("game/act1/omega_r.png"),
            act_transition: assets.load("game/act_transition.png"),
            ac1bg: assets.load("audio/ambience_act1.ogg"),
            ac2bg: assets.load("audio/ambience_act2.ogg"),
            tans_act: assets.load("audio/transition_act1.ogg"),
        }
    }
}

/// Single `OnEnter(Desktop)` setup that rebuilds the desktop for the current
/// [`StoryBeat`](crate::engine::scripted_events::StoryBeat) and (idempotently)
/// spawns the background music track.
fn setup_desktop_for_beat(
    mut cmd: Commands,
    assets: Res<FileAssets>,
    state: Res<UnlockState>,
    music_query: Query<(), (With<Music>, With<DespawnOnExit<Screen>>)>,
) {
    if music_query.is_empty() {
        let track = if state.story.is_act2() {
            assets.ac2bg.clone()
        } else {
            assets.ac1bg.clone()
        };
        cmd.spawn((music(track), DespawnOnExit(Screen::Desktop)));
    }

    cmd.queue(ApplyBeatCommand(state.story.beat));
}
