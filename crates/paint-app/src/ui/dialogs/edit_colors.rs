//! Windows XP Edit Colors dialog (48 basic colors + 16 custom slots + HSL/RGB sliders).

use egui::{Color32, Context, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, Window};
use paint_core::palette::{
    hsl_win_to_rgb, rgb_to_hsl_win, ColorRgba, BLACK, WHITE, XP_BASIC_COLORS_48,
};

#[derive(Debug)]
pub struct EditColorsDialog {
    pub is_open: bool,
    pub custom_colors: [ColorRgba; 16],
    pub selected_custom_idx: usize,
    pub current_color: ColorRgba,
    pub hue: u8,
    pub sat: u8,
    pub lum: u8,
    // String fields for text editing
    pub hue_str: String,
    pub sat_str: String,
    pub lum_str: String,
    pub red_str: String,
    pub green_str: String,
    pub blue_str: String,
    pub alpha_str: String,
}

impl Default for EditColorsDialog {
    fn default() -> Self {
        let default_custom = [WHITE; 16];
        let mut dlg = Self {
            is_open: false,
            custom_colors: default_custom,
            selected_custom_idx: 0,
            current_color: BLACK,
            hue: 0,
            sat: 0,
            lum: 0,
            hue_str: "0".to_string(),
            sat_str: "0".to_string(),
            lum_str: "0".to_string(),
            red_str: "0".to_string(),
            green_str: "0".to_string(),
            blue_str: "0".to_string(),
            alpha_str: "255".to_string(),
        };
        dlg.set_color(BLACK);
        dlg
    }
}

impl EditColorsDialog {
    pub fn open(&mut self, initial_color: ColorRgba) {
        self.is_open = true;
        self.set_color(initial_color);
    }

    pub fn set_color(&mut self, color: ColorRgba) {
        self.current_color = color;
        let (h, s, l) = rgb_to_hsl_win(color.r, color.g, color.b);
        self.hue = h;
        self.sat = s;
        self.lum = l;

        self.hue_str = format!("{}", h);
        self.sat_str = format!("{}", s);
        self.lum_str = format!("{}", l);
        self.red_str = format!("{}", color.r);
        self.green_str = format!("{}", color.g);
        self.blue_str = format!("{}", color.b);
        self.alpha_str = format!("{}", color.a);
    }

    fn sync_from_hsl(&mut self) {
        let (r, g, b) = hsl_win_to_rgb(self.hue, self.sat, self.lum);
        self.current_color = ColorRgba::rgba(r, g, b, self.current_color.a);
        self.red_str = format!("{}", r);
        self.green_str = format!("{}", g);
        self.blue_str = format!("{}", b);
        self.hue_str = format!("{}", self.hue);
        self.sat_str = format!("{}", self.sat);
        self.lum_str = format!("{}", self.lum);
    }

    fn sync_from_rgb(&mut self) {
        let (h, s, l) = rgb_to_hsl_win(
            self.current_color.r,
            self.current_color.g,
            self.current_color.b,
        );
        self.hue = h;
        self.sat = s;
        self.lum = l;
        self.hue_str = format!("{}", h);
        self.sat_str = format!("{}", s);
        self.lum_str = format!("{}", l);
        self.red_str = format!("{}", self.current_color.r);
        self.green_str = format!("{}", self.current_color.g);
        self.blue_str = format!("{}", self.current_color.b);
    }

    pub fn ui(&mut self, ctx: &Context) -> Option<ColorRgba> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut close = false;

        Window::new("Edit Colors")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(450.0);

                ui.horizontal(|ui| {
                    // Left Column: Basic Colors & Custom Colors
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Basic colors:").strong());
                        self.render_basic_palette(ui);

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Custom colors:").strong());
                        self.render_custom_palette(ui);

                        ui.add_space(8.0);
                        if ui.button("Add to Custom Colors").clicked() {
                            self.custom_colors[self.selected_custom_idx] = self.current_color;
                            self.selected_custom_idx = (self.selected_custom_idx + 1) % 16;
                        }
                    });

                    ui.separator();

                    // Right Column: Spectrum Box, Lum Slider, Numerical Inputs
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // 2D Hue / Saturation spectrum
                            self.render_spectrum_box(ui);

                            // 1D Luminance slider
                            self.render_luminance_slider(ui);
                        });

                        ui.add_space(8.0);

                        // Color preview swatch
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let c32 = Color32::from_rgba_unmultiplied(
                                self.current_color.r,
                                self.current_color.g,
                                self.current_color.b,
                                self.current_color.a,
                            );
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::new(48.0, 24.0), Sense::hover());
                            // Draw checkerboard behind preview if alpha < 255
                            if self.current_color.a < 255 {
                                for r in 0..3 {
                                    for c in 0..6 {
                                        let sq_col = if (r + c) % 2 == 0 {
                                            Color32::from_gray(240)
                                        } else {
                                            Color32::from_gray(200)
                                        };
                                        let sq_rect = Rect::from_min_size(
                                            rect.min + Vec2::new(c as f32 * 8.0, r as f32 * 8.0),
                                            Vec2::splat(8.0),
                                        );
                                        ui.painter().rect_filled(sq_rect, 0.0, sq_col);
                                    }
                                }
                            }
                            ui.painter().rect_filled(rect, 0.0, c32);
                            ui.painter().rect_stroke(
                                rect,
                                0.0,
                                Stroke::new(1.0_f32, Color32::BLACK),
                                StrokeKind::Inside,
                            );
                        });

                        ui.add_space(6.0);

                        // Numerical input grid (HSL / RGB / Alpha)
                        egui::Grid::new("color_coords_grid")
                            .num_columns(4)
                            .show(ui, |ui| {
                                ui.label("Hue:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.hue_str)
                                            .desired_width(45.0),
                                    )
                                    .lost_focus()
                                {
                                    self.hue = self
                                        .hue_str
                                        .trim()
                                        .parse::<u8>()
                                        .unwrap_or(self.hue)
                                        .clamp(0, 240);
                                    self.sync_from_hsl();
                                }

                                ui.label("Red:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.red_str)
                                            .desired_width(45.0),
                                    )
                                    .lost_focus()
                                {
                                    let r = self
                                        .red_str
                                        .trim()
                                        .parse::<u8>()
                                        .unwrap_or(self.current_color.r);
                                    self.current_color.r = r;
                                    self.sync_from_rgb();
                                }
                                ui.end_row();

                                ui.label("Sat:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.sat_str)
                                            .desired_width(45.0),
                                    )
                                    .lost_focus()
                                {
                                    self.sat = self
                                        .sat_str
                                        .trim()
                                        .parse::<u8>()
                                        .unwrap_or(self.sat)
                                        .clamp(0, 240);
                                    self.sync_from_hsl();
                                }

                                ui.label("Green:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.green_str)
                                            .desired_width(45.0),
                                    )
                                    .lost_focus()
                                {
                                    let g = self
                                        .green_str
                                        .trim()
                                        .parse::<u8>()
                                        .unwrap_or(self.current_color.g);
                                    self.current_color.g = g;
                                    self.sync_from_rgb();
                                }
                                ui.end_row();

                                ui.label("Lum:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.lum_str)
                                            .desired_width(45.0),
                                    )
                                    .lost_focus()
                                {
                                    self.lum = self
                                        .lum_str
                                        .trim()
                                        .parse::<u8>()
                                        .unwrap_or(self.lum)
                                        .clamp(0, 240);
                                    self.sync_from_hsl();
                                }

                                ui.label("Blue:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.blue_str)
                                            .desired_width(45.0),
                                    )
                                    .lost_focus()
                                {
                                    let b = self
                                        .blue_str
                                        .trim()
                                        .parse::<u8>()
                                        .unwrap_or(self.current_color.b);
                                    self.current_color.b = b;
                                    self.sync_from_rgb();
                                }
                                ui.end_row();

                                ui.label("Alpha:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut self.alpha_str)
                                            .desired_width(45.0),
                                    )
                                    .lost_focus()
                                {
                                    let a = self
                                        .alpha_str
                                        .trim()
                                        .parse::<u8>()
                                        .unwrap_or(self.current_color.a);
                                    self.current_color.a = a;
                                    self.alpha_str = format!("{}", a);
                                }

                                ui.add(
                                    egui::Slider::new(&mut self.current_color.a, 0..=255).text(""),
                                );
                                self.alpha_str = format!("{}", self.current_color.a);
                                ui.end_row();
                            });
                    });
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                        if ui.button("OK").clicked() {
                            result = Some(self.current_color);
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

    fn render_basic_palette(&mut self, ui: &mut Ui) {
        // 6 rows x 8 cols
        for row in 0..6 {
            ui.horizontal(|ui| {
                for col in 0..8 {
                    let idx = row * 8 + col;
                    let color = XP_BASIC_COLORS_48[idx];
                    let c32 = Color32::from_rgb(color.r, color.g, color.b);
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(18.0, 16.0), Sense::click());
                    ui.painter().rect_filled(rect, 0.0, c32);
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0_f32, Color32::from_gray(128)),
                        StrokeKind::Inside,
                    );

                    if response.clicked() {
                        self.set_color(color);
                    }
                }
            });
        }
    }

    fn render_custom_palette(&mut self, ui: &mut Ui) {
        // 2 rows x 8 cols
        for row in 0..2 {
            ui.horizontal(|ui| {
                for col in 0..8 {
                    let idx = row * 8 + col;
                    let color = self.custom_colors[idx];
                    let c32 = Color32::from_rgb(color.r, color.g, color.b);
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(18.0, 16.0), Sense::click());
                    ui.painter().rect_filled(rect, 0.0, c32);
                    let stroke_color = if self.selected_custom_idx == idx {
                        Color32::BLACK
                    } else {
                        Color32::from_gray(128)
                    };
                    let stroke_w = if self.selected_custom_idx == idx {
                        2.0_f32
                    } else {
                        1.0_f32
                    };
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(stroke_w, stroke_color),
                        StrokeKind::Inside,
                    );

                    if response.clicked() {
                        self.selected_custom_idx = idx;
                        self.set_color(color);
                    }
                }
            });
        }
    }

    fn render_spectrum_box(&mut self, ui: &mut Ui) {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(160.0, 120.0), Sense::click_and_drag());
        let painter = ui.painter();

        // Draw crosshair on spectrum
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0_f32, Color32::BLACK),
            StrokeKind::Inside,
        );

        // Approximate 2D gradient mesh or grid
        let step_x = 10;
        let step_y = 8;
        let cell_w = rect.width() / step_x as f32;
        let cell_h = rect.height() / step_y as f32;

        for ix in 0..step_x {
            for iy in 0..step_y {
                let cell_rect = Rect::from_min_size(
                    rect.min + Vec2::new(ix as f32 * cell_w, iy as f32 * cell_h),
                    Vec2::new(cell_w + 0.5, cell_h + 0.5),
                );
                let h = ((ix as f32 / step_x as f32) * 240.0) as u8;
                let s = (240.0 - (iy as f32 / step_y as f32) * 240.0) as u8;
                let (r, g, b) = hsl_win_to_rgb(h, s, 120);
                painter.rect_filled(cell_rect, 0.0, Color32::from_rgb(r, g, b));
            }
        }

        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rel_x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let rel_y = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                self.hue = (rel_x * 240.0).round() as u8;
                self.sat = ((1.0 - rel_y) * 240.0).round() as u8;
                self.sync_from_hsl();
            }
        }

        // Draw marker
        let marker_x = rect.min.x + (self.hue as f32 / 240.0) * rect.width();
        let marker_y = rect.min.y + (1.0 - (self.sat as f32 / 240.0)) * rect.height();
        painter.circle_stroke(
            egui::pos2(marker_x, marker_y),
            4.0,
            Stroke::new(1.5_f32, Color32::BLACK),
        );
        painter.circle_stroke(
            egui::pos2(marker_x, marker_y),
            3.0,
            Stroke::new(1.5_f32, Color32::WHITE),
        );
    }

    fn render_luminance_slider(&mut self, ui: &mut Ui) {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(20.0, 120.0), Sense::click_and_drag());
        let painter = ui.painter();

        // Draw vertical gradient from pure color at lum=120 to white at top (lum=240) and black at bottom (lum=0)
        let steps = 12;
        let step_h = rect.height() / steps as f32;
        for i in 0..steps {
            let l = ((steps - 1 - i) as f32 / (steps - 1) as f32 * 240.0) as u8;
            let (r, g, b) = hsl_win_to_rgb(self.hue, self.sat, l);
            let cell_rect = Rect::from_min_size(
                rect.min + Vec2::new(0.0, i as f32 * step_h),
                Vec2::new(rect.width(), step_h + 0.5),
            );
            painter.rect_filled(cell_rect, 0.0, Color32::from_rgb(r, g, b));
        }

        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0_f32, Color32::BLACK),
            StrokeKind::Inside,
        );

        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rel_y = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                self.lum = ((1.0 - rel_y) * 240.0).round() as u8;
                self.sync_from_hsl();
            }
        }

        // Draw arrow marker on slider
        let marker_y = rect.min.y + (1.0 - (self.lum as f32 / 240.0)) * rect.height();
        painter.line_segment(
            [
                egui::pos2(rect.min.x - 3.0, marker_y),
                egui::pos2(rect.max.x + 3.0, marker_y),
            ],
            Stroke::new(2.0_f32, Color32::BLACK),
        );
    }
}
