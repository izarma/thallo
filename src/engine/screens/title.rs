use bevy::prelude::*;

use crate::ui::menus::Menu;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(super::Screen::Title), open_startup_warning);
    app.add_systems(OnExit(super::Screen::Title), close_startup_warning);
}

fn open_startup_warning(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Startup);
}

fn close_startup_warning(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
