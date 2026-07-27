use bevy::prelude::*;
use bevy_egui::egui::{self};

use crate::{
    engine::{
        ascii_animation::{AsciiAnimation, render_ascii_animation},
        design_scale::DesignScale,
    },
    ui::theme::{
        palette::{
            COLOR_DONE, CONTENT_FONT_SIZE, DECRYPT_THEME, FONT_SMALL, HEADER_COLOR,
            HEADING_FONT_SIZE, LABEL_COLOR,
        },
        widgets::primitives::progress_bar,
    },
};

/// Total time (seconds) the decryption progress bar takes to fill.
const DECRYPT_DURATION: f32 = 12.0;

/// Progress fractions (0.0–1.0) at which a mini-game interrupt is triggered.
/// Bit N of `minigames_triggered` is set once checkpoint N has fired.
const MINIGAME_CHECKPOINTS: &[f32] = &[0.25, 0.50, 0.75];

pub struct DecrypterOutput {
    pub complete: bool,
    pub trigger_minigame: bool,
}

pub(super) fn show_encrypted(
    ui: &mut egui::Ui,
    elapsed: &mut f32,
    minigames_triggered: &mut u8,
    dt: f32,
    paused: bool,
    scale: &DesignScale,
    animation: Option<&AsciiAnimation>,
) -> DecrypterOutput {
    if !paused && *elapsed < DECRYPT_DURATION {
        *elapsed = (*elapsed + dt).min(DECRYPT_DURATION);
        ui.ctx().request_repaint();
    }

    let progress = (*elapsed / DECRYPT_DURATION).clamp(0.0, 1.0);

    let mut trigger_minigame = false;

    // Check each minigame checkpoint — mark triggered and stub the call.
    for (i, &threshold) in MINIGAME_CHECKPOINTS.iter().enumerate() {
        let bit = 1u8 << i;
        if progress >= threshold && (*minigames_triggered & bit) == 0 {
            *minigames_triggered |= bit;
            trigger_minigame = true;
        }
    }

    // UI

    // ASCII background overlay (drawn first so content appears on top).
    if let Some(anim) = animation {
        render_ascii_animation(ui, scale, anim, *elapsed);
    }

    ui.add_space(scale.py(80.0));

    let bar_w = scale.x * 480.;
    let bar_h = scale.py(16.0);

    egui::Frame::NONE
        .outer_margin(egui::Margin {
            left: 127,
            right: 127,
            top: 40,
            bottom: 40,
        })
        .fill(egui::Color32::from_black_alpha(230))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(scale.py(36.0));

                // Title
                ui.label(
                    egui::RichText::new(if progress < 1.0 {
                        "DECRYPTING FILE"
                    } else {
                        "DECRYPTION COMPLETE"
                    })
                    .size(scale.py(HEADING_FONT_SIZE))
                    .color(HEADER_COLOR)
                    .strong(),
                );

                ui.add_space(scale.py(6.0));

                // Animated sub-label
                let dot_count = ((*elapsed / 0.45) as usize) % 4;
                let status_text = if progress < 1.0 {
                    format!("Breaking encryption layers{}", ".".repeat(dot_count))
                } else {
                    "Access granted — opening folder".to_string()
                };
                ui.label(
                    egui::RichText::new(status_text)
                        .size(scale.py(CONTENT_FONT_SIZE))
                        .color(LABEL_COLOR),
                );

                ui.add_space(scale.py(20.0));
                // Progress bar
                progress_bar(ui, progress, bar_w, bar_h);
                ui.add_space(scale.py(18.0));
                // Animated hex scan line (hidden once complete)
                if progress < 1.0 {
                    ui.label(
                        egui::RichText::new(hex_scan_line(*elapsed))
                            .size(scale.py(FONT_SMALL))
                            .color(DECRYPT_THEME)
                            .monospace(),
                    );

                    ui.add_space(scale.py(14.0));

                    for (i, &threshold) in MINIGAME_CHECKPOINTS.iter().enumerate() {
                        let bit = 1u8 << i;
                        let done = (*minigames_triggered & bit) != 0;
                        let color = if done { COLOR_DONE } else { DECRYPT_THEME };
                        let label = format!(
                            "{} LAYER_{} @ {:.0}%",
                            if done { "◉" } else { "○" },
                            i + 1,
                            threshold * 100.0
                        );
                        ui.label(
                            egui::RichText::new(label)
                                .size(scale.py(FONT_SMALL))
                                .color(color)
                                .monospace(),
                        );
                        if i + 1 < MINIGAME_CHECKPOINTS.len() {
                            ui.add_space(scale.py(8.0));
                        }
                    }
                    ui.add_space(scale.py(20.0));
                }
            });
        });

    DecrypterOutput {
        complete: progress >= 1.0,
        trigger_minigame,
    }
}

/// Generates a deterministic-looking hex scan line that changes ~30 times/sec.
fn hex_scan_line(elapsed: f32) -> String {
    let seed = (elapsed * 30.0) as u64;
    let mut v = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let mut out = String::with_capacity(56);
    for i in 0..7_u64 {
        v = v
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407 + i);
        if i > 0 {
            out.push_str("  ");
        }
        out.push_str(&format!("0x{:04X}", (v >> 48) as u16));
    }
    out
}
