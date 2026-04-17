use bevy::prelude::*;

use crate::engine::{
    asset_tracking::LoadResource,
    screens::Screen,
    system_apps::{Applications, OpenAppEvent},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UnlockState>()
        .load_resource::<MinigameAssets>();
    app.add_observer(process_scripted_event);
    app.add_systems(OnEnter(Screen::Desktop), schedule_open_chat);
    app.add_systems(
        Update,
        tick_delayed_events.run_if(in_state(Screen::Desktop)),
    );

    app.init_resource::<ActiveMinigame>()
        .add_observer(on_minigame_trigger);
}

#[derive(Event, Clone)]
pub enum ScriptedEvent {
    OpenChat,
    Unlock1,
}

#[derive(Component)]
struct DelayedEvent {
    timer: Timer,
    event: ScriptedEvent,
}

#[derive(Resource, Default, Debug)]
pub struct UnlockState {
    pub chat: bool,
    pub mg1: bool,
}

#[derive(Event, Debug, Clone)]
pub struct MinigameTrigger {
    pub checkpoint: usize,
}

#[derive(Resource, Default, Debug)]
pub struct ActiveMinigame {
    pub checkpoint: Option<usize>,
}

fn process_scripted_event(
    ev: On<ScriptedEvent>,
    mut state: ResMut<UnlockState>,
    mut cmd: Commands,
) {
    match *ev {
        ScriptedEvent::OpenChat => {
            if !state.chat {
                state.chat = true;
                info!("[Story] Chat unlocked");
                cmd.trigger(OpenAppEvent {
                    name: "Chat".to_string(),
                    app_type: Applications::Chatbox {
                        input: String::new(),
                    },
                });
            }
        }
        ScriptedEvent::Unlock1 => {
            if !state.mg1 {
                state.mg1 = true;
                info!("[Story] omega.png decrypted — Unlock1 complete");
            }
        }
    }
}

fn schedule_open_chat(mut cmd: Commands) {
    cmd.spawn(DelayedEvent {
        timer: Timer::from_seconds(5.0, TimerMode::Once),
        event: ScriptedEvent::OpenChat,
    });
}

fn tick_delayed_events(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut DelayedEvent)>,
    time: Res<Time>,
) {
    for (entity, mut delayed) in &mut query {
        delayed.timer.tick(time.delta());
        if delayed.timer.just_finished() {
            cmd.trigger(delayed.event.clone());
            cmd.entity(entity).despawn();
        }
    }
}

fn on_minigame_trigger(ev: On<MinigameTrigger>, mut state: ResMut<ActiveMinigame>) {
    // Only open one at a time — decryption timer is already paused
    // because you'll stop advancing `elapsed` while active (see step 4).
    if state.checkpoint.is_none() {
        state.checkpoint = Some(ev.checkpoint);
        info!("[Minigame] Checkpoint {} triggered", ev.checkpoint);
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MinigameAssets {
    #[dependency]
    pub hack_1_bg: Handle<Image>,
    #[dependency]
    pub hack_1_disk: Handle<Image>,
}

impl FromWorld for MinigameAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hack_1_bg: assets.load("minigames/corpus/hack_1.png"),
            hack_1_disk: assets.load("minigames/corpus/hack_1_disks-sheet.png"),
        }
    }
}
