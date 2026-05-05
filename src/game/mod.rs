use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiTextureHandle,
    egui::{self},
};

use crate::{
    engine::{
        asset_tracking::LoadResource,
        audio::music,
        file_system::FsHierarchy,
        screens::Screen,
        scripted_events::{FileDialogueTriggers, UnlockState},
        window_manager::OpenWindows,
    },
    game::{
        dialogues::{
            build_act1_dialogues, build_act1_file_triggers, build_act2_dialogues,
            build_netconn_file_triggers,
        },
        files::{build_fs, inject_secure_folder, strip_to_secure},
    },
};

mod dialogues;
mod files;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<FileAssets>();
    app.add_systems(
        OnEnter(Screen::Desktop),
        (
            setup_stuffs
                .run_if(resource_exists::<FileAssets>.and(not(resource_exists::<FsHierarchy>))),
            connect_sunday_net.run_if(|state: Res<UnlockState>| state.network_reconnected),
            setup_act2.run_if(|state: Res<UnlockState>| state.act),
        ),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct FileAssets {
    #[dependency]
    pub omega: Handle<Image>,
    #[dependency]
    pub ac1bg: Handle<AudioSource>,
    #[dependency]
    pub tans_act: Handle<AudioSource>,
}

impl FromWorld for FileAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            omega: assets.load("game/act1/omega_r.png"),
            ac1bg: assets.load("audio/ambience_act1.ogg"),
            tans_act: assets.load("audio/transition_act1.ogg"),
        }
    }
}

fn setup_stuffs(
    mut file_triggers: ResMut<FileDialogueTriggers>,
    mut cmd: Commands,
    assets: Res<FileAssets>,
) {
    cmd.spawn((music(assets.ac1bg.clone()), DespawnOnExit(Screen::Desktop)));
    cmd.insert_resource(build_fs());
    cmd.insert_resource(build_act1_dialogues());
    build_act1_file_triggers(&mut file_triggers);
}

fn connect_sunday_net(
    mut vfs: ResMut<FsHierarchy>,
    mut contexts: EguiContexts,
    assets: Res<FileAssets>,
    images: Res<Assets<Image>>,
    mut open_windows: ResMut<OpenWindows>,
    mut file_triggers: ResMut<FileDialogueTriggers>,
) {
    open_windows.windows.clear();
    file_triggers.triggers.clear();
    let size = images
        .get(&assets.omega)
        .map(|img| {
            let s = img.size_f32();
            egui::Vec2::new(s.x, s.y)
        })
        .unwrap_or(egui::Vec2::splat(64.0));
    let omega_tex = contexts.add_image(EguiTextureHandle::Weak(assets.omega.id()));
    inject_secure_folder(&mut vfs, omega_tex, size);
    build_netconn_file_triggers(&mut file_triggers);
}

fn setup_act2(
    mut cmd: Commands,
    mut vfs: ResMut<FsHierarchy>,
    mut open_windows: ResMut<OpenWindows>,
    mut file_triggers: ResMut<FileDialogueTriggers>,
) {
    strip_to_secure(&mut vfs);
    open_windows.windows.clear();
    file_triggers.triggers.clear();
    cmd.insert_resource(build_act2_dialogues());
}
