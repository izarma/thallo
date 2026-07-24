use bevy_egui::egui::{self};

use crate::{
    engine::{
        design_scale::DesignScale,
        file_system::{FileType, FsHierarchy, FsPath, LockType},
        screens::desktop::IconTextures,
        system_apps::{Applications, OpenAppEvent},
        window_manager::WindowAction,
    },
    ui::theme::widgets::{
        address_bar::address_bar,
        icon_grid::{
            ICON_DESIGN_SIZE, IconGridAction, IconGridItem, icon_for_filetype, show_icon_grid,
        },
        primitives::empty_state,
    },
};

const EXPLORER_COLS: usize = 6;

pub fn show_file_explorer(
    ui: &mut egui::Ui,
    path: &FsPath,
    selected_item: &Option<String>,
    vfs: &FsHierarchy,
    icons: &IconTextures,
    scale: &DesignScale,
) -> WindowAction {
    // Address bar
    let mut action = WindowAction::None;
    let can_go_back = path.parent().is_some();
    if address_bar(ui, path, can_go_back, scale) {
        return WindowAction::GoBack;
    }
    ui.separator();

    // Right-click context menu on the explorer background.
    let bg_rect = ui.available_rect_before_wrap();
    let bg_response = ui.interact(bg_rect, ui.id().with("explorer_bg"), egui::Sense::click());
    bg_response.context_menu(|ui| {
        if ui.button("Open Terminal Here").clicked() {
            action = WindowAction::OpenNode(OpenAppEvent {
                name: "Terminal".to_string(),
                app_type: Applications::Terminal {
                    cwd: path.clone(),
                    history: Vec::new(),
                    input: String::new(),
                },
            });
            ui.close();
        }
    });

    let Some(node) = vfs.get_node(path) else {
        empty_state(ui, "Folder not found.");
        return action;
    };
    let children: Vec<_> = match &node.file_type {
        FileType::Folder(children) => children.iter().collect(),
        _ => {
            empty_state(ui, "Not a folder.");
            return WindowAction::None;
        }
    };
    if children.is_empty() {
        empty_state(ui, "(empty)");
        return action;
    }

    let mut sorted = children.clone();
    sorted.sort_by_key(|c| !matches!(c.file_type, FileType::Folder(_)));

    let grid_items: Vec<IconGridItem> = sorted
        .iter()
        .map(|c| IconGridItem {
            id: path.join(&c.name).to_string(),
            label: c.name.clone(),
            icon: icon_for_filetype(&c.file_type, icons, c.meta.locked.is_some()),
            is_folder: matches!(c.file_type, FileType::Folder(_)),
            is_locked: c.meta.locked.is_some(),
            is_encrypted: c.meta.locked == Some(LockType::Encrypted),
        })
        .collect();

    let icon_size = scale.uniform() * ICON_DESIGN_SIZE;

    match show_icon_grid(
        ui,
        &format!("explorer_{}", path),
        &grid_items,
        selected_item,
        EXPLORER_COLS,
        icon_size,
    ) {
        IconGridAction::Selected(id) => {
            return WindowAction::Select(if id.is_empty() { None } else { Some(id) });
        }
        IconGridAction::Opened(id) => {
            if let Some(child) = sorted.iter().find(|c| path.join(&c.name).to_string() == id) {
                let child_path = path.join(&child.name);
                let is_folder = matches!(child.file_type, FileType::Folder(_));
                let is_locked = child.meta.locked.is_some();
                if is_folder && !is_locked {
                    action = WindowAction::NavigateTo { path: child_path };
                } else {
                    action = WindowAction::OpenNode(OpenAppEvent::from_fsnode(child, child_path));
                }
            }
        }
        IconGridAction::OpenTerminal(id) => {
            if let Some(child) = sorted.iter().find(|c| path.join(&c.name).to_string() == id) {
                let child_path = path.join(&child.name);
                action = WindowAction::OpenNode(OpenAppEvent {
                    name: "Terminal".to_string(),
                    app_type: Applications::Terminal {
                        cwd: child_path,
                        history: Vec::new(),
                        input: String::new(),
                    },
                });
            }
        }
        IconGridAction::None => {}
    }
    action
}
