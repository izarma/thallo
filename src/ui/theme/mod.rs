use bevy::prelude::*;

mod interaction;
mod palette;
pub mod widgets;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(interaction::plugin);
}
