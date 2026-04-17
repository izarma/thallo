use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::screens::Screen,
    ui::{menus::Menu, theme::widgets::primitives},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        startup_ui.run_if(in_state(Menu::Startup)),
    );
}

fn startup_ui(
    mut contexts: EguiContexts,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    primitives::centered_panel(ctx, "startup_menu", |ui| {
        ui.style_mut().interaction.selectable_labels = false;
        primitives::header(ui, "Have you played \"nihil.\" before?");
        primitives::label(
            ui,
            "This project relies on you going through all three endings for a better experience.",
        );
        ui.add_space(12.0);

        if primitives::button(ui, "Yes").clicked() {
            info!("enter load - yes click");
            next_screen.set(Screen::Loading);
        }

        if primitives::button(ui, "No").clicked() {
            next_menu.set(Menu::ShutDown);
        }
    });

    Ok(())
}
