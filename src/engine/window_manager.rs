use bevy::prelude::*;
use bevy_egui::egui;

use crate::engine::{
    file_system::FsPath,
    system_apps::{Applications, OpenAppEvent},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OpenWindows>();
    app.add_observer(handle_open_node_events);
    app.add_observer(handle_toggle_minimize_event);
}

#[derive(Resource, Default)]
pub struct OpenWindows {
    pub windows: Vec<WindowEntry>,
}

impl OpenWindows {
    /// Open a window, or bring it to focus if already open.
    fn open(&mut self, event: OpenAppEvent) {
        // Singleton apps: if already open (even minimized), un-minimize and return.
        let is_singleton = matches!(event.app_type, Applications::Chatbox { .. });
        if is_singleton {
            if let Some(existing) = self
                .windows
                .iter_mut()
                .find(|w| matches!(w.event.app_type, Applications::Chatbox { .. }))
            {
                existing.is_minimized = false; // bring it back up
                return;
            }
        }
        self.windows.push(WindowEntry::new(event));
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

fn handle_open_node_events(node: On<OpenAppEvent>, mut open_windows: ResMut<OpenWindows>) {
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
}
