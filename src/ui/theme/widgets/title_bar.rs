use bevy_egui::egui::{self, emath::GuiRounding};

use crate::engine::design_scale::DesignScale;

// What the user clicked in the custom title bar.
#[derive(Debug, Clone, PartialEq)]
pub enum TitleBarAction {
    None,
    Minimize,
    Close,
}

pub fn title_bar(
    ui: &mut egui::Ui,
    title: &str,
    scale: &DesignScale,
    closable: bool,
    minimizable: bool,
) -> TitleBarAction {
    let mut action = TitleBarAction::None;
    let bar_height = scale.py(40.0);

    let bar_rect = {
        let r = ui.max_rect();
        egui::Rect::from_min_size(r.min, egui::vec2(r.width(), bar_height))
    };
    let bg_slot = ui.painter().add(egui::Shape::Noop);
    ui.allocate_rect(bar_rect, egui::Sense::hover());
    // Check if this window's layer is the top-most layer in its order stack
    let is_active = ui.ctx().memory(|mem| {
        mem.layer_ids()
            .filter(|layer| layer.order == ui.layer_id().order)
            .last()
            == Some(ui.layer_id())
    });
    ui.painter().set(bg_slot, egui::Shape::Noop);

    // Title text
    let text_pos = bar_rect.center();
    let text_color = if is_active {
        egui::Color32::BLACK
    } else {
        egui::Color32::DARK_GRAY
    };
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        title,
        egui::TextStyle::Heading.resolve(ui.style()),
        text_color,
    );

    // Buttons (right-to-left)
    let button_size = egui::Vec2::splat(scale.uniform() * ui.spacing().icon_width);
    let close_center = egui::Align2::RIGHT_CENTER
        .align_size_within_rect(
            button_size,
            bar_rect.shrink2(egui::vec2(scale.px(12.0, 0.0).x, 0.0)),
        )
        .center();
    let close_rect = egui::Rect::from_center_size(close_center, button_size)
        .round_to_pixels(ui.pixels_per_point());
    let min_center = close_center
        - egui::vec2(
            button_size.x + ui.spacing().item_spacing.x + (scale.uniform() * 8.0),
            0.0,
        );
    let min_rect = egui::Rect::from_center_size(min_center, button_size)
        .round_to_pixels(ui.pixels_per_point());

    if close_button(ui, close_rect, is_active, closable, scale).clicked() && closable {
        action = TitleBarAction::Close;
    }
    if minimize_button(ui, min_rect, is_active, minimizable, scale).clicked() {
        action = TitleBarAction::Minimize;
    }
    action
}

/// Paints a native-style "Close" button using strokes
fn close_button(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    is_active: bool,
    closable: bool,
    scale: &DesignScale,
) -> egui::Response {
    let close_id = ui.auto_id_with("window_close_button");
    let response = ui.interact(rect, close_id, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), "Close window")
    });

    ui.expand_to_include_rect(response.rect);

    let visuals = ui.style().interact(&response);
    let rect = rect.shrink(scale.uniform() * 2.0).expand(visuals.expansion);

    let mut stroke = egui::Stroke::new(visuals.fg_stroke.width, egui::Color32::BLACK);
    // Dim the button when the window is inactive (unless hovered over)
    if !is_active && !response.hovered() && !response.clicked() {
        stroke.color = stroke.color.gamma_multiply(0.4);
    }
    // Grey out the close cross when closing is disabled, but keep it visible.
    if !closable {
        stroke.color = egui::Color32::from_gray(160).gamma_multiply(0.55);
    }

    ui.painter()
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter()
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);
    let border_rect = rect.expand(scale.uniform() * 5.0);
    ui.painter().rect_stroke(
        border_rect,
        egui::CornerRadius::ZERO,
        stroke,
        egui::StrokeKind::Outside,
    );
    response
}

/// Paints a native-style "Minimize" button using strokes
fn minimize_button(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    is_active: bool,
    minimizable: bool,
    scale: &DesignScale,
) -> egui::Response {
    let min_id = ui.auto_id_with("window_minimize_button");
    let response = ui.interact(rect, min_id, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), "Minimize window")
    });

    ui.expand_to_include_rect(response.rect);

    let visuals = ui.style().interact(&response);
    let rect = rect
        .shrink(scale.uniform() * 2.00)
        .expand(visuals.expansion);

    let mut stroke = egui::Stroke::new(visuals.fg_stroke.width, egui::Color32::BLACK);
    if !is_active && !response.hovered() && !response.clicked() {
        stroke.color = stroke.color.gamma_multiply(0.4);
    }

    let y_pos = rect.center().y + scale.uniform() * 2.0;

    // Grey out the close cross when closing is disabled, but keep it visible.
    if !minimizable {
        stroke.color = egui::Color32::from_gray(160).gamma_multiply(0.55);
    }
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), y_pos),
            egui::pos2(rect.right(), y_pos),
        ],
        stroke,
    );
    let border_rect = rect.expand(scale.uniform() * 5.0);
    ui.painter().rect_stroke(
        border_rect,
        egui::CornerRadius::ZERO,
        stroke,
        egui::StrokeKind::Outside,
    );
    response
}
