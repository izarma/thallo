use bevy::prelude::*;

mod apps;
pub mod menus;
mod popups;
mod theme;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((theme::plugin, menus::plugin, apps::plugin, popups::plugin));
}
