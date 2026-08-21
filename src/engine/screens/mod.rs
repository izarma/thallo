use bevy::prelude::*;

mod act_break;
pub mod desktop;
mod gameover;
mod loading;
mod title;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.add_plugins((
        title::plugin,
        loading::plugin,
        desktop::plugin,
        act_break::plugin,
        gameover::plugin,
    ));
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum Screen {
    #[default]
    Title,
    Loading,
    Desktop,
    ActBreak,
    GameOver,
}
