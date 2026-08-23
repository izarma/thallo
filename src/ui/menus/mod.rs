use bevy::prelude::*;

mod actbreak;
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
        actbreak::plugin,
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
    ConnectingSunday,
    ConnectedSunday,
    Act1Break,
    Act2Startup,
    Win,
    Lose,
}
