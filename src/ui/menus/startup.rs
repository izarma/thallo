use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::{design_scale::DesignScale, screens::Screen},
    ui::{menus::Menu, theme::widgets::primitives},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        startup_ui.run_if(in_state(Menu::Startup)),
    );
    app.add_systems(
        EguiPrimaryContextPass,
        connet_sunday_title.run_if(in_state(Menu::ConnectedSunday)),
    );
    app.add_systems(
        EguiPrimaryContextPass,
        act2_title.run_if(in_state(Menu::Act2Startup)),
    );
}

fn startup_ui(
    mut contexts: EguiContexts,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    primitives::centered_panel(ctx, "startup_menu", |ui| {
        ui.style_mut().interaction.selectable_labels = false;
        primitives::header(ui, "Have you played \"nihil.\" before?", &scale);
        primitives::label(
            ui,
            "This project relies on you going through all three endings for a better experience.",
            &scale,
        );
        ui.add_space(12.0);

        if primitives::button(ui, "Yes", &scale).clicked() {
            info!("enter load - yes click");
            next_screen.set(Screen::Loading);
        }

        if primitives::button(ui, "No", &scale).clicked() {
            next_menu.set(Menu::ShutDown);
        }
    });

    Ok(())
}

fn connet_sunday_title(
    mut contexts: EguiContexts,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    primitives::centered_panel(ctx, "reconnect_menu", |ui| {
        primitives::header(ui, "SUNDAY NETWORK", &scale);
        primitives::label(ui, "Terminal reconnected. Incoming sync detected.", &scale);
        if primitives::button(ui, "Connect", &scale).clicked() {
            next_screen.set(Screen::Loading);
            next_menu.set(Menu::None);
        }
    });
    Ok(())
}

// need to fix this with proper content
fn act2_title(
    mut contexts: EguiContexts,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    primitives::centered_panel(ctx, "act2_title", |ui| {
        primitives::header(ui, "ACT II", &scale);
        primitives::label(ui, "5 minutes before the catastrophe on Sunday", &scale);
        if primitives::button(ui, "Power On", &scale).clicked() {
            next_screen.set(Screen::Loading);
            next_menu.set(Menu::None);
        }
    });
    Ok(())
}
