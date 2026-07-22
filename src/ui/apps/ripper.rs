use bevy::prelude::*;
use bevy_egui::egui::{self};

use crate::{
    engine::design_scale::DesignScale,
    ui::theme::{
        palette::{
            COLOR_DONE, CONTENT_FONT_SIZE, FONT_SMALL, HEADER_COLOR, HEADING_FONT_SIZE,
            LABEL_COLOR, RIPPER_THEME,
        },
        widgets::primitives::progress_bar,
    },
};

const TRANSMIT_DURATION: f32 = 12.0;

pub struct NetRipperOutput {
    pub complete: bool,
    pub trigger_minigame: bool,
}

/// Relay labels — flavour text for each minigame checkpoint.
const RELAYS: &[(&str, f32)] = &[
    ("RELAY_ALPHA", 0.20),
    ("RELAY_ZETA", 0.40),
    ("RELAY_OMEGA", 0.60),
    ("UPLINK_FINAL", 0.80),
];

pub(super) fn show_netripper_transmit(
    ui: &mut egui::Ui,
    target_name: &str,
    elapsed: &mut f32,
    minigames_triggered: &mut u8,
    dt: f32,
    paused: bool,
    scale: &DesignScale,
) -> NetRipperOutput {
    if !paused && *elapsed < TRANSMIT_DURATION {
        *elapsed = (*elapsed + dt).min(TRANSMIT_DURATION);
        ui.ctx().request_repaint();
    }

    let progress = (*elapsed / TRANSMIT_DURATION).clamp(0.0, 1.0);
    let mut trigger_minigame = false;

    for (i, &(_, threshold)) in RELAYS.iter().enumerate() {
        let bit = 1u8 << i;
        if progress >= threshold && (*minigames_triggered & bit) == 0 {
            *minigames_triggered |= bit;
            trigger_minigame = true;
        }
    }

    // UI

    ui.vertical_centered(|ui| {
        ui.add_space(scale.py(36.0));

        // Title
        ui.label(
            egui::RichText::new(if progress < 1.0 {
                "TRANSMITTING VIA NETRIPPER"
            } else {
                "TRANSMISSION COMPLETE"
            })
            .size(scale.py(HEADING_FONT_SIZE))
            .color(HEADER_COLOR)
            .strong(),
        );

        ui.add_space(scale.py(4.0));

        // Target line
        ui.label(
            egui::RichText::new(format!("TARGET: {}", target_name))
                .size(scale.py(CONTENT_FONT_SIZE))
                .color(RIPPER_THEME)
                .monospace(),
        );

        ui.add_space(scale.py(10.0));

        // Animated sub-label
        let dot_count = ((*elapsed / 0.45) as usize) % 4;
        let status_text = if progress < 1.0 {
            format!("Routing signal through network{}", ".".repeat(dot_count))
        } else {
            "Signal received — Earth uplink confirmed".to_string()
        };
        ui.label(
            egui::RichText::new(status_text)
                .size(scale.py(CONTENT_FONT_SIZE))
                .color(LABEL_COLOR),
        );

        ui.add_space(scale.py(20.0));
        progress_bar(ui, progress, scale.x * 480.0, scale.py(16.0));
        ui.add_space(scale.py(18.0));

        // Animated packet scan line
        if progress < 1.0 {
            ui.label(
                egui::RichText::new(packet_scan_line(*elapsed))
                    .size(scale.py(FONT_SMALL))
                    .color(RIPPER_THEME)
                    .monospace(),
            );

            ui.add_space(scale.py(14.0));

            // Relay hop indicators
            ui.horizontal_centered(|ui| {
                for (i, &(label, _)) in RELAYS.iter().enumerate() {
                    let bit = 1u8 << i;
                    let done = (*minigames_triggered & bit) != 0;
                    let color = if done { COLOR_DONE } else { RIPPER_THEME };
                    ui.label(
                        egui::RichText::new(format!("{} {}", if done { "◉" } else { "○" }, label))
                            .size(scale.py(FONT_SMALL))
                            .color(color)
                            .monospace(),
                    );
                    if i + 1 < RELAYS.len() {
                        ui.add_space(scale.uniform() * 10.0);
                    }
                }
            });
        }
    });

    NetRipperOutput {
        complete: progress >= 1.0,
        trigger_minigame,
    }
}

/// Packet-flavoured scan line — same LCG trick as the decrypter's hex line.
fn packet_scan_line(elapsed: f32) -> String {
    let seed = (elapsed * 30.0) as u64;
    let mut v = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let mut out = String::with_capacity(64);
    // Format: PKT#XXXX [SEQ:YYYY] [ACK:ZZZZ] ... repeating
    for i in 0..3u64 {
        v = v
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407 + i);
        let pkt = (v >> 48) as u16;
        let seq = (v >> 32) as u16;
        let ack = (v >> 16) as u16;
        if i > 0 {
            out.push_str("  ");
        }
        out.push_str(&format!(
            "PKT#{:04X} [SEQ:{:04X}] [ACK:{:04X}]",
            pkt, seq, ack
        ));
    }
    out
}
