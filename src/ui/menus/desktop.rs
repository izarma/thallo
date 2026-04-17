use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    engine::{
        UiPassSystems,
        design_scale::DesignScale,
        file_system::{DESKTOP_PATH, FileType, FsHierarchy, FsPath},
        screens::{
            Screen,
            desktop::{DesktopAssets, DesktopTextures, IconTextures},
        },
        system_apps::OpenAppEvent,
    },
    ui::theme::widgets::icon_grid::{
        ICON_DESIGN_SIZE, IconGridAction, IconGridItem, icon_for_filetype, show_icon_grid,
    },
};

const DESKTOP_COLS: usize = 6;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        (show_desktop.run_if(
            in_state(Screen::Desktop)
                .and(resource_exists::<FsHierarchy>)
                .and(resource_exists::<DesktopAssets>),
        ),)
            //.before(task_bar::show_task_bar)
            .in_set(UiPassSystems::Render),
    );
}

fn show_desktop(
    mut contexts: EguiContexts,
    icons: Res<IconTextures>,
    desktop_tex: Res<DesktopTextures>,
    vfs: Res<FsHierarchy>,
    scale: Res<DesignScale>,
    mut cache: Local<DesktopCache>,
    mut selected: Local<Option<String>>,
    mut cmd: Commands,
) -> Result {
    // Only build items if vfs has changed
    if vfs.is_changed() || cache.items.is_empty() {
        cache.items = build_desktop_items(&vfs);
    }
    let grid_items: Vec<IconGridItem> = cache
        .items
        .iter()
        .map(|item| IconGridItem {
            id: item.event.name.clone(),
            label: item.event.name.clone(),
            icon: icon_for_filetype(&item.file_type, &icons, item.is_locked),
        })
        .collect();

    let icon_size = scale.uniform() * ICON_DESIGN_SIZE;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(contexts.ctx_mut()?, |ui| {
            let full_rect = ui.ctx().content_rect();
            ui.ctx().layer_painter(egui::LayerId::background()).image(
                desktop_tex.wallpaper,
                full_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            ui.add_space(40.0);
            match show_icon_grid(
                ui,
                "desktop_icons",
                &grid_items,
                &selected,
                DESKTOP_COLS,
                icon_size,
            ) {
                IconGridAction::Selected(id) => {
                    *selected = if id.is_empty() { None } else { Some(id) };
                }
                IconGridAction::Opened(id) => {
                    if let Some(event) = cache.items.iter().find(|e| e.event.name == id) {
                        cmd.trigger(event.event.clone());
                    }
                }
                IconGridAction::None => {}
            }
        });

    Ok(())
}

fn build_desktop_items(vfs: &FsHierarchy) -> Vec<DesktopItem> {
    let desktop_root = FsPath::new(DESKTOP_PATH);
    vfs.get_node(&desktop_root)
        .and_then(|node| match &node.file_type {
            FileType::Folder(children) => Some(
                children
                    .iter()
                    .map(|c| DesktopItem {
                        event: OpenAppEvent::from_fsnode(c, desktop_root.join(&c.name)),
                        file_type: c.file_type.clone(),
                        is_locked: c.meta.locked.is_some(),
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

#[derive(Default)]
struct DesktopCache {
    items: Vec<DesktopItem>,
}

struct DesktopItem {
    event: OpenAppEvent,
    file_type: FileType,
    is_locked: bool,
}

// fn show_task_bar() -> Result {
//     let bar_h = scale.py(TASKBAR_DESIGN_H);
//     let start_size = scale.px(START_DESIGN_W, TASKBAR_DESIGN_H);
//     let tab_size = scale.px(TAB_DESIGN_W, TAB_DESIGN_H);
//     Ok(())
// }
