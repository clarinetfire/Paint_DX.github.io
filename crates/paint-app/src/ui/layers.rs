//! Layers panel UI.

use egui::Ui;

#[derive(Debug, Default)]
pub struct LayersView {
    pub is_open: bool,
}

impl LayersView {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Layers");
        ui.separator();
        ui.label("Layer 1 (Background)");
    }
}
