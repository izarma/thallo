use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::engine::UiPassSystems;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CurrentCursor>()
        .init_resource::<CursorEguiTexture>()
        .add_systems(
            EguiPrimaryContextPass,
            (cache_cursor_texture, draw_cursor)
                .chain()
                .in_set(UiPassSystems::Render),
        );
}

/// Handle of the image that should currently be shown as the in-game cursor.
///
/// This replaces the OS-level custom cursor so that the cursor is rendered
/// inside Bevy's view target and therefore receives the post-processing shader.
#[derive(Resource, Default, Clone)]
pub struct CurrentCursor {
    pub handle: Handle<Image>,
}

#[derive(Resource, Default, Clone, Copy)]
struct CursorEguiTexture {
    handle_id: Option<AssetId<Image>>,
    texture_id: Option<egui::TextureId>,
}

fn cache_cursor_texture(
    mut contexts: EguiContexts,
    current: Res<CurrentCursor>,
    mut cached: ResMut<CursorEguiTexture>,
) {
    let new_id = current.handle.id();

    if cached.handle_id == Some(new_id) {
        return;
    }

    if let Some(old_id) = cached.handle_id {
        contexts.remove_image(old_id);
    }

    cached.handle_id = Some(new_id);
    cached.texture_id = Some(contexts.add_image(EguiTextureHandle::Weak(new_id)));
}

fn draw_cursor(
    mut contexts: EguiContexts,
    current: Res<CurrentCursor>,
    cached: Res<CursorEguiTexture>,
    images: Res<Assets<Image>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Some(texture_id) = cached.texture_id else {
        return;
    };
    let Some(image) = images.get(&current.handle) else {
        return;
    };
    let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) else {
        return;
    };

    let scale = ctx.pixels_per_point();
    let size = egui::vec2(image.width() as f32 / scale, image.height() as f32 / scale);

    egui::Area::new(egui::Id::new("game_cursor"))
        .order(egui::Order::Tooltip)
        .interactable(false)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            ui.add(egui::Image::new(egui::load::SizedTexture::new(
                texture_id, size,
            )));
        });
}
