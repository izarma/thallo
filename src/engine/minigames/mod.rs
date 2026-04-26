use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<MinigameAssets>();
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
            hack1_disk: assets.load("minigames/hack_1_disk-sheet.png"),
            hack2: assets.load("minigames/hack_2.png"),
            hack2_lock: assets.load("minigames/hack_2_lock.png"),
        }
    }
}
