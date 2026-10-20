//! Stretch and Skew dialog.

use egui::{Context, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StretchSkewParams {
    pub stretch_x_pct: u32,
    pub stretch_y_pct: u32,
    pub skew_x_deg: i32,
    pub skew_y_deg: i32,
}

#[derive(Debug)]
pub struct StretchSkewDialog {
    pub is_open: bool,
    pub stretch_x_str: String,
    pub stretch_y_str: String,
    pub skew_x_str: String,
    pub skew_y_str: String,
}

impl Default for StretchSkewDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            stretch_x_str: "100".to_string(),
            stretch_y_str: "100".to_string(),
            skew_x_str: "0".to_string(),
            skew_y_str: "0".to_string(),
        }
    }
}

impl StretchSkewDialog {
    pub fn open(&mut self) {
        self.is_open = true;
        self.stretch_x_str = "100".to_string();
        self.stretch_y_str = "100".to_string();
        self.skew_x_str = "0".to_string();
        self.skew_y_str = "0".to_string();
    }

    pub fn ui(&mut self, ctx: &Context) -> Option<StretchSkewParams> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut close = false;

        Window::new("Stretch and Skew")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(280.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Stretch").strong());
                    egui::Grid::new("stretch_grid")
                        .num_columns(3)
                        .show(ui, |ui| {
                            ui.label("Horizontal:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.stretch_x_str)
                                    .desired_width(60.0),
                            );
                            ui.label("%");
                            ui.end_row();

                            ui.label("Vertical:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.stretch_y_str)
                                    .desired_width(60.0),
                            );
                            ui.label("%");
                            ui.end_row();
                        });
                });

                ui.add_space(4.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Skew").strong());
                    egui::Grid::new("skew_grid").num_columns(3).show(ui, |ui| {
                        ui.label("Horizontal:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.skew_x_str).desired_width(60.0),
                        );
                        ui.label("Degrees");
                        ui.end_row();

                        ui.label("Vertical:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.skew_y_str).desired_width(60.0),
                        );
                        ui.label("Degrees");
                        ui.end_row();
                    });
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                        if ui.button("OK").clicked() {
                            let stretch_x = self
                                .stretch_x_str
                                .trim()
                                .parse::<u32>()
                                .unwrap_or(100)
                                .clamp(1, 500);
                            let stretch_y = self
                                .stretch_y_str
                                .trim()
                                .parse::<u32>()
                                .unwrap_or(100)
                                .clamp(1, 500);
                            let skew_x = self
                                .skew_x_str
                                .trim()
                                .parse::<i32>()
                                .unwrap_or(0)
                                .clamp(-89, 89);
                            let skew_y = self
                                .skew_y_str
                                .trim()
                                .parse::<i32>()
                                .unwrap_or(0)
                                .clamp(-89, 89);

                            result = Some(StretchSkewParams {
                                stretch_x_pct: stretch_x,
                                stretch_y_pct: stretch_y,
                                skew_x_deg: skew_x,
                                skew_y_deg: skew_y,
                            });
                            close = true;
                        }
                    });
                });
            });

        if close {
            self.is_open = false;
        }

        result
    }
}
