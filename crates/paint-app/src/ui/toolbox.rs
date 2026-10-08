//! Toolbox UI on the left panel (16 classic tools).

use egui::Ui;
use paint_core::tools::ToolKind;

#[derive(Debug)]
pub struct ToolboxView {
    pub active_tool: ToolKind,
}

impl Default for ToolboxView {
    fn default() -> Self {
        Self {
            active_tool: ToolKind::Pencil,
        }
    }
}

impl ToolboxView {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Tools");
        ui.separator();
        egui::Grid::new("toolbox_grid")
            .num_columns(2)
            .show(ui, |ui| {
                for (idx, &tool) in ToolKind::ALL.iter().enumerate() {
                    let is_selected = self.active_tool == tool;
                    let label = match tool {
                        ToolKind::FreeformSelect => "✂ Free",
                        ToolKind::Select => "⬚ Select",
                        ToolKind::Eraser => "⌫ Erase",
                        ToolKind::Fill => "🪣 Fill",
                        ToolKind::PickColor => "💧 Pick",
                        ToolKind::Magnifier => "🔍 Zoom",
                        ToolKind::Pencil => "✏ Pencil",
                        ToolKind::Brush => "🖌 Brush",
                        ToolKind::Airbrush => "💨 Spray",
                        ToolKind::Text => "A Text",
                        ToolKind::Line => "╱ Line",
                        ToolKind::Curve => "∿ Curve",
                        ToolKind::Rectangle => "▭ Rect",
                        ToolKind::Polygon => "⬡ Poly",
                        ToolKind::Ellipse => "◯ Oval",
                        ToolKind::RoundedRectangle => "▢ Round",
                    };

                    if ui.selectable_label(is_selected, label).clicked() {
                        self.active_tool = tool;
                    }

                    if idx % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    }
}
