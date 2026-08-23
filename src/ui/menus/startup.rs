use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::{
        design_scale::DesignScale,
        screens::Screen,
        video::{VideoPlayers, spawn_video_player},
    },
    ui::{menus::Menu, theme::widgets::primitives},
};

const DISCLAIMER_VIDEO_PATH: &str = "assets/cutscenes/disclaimer.mp4";

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Startup), spawn_startup_video);
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

fn spawn_startup_video(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut players: NonSendMut<VideoPlayers>,
) {
    let Some((entity, image_handle)) = spawn_video_player(
        &mut commands,
        &mut images,
        &mut players,
        DISCLAIMER_VIDEO_PATH,
        true,
    ) else {
        return;
    };

    commands.entity(entity).insert((
        Name::new("StartupVideo"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode::new(image_handle),
        // Keep the video visible only while the startup menu is active.
        DespawnOnExit(Menu::Startup),
    ));
}

fn startup_ui(
    mut contexts: EguiContexts,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    primitives::low_centered_panel(ctx, "startup_menu", |ui| {
        ui.style_mut().interaction.selectable_labels = false;

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
