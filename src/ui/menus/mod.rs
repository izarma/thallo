use bevy::prelude::*;

pub mod desktop;
mod lose;
mod shutdown;
mod startup;
mod win;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();
    app.add_plugins((
        startup::plugin,
        shutdown::plugin,
        desktop::plugin,
        win::plugin,
        lose::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Startup,
    ShutDown,
    Win,
    Lose,
}
