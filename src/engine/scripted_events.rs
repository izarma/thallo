use std::collections::HashSet;

use bevy::prelude::*;

use crate::engine::{
    CoreSystems,
    dialogue_runner::{DialogueLine, DialogueRunner, Dialogues, derive_state},
    screens::Screen,
    system_apps::{Applications, ChatBoxState, OpenAlertEvent, OpenAppEvent, SystemAlerts},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UnlockState>();
    app.init_resource::<FileDialogueTriggers>();
    app.add_observer(on_scripted_event);
    app.add_observer(on_file_opened);
    app.add_systems(OnEnter(Screen::Desktop), schedule_open_chat);
    app.add_systems(
        Update,
        (tick_delayed_events, check_dialogue_triggers)
            .in_set(CoreSystems::Logic)
            .run_if(in_state(Screen::Desktop)),
    );
}

#[derive(Event, Clone, Debug, PartialEq)]
pub enum ScriptedEventTrigger {
    OpenChat,
    FileTrigger(String),
    ChatTrigger(ChatTriggerType),
    BeginReboot(RebootSequence),
    TransmitSecure,
    TransmitSOS,
    RipperFailed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatTriggerType {
    FileTransfer(NewFileReceiving),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RebootSequence {
    NetworkConnect,
    ActTrans,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NewFileReceiving {
    BruteForce,
    NetRipper,
}

#[derive(Resource, Default, Debug)]
pub struct UnlockState {
    pub chat: bool,
    pub bruteforce: bool,
    pub netripper: bool,
    pub network_reconnected: bool,
    pub act: bool, // true = act 2 | maybe can use bitmask instead to handle game endings
    pub secure_transmitted: bool,
}

fn on_scripted_event(
    ev: On<ScriptedEventTrigger>,
    mut state: ResMut<UnlockState>,
    mut cmd: Commands,
    mut triggers: ResMut<FileDialogueTriggers>,
    mut dialogues: ResMut<Dialogues>,
    mut runner: ResMut<DialogueRunner>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    match ev.clone() {
        ScriptedEventTrigger::OpenChat => {
            effect_open_chat(&mut state, &mut runner, &dialogues, &mut cmd)
        }
        ScriptedEventTrigger::FileTrigger(name) => {
            effect_file_trigger(name, &mut triggers, &mut dialogues, &mut cmd)
        }
        ScriptedEventTrigger::BeginReboot(boot) => {
            effect_reboot(&mut state, boot, &mut next_screen)
        }
        ScriptedEventTrigger::ChatTrigger(trigger) => match trigger {
            ChatTriggerType::FileTransfer(recv) => effect_open_file_transfer_alert(&mut cmd, recv),
        },
        ScriptedEventTrigger::TransmitSecure => {
            effect_transmit_secure(&mut state, &mut dialogues, &mut cmd)
        }
        ScriptedEventTrigger::TransmitSOS => effect_transmit_sos(&mut dialogues),
        ScriptedEventTrigger::RipperFailed => effect_game_over(&mut state, &mut next_screen),
    }
}

pub enum FileTriggerOperation {
    Any, // OR
    All, // AND
}

pub struct DialogueTrigger {
    pub files: Vec<String>,
    pub oper: FileTriggerOperation,
    pub lines: Vec<DialogueLine>,
    pub on_complete: Option<ScriptedEventTrigger>,
    fired: bool,
}

#[derive(Resource, Default)]
pub struct FileDialogueTriggers {
    pub triggers: Vec<DialogueTrigger>,
    opened: HashSet<String>,
}

impl FileDialogueTriggers {
    pub fn register(
        &mut self,
        files: &[&str],
        oper: FileTriggerOperation,
        lines: Vec<DialogueLine>,
    ) {
        self.triggers.push(DialogueTrigger {
            files: files.iter().map(|s| s.to_string()).collect(),
            oper,
            lines,
            on_complete: None,
            fired: false,
        });
    }
    pub fn register_event(
        &mut self,
        files: &[&str],
        oper: FileTriggerOperation,
        event: ScriptedEventTrigger,
    ) {
        self.triggers.push(DialogueTrigger {
            files: files.iter().map(|s| s.to_string()).collect(),
            oper,
            lines: vec![],
            on_complete: Some(event),
            fired: false,
        });
    }
    pub fn notify_opened(&mut self, name: &str) -> (Vec<DialogueLine>, Vec<ScriptedEventTrigger>) {
        self.opened.insert(name.to_string());
        let opened = &self.opened;
        let mut lines = Vec::new();
        let mut events = Vec::new();

        let mut result = Vec::new();
        for trigger in self.triggers.iter_mut().filter(|t| !t.fired) {
            let ready = match trigger.oper {
                FileTriggerOperation::Any => trigger.files.iter().any(|f| opened.contains(f)),
                FileTriggerOperation::All => trigger.files.iter().all(|f| opened.contains(f)),
            };
            if ready {
                trigger.fired = true;
                result.extend(trigger.lines.iter().cloned());
                lines.extend(trigger.lines.iter().cloned());
                if let Some(ev) = trigger.on_complete.clone() {
                    events.push(ev);
                }
            }
        }
        (lines, events)
    }
}

// Trigger Effects

fn effect_open_chat(
    state: &mut UnlockState,
    runner: &mut DialogueRunner,
    dialogues: &Dialogues,
    cmd: &mut Commands,
) {
    if state.chat && !state.act {
        return;
    }
    state.chat = true;
    debug!("[Story] Chat unlocked");
    if matches!(runner.state, ChatBoxState::Done) {
        runner.state = derive_state(&dialogues.lines, dialogues.index);
    }

    cmd.trigger(open_chatbox());
}

fn effect_file_trigger(
    name: String,
    triggers: &mut FileDialogueTriggers,
    dialogues: &mut Dialogues,
    cmd: &mut Commands,
) {
    let (new_lines, direct_events) = triggers.notify_opened(&name);
    if !new_lines.is_empty() {
        dialogues.add_lines(new_lines);
        cmd.trigger(open_chatbox());
    }
    for ev in direct_events {
        cmd.trigger(ev);
    }
}

fn effect_open_file_transfer_alert(cmd: &mut Commands, recv: NewFileReceiving) {
    info!("[Story] File transfer alert triggered");
    cmd.trigger(OpenAlertEvent::from_scripted_event(
        "Incoming Transfer".to_string(),
        SystemAlerts::FileTransfer(recv),
    ));
}

fn effect_reboot(
    state: &mut UnlockState,
    boot: RebootSequence,
    next_screen: &mut NextState<Screen>,
) {
    match boot {
        RebootSequence::NetworkConnect => {
            state.network_reconnected = true;
            info!("network reconnected?");
            next_screen.set(Screen::ActBreak);
        }
        RebootSequence::ActTrans => {
            state.act = true;
            info!("act switch");
            next_screen.set(Screen::ActBreak);
        }
    }
}

fn effect_transmit_secure(state: &mut UnlockState, dialogues: &mut Dialogues, cmd: &mut Commands) {
    if state.secure_transmitted {
        return;
    }
    state.secure_transmitted = true;
    dialogues.add_lines(vec![
        DialogueLine::new(false, "It should've reached Earth hopefully"),
        DialogueLine::new(true, "now what?"),
        DialogueLine::new(false, "Lets send an SOS, and best we can do is hope. Use the netripper sos command from your terminal"),
    ]);
    cmd.trigger(open_chatbox());
}

fn effect_transmit_sos(dialogues: &mut Dialogues) {
    info!("[Story] SOS transmitted — ending act");
    dialogues.add_lines(vec![
        DialogueLine::new(false, "I guess this is it then."),
        DialogueLine::new(true, "yeah I geuess it is.")
            .on_complete(ScriptedEventTrigger::BeginReboot(RebootSequence::ActTrans)),
    ]);
}

fn effect_game_over(state: &mut UnlockState, next_screen: &mut NextState<Screen>) {
    // we will go back to act2 start title here - unlockstate needs to be updated accordinly
    state.netripper = false;
    state.secure_transmitted = false;
    next_screen.set(Screen::ActBreak);
}

fn open_chatbox() -> OpenAppEvent {
    OpenAppEvent {
        name: "Chat".to_string(),
        app_type: Applications::Chatbox,
    }
}

// Delayed Events

#[derive(Component)]
struct DelayedEvent {
    timer: Timer,
    event: ScriptedEventTrigger,
}

fn schedule_open_chat(mut cmd: Commands) {
    cmd.spawn(DelayedEvent {
        timer: Timer::from_seconds(10.0, TimerMode::Once), // Chat Unlock Timer
        event: ScriptedEventTrigger::OpenChat,
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

fn on_file_opened(ev: On<OpenAppEvent>, mut cmd: Commands) {
    let is_file_app = matches!(
        ev.app_type,
        Applications::TextViewer { .. } | Applications::ImageViewer { .. }
    );
    if !is_file_app {
        return;
    }
    cmd.spawn(DelayedEvent {
        timer: Timer::from_seconds(5.0, TimerMode::Once),
        event: ScriptedEventTrigger::FileTrigger(ev.name.clone()),
    });
}

// Dialogues

fn check_dialogue_triggers(
    dialogues: Res<Dialogues>,
    mut last_index: Local<usize>,
    mut cmd: Commands,
) {
    if !dialogues.is_changed() || dialogues.index == *last_index {
        return;
    }
    for i in *last_index..dialogues.index {
        if let Some(event) = dialogues.lines.get(i).and_then(|l| l.on_complete.clone()) {
            info!("[Story] on_complete fired for dialogue line {}", i);
            cmd.trigger(event);
        }
    }
    *last_index = dialogues.index;
}
