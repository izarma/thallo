use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::{design_scale::DesignScale, screens::Screen},
    ui::{menus::Menu, theme::widgets::primitives},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        lose_menu.run_if(in_state(Menu::Lose)),
    );
}

fn lose_menu(
    mut contexts: EguiContexts,
    scale: Res<DesignScale>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut app_exit: MessageWriter<AppExit>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    primitives::centered_panel(ctx, "lose_menu", |ui| {
        primitives::header(ui, "CONNECTION LOST", &scale);
        primitives::label(
            ui,
            "The catastrophe arrived before you could send the SOS.",
            &scale,
        );
        if primitives::button(ui, "Retry", &scale).clicked() {
            next_screen.set(Screen::ActBreak);
        }
        if primitives::button(ui, "Quit", &scale).clicked() {
            app_exit.write(AppExit::Success);
        }
    });
    Ok(())
}
