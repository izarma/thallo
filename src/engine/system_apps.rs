use bevy::ecs::event::Event;
use bevy_egui::egui::{TextureId, Vec2};

use crate::engine::file_system::{FileType, FsNode, FsPath, LockType};

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
    },
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
                    texture_id: tex.clone(),
                    size: size.clone(),
                },
            },
        };
        Self {
            name: node.name.clone(),
            app_type,
        }
    }
}
