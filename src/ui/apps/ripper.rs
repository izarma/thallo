use bevy::prelude::*;
use bevy_egui::egui::{self, Color32};

use crate::ui::theme::{
    palette::{HEADER_COLOR, LABEL_COLOR},
    widgets::primitives::progress_bar,
};

const TRANSMIT_DURATION: f32 = 12.0;

const COLOR_RELAY: Color32 = Color32::from_rgb(80, 160, 200);
const COLOR_RELAY_DONE: Color32 = Color32::from_rgb(60, 200, 120);
const COLOR_RELAY_PENDING: Color32 = Color32::from_rgb(55, 55, 70);

pub struct NetRipperOutput {
    pub complete: bool,
    pub triggered_checkpoint: Option<usize>,
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
) -> NetRipperOutput {
    if !paused && *elapsed < TRANSMIT_DURATION {
        *elapsed = (*elapsed + dt).min(TRANSMIT_DURATION);
        ui.ctx().request_repaint();
    }

    let progress = (*elapsed / TRANSMIT_DURATION).clamp(0.0, 1.0);
    let mut triggered_checkpoint = None;

    for (i, &(_, threshold)) in RELAYS.iter().enumerate() {
        let bit = 1u8 << i;
        if progress >= threshold && (*minigames_triggered & bit) == 0 {
            *minigames_triggered |= bit;
            triggered_checkpoint = Some(i);
        }
    }

    // UI

    ui.vertical_centered(|ui| {
        ui.add_space(36.0);

        // Title
        ui.label(
            egui::RichText::new(if progress < 1.0 {
                "TRANSMITTING VIA NETRIPPER"
            } else {
                "TRANSMISSION COMPLETE"
            })
            .size(17.0)
            .color(HEADER_COLOR)
            .strong(),
        );

        ui.add_space(4.0);

        // Target line
        ui.label(
            egui::RichText::new(format!("TARGET: {}", target_name))
                .size(11.0)
                .color(COLOR_RELAY)
                .monospace(),
        );

        ui.add_space(10.0);

        // Animated sub-label
        let dot_count = ((*elapsed / 0.45) as usize) % 4;
        let status_text = if progress < 1.0 {
            format!("Routing signal through network{}", ".".repeat(dot_count))
        } else {
            "Signal received — Earth uplink confirmed".to_string()
        };
        ui.label(
            egui::RichText::new(status_text)
                .size(11.0)
                .color(LABEL_COLOR),
        );

        ui.add_space(20.0);

        progress_bar(ui, progress, 480.0, 16.0);

        ui.add_space(18.0);

        // Animated packet scan line
        if progress < 1.0 {
            ui.label(
                egui::RichText::new(packet_scan_line(*elapsed))
                    .size(10.0)
                    .color(Color32::from_rgb(80, 160, 200))
                    .monospace(),
            );

            ui.add_space(14.0);

            // Relay hop indicators
            ui.horizontal_centered(|ui| {
                for (i, &(label, _)) in RELAYS.iter().enumerate() {
                    let bit = 1u8 << i;
                    let done = (*minigames_triggered & bit) != 0;
                    let color = if done {
                        COLOR_RELAY_DONE
                    } else {
                        COLOR_RELAY_PENDING
                    };
                    ui.label(
                        egui::RichText::new(format!("{} {}", if done { "◉" } else { "○" }, label))
                            .size(9.0)
                            .color(color)
                            .monospace(),
                    );
                    if i + 1 < RELAYS.len() {
                        ui.add_space(10.0);
                    }
                }
            });
        }
    });

    NetRipperOutput {
        complete: progress >= 1.0,
        triggered_checkpoint,
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
