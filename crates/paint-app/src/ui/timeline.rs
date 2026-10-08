//! Animation timeline panel UI.

use egui::Ui;

#[derive(Debug, Default)]
pub struct TimelineView {
    pub is_open: bool,
    pub current_frame: usize,
}

impl TimelineView {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Timeline");
        ui.separator();
        ui.label(format!("Frame {} / 1", self.current_frame + 1));
    }
}
