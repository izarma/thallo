use std::collections::HashSet;

use bevy::prelude::*;

use crate::engine::{
    CoreSystems,
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
    ChatTrigger(ChatTriggerType), // add required shit later
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatTriggerType {
    FileTransfer(NewFileReceiving),
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
}

fn on_scripted_event(
    ev: On<ScriptedEventTrigger>,
    mut state: ResMut<UnlockState>,
    mut cmd: Commands,
    mut triggers: ResMut<FileDialogueTriggers>,
    mut dialogues: ResMut<Dialogues>,
) {
    match ev.clone() {
        ScriptedEventTrigger::OpenChat => effect_unlock_chat(&mut state, &mut cmd),
        ScriptedEventTrigger::FileTrigger(name) => {
            effect_file_trigger(name, &mut triggers, &mut dialogues, &mut cmd)
        }
        ScriptedEventTrigger::ChatTrigger(trigger) => match trigger {
            ChatTriggerType::FileTransfer(recv) => effect_open_file_transfer_alert(&mut cmd, recv),
        },
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
            fired: false,
        });
    }
    pub fn notify_opened(&mut self, name: &str) -> Vec<DialogueLine> {
        self.opened.insert(name.to_string());
        let opened = &self.opened;
        let mut result = Vec::new();
        for trigger in self.triggers.iter_mut().filter(|t| !t.fired) {
            let ready = match trigger.oper {
                FileTriggerOperation::Any => trigger.files.iter().any(|f| opened.contains(f)),
                FileTriggerOperation::All => trigger.files.iter().all(|f| opened.contains(f)),
            };
            if ready {
                trigger.fired = true;
                result.extend(trigger.lines.iter().cloned());
            }
        }
        result
    }
}

// Trigger Effects

fn effect_unlock_chat(state: &mut UnlockState, cmd: &mut Commands) {
    if state.chat {
        return;
    }
    state.chat = true;
    info!("[Story] Chat unlocked");
    cmd.trigger(open_chatbox());
}

fn effect_file_trigger(
    name: String,
    triggers: &mut FileDialogueTriggers,
    dialogues: &mut Dialogues,
    cmd: &mut Commands,
) {
    let new_lines = triggers.notify_opened(&name);
    if !new_lines.is_empty() {
        dialogues.add_lines(new_lines);
        cmd.trigger(open_chatbox());
    }
}

fn effect_open_file_transfer_alert(cmd: &mut Commands, recv: NewFileReceiving) {
    info!("[Story] File transfer alert triggered");
    cmd.trigger(OpenAlertEvent::from_scripted_event(
        "Incoming Transfer".to_string(),
        SystemAlerts::FileTransfer(recv),
    ));
}

fn open_chatbox() -> OpenAppEvent {
    OpenAppEvent {
        name: "Chat".to_string(),
        app_type: Applications::Chatbox {
            input: String::new(),
            state: ChatBoxState::AnonTyping { elapsed: 0.0 },
            displayed: Vec::new(),
        },
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
        timer: Timer::from_seconds(1.0, TimerMode::Once), // cahnge to 5 later
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

/// `speaker: false` = anon, `speaker: true` = player.
#[derive(Debug, PartialEq, Clone)]
pub struct DialogueLine {
    pub speaker: bool,
    pub text: String,
    pub on_complete: Option<ScriptedEventTrigger>,
}

impl DialogueLine {
    pub fn new(speaker: bool, text: &str) -> Self {
        Self {
            speaker,
            text: text.to_string(),
            on_complete: None,
        }
    }
    pub fn on_complete(mut self, event: ScriptedEventTrigger) -> Self {
        self.on_complete = Some(event);
        self
    }
}

#[derive(Resource)]
pub struct Dialogues {
    pub lines: Vec<DialogueLine>,
    pub index: usize,
}

impl Dialogues {
    pub fn add_lines(&mut self, lines: Vec<DialogueLine>) {
        self.lines.extend(lines);
    }
    // why is this needed - if needed why not used?
    pub fn has_unplayed(&self) -> bool {
        self.index < self.lines.len()
    }
}

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
