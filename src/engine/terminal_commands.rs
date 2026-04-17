use crate::engine::{
    file_system::{FileType, FsHierarchy, FsPath, HOME_PATH},
    system_apps::OpenAppEvent,
};

pub fn execute_command(
    raw: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &FsHierarchy,
) -> Option<OpenAppEvent> {
    let mut parts = raw.trim().splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();

    match cmd {
        "ls" => {
            let target = if arg.is_empty() {
                cwd.clone()
            } else {
                cwd.join(arg)
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
            None
        }

        "cd" => {
            let target = if arg.is_empty() || arg == "~" {
                FsPath::new(HOME_PATH)
            } else if arg == ".." {
                cwd.parent().unwrap_or_else(|| cwd.clone())
            } else {
                cwd.join(arg)
            };
            match vfs.get_node(&target) {
                Some(node)
                    if matches!(node.file_type, FileType::Folder(_)) && node.is_accessible() =>
                {
                    *cwd = target;
                }
                Some(node) if !node.is_accessible() => {
                    history.push(format!("cd: {}: Permission denied", arg));
                }
                Some(_) => history.push(format!("cd: {}: Not a directory", arg)),
                None => history.push(format!("cd: {}: No such file or directory", arg)),
            }
            None
        }
        "help" => {
            history.push("Available commands:".into());
            history.push("".into());
            for cmd in COMMANDS {
                let entry = if cmd.args.is_empty() {
                    format!("  {:<10}  {}", cmd.name, cmd.desc)
                } else {
                    format!(
                        "  {:<10}  {}",
                        format!("{} {}", cmd.name, cmd.args),
                        cmd.desc
                    )
                };
                history.push(entry);
            }
            None
        }
        "pwd" => {
            history.push(cwd.to_string());
            None
        }
        "clear" => {
            history.clear();
            None
        }
        "cat" => {
            let target = cwd.join(arg);
            match vfs.get_node(&target) {
                Some(node) if !node.is_accessible() => {
                    history.push(format!("cat: {}: Permission denied", arg));
                }
                Some(node) => match node.read_text() {
                    Ok(text) => {
                        for line in text.lines() {
                            history.push(line.to_string());
                        }
                    }
                    Err(e) => history.push(format!("cat: {}: {}", arg, e)),
                },
                None => history.push(format!("cat: {}: No such file or directory", arg)),
            }
            None
        }
        "open" => {
            if arg.is_empty() {
                history.push("open: missing operand".into());
                return None;
            }
            let target = cwd.join(arg);
            match vfs.get_node(&target) {
                Some(node) if !node.is_accessible() => {
                    history.push(format!("open: {}: Permission denied", arg));
                    None
                }
                Some(node) => {
                    // Folders just cd into them
                    if matches!(node.file_type, FileType::Folder(_)) {
                        *cwd = target;
                        None
                    } else {
                        Some(OpenAppEvent::from_fsnode(node, target))
                    }
                }
                None => {
                    history.push(format!("open: {}: No such file or directory", arg));
                    None
                }
            }
        }

        "" => None,
        other => {
            history.push(format!("st-os: command not found: {}", other));
            // fuzzy-ish hint — find any command that starts with what they typed
            if let Some(closest) = COMMANDS.iter().find(|c| c.name.starts_with(&other[..1])) {
                history.push(format!("  did you mean: {}?", closest.name));
            }
            None
        }
    }
}

struct CommandDef {
    name: &'static str,
    args: &'static str,
    desc: &'static str,
}

const COMMANDS: &[CommandDef] = &[
    CommandDef {
        name: "ls",
        args: "",
        desc: "list contents of current or given directory",
    },
    CommandDef {
        name: "cd",
        args: "<dir>",
        desc: "change directory (.. to go up, ~ for home)",
    },
    CommandDef {
        name: "pwd",
        args: "",
        desc: "print current directory path",
    },
    CommandDef {
        name: "cat",
        args: "<file>",
        desc: "print contents of a text file",
    },
    CommandDef {
        name: "open",
        args: "<file>",
        desc: "open a file or folder in a new window",
    },
    CommandDef {
        name: "clear",
        args: "",
        desc: "clear terminal history",
    },
    CommandDef {
        name: "help",
        args: "",
        desc: "show this message",
    },
];
