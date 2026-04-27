use bevy::prelude::*;

pub mod desktop;
mod shutdown;
mod startup;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();
    app.add_plugins((startup::plugin, shutdown::plugin, desktop::plugin));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Startup,
    ShutDown,
}
