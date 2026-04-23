use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiTextureHandle,
    egui::{self},
};

use crate::{
    engine::{
        asset_tracking::LoadResource, file_system::FsHierarchy, screens::Screen,
        scripted_events::FileDialogueTriggers,
    },
    game::{
        dialogues::{build_act1_file_triggers, build_dialogues},
        files::build_fs_hierarchy,
    },
};

mod dialogues;
mod files;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<Act1Assets>();
    app.add_systems(
        OnEnter(Screen::Desktop),
        setup_act1_stuffs
            .run_if(resource_exists::<Act1Assets>)
            .run_if(not(resource_exists::<FsHierarchy>)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct Act1Assets {
    #[dependency]
    pub omega: Handle<Image>,
}

impl FromWorld for Act1Assets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            omega: assets.load("game/act1/omega_r.png"),
        }
    }
}

fn setup_act1_stuffs(
    mut contexts: EguiContexts,
    assets: Res<Act1Assets>,
    images: Res<Assets<Image>>,
    mut file_triggers: ResMut<FileDialogueTriggers>,
    mut cmd: Commands,
) {
    // File hierarchy
    let size = images
        .get(&assets.omega)
        .map(|img| {
            let s = img.size_f32();
            egui::Vec2::new(s.x, s.y)
        })
        .unwrap_or(egui::Vec2::splat(64.0));
    let omega_tex = contexts.add_image(EguiTextureHandle::Weak(assets.omega.id()));
    cmd.insert_resource(build_fs_hierarchy(omega_tex, size));

    // Dialogues
    cmd.insert_resource(build_dialogues());
    build_act1_file_triggers(&mut file_triggers);
}
