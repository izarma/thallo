use bevy::prelude::*;
use bevy_egui::egui;

use crate::engine::{
    file_system::FsPath,
    scripted_events::{ScriptedEventTrigger, UnlockState},
    system_apps::{Applications, OpenAlertEvent, OpenAppEvent, SystemAlerts},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OpenWindows>();
    app.add_observer(handle_open_node_events);
    app.add_observer(handle_toggle_minimize_event);
    app.add_observer(handle_close_window_event);
}

/// Maximum number of open windows allowed for a single taskbar group.
/// When this cap is exceeded, the oldest windows of that group are closed.
const MAX_WINDOWS_PER_GROUP: usize = 25;

#[derive(Resource, Default)]
pub struct OpenWindows {
    pub windows: Vec<WindowEntry>,
}

impl OpenWindows {
    /// Open a window, or bring it to focus if already open.
    fn open(&mut self, event: OpenAppEvent) {
        // Singleton apps: if already open (even minimized), un-minimize and return.
        let is_singleton = matches!(event.app_type, Applications::Chatbox);
        if is_singleton
            && let Some(existing) = self
                .windows
                .iter_mut()
                .find(|w| matches!(w.event.app_type, Applications::Chatbox))
        {
            existing.is_minimized = false;
            return;
        }
        // Enforce the per-group cap, closing the oldest windows first.
        let type_name = event.app_type.type_name();

        self.windows.push(WindowEntry::new(event));
        let group_count = self
            .windows
            .iter()
            .filter(|w| w.event.app_type.type_name() == type_name)
            .count();
        if group_count > MAX_WINDOWS_PER_GROUP {
            let mut to_close = group_count - MAX_WINDOWS_PER_GROUP;
            self.windows.retain(|w| {
                if to_close == 0 || w.event.app_type.type_name() != type_name {
                    true
                } else {
                    to_close -= 1;
                    false
                }
            });
        }
    }
}

/// One entry per open window on the desktop - derived from OpenNodeEvent that adds it in OpenWindows Resource
#[derive(Debug, Clone)]
pub struct WindowEntry {
    pub id: egui::Id,
    pub event: OpenAppEvent,
    pub is_minimized: bool,
    pub is_open: bool, // used to close by X button
}

impl WindowEntry {
    pub fn new(event: OpenAppEvent) -> Self {
        Self {
            id: egui::Id::new(uuid::Uuid::new_v4()),
            event,
            is_minimized: false,
            is_open: true,
        }
    }
}

fn handle_open_node_events(
    node: On<OpenAppEvent>,
    mut open_windows: ResMut<OpenWindows>,
    state: Res<UnlockState>,
    mut cmd: Commands,
) {
    if matches!(node.app_type, Applications::Decrypter { .. }) && !state.bruteforce {
        cmd.trigger(OpenAlertEvent {
            name: node.name.clone(),
            alert: SystemAlerts::EncryptedError,
        });
        return;
    }

    open_windows.open(node.clone());
}

/// Triggered to toggle the minimized state of a specific window.
/// `id` must match a [`WindowEntry::id`] in [`OpenWindows`].
#[derive(Event, Debug, Clone)]
pub struct ToggleMinimizeEvent {
    pub id: egui::Id,
}

fn handle_toggle_minimize_event(
    event: On<ToggleMinimizeEvent>,
    mut open_windows: ResMut<OpenWindows>,
) {
    if let Some(entry) = open_windows.windows.iter_mut().find(|w| w.id == event.id) {
        entry.is_minimized = !entry.is_minimized;
    }
}

/// Triggered to close a specific window.
/// `id` must match a [`WindowEntry::id`] in [`OpenWindows`].
#[derive(Event, Debug, Clone)]
pub struct CloseWindowEvent {
    pub id: egui::Id,
}

fn handle_close_window_event(event: On<CloseWindowEvent>, mut open_windows: ResMut<OpenWindows>) {
    if let Some(entry) = open_windows.windows.iter_mut().find(|w| w.id == event.id) {
        entry.is_open = false;
    }
}

pub enum WindowAction {
    None,
    Select(Option<String>),
    /// Navigate this window into a child folder.
    NavigateTo {
        path: FsPath,
    },
    /// Open a non-folder node in a new window.
    OpenNode(OpenAppEvent),
    /// Go up one directory level using path.parent()
    GoBack,
    UnlockAttempt {
        path: FsPath,
    },
    DecryptComplete {
        path: FsPath,
    },
    RipperComplete(Option<ScriptedEventTrigger>),
}
