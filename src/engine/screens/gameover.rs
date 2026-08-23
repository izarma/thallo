use bevy::prelude::*;

use crate::{engine::scripted_events::GameOverEvent, ui::menus::Menu};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_game_over);
}

fn on_game_over(ev: On<GameOverEvent>, mut next_menu: ResMut<NextState<Menu>>) {
    match *ev {
        GameOverEvent::Win => next_menu.set(Menu::Win),
        GameOverEvent::Lose => next_menu.set(Menu::Lose),
    }
}
