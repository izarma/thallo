use bevy::prelude::*;

pub mod button_textures;
mod interaction;
pub mod palette;
pub mod widgets;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((button_textures::plugin, interaction::plugin));
}
