use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::{
    engine::{
        Pause, UiPassSystems,
        design_scale::DesignScale,
        file_system::FsHierarchy,
        minigames::{MinigameTrigger, MinigameType},
        screens::{
            Screen,
            desktop::{DesktopTextures, IconTextures},
        },
        scripted_events::{Dialogues, ScriptedEventTrigger, UnlockState},
        system_apps::{Applications, OpenAppEvent},
        terminal_commands::{CommandOutput, execute_command},
        window_manager::{OpenWindows, ToggleMinimizeEvent, WindowAction},
    },
    ui::{
        apps::{
            chatbox::{commit_player_line, show_chatbox},
            decrypter::show_encrypted,
            file_explorer::show_file_explorer,
            image_viewer::show_image_viewer,
            ripper::show_netripper_transmit,
            terminal::show_terminal,
            text_viewer::show_text_viewer,
            unlocker::show_unlocker,
        },
        theme::widgets::title_bar::{TitleBarAction, title_bar},
    },
};

mod chatbox;
mod decrypter;
mod file_explorer;
mod image_viewer;
mod ripper;
pub mod settings_menu;
mod terminal;
mod text_viewer;
mod unlocker;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        (show_open_windows)
            .run_if(
                in_state(Screen::Desktop)
                    .and(resource_exists::<FsHierarchy>)
                    .and(resource_exists::<Dialogues>)
                    .and(resource_exists::<IconTextures>),
            )
            .in_set(UiPassSystems::Render),
    );
}

const WINDOW_DESIGN_W: f32 = 1040.0;
const WINDOW_DESIGN_H: f32 = 718.0;
const WINDOW_PAD_X: f32 = 6.0;
const WINDOW_PAD_BOT: f32 = 6.0;

fn show_open_windows(
    mut cmd: Commands,
    mut contexts: EguiContexts,
    mut open_windows: ResMut<OpenWindows>,
    mut vfs: ResMut<FsHierarchy>,
    mut dialogues: ResMut<Dialogues>,
    icons: Res<IconTextures>,
    tex: Res<DesktopTextures>,
    scale: Res<DesignScale>,
    time: Res<Time>,
    paused: Res<State<Pause>>,
    unlock_state: Res<UnlockState>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let top_layer_window_id = ctx.memory(|mem| {
        mem.layer_ids()
            .filter(|layer| layer.order == egui::Order::Middle)
            .last()
            .map(|layer| layer.id)
    });
    let window_frame = egui::Frame::NONE;
    let window_size = scale.px(WINDOW_DESIGN_W, WINDOW_DESIGN_H);
    let dt = time.delta_secs();
    //let paused = active_mg.checkpoint.is_some();
    let mut actions: Vec<(egui::Id, WindowAction)> = Vec::new();
    for entry in open_windows.windows.iter_mut() {
        if entry.is_minimized {
            continue;
        }
        let is_focused = Some(entry.id) == top_layer_window_id;
        egui::Window::new(&entry.event.name)
            .id(entry.id)
            .resizable(false)
            .fade_in(true)
            .frame(window_frame)
            .collapsible(false)
            .fixed_size(window_size)
            .title_bar(false)
            .show(ctx, |ui| {
                let bg_rect = ui.max_rect();
                ui.painter().image(
                    tex.window,
                    bg_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                ui.style_mut().interaction.selectable_labels = false;
                match title_bar(ui, &entry.event.name, &scale) {
                    TitleBarAction::Minimize => {
                        // Will be picked up next frame by the observer
                        cmd.trigger(ToggleMinimizeEvent { id: entry.id });
                    }
                    TitleBarAction::Close => {
                        entry.is_open = false;
                    }
                    TitleBarAction::None => {}
                }
                egui::Frame::new()
                    .inner_margin(egui::Margin {
                        left: (scale.x * WINDOW_PAD_X) as i8,
                        right: (scale.x * WINDOW_PAD_X) as i8,
                        top: 0,
                        bottom: scale.py(WINDOW_PAD_BOT) as i8,
                    })
                    .show(ui, |ui| {
                        let action = match &mut entry.event.app_type {
                            Applications::TextViewer { content } => {
                                show_text_viewer(ui, content);
                                WindowAction::None
                            }
                            Applications::FileExplorer {
                                path,
                                selected_item,
                            } => show_file_explorer(ui, path, selected_item, &vfs, &icons, &scale),
                            Applications::ImageViewer { texture_id, size } => {
                                show_image_viewer(ui, *texture_id, *size, &scale);
                                WindowAction::None
                            }
                            Applications::Unlocker { path, input } => {
                                if show_unlocker(ui, input) {
                                    WindowAction::UnlockAttempt { path: path.clone() } // is this alright?
                                } else {
                                    WindowAction::None
                                }
                            }
                            Applications::Terminal {
                                cwd,
                                history,
                                input,
                            } => {
                                ui.painter().image(
                                    tex.terminal,
                                    ui.max_rect(),
                                    egui::Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    egui::Color32::WHITE,
                                );
                                if let Some(cmd_str) =
                                    show_terminal(ui, cwd, history, input, &scale, is_focused)
                                {
                                    history.push(format!("> {}", cmd_str));
                                    let cmd_output = execute_command(
                                        &cmd_str,
                                        cwd,
                                        history,
                                        &mut *vfs,
                                        &unlock_state,
                                    );
                                    match cmd_output {
                                        CommandOutput::OpenApp(event) => {
                                            cmd.trigger(event);
                                        }
                                        CommandOutput::None => {}
                                    }
                                    input.clear();
                                }
                                WindowAction::None
                            }
                            Applications::Chatbox {
                                input,
                                state,
                                displayed,
                            } => {
                                ui.painter().image(
                                    tex.chatbox,
                                    ui.max_rect(),
                                    egui::Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    egui::Color32::WHITE,
                                );
                                let send = show_chatbox(
                                    ui,
                                    displayed,
                                    state,
                                    input,
                                    &mut dialogues,
                                    dt,
                                    &scale,
                                    is_focused,
                                );
                                if send {
                                    commit_player_line(displayed, input, state, &mut dialogues);
                                }
                                WindowAction::None
                            }
                            Applications::Decrypter {
                                path,
                                max_tries: _, // check later
                                elapsed,
                                minigames_triggered,
                            } => {
                                let output =
                                    show_encrypted(ui, elapsed, minigames_triggered, dt, paused.0);
                                if let Some(idx) = output.triggered_checkpoint {
                                    cmd.trigger(MinigameTrigger {
                                        checkpoint: idx,
                                        game_type: MinigameType::BruteForce,
                                    });
                                }

                                if output.complete {
                                    WindowAction::DecryptComplete { path: path.clone() }
                                } else {
                                    WindowAction::None
                                }
                            }
                            Applications::Ripper {
                                path,
                                elapsed,
                                minigames_triggered,
                                on_complete,
                                ..
                            } => {
                                let target =
                                    path.as_ref().map(|p| p.file_name()).unwrap_or("UNKNOWN");
                                let output = show_netripper_transmit(
                                    ui,
                                    target,
                                    elapsed,
                                    minigames_triggered,
                                    dt,
                                    paused.0,
                                );
                                if let Some(idx) = output.triggered_checkpoint {
                                    cmd.trigger(MinigameTrigger {
                                        checkpoint: idx,
                                        game_type: MinigameType::NetRipper,
                                    });
                                }
                                if output.complete {
                                    WindowAction::RipperComplete(on_complete.clone())
                                } else {
                                    WindowAction::None
                                }
                            }
                        };
                        actions.push((entry.id, action));
                    });
            });
    }
    for (id, action) in actions {
        let Some(entry) = open_windows.windows.iter_mut().find(|w| w.id == id) else {
            continue; // window was already closed this frame — skip gracefully
        };
        match action {
            WindowAction::None => {}

            WindowAction::NavigateTo { path } => {
                if let Applications::FileExplorer {
                    path: current,
                    selected_item,
                } = &mut entry.event.app_type
                {
                    entry.event.name = path.file_name().to_string();
                    *current = path;
                    *selected_item = None;
                }
            }

            WindowAction::GoBack => {
                if let Applications::FileExplorer { path, .. } = &mut entry.event.app_type {
                    if let Some(parent) = path.parent() {
                        entry.event.name = parent.file_name().to_string();
                        *path = parent;
                    }
                }
            }

            WindowAction::OpenNode(event) => {
                cmd.trigger(event);
            }
            WindowAction::UnlockAttempt { path } => {
                let password =
                    if let Applications::Unlocker { path: _, input } = &entry.event.app_type {
                        input.clone()
                    } else {
                        String::new()
                    };

                if vfs.unlock_with_password(&path, &password).is_ok() {
                    if let Some(node) = vfs.get_node(&path) {
                        let fresh_event = OpenAppEvent::from_fsnode(node, path);
                        entry.event.app_type = fresh_event.app_type;
                    }
                } else {
                    if let Applications::Unlocker { path: _, input } = &mut entry.event.app_type {
                        input.clear();
                    }
                    debug!("Failed to unlock node at {}", path);
                }
            }
            WindowAction::DecryptComplete { path } => {
                if vfs.crack_encrypted(&path).is_ok() {
                    if let Some(node) = vfs.get_node(&path) {
                        let fresh_event = OpenAppEvent::from_fsnode(node, path);
                        entry.event.app_type = fresh_event.app_type;
                    }
                } else {
                    debug!("Failed to crack encrypted node at {}", path);
                }
            }
            WindowAction::RipperComplete(event) => match event {
                Some(ScriptedEventTrigger::RipperFailed) => {
                    cmd.trigger(ScriptedEventTrigger::RipperFailed);
                    entry.is_open = false;
                }
                other => {
                    if let Some(ev) = other {
                        cmd.trigger(ev);
                    }
                    entry.is_open = false;
                }
            },
            WindowAction::Select(new_selection) => {
                if let Applications::FileExplorer { selected_item, .. } = &mut entry.event.app_type
                {
                    *selected_item = new_selection;
                }
            }
        }
    }

    // Prune windows closed via the × button
    open_windows.windows.retain(|w| w.is_open);
    Ok(())
}
