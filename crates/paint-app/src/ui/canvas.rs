//! Canvas rendering and interaction area.

use egui::Ui;
use paint_core::image::PixelBuffer;

#[derive(Debug)]
pub struct CanvasView {
    pub zoom: f32,
    pub show_grid: bool,
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            show_grid: false,
        }
    }
}

impl CanvasView {
    pub fn ui(&mut self, ui: &mut Ui, buffer: &PixelBuffer) {
        ui.vertical(|ui| {
            ui.label(format!(
                "Canvas: {} x {} (Zoom: {:.0}%)",
                buffer.width(),
                buffer.height(),
                self.zoom * 100.0
            ));
        });
    }
}
