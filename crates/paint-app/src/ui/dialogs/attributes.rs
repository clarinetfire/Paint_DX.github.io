//! Image Attributes dialog (Canvas dimensions, units, monochrome conversion).

use egui::{Context, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitChoice {
    Inches,
    Centimeters,
    Pixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorModeChoice {
    BlackAndWhite,
    Colors,
}

#[derive(Debug, Clone, Copy)]
pub struct AttributesParams {
    pub width: u32,
    pub height: u32,
    pub color_mode: ColorModeChoice,
}

#[derive(Debug)]
pub struct AttributesDialog {
    pub is_open: bool,
    pub width_str: String,
    pub height_str: String,
    pub unit: UnitChoice,
    pub color_mode: ColorModeChoice,
    current_px_w: u32,
    current_px_h: u32,
}

impl Default for AttributesDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            width_str: "640".to_string(),
            height_str: "480".to_string(),
            unit: UnitChoice::Pixels,
            color_mode: ColorModeChoice::Colors,
            current_px_w: 640,
            current_px_h: 480,
        }
    }
}

const DPI: f32 = 96.0;
const CM_PER_INCH: f32 = 2.54;

impl AttributesDialog {
    pub fn open(&mut self, current_w: u32, current_h: u32) {
        self.is_open = true;
        self.current_px_w = current_w;
        self.current_px_h = current_h;
        self.unit = UnitChoice::Pixels;
        self.color_mode = ColorModeChoice::Colors;
        self.update_strings_from_pixels();
    }

    fn update_strings_from_pixels(&mut self) {
        match self.unit {
            UnitChoice::Pixels => {
                self.width_str = format!("{}", self.current_px_w);
                self.height_str = format!("{}", self.current_px_h);
            }
            UnitChoice::Inches => {
                self.width_str = format!("{:.2}", self.current_px_w as f32 / DPI);
                self.height_str = format!("{:.2}", self.current_px_h as f32 / DPI);
            }
            UnitChoice::Centimeters => {
                self.width_str = format!("{:.2}", (self.current_px_w as f32 / DPI) * CM_PER_INCH);
                self.height_str = format!("{:.2}", (self.current_px_h as f32 / DPI) * CM_PER_INCH);
            }
        }
    }

    fn sync_pixels_from_strings(&mut self) {
        let w_val: f32 = self
            .width_str
            .trim()
            .parse()
            .unwrap_or(self.current_px_w as f32);
        let h_val: f32 = self
            .height_str
            .trim()
            .parse()
            .unwrap_or(self.current_px_h as f32);

        match self.unit {
            UnitChoice::Pixels => {
                self.current_px_w = (w_val as u32).max(1);
                self.current_px_h = (h_val as u32).max(1);
            }
            UnitChoice::Inches => {
                self.current_px_w = (w_val * DPI).round().max(1.0) as u32;
                self.current_px_h = (h_val * DPI).round().max(1.0) as u32;
            }
            UnitChoice::Centimeters => {
                self.current_px_w = ((w_val / CM_PER_INCH) * DPI).round().max(1.0) as u32;
                self.current_px_h = ((h_val / CM_PER_INCH) * DPI).round().max(1.0) as u32;
            }
        }
    }

    pub fn ui(&mut self, ctx: &Context) -> Option<AttributesParams> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut close = false;

        Window::new("Attributes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(320.0);

                egui::Grid::new("attr_inputs_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Width:");
                        ui.add(egui::TextEdit::singleline(&mut self.width_str).desired_width(80.0));
                        ui.end_row();

                        ui.label("Height:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.height_str).desired_width(80.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Units").strong());
                        let prev_unit = self.unit;
                        ui.radio_value(&mut self.unit, UnitChoice::Inches, "Inches");
                        ui.radio_value(&mut self.unit, UnitChoice::Centimeters, "Cm");
                        ui.radio_value(&mut self.unit, UnitChoice::Pixels, "Pixels");
                        if prev_unit != self.unit {
                            self.sync_pixels_from_strings();
                            self.update_strings_from_pixels();
                        }
                    });

                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Colors").strong());
                        ui.radio_value(
                            &mut self.color_mode,
                            ColorModeChoice::BlackAndWhite,
                            "Black and white",
                        );
                        ui.radio_value(&mut self.color_mode, ColorModeChoice::Colors, "Colors");
                    });
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("Default").clicked() {
                        self.current_px_w = 640;
                        self.current_px_h = 480;
                        self.update_strings_from_pixels();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                        if ui.button("OK").clicked() {
                            self.sync_pixels_from_strings();
                            result = Some(AttributesParams {
                                width: self.current_px_w.clamp(1, 16384),
                                height: self.current_px_h.clamp(1, 16384),
                                color_mode: self.color_mode,
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
