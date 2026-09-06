use bevy::ecs::resource::Resource;
use bevy_egui::egui::{TextureId, Vec2};

pub const HOME_PATH: &str = "Home";
pub const DESKTOP_PATH: &str = "Home/Desktop";

/// File System Path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FsPath(String);

impl FsPath {
    /// new `FsPath` from string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// Returns the path as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Returns an iterator over the path items
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/').filter(|s| !s.is_empty())
    }
    /// adds child to path
    pub fn join(&self, name: &str) -> Self {
        Self(format!("{}/{}", self.0.trim_end_matches('/'), name))
    }
    /// Returns the parent path, if it exists.
    pub fn parent(&self) -> Option<Self> {
        let s = self.0.trim_end_matches('/');
        let idx = s.rfind('/')?;
        Some(Self(s[..idx].to_string()))
    }
    /// Returns the file name of the path.
    pub fn file_name(&self) -> &str {
        self.0
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or(&self.0)
    }
}

impl std::fmt::Display for FsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// File System Types
/// FileType Enum can store a Folder (List of Nodes) or the type of the file
#[derive(Debug, Clone)]
pub enum FileType {
    Folder(Vec<FsNode>),
    TextFile(String),
    Image(TextureId, Vec2),
}

/// Each Node has a name & FileType
#[derive(Debug, Clone)]
pub struct FsNode {
    pub name: String,
    pub file_type: FileType,
    pub meta: NodeMeta, // just locked info for now
}

/// VFS Hierarchy Stored as resource - initialized on entering Desktop - stores root node
#[derive(Resource)]
pub struct FsHierarchy {
    pub root: FsNode,
}

/// Node Metadata
#[derive(Debug, Clone, Default)]
pub struct NodeMeta {
    pub locked: Option<LockType>, // None = open, Some = restricted
}

/// For Locked Folders / Files
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    Password(String), // hashed or plaintext for a game is fine
    Encrypted,        // unlocked by game event / specific string
}

#[allow(dead_code)]
impl FsNode {
    pub fn folder(name: &str) -> Self {
        Self {
            name: name.to_string(),
            file_type: FileType::Folder(Vec::new()),
            meta: NodeMeta::default(),
        }
    }
    pub fn text_file(name: &str, content: &str) -> Self {
        Self {
            name: name.to_string(),
            file_type: FileType::TextFile(content.to_string()),
            meta: NodeMeta::default(),
        }
    }
    pub fn img_file(name: &str, img: TextureId, size: Vec2) -> Self {
        Self {
            name: name.to_string(),
            file_type: FileType::Image(img, size),
            meta: NodeMeta::default(),
        }
    }
    pub fn password_folder(name: &str, password: &str) -> Self {
        Self {
            name: name.to_string(),
            file_type: FileType::Folder(Vec::new()),
            meta: NodeMeta {
                locked: Some(LockType::Password(password.to_string())),
            },
        }
    }
    pub fn encrypted_folder(name: &str) -> Self {
        Self {
            name: name.to_string(),
            file_type: FileType::Folder(Vec::new()),
            meta: NodeMeta {
                locked: Some(LockType::Encrypted),
            },
        }
    }

    // Lock Helpers

    /// Returns true if this node can be entered/read freely
    pub fn is_accessible(&self) -> bool {
        self.meta.locked.is_none()
    }
    /// Try to unlock with a password. Returns Ok(()) on success.
    pub fn try_unlock_password(&mut self, input: &str) -> Result<(), FsError> {
        match &self.meta.locked {
            Some(LockType::Password(pw)) if pw.eq_ignore_ascii_case(input) => {
                self.meta.locked = None;
                Ok(())
            }
            Some(LockType::Password(_)) => Err(FsError::WrongPassword),
            Some(LockType::Encrypted) => Err(FsError::IsEncrypted),
            None => Ok(()), // already open
        }
    }
    /// Force-decrypt an encrypted node after the player completes the
    /// decryption progress sequence
    pub fn force_decrypt(&mut self) -> Result<(), FsError> {
        match &self.meta.locked {
            Some(LockType::Encrypted) => {
                self.meta.locked = None;
                Ok(())
            }
            Some(LockType::Password(_)) => Err(FsError::NeedsPassword),
            None => Ok(()),
        }
    }

    // Tree Hierarchy Helpers

    /// Add a child to a Folder node
    pub fn push_child(&mut self, child: FsNode) -> Result<(), FsError> {
        match &mut self.file_type {
            FileType::Folder(children) => {
                children.push(child);
                Ok(())
            }
            _ => Err(FsError::NotAFolder),
        }
    }

    /// Get an immutable child by name (does NOT check locks)
    pub fn get_child(&self, name: &str) -> Option<&FsNode> {
        match &self.file_type {
            FileType::Folder(children) => children.iter().find(|c| c.name == name),
            _ => None,
        }
    }

    /// Get a mutable child by name
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut FsNode> {
        match &mut self.file_type {
            FileType::Folder(children) => children.iter_mut().find(|c| c.name == name),
            _ => None,
        }
    }

    /// List child names (for terminal `ls`)
    pub fn list_children(&self) -> Vec<&str> {
        match &self.file_type {
            FileType::Folder(children) => children.iter().map(|c| c.name.as_str()).collect(),
            _ => vec![],
        }
    }

    /// Read text content (for terminal `cat`)
    pub fn read_text(&self) -> Result<&str, FsError> {
        match &self.file_type {
            FileType::TextFile(content) => Ok(content.as_str()),
            FileType::Folder(_) => Err(FsError::IsAFolder),
            FileType::Image(_, _) => Err(FsError::NotReadable),
        }
    }
}

impl FsHierarchy {
    // Path Traversal

    /// Walk a slash-separated path and return an immutable reference.
    pub fn get_node(&self, path: &FsPath) -> Option<&FsNode> {
        let mut current = &self.root;
        for segment in path.segments() {
            if segment == self.root.name {
                continue;
            }
            current = current.get_child(segment)?;
        }
        Some(current)
    }

    /// Walk a path and return a mutable reference — needed for unlocking nodes
    pub fn get_node_mut(&mut self, path: &FsPath) -> Option<&mut FsNode> {
        let mut current = &mut self.root;
        for segment in path.segments() {
            if segment == current.name {
                continue;
            }
            current = current.get_child_mut(segment)?;
        }
        Some(current)
    }

    /// Unlock a node at a path using a password
    pub fn unlock_with_password(&mut self, path: &FsPath, password: &str) -> Result<(), FsError> {
        self.get_node_mut(path)
            .ok_or(FsError::NotFound)?
            .try_unlock_password(password)
    }

    /// Force-unlock an encrypted node once the player finishes the decryption
    /// progress sequence.
    pub fn crack_encrypted(&mut self, path: &FsPath) -> Result<(), FsError> {
        self.get_node_mut(path)
            .ok_or(FsError::NotFound)?
            .force_decrypt()
    }
}

#[derive(Debug)]
pub enum FsError {
    NotFound,
    NotAFolder,
    IsAFolder,
    NotReadable,
    WrongPassword,
    NeedsPassword,
    IsEncrypted,
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::NotFound => write!(f, "No such file or directory"),
            FsError::NotAFolder => write!(f, "Not a directory"),
            FsError::IsAFolder => write!(f, "Is a directory"),
            FsError::NotReadable => write!(f, "Cannot read binary file as text"),
            FsError::WrongPassword => write!(f, "Incorrect password"),
            FsError::NeedsPassword => write!(f, "This node requires a password"),
            FsError::IsEncrypted => write!(f, "This node requires a decryption key"),
        }
    }
}
