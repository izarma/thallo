use bevy::ecs::event::Event;
use bevy_egui::egui::{TextureId, Vec2};

use crate::engine::{
    file_system::{FileType, FsNode, FsPath, LockType},
    scripted_events::DialogueLine,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Applications {
    FileExplorer {
        path: FsPath,
        selected_item: Option<String>,
    },
    TextViewer {
        content: String,
    },
    ImageViewer {
        texture_id: TextureId,
        size: Vec2,
    },
    Unlocker {
        path: FsPath,
        input: String,
    },
    Decrypter {
        path: FsPath,
        max_tries: Option<u8>,
        /// Seconds elapsed in the decryption animation (0.0 → DECRYPT_DURATION).
        elapsed: f32,
        /// Bitmask — bit N is set once minigame checkpoint N has been triggered.
        minigames_triggered: u8,
    },
    Terminal {
        // Current Working Directory
        cwd: FsPath,
        history: Vec<String>,
        input: String,
    },
    Chatbox {
        input: String,
        state: ChatBoxState,
        displayed: Vec<DialogueLine>, // need to understand this better
    },
}

impl Applications {
    pub fn type_name(&self) -> &'static str {
        match self {
            Applications::FileExplorer { .. } => "Explorer",
            Applications::TextViewer { .. } => "Text Viewer",
            Applications::ImageViewer { .. } => "Image Viewer",
            Applications::Unlocker { .. } => "Locked",
            Applications::Decrypter { .. } => "Encrypted",
            Applications::Terminal { .. } => "Terminal",
            Applications::Chatbox { .. } => "Chatbox",
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct OpenAppEvent {
    pub name: String,
    pub app_type: Applications,
}

impl OpenAppEvent {
    pub fn from_fsnode(node: &FsNode, path: FsPath) -> Self {
        // why does this function need node and path seperately?
        let app_type = match &node.meta.locked {
            Some(LockType::Password(_)) => Applications::Unlocker {
                path: path.clone(),
                input: String::new(),
            },
            Some(LockType::Encrypted { .. }) => Applications::Decrypter {
                path: path.clone(),
                max_tries: None,
                elapsed: 0.0,
                minigames_triggered: 0,
            },
            None => match &node.file_type {
                FileType::TextFile(content) => Applications::TextViewer {
                    content: content.clone(),
                },
                FileType::Folder(_) => Applications::FileExplorer {
                    path: path.clone(),
                    selected_item: None,
                },
                FileType::Image(tex, size) => Applications::ImageViewer {
                    texture_id: *tex,
                    size: *size,
                },
            },
        };
        Self {
            name: node.name.clone(),
            app_type,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChatBoxState {
    AnonTyping { elapsed: f32 },
    PlayerReady { chars_revealed: usize },
    Done,
}

#[derive(Debug, Clone)]
pub enum SystemAlerts {
    FileTransfer(String),
    EncryptedError,
}

#[derive(Event)]
pub struct OpenAlertEvent {
    pub name: String,
    pub alert: SystemAlerts,
}

impl OpenAlertEvent {
    //do we have a from fs node as well?
    pub fn from_scripted_event(name: String, alert: SystemAlerts) -> Self {
        println!("{}", name);
        Self { name, alert }
    }
}
