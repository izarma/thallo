use bevy::prelude::*;

use crate::engine::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<MinigameAssets>()
        .init_resource::<ActiveMinigame>()
        .add_observer(on_minigame_trigger);
}

#[derive(Resource, Default)]
pub struct ActiveMinigame(pub Option<Minigame>);

pub struct Minigame {
    game_type: MinigameType,
    checkpoint: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum MinigameType {
    BruteForce,
    NetRipper,
}

#[derive(Event, Debug, Clone)]
pub struct MinigameTrigger {
    pub checkpoint: usize,
    pub game_type: MinigameType,
}

fn on_minigame_trigger(ev: On<MinigameTrigger>, mut state: ResMut<ActiveMinigame>) {
    if state.0.is_none() {
        state.0 = Some(Minigame {
            game_type: ev.game_type,
            checkpoint: ev.checkpoint,
        });
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MinigameAssets {
    #[dependency]
    pub hack1: Handle<Image>,
    #[dependency]
    pub hack1_disk: Handle<Image>,
    #[dependency]
    pub hack2: Handle<Image>,
    #[dependency]
    pub hack2_lock: Handle<Image>,
}

impl FromWorld for MinigameAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hack1: assets.load("minigames/hack_1.png"),
            hack1_disk: assets.load("minigames/hack_1_disks-sheet.png"),
            hack2: assets.load("minigames/hack_2.png"),
            hack2_lock: assets.load("minigames/hack_2_lock.png"),
        }
    }
}
