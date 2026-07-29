use bevy::prelude::*;

use crate::{engine::scripted_events::UnlockState, ui::menus::Menu};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(super::Screen::ActBreak), open_break_menu);
}

fn open_break_menu(mut next_menu: ResMut<NextState<Menu>>, state: Res<UnlockState>) {
    if state.story.is_connecting_sunday() {
        next_menu.set(Menu::ConnectingSunday);
    } else if state.story.is_act2() {
        next_menu.set(Menu::Act1Break);
    }
}
