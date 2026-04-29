use bevy::prelude::*;

use crate::{engine::scripted_events::UnlockState, ui::menus::Menu};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(super::Screen::Title), open_startup_warning);
    app.add_systems(OnExit(super::Screen::Title), close_startup_warning);
}

fn open_startup_warning(mut next_menu: ResMut<NextState<Menu>>, state: Res<UnlockState>) {
    if state.network_reconnected && !state.act {
        next_menu.set(Menu::ConnectedSunday);
    } else if state.act {
        next_menu.set(Menu::Act2Startup);
    } else {
        next_menu.set(Menu::Startup);
    }
}

fn close_startup_warning(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
