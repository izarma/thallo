use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::design_scale::DesignScale,
    ui::{menus::Menu, theme::widgets::primitives},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, win_menu.run_if(in_state(Menu::Win)));
}

fn win_menu(
    mut contexts: EguiContexts,
    scale: Res<DesignScale>,
    mut app_exit: MessageWriter<AppExit>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    primitives::centered_panel(ctx, "win_menu", |ui| {
        primitives::header(ui, "TRANSMISSION RECEIVED", &scale);
        primitives::label(ui, "something something", &scale);
        if primitives::button(ui, "Quit", &scale).clicked() {
            app_exit.write(AppExit::Success);
        }
    });
    Ok(())
}
