use bevy::prelude::*;

use crate::ui::menus::Menu;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(super::Screen::GameOver), open_game_over_menu);
}

fn open_game_over_menu(mut next_menu: ResMut<NextState<Menu>>) {
    // add input for win/lose
    if true {
        next_menu.set(Menu::Win);
    } else {
        next_menu.set(Menu::Lose);
    }
}
