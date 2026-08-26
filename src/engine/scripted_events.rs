use std::collections::HashSet;

use bevy::prelude::*;

use crate::engine::{
    CoreSystems,
    dialogue_runner::{DialogueLine, DialogueRunner, Dialogues, derive_state},
    screens::Screen,
    system_apps::{Applications, ChatBoxState, OpenAlertEvent, OpenAppEvent, SystemAlerts},
};
use crate::game::beats::ApplyBeatCommand;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UnlockState>();
    app.init_resource::<FileDialogueTriggers>();
    app.init_resource::<Act2Timer>();
    app.add_observer(on_scripted_event);
    app.add_observer(on_file_opened);
    app.add_systems(OnEnter(Screen::Desktop), schedule_open_chat);
    app.add_systems(
        Update,
        (tick_delayed_events, check_dialogue_triggers)
            .in_set(CoreSystems::Logic)
            .run_if(in_state(Screen::Desktop)),
    );
    app.add_systems(Update, tick_act2_timer.run_if(in_state(Screen::Desktop)));
}

#[derive(Event, Clone, Debug, PartialEq)]
pub enum ScriptedEventTrigger {
    OpenChat,
    FileTrigger(String),
    ChatTrigger(ChatTriggerType),
    FileTransferComplete(NewFileReceiving),
    BeginReboot(RebootSequence),
    TransmitSecure,
    TransmitSOS,
    RipperFailed,
    Win,
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

/// The single source of truth for narrative position.
///
/// Replaces the old set of derived booleans (`network_reconnected`, `act`) with
/// one addressable value. [`apply_beat`](crate::game::beats::apply_beat) rebuilds
/// every piece of state that depends on this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StoryBeat {
    /// Desktop + chat + the Act 1 file puzzle.
    #[default]
    Act1Intro,
    /// `[SECURE]` folder injected, the player hunts for `omega`.
    Act1Connecting,
    /// NetRipper + the SOS countdown.
    Act2Sos,
    /// Terminal beat: the player won.
    Win,
    /// Terminal beat: the player lost.
    Lose,
}

impl StoryBeat {
    /// Every beat, in narrative order, for the dev jump panel.
    #[cfg(feature = "dev")]
    pub const ALL: [StoryBeat; 5] = [
        StoryBeat::Act1Intro,
        StoryBeat::Act1Connecting,
        StoryBeat::Act2Sos,
        StoryBeat::Win,
        StoryBeat::Lose,
    ];
}

/// Top-level progression state.
///
/// This intentionally groups *feature unlocks* and *story beats* into named
/// sub-structs so the code can talk about `state.programs.bruteforce` or
/// `state.story.is_act2()` instead of six flat booleans.
#[derive(Resource, Default, Debug)]
pub struct UnlockState {
    pub programs: ProgramsUnlocked,
    pub story: StoryProgress,
}

/// Features/programs the player has unlocked.
#[derive(Default, Debug)]
pub struct ProgramsUnlocked {
    pub chat: bool,
    pub bruteforce: bool,
    pub netripper: bool,
}

impl ProgramsUnlocked {
    /// The canonical program unlocks for a beat, before any mid-beat unlocks.
    fn for_beat(beat: StoryBeat) -> Self {
        match beat {
            StoryBeat::Act1Intro => ProgramsUnlocked {
                chat: false,
                bruteforce: false,
                netripper: false,
            },
            StoryBeat::Act1Connecting => ProgramsUnlocked {
                chat: true,
                bruteforce: true,
                netripper: false,
            },
            StoryBeat::Act2Sos => ProgramsUnlocked {
                chat: true,
                bruteforce: true,
                netripper: false,
            },
            StoryBeat::Win | StoryBeat::Lose => ProgramsUnlocked {
                chat: true,
                bruteforce: true,
                netripper: true,
            },
        }
    }
}

/// Story progression flags that drive screen/menu transitions and narrative checks.
#[derive(Default, Debug)]
pub struct StoryProgress {
    /// Authoritative narrative position.
    pub beat: StoryBeat,
    /// Mid-Act-2 flag: set once the `[SECURE]` payload has been transmitted,
    /// which gates the `netripper sos` command.
    pub secure_transmitted: bool,
}

impl StoryProgress {
    /// True once the player has reached Act 2.
    pub fn is_act2(&self) -> bool {
        matches!(self.beat, StoryBeat::Act2Sos)
    }

    /// True during the "connect to Sunday" interstitial (after reconnect but before Act 2).
    pub fn is_connecting_sunday(&self) -> bool {
        matches!(self.beat, StoryBeat::Act1Connecting)
    }

    /// True when the connecting-Sunday transition sound effect should play.
    /// Act 2 no longer plays a break sfx (the single title card is silent).
    pub fn should_play_break_sfx(&self) -> bool {
        self.is_connecting_sunday()
    }
}

impl UnlockState {
    /// Reset [`StoryProgress`] and [`ProgramsUnlocked`] to the canonical values for `beat`.
    ///
    /// This only touches the *flags*; rebuilding the VFS, dialogues, triggers and
    /// transient UI state is done by [`crate::game::beats::apply_beat`], which is the
    /// only entry point that should call this.
    pub(crate) fn apply_beat(&mut self, beat: StoryBeat) {
        self.story.beat = beat;
        self.story.secure_transmitted = false;
        self.programs = ProgramsUnlocked::for_beat(beat);
    }
}

/// One-shot result of a run, emitted once before [`Screen::GameOver`] is
/// entered. The game-over screen reads it to pick the win or lose menu.
#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOverEvent {
    Win,
    Lose,
}

/// Overall Act 2 countdown. Starts once the Act 2 desktop loads and, when it
/// hits zero, forces a loss unless the SOS transmission has already completed.
pub const ACT2_TIME_LIMIT: f32 = (5.0 * 60.0) + 1.0;

#[derive(Resource)]
pub struct Act2Timer {
    pub remaining: f32,
    pub active: bool,
}

impl Default for Act2Timer {
    fn default() -> Self {
        Self {
            remaining: ACT2_TIME_LIMIT,
            active: false,
        }
    }
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
        ScriptedEventTrigger::FileTransferComplete(recv) => {
            effect_file_transfer_complete(&mut state, recv)
        }
        ScriptedEventTrigger::BeginReboot(boot) => effect_reboot(boot, &mut cmd, &mut next_screen),
        ScriptedEventTrigger::ChatTrigger(trigger) => match trigger {
            ChatTriggerType::FileTransfer(recv) => effect_open_file_transfer_alert(&mut cmd, recv),
        },
        ScriptedEventTrigger::TransmitSecure => {
            effect_transmit_secure(&mut state.story, &mut dialogues, &mut cmd)
        }
        ScriptedEventTrigger::TransmitSOS => effect_transmit_sos(&mut dialogues),
        ScriptedEventTrigger::RipperFailed => effect_lose(&mut cmd, &mut next_screen),
        ScriptedEventTrigger::Win => effect_win(&mut cmd, &mut next_screen),
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

    /// Remove every registered trigger and forget which files have been opened.
    ///
    /// Called by [`apply_beat`](crate::game::beats::apply_beat) so rebuilding state
    /// for a beat never leaks the previous beat's `opened` set.
    pub fn clear(&mut self) {
        self.triggers.clear();
        self.opened.clear();
    }

    pub fn notify_opened(&mut self, name: &str) -> (Vec<DialogueLine>, Vec<ScriptedEventTrigger>) {
        self.opened.insert(name.to_string());
        let opened = &self.opened;
        let mut lines = Vec::new();
        let mut events = Vec::new();

        for trigger in self.triggers.iter_mut().filter(|t| !t.fired) {
            let ready = match trigger.oper {
                FileTriggerOperation::Any => trigger.files.iter().any(|f| opened.contains(f)),
                FileTriggerOperation::All => trigger.files.iter().all(|f| opened.contains(f)),
            };
            if ready {
                trigger.fired = true;
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
    if state.programs.chat && !state.story.is_act2() {
        return;
    }
    state.programs.chat = true;
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

/// Unlock a program once its file-transfer alert has finished playing out.
fn effect_file_transfer_complete(state: &mut UnlockState, recv: NewFileReceiving) {
    match recv {
        NewFileReceiving::BruteForce => state.programs.bruteforce = true,
        NewFileReceiving::NetRipper => state.programs.netripper = true,
    }
    info!("[Story] Program unlocked: {:?}", recv);
}

fn effect_reboot(boot: RebootSequence, cmd: &mut Commands, next_screen: &mut NextState<Screen>) {
    match boot {
        RebootSequence::NetworkConnect => {
            info!("network reconnected?");
            cmd.queue(ApplyBeatCommand(StoryBeat::Act1Connecting));
            next_screen.set(Screen::ActBreak);
        }
        RebootSequence::ActTrans => {
            info!("act switch");
            cmd.queue(ApplyBeatCommand(StoryBeat::Act2Sos));
            next_screen.set(Screen::ActBreak);
        }
    }
}

fn effect_transmit_secure(
    story: &mut StoryProgress,
    dialogues: &mut Dialogues,
    cmd: &mut Commands,
) {
    if story.secure_transmitted {
        return;
    }
    story.secure_transmitted = true;
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
        DialogueLine::new(true, "yeah I geuess it is.").on_complete(ScriptedEventTrigger::Win),
    ]);
}

fn effect_win(cmd: &mut Commands, next_screen: &mut NextState<Screen>) {
    info!("[Story] Act complete — win");
    cmd.queue(ApplyBeatCommand(StoryBeat::Win));
    cmd.trigger(GameOverEvent::Win);
    next_screen.set(Screen::GameOver);
}

fn effect_lose(cmd: &mut Commands, next_screen: &mut NextState<Screen>) {
    info!("[Story] Act failed — lose");
    cmd.queue(ApplyBeatCommand(StoryBeat::Lose));
    cmd.trigger(GameOverEvent::Lose);
    next_screen.set(Screen::GameOver);
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

fn tick_act2_timer(
    time: Res<Time>,
    mut timer: ResMut<Act2Timer>,
    mut cmd: Commands,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if !timer.active {
        return;
    }
    timer.remaining -= time.delta_secs();
    if timer.remaining <= 0.0 {
        timer.remaining = 0.0;
        timer.active = false;
        effect_lose(&mut cmd, &mut next_screen);
    }
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
