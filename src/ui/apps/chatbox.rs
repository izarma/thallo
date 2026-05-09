use bevy_egui::egui::{
    self, FontId, RichText, ScrollArea, scroll_area::ScrollBarVisibility, style::ScrollStyle,
};

use crate::{
    engine::{
        design_scale::DesignScale,
        dialogue_runner::{DialogueLine, DialogueRunner, Dialogues, derive_state},
        system_apps::ChatBoxState,
    },
    ui::theme::palette::{
        CHATBOX_ANON, CONTENT_FONT_SIZE, DEEP_RED_THEME, FONT_CHAT, FONT_CHAT_SM, LABEL_COLOR,
        LABEL_META, RED_CONTRAST_THEME,
    },
};

// Layout ratios (relative to the content rect below the title bar)

/// How far down the decorative header strip extends — skip past it.
const HEADER_RATIO: f32 = 0.155;
/// Height of the bottom status bar region.
const STATUS_RATIO: f32 = 0.402;
/// Margin around the status.
const STATUS_MARGIN: f32 = 20.0;
/// Width of the right contact-panel sidebar (logo area).
const SIDEBAR_RATIO: f32 = 0.288;

const SEND_BUTTON_HEIGHT: f32 = 48.0;
const SEND_BUTTON_WIDTH: f32 = 120.0;

pub fn show_chatbox(
    ui: &mut egui::Ui,
    runner: &mut DialogueRunner,
    dialogues: &mut Dialogues,
    scale: &DesignScale,
    is_focused: bool,
) -> bool {
    let total = ui.max_rect();
    let chat_w = total.width() * (1.0 - SIDEBAR_RATIO);
    let header_h = total.height() * HEADER_RATIO;
    let status_h = total.height() * STATUS_RATIO;
    let msg_h = total.height() - header_h - status_h;

    let msg_rect = egui::Rect::from_min_size(
        egui::pos2(total.min.x, total.min.y + header_h),
        egui::vec2(chat_w, msg_h),
    );
    let status_rect = egui::Rect::from_min_size(
        egui::pos2(total.min.x, total.min.y + header_h + msg_h),
        egui::vec2(chat_w, status_h),
    );
    handle_player_input(runner, dialogues, ui.ctx(), is_focused);
    render_messages(ui, msg_rect, &runner.displayed, &runner.state, scale);
    render_status_bar(ui, status_rect, runner, dialogues, scale, is_focused)
}

// State machine

fn handle_player_input(
    runner: &mut DialogueRunner,
    dialogues: &Dialogues,
    ctx: &egui::Context,
    is_focused: bool,
) {
    let ChatBoxState::PlayerReady { chars_revealed } = &mut runner.state else {
        return;
    };
    if !is_focused {
        return;
    }
    let Some(line) = dialogues.lines.get(dialogues.index) else {
        return;
    };
    let total_chars = line.text.chars().count();
    let new_chars = ctx.input(|i| {
        i.events
            .iter()
            .filter(|e| matches!(e, egui::Event::Text(_)))
            .count()
    });
    if new_chars > 0 && *chars_revealed < total_chars {
        *chars_revealed = (*chars_revealed + new_chars).min(total_chars);
        runner.input = line.text.chars().take(*chars_revealed).collect();
    }
}

// Message area

fn render_messages(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    displayed: &[DialogueLine],
    state: &ChatBoxState,
    scale: &DesignScale,
) {
    let font = FontId::proportional(scale.py(FONT_CHAT));
    let font_sm = FontId::proportional(scale.py(FONT_CHAT_SM));

    let mut msg_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: (scale.x * 14.0) as i8,
            right: (scale.x * 4.0) as i8,
            top: 0,
            bottom: (scale.y * 8.0) as i8,
        })
        .show(&mut msg_ui, |ui| {
            // Messages fill available height; "anon is typing" pins to bottom.
            let indicator_gap = scale.py(4.0);
            let indicator_h = scale.py(FONT_CHAT_SM) + indicator_gap;
            let scroll_h = ui.available_height() - indicator_h;
            ui.spacing_mut().scroll = ScrollStyle {
                floating: true,
                bar_width: scale.uniform() * 10.0,
                bar_outer_margin: scale.uniform() * 2.0,
                floating_width: scale.uniform() * 4.0,
                floating_allocated_width: scale.uniform() * 20.0,
                foreground_color: true,
                dormant_background_opacity: 0.0,
                active_background_opacity: 0.4,
                interact_background_opacity: 0.7,
                dormant_handle_opacity: 0.0,
                active_handle_opacity: 0.6,
                interact_handle_opacity: 1.0,
                ..ScrollStyle::solid()
            };
            ScrollArea::vertical()
                .max_height(scroll_h)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = scale.py(8.0);
                    for DialogueLine { speaker, text, .. } in displayed {
                        bubble(ui, text, *speaker, &font, &font_sm);
                    }
                });

            // "anon is typing…" indicator below scroll area.
            if matches!(state, ChatBoxState::AnonTyping { .. }) {
                ui.add_space(scale.py(2.0));
                ui.label(
                    RichText::new("anon is typing…")
                        .font(font.clone())
                        .color(LABEL_META)
                        .italics(),
                );
            }
        });
}

// Status bar / input row

fn render_status_bar(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    runner: &mut DialogueRunner,
    dialogues: &Dialogues,
    scale: &DesignScale,
    is_focused: bool,
) -> bool {
    let mut send = false;
    let font = FontId::proportional(scale.py(FONT_CHAT_SM));
    let mut ready_to_send = false;

    // Paint a solid fill over the baked-in "Status: Online" text so our
    // overlay is readable regardless of window state.
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, DEEP_RED_THEME);

    let mut status_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: (scale.x * STATUS_MARGIN) as i8,
            right: (scale.x * STATUS_MARGIN) as i8,
            top: (scale.y * STATUS_MARGIN) as i8,
            bottom: (scale.y * STATUS_MARGIN) as i8,
        })
        .show(&mut status_ui, |ui| {
            match &runner.state {
                ChatBoxState::AnonTyping { .. } => {
                    ui.label(
                        RichText::new("| Status: Online")
                            .font(font.clone())
                            .color(LABEL_META)
                            .italics(),
                    );
                }

                ChatBoxState::PlayerReady { chars_revealed } => {
                    let total_chars = dialogues
                        .lines
                        .get(dialogues.index)
                        .map(|l| l.text.chars().count())
                        .unwrap_or(0);
                    ready_to_send = *chars_revealed >= total_chars;
                    if runner.input.is_empty() {
                        ui.label(
                            RichText::new("[PRESS ANY KEYS TO TYPE THE RESPONSE]")
                                .font(font.clone())
                                .color(LABEL_META),
                        );
                    } else {
                        ui.label(
                            RichText::new(runner.input.as_str())
                                .font(font.clone())
                                .color(LABEL_COLOR),
                        );
                    }
                }

                ChatBoxState::Done => {
                    ui.label(
                        RichText::new("| Status: Offline")
                            .font(font.clone())
                            .color(LABEL_META)
                            .italics(),
                    );
                }
            };
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                let send_clicked = ui
                    .add_enabled_ui(ready_to_send, |ui| {
                        ui.add_sized(
                            scale.px(SEND_BUTTON_WIDTH, SEND_BUTTON_HEIGHT),
                            egui::Button::new(
                                RichText::new("Send")
                                    .size(scale.py(CONTENT_FONT_SIZE))
                                    .color(RED_CONTRAST_THEME)
                                    .strong(),
                            ),
                        )
                    })
                    .inner // Extract the response from the inner Ui
                    .clicked();

                let enter_pressed = is_focused && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if send_clicked || enter_pressed {
                    send = true;
                }
            });
        });
    send
}

/// Finalise the player's turn after Send/Enter.  Updates `runner` and advances
/// the dialogue index.
pub fn commit_player_line(runner: &mut DialogueRunner, dialogues: &mut Dialogues) {
    if let Some(line) = dialogues.lines.get(dialogues.index) {
        runner.displayed.push(DialogueLine::new(true, &line.text));
    }
    runner.input.clear();
    dialogues.index += 1;
    runner.state = derive_state(&dialogues.lines, dialogues.index);
}

// Chat bubble

fn bubble(ui: &mut egui::Ui, text: &str, is_player: bool, font: &FontId, font_sm: &FontId) {
    if text.is_empty() {
        return;
    }
    ui.separator();
    let name = if is_player { "operator" } else { "anon" };
    let name_color = if is_player { LABEL_META } else { CHATBOX_ANON };
    let max_w = ui.available_width() * 0.75;

    let alignment = if is_player {
        egui::Align::Max
    } else {
        egui::Align::Min
    };

    ui.with_layout(egui::Layout::top_down(alignment), |ui| {
        ui.set_max_width(max_w);
        ui.label(
            RichText::new(name)
                .font(font_sm.clone())
                .color(name_color)
                .strong(),
        );
        ui.label(RichText::new(text).font(font.clone()).color(LABEL_COLOR));
    });
}
