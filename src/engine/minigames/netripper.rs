use bevy_egui::egui;
use rand::Rng;

use crate::engine::{design_scale::DesignScale, minigames::MinigameTextures};

#[derive(Clone)]
pub struct NetRipperState {
    pub nodes: [[NetNode; 3]; 3],
    pub solved: bool,
}

#[derive(Clone, Copy, Default)]
pub struct NetNode {
    /// Logical connections: [0: Down-Right, 1: Down-Left, 2: Up-Left, 3: Up-Right]
    pub lines: [bool; 4],
    pub rotation: u8, // 0 to 3 (representing 0, 90, 180, 270 degrees)
}

impl NetNode {
    pub fn current_lines(&self) -> [bool; 4] {
        let mut rotated = [false; 4];
        for i in 0..4 {
            // Clockwise rotation shift
            rotated[i] = self.lines[(i + 4 - self.rotation as usize) % 4];
        }
        rotated
    }
}

impl NetRipperState {
    /// Generates a guaranteed solvable board by creating valid connections,
    /// and then scrambling the rotations.
    pub fn new_random() -> Self {
        let mut nodes = [[NetNode::default(); 3]; 3];
        let mut rng = rand::rng();

        // 1. Generate a solved board
        for y in 0..3 {
            for x in 0..3 {
                // Randomly connect Top-Right to neighbor's Bottom-Left
                if x < 2 && rng.random_bool(0.6) {
                    nodes[y][x].lines[0] = true;
                    nodes[y][x + 1].lines[2] = true;
                }
                // Randomly connect Bottom-Right to neighbor's Top-Left
                if y < 2 && rng.random_bool(0.6) {
                    nodes[y][x].lines[1] = true;
                    nodes[y + 1][x].lines[3] = true;
                }
            }
        }

        // 2. Scramble the rotations
        for y in 0..3 {
            for x in 0..3 {
                nodes[y][x].rotation = rng.random_range(0..4);
            }
        }

        Self {
            nodes,
            solved: false,
        }
    }

    /// Checks if all lines align perfectly with their neighbors and no lines point out of bounds.
    pub fn check_win(&self) -> bool {
        for y in 0..3 {
            for x in 0..3 {
                let lines = self.nodes[y][x].current_lines();

                // Check Top-Right (0) against neighbor's Bottom-Left (2)
                if x == 2 {
                    if lines[0] {
                        return false;
                    }
                } else if lines[0] != self.nodes[y][x + 1].current_lines()[2] {
                    return false;
                }

                // Check Bottom-Right (1) against neighbor's Top-Left (3)
                if y == 2 {
                    if lines[1] {
                        return false;
                    }
                } else if lines[1] != self.nodes[y + 1][x].current_lines()[3] {
                    return false;
                }

                // Check Bottom-Left (2) against neighbor's Top-Right (0)
                if x == 0 {
                    if lines[2] {
                        return false;
                    }
                } else if lines[2] != self.nodes[y][x - 1].current_lines()[0] {
                    return false;
                }

                // Check Top-Left (3) against neighbor's Bottom-Right (1)
                if y == 0 {
                    if lines[3] {
                        return false;
                    }
                } else if lines[3] != self.nodes[y - 1][x].current_lines()[1] {
                    return false;
                }
            }
        }
        true
    }
}

pub(super) fn render_netripper(
    ctx: &egui::Context,
    screen: egui::Rect,
    center: egui::Pos2,
    state: &mut NetRipperState,
    scale: &DesignScale,
    textures: &MinigameTextures,
) {
    let display_scale = 2.0;
    let bg_size = scale.px(512.0, 512.0) * display_scale;

    // Scale node spacing relatively to the background image dimensions to ensure alignment
    let spacing_x = bg_size.x * 0.1367;
    let spacing_y = bg_size.y * 0.1367;
    let node_radius = spacing_x * 0.55;

    egui::Area::new(egui::Id::new("netripper_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            let (_, painter) = ui.allocate_painter(screen.size(), egui::Sense::click());

            // 1. Scrim Background
            painter.rect_filled(
                screen,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(220),
            );

            // Render static base background
            let bg_rect = egui::Rect::from_center_size(center, bg_size);
            painter.image(
                textures.hack1,
                bg_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            let mut interaction = None;

            // 4. Draw the Nodes & Lines
            for y in 0..3 {
                for x in 0..3 {
                    // Map grid logically to visual diamond shape
                    let pos = center
                        + egui::vec2(
                            (x as f32 - y as f32) * spacing_x,
                            (x as f32 + y as f32 - 2.0) * spacing_y,
                        );

                    let id = egui::Id::new("node").with(x).with(y);
                    let rect = egui::Rect::from_center_size(
                        pos,
                        egui::vec2(node_radius * 2.0, node_radius * 2.0),
                    );
                    let response = ui.interact(rect, id, egui::Sense::click());

                    if response.clicked() {
                        interaction = Some((x, y, 1));
                    } // CW
                    if response.secondary_clicked() {
                        interaction = Some((x, y, 3));
                    } // CCW

                    // Draw Node Background (Slightly darker when idle, brightens when hovered)
                    let bg_color = if response.hovered() {
                        egui::Color32::from_white_alpha(40)
                    } else {
                        egui::Color32::from_black_alpha(150)
                    };
                    painter.circle_filled(pos, node_radius, bg_color);
                    painter.circle_stroke(
                        pos,
                        node_radius,
                        egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
                    );

                    // Calculate Stroke based on uniform scale so lines don't get fat/thin on window stretch
                    let line_stroke = egui::Stroke::new(
                        2.0 * scale.uniform() * display_scale,
                        egui::Color32::DARK_RED,
                    );

                    let lines = state.nodes[y][x].current_lines();

                    // Vectors matching the adjacency logic: Down-Right, Down-Left, Up-Left, Up-Right
                    let dirs = [
                        egui::vec2(spacing_x, spacing_y),
                        egui::vec2(-spacing_x, spacing_y),
                        egui::vec2(-spacing_x, -spacing_y),
                        egui::vec2(spacing_x, -spacing_y),
                    ];

                    for i in 0..4 {
                        if lines[i] {
                            let end = pos + dirs[i].normalized() * node_radius;
                            painter.line_segment([pos, end], line_stroke);

                            // Glowing cap at the edge of the line
                            painter.circle_filled(
                                end,
                                3.0 * scale.uniform() * display_scale,
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }
            }

            // 5. Apply Interactions
            if let Some((x, y, rot)) = interaction {
                state.nodes[y][x].rotation = (state.nodes[y][x].rotation + rot) % 4;
                state.solved = state.check_win();
            }
        });
}
