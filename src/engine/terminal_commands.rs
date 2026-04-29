use crate::engine::{
    file_system::{FileType, FsError, FsHierarchy, FsPath, HOME_PATH},
    system_apps::OpenAppEvent,
};

type CommandFn = fn(&str, &mut FsPath, &mut Vec<String>, &mut FsHierarchy) -> Option<OpenAppEvent>;

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
        desc: "list contents of current or given directory",
        handler: cmd_ls,
    },
    CommandDef {
        name: "cd",
        args: "<dir>",
        desc: "change directory (.. to go up, ~ for home)",
        handler: cmd_cd,
    },
    CommandDef {
        name: "open",
        args: "<file>",
        desc: "open a file or folder in a new window",
        handler: cmd_open,
    },
    CommandDef {
        name: "unlock",
        args: "<file> <password>",
        desc: "unlock a password-protected file or folder",
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
    // CommandDef {
    //     name: "pwd",
    //     args: "",
    //     desc: "print current directory path",
    //     handler: cmd_pwd,
    // },
    // CommandDef {
    //     name: "cat",
    //     args: "<file>",
    //     desc: "print contents of a text file",
    //     handler: cmd_cat,
    // },
];

pub fn execute_command(
    raw: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> Option<OpenAppEvent> {
    let mut parts = raw.trim().splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();

    if cmd.is_empty() {
        return None;
    }

    if let Some(def) = COMMANDS.iter().find(|c| c.name == cmd) {
        (def.handler)(arg, cwd, history, vfs)
    } else {
        history.push(format!("st-os: command not found: {}", cmd));
        // fuzzy-ish hint — find any command that starts with what they typed
        if let Some(closest) = COMMANDS.iter().find(|c| c.name.starts_with(&cmd[..1])) {
            history.push(format!("  did you mean: {}?", closest.name));
        }
        None
    }
}

fn cmd_ls(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> Option<OpenAppEvent> {
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

fn cmd_cd(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> Option<OpenAppEvent> {
    let target = if arg.is_empty() || arg == "~" {
        FsPath::new(HOME_PATH)
    } else if arg == ".." {
        cwd.parent().unwrap_or_else(|| cwd.clone())
    } else {
        cwd.join(arg)
    };
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
    None
}

fn cmd_open(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> Option<OpenAppEvent> {
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

fn cmd_clear(
    _arg: &str,
    _cwd: &mut FsPath,
    history: &mut Vec<String>,
    _vfs: &mut FsHierarchy,
) -> Option<OpenAppEvent> {
    history.clear();
    None
}

fn cmd_help(
    _arg: &str,
    _cwd: &mut FsPath,
    history: &mut Vec<String>,
    _vfs: &mut FsHierarchy,
) -> Option<OpenAppEvent> {
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

fn cmd_unlock(
    arg: &str,
    cwd: &mut FsPath,
    history: &mut Vec<String>,
    vfs: &mut FsHierarchy,
) -> Option<OpenAppEvent> {
    let mut parts = arg.splitn(2, ' ');
    let path_str = parts.next().unwrap_or("").trim();
    let password = parts.next().unwrap_or("").trim();

    if path_str.is_empty() || password.is_empty() {
        history.push("usage: unlock <file|dir> <password>".into());
        return None;
    }

    let target = cwd.join(path_str);
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
    None
}

// fn cmd_pwd(
//     _arg: &str,
//     cwd: &mut FsPath,
//     history: &mut Vec<String>,
//     _vfs: &mut FsHierarchy,
// ) -> Option<OpenAppEvent> {
//     history.push(cwd.to_string());
//     None
// }

// fn cmd_cat(
//     arg: &str,
//     cwd: &mut FsPath,
//     history: &mut Vec<String>,
//     vfs: &mut FsHierarchy,
// ) -> Option<OpenAppEvent> {
//     let target = cwd.join(arg);
//     match vfs.get_node(&target) {
//         Some(node) if !node.is_accessible() => {
//             history.push(format!("cat: {}: Permission denied", arg));
//         }
//         Some(node) => match node.read_text() {
//             Ok(text) => {
//                 for line in text.lines() {
//                     history.push(line.to_string());
//                 }
//             }
//             Err(e) => history.push(format!("cat: {}: {}", arg, e)),
//         },
//         None => history.push(format!("cat: {}: No such file or directory", arg)),
//     }
//     None
// }
