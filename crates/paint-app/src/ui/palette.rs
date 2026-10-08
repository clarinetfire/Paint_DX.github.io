//! Palette UI at the bottom (28 colors + FG/BG indicator).

use egui::{Color32, Ui, Vec2};
use paint_core::palette::{ColorRgba, BLACK, WHITE, XP_PALETTE_28};

#[derive(Debug)]
pub struct PaletteView {
    pub fg_color: ColorRgba,
    pub bg_color: ColorRgba,
}

impl Default for PaletteView {
    fn default() -> Self {
        Self {
            fg_color: BLACK,
            bg_color: WHITE,
        }
    }
}

impl PaletteView {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Foreground / Background swatch preview
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let fg_c32 = Color32::from_rgba_unmultiplied(
                        self.fg_color.r,
                        self.fg_color.g,
                        self.fg_color.b,
                        self.fg_color.a,
                    );
                    let bg_c32 = Color32::from_rgba_unmultiplied(
                        self.bg_color.r,
                        self.bg_color.g,
                        self.bg_color.b,
                        self.bg_color.a,
                    );
                    let (rect, _response) =
                        ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, bg_c32);
                    let fg_rect = egui::Rect::from_min_size(rect.min, Vec2::new(16.0, 16.0));
                    ui.painter().rect_filled(fg_rect, 0.0, fg_c32);
                });
            });

            ui.separator();

            // 28-color grid (2 rows of 14)
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    for color in &XP_PALETTE_28[0..14] {
                        self.render_swatch(ui, *color);
                    }
                });
                ui.horizontal(|ui| {
                    for color in &XP_PALETTE_28[14..28] {
                        self.render_swatch(ui, *color);
                    }
                });
            });
        });
    }

    fn render_swatch(&mut self, ui: &mut Ui, color: ColorRgba) {
        let c32 = Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::click());
        ui.painter().rect_filled(rect, 0.0, c32);
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0_f32, Color32::from_gray(128)),
            egui::StrokeKind::Inside,
        );

        if response.clicked() {
            self.fg_color = color;
        } else if response.secondary_clicked() {
            self.bg_color = color;
        }
    }
}
