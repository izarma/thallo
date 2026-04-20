use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    engine::{
        UiPassSystems,
        design_scale::DesignScale,
        file_system::{DESKTOP_PATH, FileType, FsHierarchy, FsPath, HOME_PATH},
        screens::{
            Screen,
            desktop::{DesktopAssets, DesktopTextures, IconTextures},
        },
        system_apps::{Applications, OpenAppEvent},
        window_manager::{OpenWindows, ToggleMinimizeEvent},
    },
    ui::{
        apps::settings_menu::show_settings_window,
        theme::widgets::{
            icon_grid::{
                ICON_DESIGN_SIZE, IconGridAction, IconGridItem, icon_for_filetype, show_icon_grid,
            },
            task_bar::{
                GroupedWindow, START_DESIGN_W, TAB_DESIGN_H, TAB_DESIGN_W, TASKBAR_DESIGN_H,
                menu_item, taskbar_app_button, taskbar_group_button,
            },
        },
    },
};

const DESKTOP_COLS: usize = 6;
const MAX_UNGROUPED_WINDOWS: usize = 7;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<TaskBarState>().add_systems(
        EguiPrimaryContextPass,
        (show_desktop, show_task_bar, show_settings_egui_window)
            .chain()
            .run_if(
                in_state(Screen::Desktop)
                    .and(resource_exists::<FsHierarchy>)
                    .and(resource_exists::<DesktopAssets>),
            )
            .in_set(UiPassSystems::Render),
    );
}

fn show_desktop(
    mut contexts: EguiContexts,
    icons: Res<IconTextures>,
    desktop_tex: Res<DesktopTextures>,
    vfs: Res<FsHierarchy>,
    scale: Res<DesignScale>,
    mut cache: Local<Vec<DesktopItem>>,
    mut selected: Local<Option<String>>,
    mut cmd: Commands,
) -> Result {
    // Only build items if vfs has changed
    if vfs.is_changed() || cache.is_empty() {
        *cache = build_desktop_items(&vfs);
    }
    let grid_items: Vec<IconGridItem> = cache
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
                    if let Some(event) = cache.iter().find(|e| e.event.name == id) {
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

struct DesktopItem {
    event: OpenAppEvent,
    file_type: FileType,
    is_locked: bool,
}

#[derive(Resource, Default)]
struct TaskBarState {
    pub settings_open: bool,
}

fn show_task_bar(
    mut contexts: EguiContexts,
    open_windows: Res<OpenWindows>,
    desktop_tex: Option<Res<DesktopTextures>>,
    scale: Res<DesignScale>,
    mut cmd: Commands,
    mut state: ResMut<TaskBarState>,
    mut app_exit: MessageWriter<AppExit>,
) -> Result {
    let bar_h = scale.py(TASKBAR_DESIGN_H);
    let start_size = scale.px(START_DESIGN_W, TASKBAR_DESIGN_H);
    let tab_size = scale.px(TAB_DESIGN_W, TAB_DESIGN_H);

    egui::TopBottomPanel::bottom("Task Bar")
        .exact_height(bar_h)
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120))
                .inner_margin(egui::Margin::symmetric(0, 0)),
        )
        .show_separator_line(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let start_clicked = if let Some(dt) = desktop_tex.as_deref() {
                    let sized = egui::load::SizedTexture::new(dt.start, start_size);
                    ui.add(egui::Button::image(sized).frame(false))
                } else {
                    ui.button("Start") // graceful fallback while asset loads
                };

                // start menu logic
                egui::Popup::menu(&start_clicked).show(|ui| {
                    ui.set_min_width(200.0);
                    ui.heading("Start Menu");
                    if menu_item(ui, "Terminal").clicked() {
                        cmd.trigger(OpenAppEvent {
                            name: "Terminal".to_string(),
                            app_type: Applications::Terminal {
                                cwd: FsPath::new("Home"),
                                history: Vec::new(),
                                input: String::new(),
                            },
                        });
                    }
                    if menu_item(ui, "File Explorer").clicked() {
                        cmd.trigger(OpenAppEvent {
                            name: HOME_PATH.to_string(),
                            app_type: Applications::FileExplorer {
                                path: FsPath::new(HOME_PATH),
                                selected_item: None,
                            },
                        });
                    }
                    if menu_item(ui, "Settings").clicked() {
                        state.settings_open = true;
                    }
                    if menu_item(ui, "Shut Down").clicked() {
                        app_exit.write(AppExit::Success);
                    }
                });
                let total = open_windows.windows.len();
                if total > MAX_UNGROUPED_WINDOWS {
                    let mut groups: Vec<(
                        &'static str,
                        Vec<GroupedWindow>,
                        bool,
                        Option<egui::TextureId>,
                    )> = Vec::new();

                    for entry in open_windows.windows.iter() {
                        let key = entry.event.app_type.type_name();
                        if let Some(g) = groups.iter_mut().find(|g| g.0 == key) {
                            if !entry.is_minimized {
                                g.2 = true; // at least one visible → group is active
                            }
                            g.1.push(GroupedWindow {
                                id: entry.id,
                                name: entry.event.name.clone(),
                                is_minimized: entry.is_minimized,
                            });
                        } else {
                            let icon = desktop_tex
                                .as_deref()
                                .map(|dt| dt.icon_for_app(&entry.event.app_type));
                            groups.push((
                                key,
                                vec![GroupedWindow {
                                    id: entry.id,
                                    name: entry.event.name.clone(),
                                    is_minimized: entry.is_minimized,
                                }],
                                !entry.is_minimized,
                                icon,
                            ));
                        }
                    }

                    for (key, windows, is_active, icon) in &groups {
                        if windows.len() == 1 {
                            // Single window — render as a normal tab using its actual name
                            if taskbar_app_button(ui, &windows[0].name, *is_active, *icon, tab_size)
                                .clicked()
                            {
                                cmd.trigger(ToggleMinimizeEvent { id: windows[0].id });
                            }
                        } else {
                            // Multiple windows of the same type — render grouped with popup
                            if let Some(window_id) =
                                taskbar_group_button(ui, *key, *is_active, *icon, tab_size, windows)
                            {
                                cmd.trigger(ToggleMinimizeEvent { id: window_id });
                            }
                        }
                    }
                } else {
                    for entry in open_windows.windows.iter() {
                        let is_active = !entry.is_minimized;
                        let icon = desktop_tex
                            .as_deref()
                            .map(|dt| dt.icon_for_app(&entry.event.app_type));
                        if taskbar_app_button(ui, &entry.event.name, is_active, icon, tab_size)
                            .clicked()
                        {
                            cmd.trigger(ToggleMinimizeEvent { id: entry.id });
                        }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(6.0);

                    // Clock label
                    let now = chrono::Local::now();
                    let time_str = now.format("%H:%M").to_string();
                    ui.label(
                        egui::RichText::new(time_str)
                            .size((13.0 * (tab_size.y / TASKBAR_DESIGN_H)).max(8.0))
                            .color(egui::Color32::WHITE),
                    );

                    ui.add_space(4.0);
                });
            })
        });
    Ok(())
}

fn show_settings_egui_window(
    mut contexts: EguiContexts,
    mut state: ResMut<TaskBarState>,
    mut global_volume: ResMut<GlobalVolume>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) -> Result {
    if !state.settings_open {
        return Ok(());
    }
    let Ok(mut win) = primary_window.single_mut() else {
        return Ok(());
    };
    let mut window_mode = win.mode.clone();
    let mut resolution = win.resolution.clone();
    let mut decorations = win.decorations;

    let mut is_open = state.settings_open;
    egui::Window::new("Settings")
        .open(&mut is_open) // the × button sets this to false
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut()?, |ui| {
            ui.set_min_width(280.0);
            show_settings_window(
                ui,
                &mut global_volume,
                &mut window_mode,
                &mut resolution,
                &mut decorations,
            );
        });

    if win.mode != window_mode {
        win.mode = window_mode;
    }
    if win.resolution != resolution {
        win.resolution = resolution;
    }
    if win.decorations != decorations {
        win.decorations = decorations;
    }

    state.settings_open = is_open; // propagate close from × button
    Ok(())
}
