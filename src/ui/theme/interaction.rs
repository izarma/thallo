use crate::engine::{asset_tracking::LoadResource, audio::sound_effect, screens::Screen};
use bevy::prelude::*;
use rand::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(apply_interaction_palette_on_click);
    app.add_observer(apply_interaction_palette_on_over);
    app.add_observer(apply_interaction_palette_on_out);
    app.add_observer(apply_interaction_palette_on_release);

    app.load_resource::<InteractionAssets>();
    app.add_observer(play_sound_effect_on_click);
    app.add_observer(play_sound_effect_on_over);

    app.add_systems(
        Update,
        play_sound_effect_on_keypress.run_if(in_state(Screen::Desktop)),
    );
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette_on_click(
    click: On<Pointer<Click>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(click.event_target()) else {
        return;
    };

    *bg = palette.pressed.into();
}

fn apply_interaction_palette_on_release(
    click: On<Pointer<Release>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(click.event_target()) else {
        return;
    };

    *bg = palette.hovered.into();
}

fn apply_interaction_palette_on_over(
    over: On<Pointer<Over>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(over.event_target()) else {
        return;
    };

    *bg = palette.hovered.into();
}

fn apply_interaction_palette_on_out(
    out: On<Pointer<Out>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(out.event_target()) else {
        return;
    };

    *bg = palette.none.into();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct InteractionAssets {
    #[dependency]
    hover: Handle<AudioSource>,
    #[dependency]
    click: Vec<Handle<AudioSource>>,
    #[dependency]
    keypress: Vec<Handle<AudioSource>>,
}

impl FromWorld for InteractionAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hover: assets.load("audio/sfx/button_hover.ogg"),
            click: vec![
                assets.load("audio/sfx/mouse1.ogg"),
                assets.load("audio/sfx/mouse2.ogg"),
                assets.load("audio/sfx/mouse3.ogg"),
            ],
            keypress: vec![
                assets.load("audio/sfx/key1.ogg"),
                assets.load("audio/sfx/key2.ogg"),
                assets.load("audio/sfx/key3.ogg"),
                assets.load("audio/sfx/key4.ogg"),
            ],
        }
    }
}

fn play_sound_effect_on_click(
    _: On<Pointer<Click>>,
    interaction_assets: If<Res<InteractionAssets>>,
    mut commands: Commands,
) {
    let mut rng = rand::rng();
    if let Some(random_click_sfx) = interaction_assets.click.choose(&mut rng) {
        commands.spawn(sound_effect(random_click_sfx.clone()));
    }
}

fn play_sound_effect_on_over(
    _: On<Pointer<Over>>,
    interaction_assets: If<Res<InteractionAssets>>,
    mut commands: Commands,
) {
    commands.spawn(sound_effect(interaction_assets.hover.clone()));
}

fn play_sound_effect_on_keypress(
    keys: Res<ButtonInput<KeyCode>>,
    interaction_assets: Option<Res<InteractionAssets>>,
    mut commands: Commands,
) {
    if let Some(assets) = interaction_assets {
        if keys.get_just_pressed().next().is_some() {
            let mut rng = rand::rng();
            if let Some(random_key_sfx) = assets.keypress.choose(&mut rng) {
                commands.spawn(sound_effect(random_key_sfx.clone()));
            }
        }
    }
}
