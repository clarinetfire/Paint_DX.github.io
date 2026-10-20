//! Palette UI at the bottom (28 colors + FG/BG indicator + Opacity slider).

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};
use paint_core::palette::{ColorRgba, RetroPalettePreset, BLACK, WHITE, XP_PALETTE_28};

#[derive(Debug)]
pub struct PaletteView {
    pub fg_color: ColorRgba,
    pub bg_color: ColorRgba,
    pub swatches: [ColorRgba; 28],
    pub active_preset: Option<RetroPalettePreset>,
}

impl Default for PaletteView {
    fn default() -> Self {
        Self {
            fg_color: BLACK,
            bg_color: WHITE,
            swatches: XP_PALETTE_28,
            active_preset: Some(RetroPalettePreset::WindowsXp),
        }
    }
}

fn draw_checkerboard(painter: &Painter, rect: Rect, grid_size: f32) {
    let cols = (rect.width() / grid_size).ceil() as usize;
    let rows = (rect.height() / grid_size).ceil() as usize;
    for r in 0..rows {
        for c in 0..cols {
            let color = if (r + c) % 2 == 0 {
                Color32::from_gray(240)
            } else {
                Color32::from_gray(200)
            };
            let min = Pos2::new(
                (rect.min.x + c as f32 * grid_size).min(rect.max.x),
                (rect.min.y + r as f32 * grid_size).min(rect.max.y),
            );
            let max = Pos2::new(
                (min.x + grid_size).min(rect.max.x),
                (min.y + grid_size).min(rect.max.y),
            );
            painter.rect_filled(Rect::from_min_max(min, max), 0.0, color);
        }
    }
}

impl PaletteView {
    pub fn ui(&mut self, ui: &mut Ui) -> Option<usize> {
        let mut double_clicked = None;

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

                    // BG box
                    let bg_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + 8.0, rect.min.y + 8.0),
                        Vec2::new(20.0, 20.0),
                    );
                    draw_checkerboard(ui.painter(), bg_rect, 4.0);
                    ui.painter().rect_filled(bg_rect, 0.0, bg_c32);
                    ui.painter().rect_stroke(
                        bg_rect,
                        0.0,
                        Stroke::new(1.0_f32, Color32::from_gray(100)),
                        StrokeKind::Inside,
                    );

                    // FG box
                    let fg_rect = egui::Rect::from_min_size(rect.min, Vec2::new(20.0, 20.0));
                    draw_checkerboard(ui.painter(), fg_rect, 4.0);
                    ui.painter().rect_filled(fg_rect, 0.0, fg_c32);
                    ui.painter().rect_stroke(
                        fg_rect,
                        0.0,
                        Stroke::new(1.0_f32, Color32::BLACK),
                        StrokeKind::Inside,
                    );
                });
            });

            ui.separator();

            // 28-color grid (2 rows of 14)
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    for idx in 0..14 {
                        if self.render_swatch(ui, idx) {
                            double_clicked = Some(idx);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    for idx in 14..28 {
                        if self.render_swatch(ui, idx) {
                            double_clicked = Some(idx);
                        }
                    }
                });
            });

            ui.separator();

            // Retro Palette Preset Selector
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Preset:").size(11.0));
                    let current_label = match self.active_preset {
                        Some(p) => p.name(),
                        None => "Custom",
                    };
                    egui::ComboBox::from_id_salt("palette_preset_combo")
                        .width(130.0)
                        .selected_text(current_label)
                        .show_ui(ui, |ui| {
                            for preset in RetroPalettePreset::ALL {
                                let is_selected = self.active_preset == Some(preset);
                                if ui.selectable_label(is_selected, preset.name()).clicked() {
                                    self.apply_preset(preset);
                                }
                            }
                        });
                });
            });

            ui.separator();

            // Opacity slider
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Opacity:").size(11.0));
                    let mut alpha_pct = (self.fg_color.a as f32 / 255.0 * 100.0).round() as u32;
                    ui.spacing_mut().slider_width = 70.0;
                    if ui
                        .add(egui::Slider::new(&mut alpha_pct, 0..=100).suffix("%"))
                        .changed()
                    {
                        self.fg_color.a = ((alpha_pct as f32 / 100.0) * 255.0).round() as u8;
                    }
                });
            });
        });

        double_clicked
    }

    pub fn apply_preset(&mut self, preset: RetroPalettePreset) {
        self.swatches = preset.swatches();
        self.active_preset = Some(preset);
    }

    fn render_swatch(&mut self, ui: &mut Ui, idx: usize) -> bool {
        let color = self.swatches[idx];
        let c32 = Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::click());
        if color.a < 255 {
            draw_checkerboard(ui.painter(), rect, 4.0);
        }
        ui.painter().rect_filled(rect, 0.0, c32);
        ui.painter().rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0_f32, Color32::from_gray(128)),
            StrokeKind::Inside,
        );

        if response.double_clicked() {
            return true;
        }

        if response.clicked() {
            // Retain opacity or use swatch opacity
            self.fg_color = color;
        } else if response.secondary_clicked() {
            self.bg_color = color;
        }

        false
    }
}
