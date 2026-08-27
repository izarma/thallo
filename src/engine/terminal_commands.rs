use crate::engine::{
    file_system::{FileType, FsError, FsHierarchy, FsPath, HOME_PATH, LockType},
    scripted_events::{ScriptedEventTrigger, UnlockState},
    system_apps::{Applications, OpenAppEvent},
};

type CommandFn = fn(&str, &mut FsPath, &mut Vec<String>, &mut FsHierarchy) -> CommandOutput;

struct CommandDef {
    name: &'static str,
    args: &'static str,
    desc: &'static str,
    handler: CommandFn,
}

// need to add conditional bruteforce & netripper commands
const COMMANDS: &[CommandDef] = &[
    CommandDef {
        name: "ls",
        args: "",
        desc: "list content of current/given directory",
        handler: cmd_ls,
    },
    CommandDef {
        name: "cd",
        args: "<dir>",
        desc: "change directory",
        handler: cmd_cd,
    },
    CommandDef {
        name: "open",
        args: "<file>",
        desc: "open a file or folder",
        handler: cmd_open,
    },
    CommandDef {
        name: "unlock",
        args: "<file> <password>",
        desc: "unlock a file or folder",
        handler: cmd_unlock,
    },
    CommandDef {
        name: "clear",
        args: "",
        desc: "clear terminal history",
        handler: cmd_clear,
    },
    CommandDef {
        name: "help",
        args: "",
        desc: "show this message",
        handler: cmd_help,
    },
    CommandDef {
        name: "programs",
        args: "",
        desc: "list installed programs",
        handler: |_, _, _, _| CommandOutput::None, // unreachable; handled in execute_command
    },
];

pub enum CommandOutput {
    None,
    OpenApp(OpenAppEvent),
}

pub fn execute_command(
    raw: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
    state: &UnlockState,
) -> CommandOutput {
    let mut parts = raw.trim().splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();

    if cmd.is_empty() {
        return CommandOutput::None;
    }

    match cmd {
        "bruteforce" if state.programs.bruteforce => return cmd_bruteforce(arg, cwd, history, vfs),
        "bruteforce" => {
            history.push("st-os: command not found: bruteforce".into());
            return CommandOutput::None;
        }
        "netripper" if state.programs.netripper => {
            return cmd_netripper(arg, cwd, history, vfs, state);
        }
        "netripper" => {
            history.push("st-os: command not found: netripper".into());
            return CommandOutput::None;
        }
        "programs" => {
            let mut installed = Vec::new();
            if state.programs.bruteforce {
                installed.push("  bruteforce   decrypt an encrypted file or folder");
            }
            if state.programs.netripper {
                installed.push("  netripper   transmit data through the spacenet");
            }
            if installed.is_empty() {
                history.push("No programs installed.".into());
            } else {
                history.push("Installed programs:".into());
                history.push("".into());
                for line in installed {
                    history.push(line.into());
                }
            }
            return CommandOutput::None;
        }
        _ => {}
    }

    if let Some(def) = COMMANDS.iter().find(|c| c.name == cmd) {
        (def.handler)(arg, cwd, history, vfs)
    } else {
        history.push(format!("st-os: command not found: {}", cmd));
        // fuzzy-ish hint — find any command that starts with what they typed
        if let Some(closest) = COMMANDS.iter().find(|c| c.name.starts_with(&cmd[..1])) {
            history.push(format!("  did you mean: {}?", closest.name));
        }
        CommandOutput::None
    }
}

fn cmd_ls(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> CommandOutput {
    let target = if arg.is_empty() {
        cwd.clone()
    } else {
        resolve_path(arg, cwd)
    };
    match vfs.get_node(&target) {
        Some(node) if node.is_accessible() => {
            let children = node.list_children();
            if children.is_empty() {
                history.push("(empty)".into());
            } else {
                history.push(children.join("  "));
            }
        }
        Some(_) => history.push(format!("ls: {}: Permission denied", target)),
        None => history.push(format!("ls: {}: No such file or directory", target)),
    }
    CommandOutput::None
}

/// Resolve a path argument, expanding `~` and handling `..` / `.`.
fn resolve_path(arg: &str, cwd: &FsPath) -> FsPath {
    if arg.is_empty() || arg == "~" {
        return FsPath::new(HOME_PATH);
    }

    // Expand leading "~/" into the home directory.
    let arg = if let Some(rest) = arg.strip_prefix("~/") {
        format!("{}/{}", HOME_PATH, rest)
    } else {
        arg.to_string()
    };

    // Normalize a leading "/" to mean "under HOME_PATH".
    let normalized = if arg.starts_with('/') {
        let without_slash = arg.trim_start_matches('/');
        if without_slash.is_empty() || without_slash == HOME_PATH {
            HOME_PATH.to_string()
        } else if without_slash.starts_with(&format!("{}/", HOME_PATH)) {
            without_slash.to_string()
        } else {
            format!("{}/{}", HOME_PATH, without_slash)
        }
    } else {
        arg
    };

    // Start from the current directory for relative paths, or from root for
    // absolute ones ("/Home/..." or "Home/...").
    let is_absolute = normalized == HOME_PATH || normalized.starts_with(&format!("{}/", HOME_PATH));
    let mut components: Vec<&str> = if is_absolute {
        Vec::new()
    } else {
        cwd.segments().collect()
    };

    for component in normalized.split('/') {
        match component {
            "" | "." => continue,
            ".." => {
                components.pop();
            }
            other => components.push(other),
        }
    }

    let result = if components.is_empty() {
        HOME_PATH.to_string()
    } else {
        components.join("/")
    };

    // Keep cwd in canonical form: every path lives under HOME_PATH.
    if result == HOME_PATH || result.starts_with(&format!("{}/", HOME_PATH)) {
        FsPath::new(result)
    } else {
        FsPath::new(format!("{}/{}", HOME_PATH, result))
    }
}

fn cmd_cd(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> CommandOutput {
    let target = resolve_path(arg, cwd);
    match vfs.get_node(&target) {
        Some(node) if matches!(node.file_type, FileType::Folder(_)) && node.is_accessible() => {
            *cwd = target;
        }
        Some(node) if !node.is_accessible() => {
            history.push(format!("cd: {}: Permission denied", arg));
        }
        Some(_) => history.push(format!("cd: {}: Not a directory", arg)),
        None => history.push(format!("cd: {}: No such file or directory", arg)),
    }
    CommandOutput::None
}

fn cmd_open(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> CommandOutput {
    if arg.is_empty() {
        history.push("open: missing operand".into());
        return CommandOutput::None;
    }
    let target = resolve_path(arg, cwd);
    match vfs.get_node(&target) {
        Some(node) if !node.is_accessible() => {
            history.push(format!("open: {}: Permission denied", arg));
            CommandOutput::None
        }
        Some(node) => {
            if matches!(node.file_type, FileType::Folder(_)) {
                *cwd = target;
                CommandOutput::None
            } else {
                CommandOutput::OpenApp(OpenAppEvent::from_fsnode(node, target))
            }
        }
        None => {
            history.push(format!("open: {}: No such file or directory", arg));
            CommandOutput::None
        }
    }
}

fn cmd_clear(
    _arg: &str,
    _cwd: &mut FsPath,
    history: &mut Vec<String>,
    _vfs: &mut FsHierarchy,
) -> CommandOutput {
    history.clear();
    CommandOutput::None
}

/// Wrap `text` into lines no longer than `max_width` characters.
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let separator_len = usize::from(!current.is_empty());
        if current.len() + separator_len + word.len() > max_width {
            if !current.is_empty() {
                lines.push(current);
            }
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn cmd_help(
    _arg: &str,
    _cwd: &mut FsPath,
    history: &mut Vec<String>,
    _vfs: &mut FsHierarchy,
) -> CommandOutput {
    history.push("Available commands:".into());
    history.push("".into());

    // Keep wrapped help lines inside the terminal frame
    // (matches the "┌─ ST-OS TERMINAL ─...┐" header width).
    const TOTAL_WIDTH: usize = 54;

    let max_usage_width = COMMANDS
        .iter()
        .map(|cmd| {
            if cmd.args.is_empty() {
                cmd.name.len()
            } else {
                cmd.name.len() + 1 + cmd.args.len()
            }
        })
        .max()
        .unwrap_or(0);

    let continuation_indent = format!("  {:<width$}  ", "", width = max_usage_width);
    let desc_width = TOTAL_WIDTH.saturating_sub(continuation_indent.len());

    for cmd in COMMANDS {
        let usage = if cmd.args.is_empty() {
            cmd.name.to_string()
        } else {
            format!("{} {}", cmd.name, cmd.args)
        };
        let wrapped = wrap_text(cmd.desc, desc_width);
        for (i, line) in wrapped.into_iter().enumerate() {
            if i == 0 {
                history.push(format!(
                    "  {:<width$}  {}",
                    usage,
                    line,
                    width = max_usage_width
                ));
            } else {
                history.push(format!("{}{}", continuation_indent, line));
            }
        }
    }
    CommandOutput::None
}

fn cmd_unlock(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> CommandOutput {
    let mut parts = arg.splitn(2, ' ');
    let path_str = parts.next().unwrap_or("").trim();
    let password = parts.next().unwrap_or("").trim();

    if path_str.is_empty() || password.is_empty() {
        history.push("usage: unlock <file|dir> <password>".into());
        return CommandOutput::None;
    }

    let target = resolve_path(path_str, cwd);
    match vfs.unlock_with_password(&target, password) {
        Ok(()) => {
            history.push(format!("unlock: {} is now unlocked", path_str));
        }
        Err(FsError::WrongPassword) => {
            history.push(format!("unlock: {}: Incorrect password", path_str));
        }
        Err(FsError::IsEncrypted) => {
            history.push(format!(
                "unlock: {}: Not password-protected — file is encrypted",
                path_str
            ));
        }
        Err(FsError::NotFound) => {
            history.push(format!("unlock: {}: No such file or directory", path_str));
        }
        Err(e) => {
            history.push(format!("unlock: {}: {}", path_str, e));
        }
    }
    CommandOutput::None
}

fn cmd_netripper(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
    state: &UnlockState,
) -> CommandOutput {
    if arg.is_empty() {
        history.push("usage: netripper <file|dir|sos>".into());
        return CommandOutput::None;
    }
    if arg.eq_ignore_ascii_case("sos") {
        if !state.story.secure_transmitted {
            history.push("netripper: Invalid Payload".into());
            return CommandOutput::None;
        }
        history.push("netripper: Ripping SOS through spacenet".into());
        return CommandOutput::OpenApp(OpenAppEvent {
            name: "SOS Transmission".to_string(),
            app_type: Applications::Ripper {
                path: None,
                max_tries: Some(3),
                elapsed: 0.0,
                minigames_triggered: 0,
                on_complete: Some(ScriptedEventTrigger::TransmitSOS),
            },
        });
    }
    let target = cwd.join(arg);
    match vfs.get_node(&target) {
        None => {
            history.push(format!("netripper: {}: No such file or directory", arg));
            CommandOutput::None
        }
        Some(node) if !node.is_accessible() => {
            history.push(format!(
                "netripper: {}: Decrypt the file first before transmitting",
                arg
            ));
            CommandOutput::None
        }
        Some(node) => {
            let on_complete = match node.name.as_str() {
                "[SECURE]" => Some(ScriptedEventTrigger::TransmitSecure),
                _ => None,
            };
            history.push(format!("netripper: Ripping {}", arg));
            CommandOutput::OpenApp(OpenAppEvent {
                name: node.name.clone(),
                app_type: Applications::Ripper {
                    path: Some(target),
                    max_tries: Some(3),
                    elapsed: 0.0,
                    minigames_triggered: 0,
                    on_complete,
                },
            })
        }
    }
}

fn cmd_bruteforce(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> CommandOutput {
    if arg.is_empty() {
        history.push("usage: bruteforce <file|dir>".into());
        return CommandOutput::None;
    }

    let target = cwd.join(arg);
    match vfs.get_node(&target) {
        None => {
            history.push(format!("bruteforce: {}: No such file or directory", arg));
            CommandOutput::None
        }
        Some(node) if node.is_accessible() => {
            history.push(format!("bruteforce: {}: File is not encrypted", arg));
            CommandOutput::None
        }
        Some(node) if matches!(node.meta.locked, Some(LockType::Password(_))) => {
            history.push(format!(
                "bruteforce: {}: File is password-protected, not encrypted — use unlock",
                arg
            ));
            CommandOutput::None
        }
        Some(node) => {
            history.push(format!("bruteforce: decrypter starting for {}", arg));
            CommandOutput::OpenApp(OpenAppEvent {
                name: node.name.clone(),
                app_type: Applications::Decrypter {
                    path: target,
                    max_tries: None,
                    elapsed: 0.0,
                    minigames_triggered: 0,
                },
            })
        }
    }
}
