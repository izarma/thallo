use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::engine::{UiPassSystems, asset_tracking::LoadResource};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<ButtonAssets>();
    app.add_systems(
        EguiPrimaryContextPass,
        cache_button_textures
            .run_if(resource_exists::<ButtonAssets>.and(not(resource_exists::<ButtonTextures>)))
            .in_set(UiPassSystems::CacheTextures),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct ButtonAssets {
    #[dependency]
    button: Handle<Image>,
    #[dependency]
    reveal_button: Handle<Image>,
}

impl FromWorld for ButtonAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            button: assets.load("ui/button.png"),
            reveal_button: assets.load("ui/reveal_button.png"),
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct ButtonTextures {
    pub button: egui::TextureId,
    pub reveal_button: egui::TextureId,
}

fn cache_button_textures(mut contexts: EguiContexts, assets: Res<ButtonAssets>, mut cmd: Commands) {
    cmd.insert_resource(ButtonTextures {
        button: contexts.add_image(EguiTextureHandle::Weak(assets.button.id())),
        reveal_button: contexts.add_image(EguiTextureHandle::Weak(assets.reveal_button.id())),
    });
}
