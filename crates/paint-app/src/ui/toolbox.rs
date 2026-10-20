//! Toolbox UI on the left panel (16 classic tools + options area).

use crate::ui::canvas::SymmetryMode;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use paint_core::palette::ColorRgba;
use paint_core::tools::{
    AirbrushSize, BrushShape, EraserSize, FillPattern, FillStyle, FontFamilyChoice, MixerEffect,
    MixerTray, SelectionMode, StampShape, ToolKind, WackyBrushShape,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolboxMode {
    #[default]
    Classic,
    KidPix,
}

/// Helper function to handle mouse wheel scrolling when hovering any widget/value
pub fn handle_hover_scroll<T>(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut T,
    step: T,
    min: T,
    max: T,
) where
    T: Copy + PartialOrd + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    if response.hovered() {
        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_y > 0.0 {
            let next = *value + step;
            if next <= max {
                *value = next;
            } else {
                *value = max;
            }
        } else if scroll_y < 0.0 {
            if *value >= min + step {
                *value = *value - step;
            } else {
                *value = min;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolboxView {
    pub mode: ToolboxMode,
    pub active_tool: ToolKind,
    pub line_width: u32,
    pub pencil_size: u32,
    pub eraser_size: EraserSize,
    pub brush_shape: BrushShape,
    pub airbrush_size: AirbrushSize,
    pub fill_style: FillStyle,
    pub selection_mode: SelectionMode,
    pub font_family: FontFamilyChoice,
    pub font_size: u32,
    pub magnifier_zoom: f32,
    pub erase_to_transparent: bool,
    pub fill_tolerance: u8,
    pub anti_aliasing: bool,
    pub wacky_brush_shape: WackyBrushShape,
    pub wacky_brush_size: u32,
    pub stamp_shape: StampShape,
    pub stamp_scale: u32,
    pub stamp_rotation: u16,
    pub fill_pattern: FillPattern,
    pub text_rotation: u16,
    pub mixer_tray: MixerTray,
    pub active_mixer_effect: MixerEffect,
    pub pending_mixer_effect: Option<MixerEffect>,
    // Precision Mode / Mode Minutieux (Kid Pix)
    pub precision_mode: bool,
    pub wacky_spacing: u32,
    pub wacky_continuous: bool,
    pub wacky_angle: f32,
    pub wacky_auto_tangent: bool,
    pub wacky_density: f32,
    pub wacky_scatter_radius: u32,
    pub wacky_seed_locked: bool,
    pub wacky_seed_value: u64,
    pub stamp_fine_scale: f32,
    pub stamp_free_rotation: f32,
    pub stamp_continuous_trail: bool,
    pub stamp_trail_spacing: u32,
    pub stamp_tint_mode: usize,
    pub pattern_scale: u32,
    pub pattern_offset_x: i32,
    pub pattern_offset_y: i32,
    pub mixer_scope_selection: bool,
}

impl Default for ToolboxView {
    fn default() -> Self {
        Self {
            mode: ToolboxMode::Classic,
            active_tool: ToolKind::Pencil,
            line_width: 1,
            pencil_size: 1,
            eraser_size: EraserSize::Medium,
            brush_shape: BrushShape::RoundMedium,
            airbrush_size: AirbrushSize::Medium,
            fill_style: FillStyle::OutlineOnly,
            selection_mode: SelectionMode::Transparent,
            font_family: FontFamilyChoice::Pixel8x8,
            font_size: 16,
            magnifier_zoom: 4.0,
            erase_to_transparent: false,
            fill_tolerance: 0,
            anti_aliasing: false,
            wacky_brush_shape: WackyBrushShape::Rainbow,
            wacky_brush_size: 4,
            stamp_shape: StampShape::Smiley,
            stamp_scale: 1,
            stamp_rotation: 0,
            fill_pattern: FillPattern::Checkerboard,
            text_rotation: 0,
            mixer_tray: MixerTray::Tray1,
            active_mixer_effect: MixerEffect::Wave,
            pending_mixer_effect: None,
            precision_mode: false,
            wacky_spacing: 6,
            wacky_continuous: false,
            wacky_angle: 0.0,
            wacky_auto_tangent: true,
            wacky_density: 1.0,
            wacky_scatter_radius: 0,
            wacky_seed_locked: false,
            wacky_seed_value: 123456,
            stamp_fine_scale: 1.0,
            stamp_free_rotation: 0.0,
            stamp_continuous_trail: false,
            stamp_trail_spacing: 16,
            stamp_tint_mode: 0,
            pattern_scale: 1,
            pattern_offset_x: 0,
            pattern_offset_y: 0,
            mixer_scope_selection: true,
        }
    }
}

impl ToolboxView {
    pub fn increase_tool_size(&mut self) {
        match self.active_tool {
            ToolKind::Brush => {
                self.brush_shape = match self.brush_shape {
                    BrushShape::RoundSmall => BrushShape::RoundMedium,
                    BrushShape::RoundMedium => BrushShape::RoundLarge,
                    BrushShape::SquareSmall => BrushShape::SquareMedium,
                    BrushShape::SquareMedium => BrushShape::SquareLarge,
                    BrushShape::ForwardSlashSmall => BrushShape::ForwardSlashMedium,
                    BrushShape::ForwardSlashMedium => BrushShape::ForwardSlashLarge,
                    BrushShape::BackslashSmall => BrushShape::BackslashMedium,
                    BrushShape::BackslashMedium => BrushShape::BackslashLarge,
                    other => other,
                };
            }
            ToolKind::WackyBrush => {
                self.wacky_brush_size = (self.wacky_brush_size + 1).min(24);
            }
            ToolKind::Stamp => {
                self.stamp_scale = (self.stamp_scale + 1).min(4);
            }
            ToolKind::Eraser => {
                self.eraser_size = match self.eraser_size {
                    EraserSize::Small => EraserSize::Medium,
                    EraserSize::Medium => EraserSize::Large,
                    EraserSize::Large => EraserSize::XLarge,
                    EraserSize::XLarge => EraserSize::XLarge,
                };
            }
            ToolKind::Airbrush => {
                self.airbrush_size = match self.airbrush_size {
                    AirbrushSize::Small => AirbrushSize::Medium,
                    AirbrushSize::Medium => AirbrushSize::Large,
                    AirbrushSize::Large => AirbrushSize::Large,
                };
            }
            ToolKind::Text => {
                self.font_size = (self.font_size + 2).min(72);
            }
            ToolKind::Pencil => {
                self.pencil_size = (self.pencil_size + 1).min(64);
            }
            ToolKind::Line | ToolKind::Curve if self.line_width < 5 => {
                self.line_width += 1;
            }
            _ => {}
        }
    }

    pub fn decrease_tool_size(&mut self) {
        match self.active_tool {
            ToolKind::Brush => {
                self.brush_shape = match self.brush_shape {
                    BrushShape::RoundLarge => BrushShape::RoundMedium,
                    BrushShape::RoundMedium => BrushShape::RoundSmall,
                    BrushShape::SquareLarge => BrushShape::SquareMedium,
                    BrushShape::SquareMedium => BrushShape::SquareSmall,
                    BrushShape::ForwardSlashLarge => BrushShape::ForwardSlashMedium,
                    BrushShape::ForwardSlashMedium => BrushShape::ForwardSlashSmall,
                    BrushShape::BackslashLarge => BrushShape::BackslashMedium,
                    BrushShape::BackslashMedium => BrushShape::BackslashSmall,
                    other => other,
                };
            }
            ToolKind::WackyBrush => {
                self.wacky_brush_size = self.wacky_brush_size.saturating_sub(1).max(1);
            }
            ToolKind::Stamp => {
                self.stamp_scale = self.stamp_scale.saturating_sub(1).max(1);
            }
            ToolKind::Eraser => {
                self.eraser_size = match self.eraser_size {
                    EraserSize::XLarge => EraserSize::Large,
                    EraserSize::Large => EraserSize::Medium,
                    EraserSize::Medium => EraserSize::Small,
                    EraserSize::Small => EraserSize::Small,
                };
            }
            ToolKind::Airbrush => {
                self.airbrush_size = match self.airbrush_size {
                    AirbrushSize::Large => AirbrushSize::Medium,
                    AirbrushSize::Medium => AirbrushSize::Small,
                    AirbrushSize::Small => AirbrushSize::Small,
                };
            }
            ToolKind::Text => {
                self.font_size = self.font_size.saturating_sub(2).max(8);
            }
            ToolKind::Pencil => {
                self.pencil_size = self.pencil_size.saturating_sub(1).max(1);
            }
            ToolKind::Line | ToolKind::Curve if self.line_width > 1 => {
                self.line_width -= 1;
            }
            _ => {}
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, symmetry_mode: &mut SymmetryMode, show_grid: &mut bool) {
        ui.vertical_centered(|ui| {
            ui.add_space(2.0);

            // Tool set mode switch: Classic XP vs Kid Pix
            ui.horizontal(|ui| {
                let classic_selected = self.mode == ToolboxMode::Classic;
                if ui
                    .selectable_label(classic_selected, egui::RichText::new("🎨 XP").size(10.5))
                    .on_hover_text("Classic Windows XP Tools")
                    .clicked()
                {
                    self.mode = ToolboxMode::Classic;
                }
                let kidpix_selected = self.mode == ToolboxMode::KidPix;
                if ui
                    .selectable_label(
                        kidpix_selected,
                        egui::RichText::new("🤪 Kid Pix").size(10.5),
                    )
                    .on_hover_text("Kid Pix Wacky Tools & Rubber Stamps")
                    .clicked()
                {
                    self.mode = ToolboxMode::KidPix;
                    if self.active_tool == ToolKind::Brush || self.active_tool == ToolKind::Pencil {
                        self.active_tool = ToolKind::WackyBrush;
                    }
                }
            });

            ui.add_space(2.0);

            // Tools 2-column Grid
            egui::Grid::new("toolbox_grid")
                .num_columns(2)
                .spacing([3.0, 3.0])
                .show(ui, |ui| {
                    let classic_items = [
                        (ToolKind::FreeformSelect, "✂ Free", "Free-Form Select"),
                        (ToolKind::Select, "⬚ Select", "Select Rectangular Area"),
                        (ToolKind::Move, "✋ Move", "Move Layer Content (V / H)"),
                        (ToolKind::Eraser, "⌫ Erase", "Eraser / Color Eraser"),
                        (ToolKind::Fill, "🪣 Fill", "Fill With Color"),
                        (ToolKind::PickColor, "💧 Pick", "Pick Color (Pipette)"),
                        (ToolKind::Magnifier, "🔍 Zoom", "Magnifier"),
                        (ToolKind::Pencil, "✏ Pencil", "Pencil (1px freeform)"),
                        (ToolKind::Brush, "🖌 Brush", "Brush"),
                        (ToolKind::Airbrush, "💨 Spray", "Airbrush Spray"),
                        (ToolKind::Text, "A Text", "Text Box"),
                        (ToolKind::Line, "╱ Line", "Straight Line"),
                        (ToolKind::Curve, "∿ Curve", "Curved Line (3-step)"),
                        (ToolKind::Rectangle, "▭ Rect", "Rectangle"),
                        (ToolKind::Polygon, "⬡ Poly", "Polygon"),
                        (ToolKind::Ellipse, "◯ Oval", "Ellipse / Circle"),
                        (ToolKind::RoundedRectangle, "▢ Round", "Rounded Rectangle"),
                    ];

                    let kidpix_items = [
                        (
                            ToolKind::WackyBrush,
                            "🪄 Wacky",
                            "Wacky Brush (12 Fun Creative Tips)",
                        ),
                        (
                            ToolKind::Stamp,
                            "🌟 Stamp",
                            "Rubber Stamps (16 Retro Pixel Stamps)",
                        ),
                        (
                            ToolKind::PatternFill,
                            "🪣 P-Fill",
                            "Pattern Fill (Retro Flood Fill)",
                        ),
                        (
                            ToolKind::ElectricMixer,
                            "⚡ Mixer",
                            "Electric Mixer (28 Wild Tray Effects)",
                        ),
                        (ToolKind::Eraser, "⌫ Erase", "Eraser / Color Eraser"),
                    ];

                    let tool_items: &[(ToolKind, &str, &str)] = match self.mode {
                        ToolboxMode::Classic => &classic_items,
                        ToolboxMode::KidPix => &kidpix_items,
                    };

                    for (idx, (tool, label, tooltip)) in tool_items.iter().enumerate() {
                        let is_selected = self.active_tool == *tool;
                        let btn = egui::Button::new(egui::RichText::new(*label).size(11.0))
                            .selected(is_selected);

                        if ui
                            .add_sized([52.0, 24.0], btn)
                            .on_hover_text(*tooltip)
                            .clicked()
                        {
                            self.active_tool = *tool;
                        }

                        if idx % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(2.0);

            // Tool options area (classic Windows XP sunken box)
            // Note: Fill and Text controls are exclusively on the top horizontal toolbar now.
            let options_rect_size = match self.active_tool {
                ToolKind::ElectricMixer => Vec2::new(108.0, 192.0),
                ToolKind::Stamp => Vec2::new(108.0, 172.0),
                ToolKind::WackyBrush => Vec2::new(108.0, 150.0),
                _ => Vec2::new(108.0, 76.0),
            };
            let (rect, _response) = ui.allocate_exact_size(options_rect_size, egui::Sense::hover());

            // Sunken border style
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(240));
            ui.painter().rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0_f32, Color32::from_gray(160)),
                egui::StrokeKind::Inside,
            );

            // Render tool-specific options inside the options box
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
            match self.active_tool {
                ToolKind::Line | ToolKind::Curve => {
                    self.render_line_width_options(&mut child_ui, rect);
                }
                ToolKind::Eraser => {
                    self.render_eraser_options(&mut child_ui, rect);
                }
                ToolKind::Brush => {
                    self.render_brush_options(&mut child_ui, rect);
                }
                ToolKind::Airbrush => {
                    self.render_airbrush_options(&mut child_ui, rect);
                }
                ToolKind::Rectangle
                | ToolKind::Polygon
                | ToolKind::Ellipse
                | ToolKind::RoundedRectangle => {
                    self.render_shape_fill_options(&mut child_ui, rect);
                }
                ToolKind::Select | ToolKind::FreeformSelect => {
                    self.render_selection_mode_options(&mut child_ui, rect);
                }
                ToolKind::Magnifier => {
                    self.render_magnifier_options(&mut child_ui, rect);
                }
                ToolKind::WackyBrush => {
                    self.render_wacky_brush_options(&mut child_ui, rect);
                }
                ToolKind::Stamp => {
                    self.render_stamp_options(&mut child_ui, rect);
                }
                ToolKind::PatternFill => {
                    self.render_pattern_fill_options(&mut child_ui, rect);
                }
                ToolKind::ElectricMixer => {
                    self.render_mixer_options(&mut child_ui, rect);
                }
                _ => {
                    // Empty options box for pencil, text, fill, move, pick color
                }
            }

            if self.mode == ToolboxMode::KidPix {
                ui.add_space(4.0);
                let label = if self.precision_mode {
                    "⚙ Affiner ▲"
                } else {
                    "⚙ Affiner ▼"
                };
                let btn = egui::Button::new(egui::RichText::new(label).size(10.0))
                    .selected(self.precision_mode);
                if ui
                    .add_sized([ui.available_width(), 19.0], btn)
                    .on_hover_text("Mode Minutieux / Précision : réglages fins pour le pixel art")
                    .clicked()
                {
                    self.precision_mode = !self.precision_mode;
                }

                if self.precision_mode {
                    self.render_kid_pix_precision_subpanel(ui);
                }
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(2.0);

            // Symmetry section
            ui.label(egui::RichText::new("Symmetry").size(11.0).strong());
            ui.add_space(2.0);
            egui::Grid::new("symmetry_grid")
                .num_columns(4)
                .spacing([2.0, 2.0])
                .show(ui, |ui| {
                    let modes = [
                        (SymmetryMode::None, "✕", "Symmetry: Off"),
                        (
                            SymmetryMode::Horizontal,
                            "↔",
                            "Horizontal Symmetry (Left / Right)",
                        ),
                        (
                            SymmetryMode::Vertical,
                            "↕",
                            "Vertical Symmetry (Top / Bottom)",
                        ),
                        (SymmetryMode::Quad, "✛", "4-Way Quad Symmetry"),
                    ];
                    for (mode, icon, tooltip) in modes {
                        let is_selected = *symmetry_mode == mode;
                        let btn = egui::Button::new(egui::RichText::new(icon).size(12.0))
                            .selected(is_selected);
                        if ui
                            .add_sized([25.0, 22.0], btn)
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            *symmetry_mode = mode;
                        }
                    }
                });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(2.0);

            // Quick toggles / Helpers
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.checkbox(show_grid, "🖽 Grid (Ctrl+G)")
                    .on_hover_text("Toggle pixel grid overlay");
                if self.active_tool == ToolKind::Eraser {
                    ui.checkbox(&mut self.erase_to_transparent, "Transparent")
                        .on_hover_text("Erase to transparent alpha instead of background color");
                }
                if self.active_tool == ToolKind::WackyBrush {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Size:").size(10.0));
                        ui.add(egui::Slider::new(&mut self.wacky_brush_size, 1..=24).text(""));
                    });
                }
                ui.checkbox(&mut self.anti_aliasing, "Anti-aliasing")
                    .on_hover_text(
                        "Toggle anti-aliasing (disabled by default for XP pixel fidelity)",
                    );
            });
        });
    }

    fn render_line_width_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let widths = [1, 2, 3, 4, 5];
        let item_height = parent_rect.height() / 5.0;

        for (i, &w) in widths.iter().enumerate() {
            let item_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + 2.0,
                    parent_rect.min.y + i as f32 * item_height,
                ),
                Vec2::new(parent_rect.width() - 4.0, item_height),
            );

            let is_selected = self.line_width == w;
            let response = ui.allocate_rect(item_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            let line_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::BLACK
            };

            let line_center_y = item_rect.center().y;
            let p1 = Pos2::new(item_rect.min.x + 10.0, line_center_y);
            let p2 = Pos2::new(item_rect.max.x - 10.0, line_center_y);

            ui.painter()
                .line_segment([p1, p2], Stroke::new(w as f32, line_color));

            if response.clicked() {
                self.line_width = w;
            }
        }
    }

    fn render_eraser_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let sizes = EraserSize::ALL;
        let item_height = parent_rect.height() / 4.0;

        for (i, &size) in sizes.iter().enumerate() {
            let item_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + 2.0,
                    parent_rect.min.y + i as f32 * item_height,
                ),
                Vec2::new(parent_rect.width() - 4.0, item_height),
            );

            let is_selected = self.eraser_size == size;
            let response = ui.allocate_rect(item_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            let border_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::BLACK
            };

            let sq_size = size.size_in_pixels() as f32 * 1.5;
            let sq_rect = Rect::from_center_size(item_rect.center(), Vec2::splat(sq_size));
            ui.painter().rect_filled(sq_rect, 0.0, Color32::WHITE);
            ui.painter().rect_stroke(
                sq_rect,
                0.0,
                Stroke::new(1.0_f32, border_color),
                egui::StrokeKind::Inside,
            );

            if response.clicked() {
                self.eraser_size = size;
            }
        }
    }

    fn render_brush_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let cols = 3;
        let rows = 4;
        let margin = 2.0;
        let cell_w = (parent_rect.width() - margin * 2.0) / cols as f32;
        let cell_h = (parent_rect.height() - margin * 2.0) / rows as f32;

        for (i, &shape) in BrushShape::ALL.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;

            let cell_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + margin + col as f32 * cell_w,
                    parent_rect.min.y + margin + row as f32 * cell_h,
                ),
                Vec2::new(cell_w, cell_h),
            );

            let is_selected = self.brush_shape == shape;
            let response = ui.allocate_rect(cell_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(cell_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(cell_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            let stroke_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::BLACK
            };

            let center = cell_rect.center();

            match shape {
                BrushShape::RoundLarge => {
                    ui.painter().circle_filled(center, 4.0, stroke_color);
                }
                BrushShape::RoundMedium => {
                    ui.painter().circle_filled(center, 2.5, stroke_color);
                }
                BrushShape::RoundSmall => {
                    ui.painter().circle_filled(center, 1.2, stroke_color);
                }
                BrushShape::SquareLarge => {
                    let r = Rect::from_center_size(center, Vec2::splat(8.0));
                    ui.painter().rect_filled(r, 0.0, stroke_color);
                }
                BrushShape::SquareMedium => {
                    let r = Rect::from_center_size(center, Vec2::splat(5.0));
                    ui.painter().rect_filled(r, 0.0, stroke_color);
                }
                BrushShape::SquareSmall => {
                    let r = Rect::from_center_size(center, Vec2::splat(2.5));
                    ui.painter().rect_filled(r, 0.0, stroke_color);
                }
                BrushShape::ForwardSlashLarge => {
                    let p1 = Pos2::new(center.x - 4.0, center.y + 4.0);
                    let p2 = Pos2::new(center.x + 4.0, center.y - 4.0);
                    ui.painter()
                        .line_segment([p1, p2], Stroke::new(2.2_f32, stroke_color));
                }
                BrushShape::ForwardSlashMedium => {
                    let p1 = Pos2::new(center.x - 2.5, center.y + 2.5);
                    let p2 = Pos2::new(center.x + 2.5, center.y - 2.5);
                    ui.painter()
                        .line_segment([p1, p2], Stroke::new(1.8_f32, stroke_color));
                }
                BrushShape::ForwardSlashSmall => {
                    let p1 = Pos2::new(center.x - 1.5, center.y + 1.5);
                    let p2 = Pos2::new(center.x + 1.5, center.y - 1.5);
                    ui.painter()
                        .line_segment([p1, p2], Stroke::new(1.2_f32, stroke_color));
                }
                BrushShape::BackslashLarge => {
                    let p1 = Pos2::new(center.x - 4.0, center.y - 4.0);
                    let p2 = Pos2::new(center.x + 4.0, center.y + 4.0);
                    ui.painter()
                        .line_segment([p1, p2], Stroke::new(2.2_f32, stroke_color));
                }
                BrushShape::BackslashMedium => {
                    let p1 = Pos2::new(center.x - 2.5, center.y - 2.5);
                    let p2 = Pos2::new(center.x + 2.5, center.y + 2.5);
                    ui.painter()
                        .line_segment([p1, p2], Stroke::new(1.8_f32, stroke_color));
                }
                BrushShape::BackslashSmall => {
                    let p1 = Pos2::new(center.x - 1.5, center.y - 1.5);
                    let p2 = Pos2::new(center.x + 1.5, center.y + 1.5);
                    ui.painter()
                        .line_segment([p1, p2], Stroke::new(1.2_f32, stroke_color));
                }
            }

            if response.clicked() {
                self.brush_shape = shape;
            }
        }
    }

    fn render_airbrush_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let sizes = AirbrushSize::ALL;
        let item_height = parent_rect.height() / 3.0;

        for (i, &size) in sizes.iter().enumerate() {
            let item_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + 2.0,
                    parent_rect.min.y + i as f32 * item_height,
                ),
                Vec2::new(parent_rect.width() - 4.0, item_height),
            );

            let is_selected = self.airbrush_size == size;
            let response = ui.allocate_rect(item_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            let dot_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::BLACK
            };

            let r = size.radius() as f32;
            let center = item_rect.center();

            // Draw a spray icon of small dots
            let dot_count = size.dots_per_spray();
            for d in 0..dot_count {
                let angle = (d as f32 / dot_count as f32) * std::f32::consts::TAU;
                let rad = ((d % 3 + 1) as f32 / 3.0) * r;
                let px = center.x + rad * angle.cos();
                let py = center.y + rad * angle.sin();
                ui.painter().rect_filled(
                    Rect::from_min_size(Pos2::new(px, py), Vec2::splat(1.5)),
                    0.0,
                    dot_color,
                );
            }

            if response.clicked() {
                self.airbrush_size = size;
            }
        }
    }

    fn render_shape_fill_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let styles = [
            (FillStyle::OutlineOnly, "Outline"),
            (FillStyle::FilledWithOutline, "Fill+Line"),
            (FillStyle::FillOnly, "Fill Only"),
        ];
        let item_height = parent_rect.height() / 3.0;

        for (i, (style, _name)) in styles.iter().enumerate() {
            let item_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + 2.0,
                    parent_rect.min.y + i as f32 * item_height,
                ),
                Vec2::new(parent_rect.width() - 4.0, item_height),
            );

            let is_selected = self.fill_style == *style;
            let response = ui.allocate_rect(item_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            let box_rect = Rect::from_center_size(item_rect.center(), Vec2::new(26.0, 16.0));
            match style {
                FillStyle::OutlineOnly => {
                    ui.painter().rect_stroke(
                        box_rect,
                        0.0,
                        Stroke::new(
                            1.5_f32,
                            if is_selected {
                                Color32::WHITE
                            } else {
                                Color32::BLACK
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                }
                FillStyle::FilledWithOutline => {
                    ui.painter().rect_filled(
                        box_rect,
                        0.0,
                        if is_selected {
                            Color32::from_gray(180)
                        } else {
                            Color32::from_gray(160)
                        },
                    );
                    ui.painter().rect_stroke(
                        box_rect,
                        0.0,
                        Stroke::new(
                            1.5_f32,
                            if is_selected {
                                Color32::WHITE
                            } else {
                                Color32::BLACK
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                }
                FillStyle::FillOnly => {
                    ui.painter().rect_filled(
                        box_rect,
                        0.0,
                        if is_selected {
                            Color32::WHITE
                        } else {
                            Color32::from_gray(140)
                        },
                    );
                }
            }

            if response.clicked() {
                self.fill_style = *style;
            }
        }
    }

    fn render_selection_mode_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let modes = [
            (SelectionMode::Opaque, "Opaque"),
            (SelectionMode::Transparent, "Transparent"),
        ];
        let item_height = parent_rect.height() / 2.0;

        for (i, (mode, name)) in modes.iter().enumerate() {
            let item_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + 2.0,
                    parent_rect.min.y + i as f32 * item_height,
                ),
                Vec2::new(parent_rect.width() - 4.0, item_height),
            );

            let is_selected = self.selection_mode == *mode;
            let response = ui.allocate_rect(item_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            let text_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::BLACK
            };

            ui.painter().text(
                item_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(12.0),
                text_color,
            );

            if response.clicked() {
                self.selection_mode = *mode;
            }
        }
    }

    fn render_magnifier_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let zoom_levels = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0];
        let item_height = parent_rect.height() / zoom_levels.len() as f32;

        for (i, &z) in zoom_levels.iter().enumerate() {
            let item_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + 2.0,
                    parent_rect.min.y + i as f32 * item_height,
                ),
                Vec2::new(parent_rect.width() - 4.0, item_height),
            );

            let is_selected = (self.magnifier_zoom - z).abs() < 0.1;
            let response = ui.allocate_rect(item_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(item_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            let text_color = if is_selected {
                Color32::WHITE
            } else {
                Color32::BLACK
            };

            ui.painter().text(
                item_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}x", z as u32),
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                self.magnifier_zoom = z;
            }
        }
    }

    fn render_wacky_brush_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let cols = 3;
        let rows = 6;
        let margin = 2.0;
        let cell_w = (parent_rect.width() - margin * 2.0) / cols as f32;
        let cell_h = (parent_rect.height() - margin * 2.0) / rows as f32;

        for (i, &shape) in WackyBrushShape::ALL.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;

            let cell_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + margin + col as f32 * cell_w,
                    parent_rect.min.y + margin + row as f32 * cell_h,
                ),
                Vec2::new(cell_w, cell_h),
            );

            let is_selected = self.wacky_brush_shape == shape;
            let response = ui.allocate_rect(cell_rect, egui::Sense::click());

            if is_selected {
                ui.painter()
                    .rect_filled(cell_rect, 0.0, Color32::from_rgb(0, 102, 204));
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(cell_rect, 0.0, Color32::from_rgb(220, 230, 245));
            }

            draw_wacky_brush_icon(ui.painter(), cell_rect.center(), shape, is_selected);

            if response.clicked() {
                self.wacky_brush_shape = shape;
            }
            response.on_hover_text(shape.display_name());
        }
    }

    fn render_stamp_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let cols = 4;
        let rows = 4;
        let margin = 2.0;
        let grid_w = parent_rect.width() - margin * 2.0;
        let cell_w = grid_w / cols as f32;
        let cell_h = 24.0_f32;

        for (i, &stamp) in StampShape::ALL.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;

            let cell_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + margin + col as f32 * cell_w,
                    parent_rect.min.y + margin + row as f32 * cell_h,
                ),
                Vec2::new(cell_w, cell_h),
            );

            let is_selected = self.stamp_shape == stamp;
            let response = ui.allocate_rect(cell_rect, egui::Sense::click());

            let bg_color = if is_selected {
                Color32::from_rgb(0, 102, 204)
            } else if response.hovered() {
                Color32::from_rgb(220, 235, 255)
            } else {
                Color32::from_rgb(250, 250, 250)
            };
            let stroke_color = if is_selected {
                Color32::from_rgb(0, 60, 140)
            } else {
                Color32::from_gray(190)
            };
            ui.painter().rect_filled(cell_rect, 1.0, bg_color);
            ui.painter().rect_stroke(
                cell_rect,
                1.0,
                Stroke::new(1.0_f32, stroke_color),
                egui::StrokeKind::Inside,
            );

            // Draw crisp pixel art stamp bitmap in retro color
            let bitmap = stamp.bitmap();
            let pixel_scale = 1.35_f32;
            let start_x = (cell_rect.center().x - (12.0 * pixel_scale) / 2.0).round();
            let start_y = (cell_rect.center().y - (12.0 * pixel_scale) / 2.0).round();
            let pixel_color = if is_selected {
                Color32::WHITE
            } else {
                stamp_color(stamp)
            };

            for (r, row_str) in bitmap.iter().enumerate() {
                for (c, ch) in row_str.chars().enumerate() {
                    if ch == 'X' {
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                Pos2::new(
                                    start_x + c as f32 * pixel_scale,
                                    start_y + r as f32 * pixel_scale,
                                ),
                                Vec2::splat(pixel_scale),
                            ),
                            0.0,
                            pixel_color,
                        );
                    }
                }
            }

            if response.clicked() {
                self.stamp_shape = stamp;
            }
            response.on_hover_text(stamp.display_name());
        }

        // Direct Scale & Rotation controls under the stamp grid
        let controls_y = parent_rect.min.y + margin + rows as f32 * cell_h + 3.0;
        let controls_rect = Rect::from_min_size(
            Pos2::new(parent_rect.min.x + 2.0, controls_y),
            Vec2::new(parent_rect.width() - 4.0, parent_rect.max.y - controls_y),
        );
        let mut controls_ui = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect));
        controls_ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Scale:").size(10.0));
                for &s in &[1, 2, 3, 4] {
                    if ui
                        .selectable_label(
                            self.stamp_scale == s,
                            egui::RichText::new(format!("{}x", s)).size(9.0),
                        )
                        .clicked()
                    {
                        self.stamp_scale = s;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Rot:").size(10.0));
                for &deg in &[0, 90, 180, 270] {
                    let is_sel = self.stamp_rotation == deg;
                    if ui
                        .selectable_label(
                            is_sel,
                            egui::RichText::new(format!("{}°", deg)).size(9.0),
                        )
                        .clicked()
                    {
                        self.stamp_rotation = deg;
                    }
                }
            });
        });
    }

    fn render_pattern_fill_options(&mut self, ui: &mut Ui, parent_rect: Rect) {
        let cols = 4;
        let rows = 2;
        let margin = 3.0;
        let cell_w = (parent_rect.width() - margin * 2.0) / cols as f32;
        let cell_h = (parent_rect.height() - margin * 2.0) / rows as f32;

        for (i, &pat) in FillPattern::ALL.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;

            let cell_rect = Rect::from_min_size(
                Pos2::new(
                    parent_rect.min.x + margin + col as f32 * cell_w,
                    parent_rect.min.y + margin + row as f32 * cell_h,
                ),
                Vec2::new(cell_w, cell_h),
            );

            let is_selected = self.fill_pattern == pat;
            let response = ui.allocate_rect(cell_rect, egui::Sense::click());

            let bg_panel = if is_selected {
                Color32::from_rgb(0, 102, 204)
            } else if response.hovered() {
                Color32::from_rgb(220, 230, 245)
            } else {
                Color32::from_gray(235)
            };
            ui.painter().rect_filled(cell_rect, 1.0, bg_panel);

            // Draw real 20x16 pixel pattern swatch with 1px border
            let swatch_w = 18;
            let swatch_h = 14;
            let start_x = (cell_rect.center().x - (swatch_w as f32 / 2.0)).round();
            let start_y = (cell_rect.center().y - (swatch_h as f32 / 2.0)).round();

            let swatch_rect = Rect::from_min_size(
                Pos2::new(start_x - 1.0, start_y - 1.0),
                Vec2::new(swatch_w as f32 + 2.0, swatch_h as f32 + 2.0),
            );
            ui.painter().rect_stroke(
                swatch_rect,
                0.0,
                Stroke::new(
                    1.0_f32,
                    if is_selected {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(120)
                    },
                ),
                egui::StrokeKind::Outside,
            );

            let fg_c = if is_selected {
                ColorRgba::rgb(255, 255, 255)
            } else {
                ColorRgba::rgb(10, 10, 10)
            };
            let bg_c = if is_selected {
                ColorRgba::rgb(0, 90, 190)
            } else {
                ColorRgba::rgb(245, 245, 245)
            };

            for py in 0..swatch_h {
                for px in 0..swatch_w {
                    let col = pat.sample(px, py, fg_c, bg_c);
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            Pos2::new(start_x + px as f32, start_y + py as f32),
                            Vec2::splat(1.0),
                        ),
                        0.0,
                        Color32::from_rgba_unmultiplied(col.r, col.g, col.b, col.a),
                    );
                }
            }

            if response.clicked() {
                self.fill_pattern = pat;
            }
            response.on_hover_text(pat.display_name());
        }
    }

    fn render_mixer_options(&mut self, ui: &mut Ui, _parent_rect: Rect) {
        ui.vertical_centered(|ui| {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("⚡ Electric Mixer")
                    .size(10.0)
                    .color(Color32::from_rgb(40, 40, 50))
                    .strong(),
            );
            ui.add_space(2.0);

            // Tray selector: Tray 1 / Tray 2
            ui.horizontal(|ui| {
                let is_tray1 = self.mixer_tray == MixerTray::Tray1;
                if ui
                    .selectable_label(is_tray1, egui::RichText::new("Tray 1").size(9.5))
                    .on_hover_text("Tray 1: Classic mix, flips, warps & filters")
                    .clicked()
                {
                    self.mixer_tray = MixerTray::Tray1;
                }
                let is_tray2 = self.mixer_tray == MixerTray::Tray2;
                if ui
                    .selectable_label(is_tray2, egui::RichText::new("Tray 2").size(9.5))
                    .on_hover_text("Tray 2: Wacky, textured & party mixers")
                    .clicked()
                {
                    self.mixer_tray = MixerTray::Tray2;
                }
            });

            ui.add_space(2.0);

            let effects = match self.mixer_tray {
                MixerTray::Tray1 => MixerEffect::TRAY_1.as_slice(),
                MixerTray::Tray2 => MixerEffect::TRAY_2.as_slice(),
            };

            egui::Grid::new("mixer_options_grid")
                .num_columns(2)
                .spacing([2.0, 2.0])
                .show(ui, |ui| {
                    for (idx, &effect) in effects.iter().enumerate() {
                        let is_active = self.active_mixer_effect == effect;
                        let label = format!("{} {}", effect.icon(), effect.short_name());
                        let btn = egui::Button::new(egui::RichText::new(label).size(8.5))
                            .selected(is_active);
                        if ui
                            .add_sized([51.0, 20.0], btn)
                            .on_hover_text(format!(
                                "{}: {}",
                                effect.display_name(),
                                effect.tooltip()
                            ))
                            .clicked()
                        {
                            self.active_mixer_effect = effect;
                        }
                        if idx % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn render_kid_pix_precision_subpanel(&mut self, ui: &mut Ui) {
        ui.add_space(3.0);
        ui.separator();
        ui.add_space(2.0);

        match self.active_tool {
            ToolKind::WackyBrush => {
                ui.label(
                    egui::RichText::new("Wacky Précision")
                        .size(9.5)
                        .strong()
                        .color(Color32::from_rgb(0, 102, 204)),
                );
                egui::Grid::new("wacky_precision_grid")
                    .num_columns(2)
                    .spacing([4.0, 3.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Taille:").size(9.0));
                        ui.add(
                            egui::DragValue::new(&mut self.wacky_brush_size)
                                .range(1..=64)
                                .suffix("px"),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Pas:").size(9.0));
                        ui.horizontal(|ui| {
                            if ui.selectable_label(!self.wacky_continuous, "Déf").clicked() {
                                self.wacky_continuous = false;
                            }
                            if ui.selectable_label(self.wacky_continuous, "1px").clicked() {
                                self.wacky_continuous = true;
                            }
                        });
                        ui.end_row();

                        if !self.wacky_continuous {
                            ui.label(egui::RichText::new("Écart:").size(9.0));
                            ui.add(
                                egui::DragValue::new(&mut self.wacky_spacing)
                                    .range(1..=100)
                                    .suffix("px"),
                            );
                            ui.end_row();
                        }

                        ui.label(egui::RichText::new("Dispersion:").size(9.0));
                        ui.add(
                            egui::DragValue::new(&mut self.wacky_scatter_radius)
                                .range(0..=48)
                                .suffix("px"),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Densité:").size(9.0));
                        let mut pct = (self.wacky_density * 100.0).round() as u32;
                        if ui
                            .add(egui::DragValue::new(&mut pct).range(10..=300).suffix("%"))
                            .changed()
                        {
                            self.wacky_density = pct as f32 / 100.0;
                        }
                        ui.end_row();

                        ui.label(egui::RichText::new("Angle:").size(9.0));
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(self.wacky_auto_tangent, "Auto")
                                .clicked()
                            {
                                self.wacky_auto_tangent = true;
                            }
                            if ui
                                .selectable_label(
                                    !self.wacky_auto_tangent,
                                    format!("{}°", self.wacky_angle as i32),
                                )
                                .clicked()
                            {
                                self.wacky_auto_tangent = false;
                            }
                        });
                        ui.end_row();

                        if !self.wacky_auto_tangent {
                            ui.label(egui::RichText::new("Rot:").size(9.0));
                            ui.add(
                                egui::DragValue::new(&mut self.wacky_angle)
                                    .range(0.0..=360.0)
                                    .speed(1.0)
                                    .suffix("°"),
                            );
                            ui.end_row();
                        }

                        ui.label(egui::RichText::new("Graine:").size(9.0));
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.wacky_seed_locked, "Fixe");
                            if self.wacky_seed_locked {
                                ui.add(egui::DragValue::new(&mut self.wacky_seed_value).speed(1));
                            }
                        });
                        ui.end_row();
                    });
            }
            ToolKind::Stamp => {
                ui.label(
                    egui::RichText::new("Stamp Précision")
                        .size(9.5)
                        .strong()
                        .color(Color32::from_rgb(0, 102, 204)),
                );
                egui::Grid::new("stamp_precision_grid")
                    .num_columns(2)
                    .spacing([4.0, 3.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Échelle:").size(9.0));
                        ui.add(
                            egui::DragValue::new(&mut self.stamp_fine_scale)
                                .range(0.5..=16.0)
                                .speed(0.1)
                                .suffix("x"),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Angle:").size(9.0));
                        ui.add(
                            egui::DragValue::new(&mut self.stamp_free_rotation)
                                .range(0.0..=360.0)
                                .speed(1.0)
                                .suffix("°"),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Style:").size(9.0));
                        let modes = ["Iconic", "Mono", "2-Tone", "Contour"];
                        egui::ComboBox::from_id_salt("stamp_tint_mode_combo")
                            .selected_text(modes[self.stamp_tint_mode.min(3)])
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                for (idx, &m) in modes.iter().enumerate() {
                                    ui.selectable_value(&mut self.stamp_tint_mode, idx, m);
                                }
                            });
                        ui.end_row();

                        ui.label(egui::RichText::new("Tracé:").size(9.0));
                        ui.checkbox(&mut self.stamp_continuous_trail, "Continu");
                        ui.end_row();

                        if self.stamp_continuous_trail {
                            ui.label(egui::RichText::new("Écart:").size(9.0));
                            ui.add(
                                egui::DragValue::new(&mut self.stamp_trail_spacing)
                                    .range(2..=120)
                                    .suffix("px"),
                            );
                            ui.end_row();
                        }
                    });
            }
            ToolKind::PatternFill => {
                ui.label(
                    egui::RichText::new("Pattern Précision")
                        .size(9.5)
                        .strong()
                        .color(Color32::from_rgb(0, 102, 204)),
                );
                egui::Grid::new("pattern_precision_grid")
                    .num_columns(2)
                    .spacing([4.0, 3.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Échelle:").size(9.0));
                        ui.horizontal(|ui| {
                            for &s in &[1, 2, 4, 8] {
                                if ui
                                    .selectable_label(self.pattern_scale == s, format!("{}x", s))
                                    .clicked()
                                {
                                    self.pattern_scale = s;
                                }
                            }
                        });
                        ui.end_row();

                        ui.label(egui::RichText::new("Origine:").size(9.0));
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.pattern_offset_x).prefix("X:"));
                            ui.add(egui::DragValue::new(&mut self.pattern_offset_y).prefix("Y:"));
                        });
                        ui.end_row();
                    });
            }
            ToolKind::ElectricMixer => {
                ui.label(
                    egui::RichText::new("Mixer Précision")
                        .size(9.5)
                        .strong()
                        .color(Color32::from_rgb(0, 102, 204)),
                );
                ui.checkbox(
                    &mut self.mixer_scope_selection,
                    "Restreindre à la sélection",
                )
                .on_hover_text(
                    "Applique l'effet uniquement dans le rectangle ou lasso de sélection",
                );
            }
            _ => {}
        }
    }
}

fn draw_wacky_brush_icon(
    painter: &egui::Painter,
    center: Pos2,
    shape: WackyBrushShape,
    is_selected: bool,
) {
    let main_col = if is_selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(30, 30, 30)
    };
    match shape {
        // 1. Procedural & Geometric
        WackyBrushShape::PineTree => {
            let foliage = if is_selected {
                Color32::from_rgb(180, 255, 180)
            } else {
                Color32::from_rgb(34, 150, 60)
            };
            let trunk = if is_selected {
                Color32::from_rgb(230, 200, 180)
            } else {
                Color32::from_rgb(140, 90, 40)
            };
            // Trunk
            painter.line_segment(
                [center + Vec2::new(0.0, 3.0), center + Vec2::new(0.0, 6.0)],
                Stroke::new(2.0_f32, trunk),
            );
            // Foliage tiers
            painter.line_segment(
                [center + Vec2::new(-4.0, 3.0), center + Vec2::new(4.0, 3.0)],
                Stroke::new(1.5_f32, foliage),
            );
            painter.line_segment(
                [center + Vec2::new(-3.0, 0.0), center + Vec2::new(3.0, 0.0)],
                Stroke::new(1.5_f32, foliage),
            );
            painter.line_segment(
                [
                    center + Vec2::new(-2.0, -3.0),
                    center + Vec2::new(2.0, -3.0),
                ],
                Stroke::new(1.5_f32, foliage),
            );
            painter.circle_filled(center - Vec2::new(0.0, 5.0), 1.5, foliage);
        }
        WackyBrushShape::ConcentricCircles => {
            let red = if is_selected {
                Color32::from_rgb(255, 180, 180)
            } else {
                Color32::from_rgb(220, 40, 40)
            };
            painter.circle_stroke(center, 6.0, Stroke::new(1.2_f32, red));
            painter.circle_stroke(center, 3.5, Stroke::new(1.2_f32, main_col));
            painter.circle_filled(center, 1.5, red);
        }
        WackyBrushShape::Starburst => {
            let yellow = if is_selected {
                Color32::from_rgb(255, 255, 200)
            } else {
                Color32::from_rgb(235, 180, 0)
            };
            painter.line_segment(
                [center - Vec2::new(0.0, 6.0), center + Vec2::new(0.0, 6.0)],
                Stroke::new(1.5_f32, yellow),
            );
            painter.line_segment(
                [center - Vec2::new(6.0, 0.0), center + Vec2::new(6.0, 0.0)],
                Stroke::new(1.5_f32, yellow),
            );
            painter.line_segment(
                [center - Vec2::new(4.0, 4.0), center + Vec2::new(4.0, 4.0)],
                Stroke::new(1.2_f32, yellow),
            );
            painter.line_segment(
                [center - Vec2::new(4.0, -4.0), center + Vec2::new(4.0, -4.0)],
                Stroke::new(1.2_f32, yellow),
            );
            painter.circle_filled(center, 1.5, Color32::WHITE);
        }
        WackyBrushShape::Swirl => {
            let cyan = if is_selected {
                Color32::from_rgb(200, 255, 255)
            } else {
                Color32::from_rgb(0, 180, 220)
            };
            painter.circle_stroke(
                center + Vec2::new(-1.0, -1.0),
                4.5,
                Stroke::new(1.2_f32, cyan),
            );
            painter.line_segment(
                [center + Vec2::new(-3.0, 2.0), center + Vec2::new(2.0, 4.0)],
                Stroke::new(1.4_f32, cyan),
            );
            painter.circle_filled(center, 1.5, main_col);
        }
        WackyBrushShape::LooperRibbon => {
            let pink = if is_selected {
                Color32::from_rgb(255, 190, 220)
            } else {
                Color32::from_rgb(230, 60, 140)
            };
            painter.circle_stroke(
                center + Vec2::new(-3.0, -1.0),
                3.0,
                Stroke::new(1.2_f32, pink),
            );
            painter.circle_stroke(
                center + Vec2::new(3.0, -1.0),
                3.0,
                Stroke::new(1.2_f32, pink),
            );
            painter.line_segment(
                [center, center + Vec2::new(-4.0, 5.0)],
                Stroke::new(1.5_f32, pink),
            );
            painter.line_segment(
                [center, center + Vec2::new(4.0, 5.0)],
                Stroke::new(1.5_f32, pink),
            );
            painter.circle_filled(center, 1.5, Color32::WHITE);
        }
        WackyBrushShape::ZigZag => {
            let yellow = if is_selected {
                Color32::from_rgb(255, 255, 180)
            } else {
                Color32::from_rgb(240, 190, 0)
            };
            let pts = [
                center + Vec2::new(2.0, -6.0),
                center + Vec2::new(-3.0, -1.0),
                center + Vec2::new(2.0, -1.0),
                center + Vec2::new(-2.0, 6.0),
            ];
            for i in 0..pts.len() - 1 {
                painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.8_f32, yellow));
            }
        }

        // 2. Particle & Dynamic Ink
        WackyBrushShape::LeakyPen => {
            let blue = if is_selected {
                Color32::from_rgb(180, 220, 255)
            } else {
                Color32::from_rgb(0, 120, 220)
            };
            // Pen nib
            painter.line_segment(
                [center - Vec2::new(0.0, 6.0), center - Vec2::new(0.0, 1.0)],
                Stroke::new(2.0_f32, main_col),
            );
            // Drip drop
            painter.line_segment(
                [center + Vec2::new(0.0, 2.0), center + Vec2::new(0.0, 5.0)],
                Stroke::new(1.8_f32, blue),
            );
            painter.circle_filled(center + Vec2::new(0.0, 5.0), 2.0, blue);
        }
        WackyBrushShape::Bubbles => {
            let blue = if is_selected {
                Color32::from_rgb(200, 230, 255)
            } else {
                Color32::from_rgb(50, 140, 220)
            };
            painter.circle_stroke(
                center - Vec2::new(2.5, 1.5),
                4.0,
                Stroke::new(1.2_f32, blue),
            );
            painter.circle_stroke(
                center + Vec2::new(3.5, 2.5),
                2.5,
                Stroke::new(1.0_f32, blue),
            );
            painter.rect_filled(
                Rect::from_min_size(center - Vec2::new(4.0, 3.5), Vec2::splat(1.2)),
                0.0,
                Color32::WHITE,
            );
        }
        WackyBrushShape::Splatter => {
            let orange = if is_selected {
                Color32::from_rgb(255, 230, 150)
            } else {
                Color32::from_rgb(220, 90, 0)
            };
            painter.circle_filled(center, 2.2, orange);
            painter.circle_filled(center + Vec2::new(-4.5, -3.0), 1.2, orange);
            painter.circle_filled(center + Vec2::new(4.0, -2.5), 1.5, orange);
            painter.circle_filled(center + Vec2::new(3.0, 4.0), 1.2, orange);
            painter.circle_filled(center + Vec2::new(-3.5, 3.5), 1.4, orange);
        }
        WackyBrushShape::NorthernLights => {
            let emerald = Color32::from_rgb(0, 230, 118);
            let cyan = Color32::from_rgb(0, 229, 255);
            let magenta = Color32::from_rgb(255, 0, 128);
            painter.line_segment(
                [
                    center + Vec2::new(-4.0, -5.0),
                    center + Vec2::new(-3.0, 5.0),
                ],
                Stroke::new(1.6_f32, emerald),
            );
            painter.line_segment(
                [center + Vec2::new(0.0, -6.0), center + Vec2::new(0.0, 6.0)],
                Stroke::new(1.6_f32, cyan),
            );
            painter.line_segment(
                [center + Vec2::new(4.0, -5.0), center + Vec2::new(3.0, 5.0)],
                Stroke::new(1.6_f32, magenta),
            );
        }
        WackyBrushShape::Confetti => {
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-4.0, -4.0), Vec2::splat(2.0)),
                0.0,
                Color32::from_rgb(255, 40, 40),
            );
            painter.circle_filled(
                center + Vec2::new(3.5, -3.0),
                1.5,
                Color32::from_rgb(255, 215, 0),
            );
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-3.0, 2.5), Vec2::new(2.5, 1.5)),
                0.0,
                Color32::from_rgb(0, 220, 240),
            );
            painter.circle_filled(
                center + Vec2::new(3.0, 3.0),
                1.5,
                Color32::from_rgb(60, 220, 60),
            );
        }
        WackyBrushShape::ConnectTheDots => {
            let blue = if is_selected {
                Color32::from_rgb(180, 220, 255)
            } else {
                Color32::from_rgb(40, 100, 220)
            };
            let p1 = center + Vec2::new(-4.0, -4.0);
            let p2 = center + Vec2::new(4.0, -2.0);
            let p3 = center + Vec2::new(0.0, 4.5);
            painter.line_segment([p1, p2], Stroke::new(1.0_f32, blue));
            painter.line_segment([p2, p3], Stroke::new(1.0_f32, blue));
            painter.circle_filled(p1, 1.8, Color32::from_rgb(255, 50, 50));
            painter.circle_filled(p2, 1.8, Color32::from_rgb(50, 200, 50));
            painter.circle_filled(p3, 1.8, Color32::from_rgb(255, 200, 0));
        }

        // 3. Texture & Special
        WackyBrushShape::Rainbow => {
            let r_col = Color32::from_rgb(255, 60, 60);
            let g_col = Color32::from_rgb(50, 210, 50);
            let b_col = Color32::from_rgb(50, 120, 255);
            painter.circle_stroke(
                center + Vec2::new(0.0, 4.0),
                7.0,
                Stroke::new(1.4_f32, r_col),
            );
            painter.circle_stroke(
                center + Vec2::new(0.0, 4.0),
                5.0,
                Stroke::new(1.4_f32, g_col),
            );
            painter.circle_stroke(
                center + Vec2::new(0.0, 4.0),
                3.0,
                Stroke::new(1.4_f32, b_col),
            );
        }
        WackyBrushShape::Caterpillar => {
            let green = Color32::from_rgb(90, 200, 40);
            let dark = Color32::from_rgb(20, 80, 20);
            // Segments
            painter.circle_filled(center + Vec2::new(-4.0, 2.0), 2.5, green);
            painter.circle_stroke(
                center + Vec2::new(-4.0, 2.0),
                2.5,
                Stroke::new(1.0_f32, dark),
            );
            painter.circle_filled(center + Vec2::new(-0.5, 0.5), 2.7, green);
            painter.circle_stroke(
                center + Vec2::new(-0.5, 0.5),
                2.7,
                Stroke::new(1.0_f32, dark),
            );
            // Head
            painter.circle_filled(center + Vec2::new(3.5, -1.5), 3.0, green);
            painter.circle_stroke(
                center + Vec2::new(3.5, -1.5),
                3.0,
                Stroke::new(1.0_f32, dark),
            );
            // Eye
            painter.circle_filled(center + Vec2::new(4.5, -2.5), 1.0, Color32::WHITE);
            painter.circle_filled(center + Vec2::new(4.8, -2.5), 0.6, Color32::BLACK);
            // Antenna
            painter.line_segment(
                [center + Vec2::new(3.5, -4.0), center + Vec2::new(4.5, -6.0)],
                Stroke::new(1.0_f32, dark),
            );
        }
        WackyBrushShape::PancakeStack => {
            let cake = Color32::from_rgb(220, 160, 70);
            let crust = Color32::from_rgb(130, 70, 20);
            let butter = Color32::from_rgb(255, 230, 60);
            let syrup = Color32::from_rgb(100, 40, 10);
            // Bottom pancake
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-6.0, 2.0), Vec2::new(12.0, 2.5)),
                1.0,
                cake,
            );
            painter.rect_stroke(
                Rect::from_min_size(center + Vec2::new(-6.0, 2.0), Vec2::new(12.0, 2.5)),
                1.0,
                Stroke::new(0.8_f32, crust),
                egui::StrokeKind::Inside,
            );
            // Middle pancake
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-5.5, -0.5), Vec2::new(11.0, 2.5)),
                1.0,
                cake,
            );
            painter.rect_stroke(
                Rect::from_min_size(center + Vec2::new(-5.5, -0.5), Vec2::new(11.0, 2.5)),
                1.0,
                Stroke::new(0.8_f32, crust),
                egui::StrokeKind::Inside,
            );
            // Top pancake
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-5.0, -3.0), Vec2::new(10.0, 2.5)),
                1.0,
                cake,
            );
            // Butter
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-1.5, -5.0), Vec2::new(3.0, 2.0)),
                0.0,
                butter,
            );
            // Syrup drip
            painter.line_segment(
                [center + Vec2::new(2.5, -1.0), center + Vec2::new(2.5, 4.0)],
                Stroke::new(1.2_f32, syrup),
            );
        }
        WackyBrushShape::FuzzerFur => {
            let fur_col = if is_selected {
                Color32::from_rgb(200, 240, 255)
            } else {
                Color32::from_rgb(80, 130, 160)
            };
            painter.circle_filled(center, 2.0, fur_col);
            for i in 0..8 {
                let ang = i as f32 * std::f32::consts::PI / 4.0;
                let len = 4.0 + (i % 3) as f32 * 1.5;
                painter.line_segment(
                    [center, center + Vec2::new(ang.cos() * len, ang.sin() * len)],
                    Stroke::new(1.0_f32, fur_col),
                );
            }
        }
        WackyBrushShape::PatternRoll => {
            let s = 2.5;
            // Roller drum 2x2 checker pattern
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-s * 2.0, -s * 2.0), Vec2::splat(s)),
                0.0,
                main_col,
            );
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(0.0, -s * 2.0), Vec2::splat(s)),
                0.0,
                Color32::from_rgb(180, 180, 180),
            );
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(-s * 2.0, 0.0), Vec2::splat(s)),
                0.0,
                Color32::from_rgb(180, 180, 180),
            );
            painter.rect_filled(
                Rect::from_min_size(center + Vec2::new(0.0, 0.0), Vec2::splat(s)),
                0.0,
                main_col,
            );
            // Frame / handle
            let handle_col = Color32::from_rgb(150, 100, 50);
            painter.line_segment(
                [center + Vec2::new(1.0, 2.0), center + Vec2::new(4.5, 5.5)],
                Stroke::new(1.5_f32, handle_col),
            );
        }
        WackyBrushShape::Alphabet => {
            let text_col = if is_selected {
                Color32::from_rgb(255, 230, 80)
            } else {
                Color32::from_rgb(220, 40, 60)
            };
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "Ab",
                egui::FontId::monospace(10.0),
                text_col,
            );
        }
    }
}

/// Signature vibrant retro palette color for each rubber stamp shape.
pub fn stamp_color(shape: StampShape) -> Color32 {
    match shape {
        StampShape::Smiley => Color32::from_rgb(235, 175, 0),
        StampShape::Heart => Color32::from_rgb(230, 35, 60),
        StampShape::Star => Color32::from_rgb(230, 180, 0),
        StampShape::MusicNote => Color32::from_rgb(142, 68, 173),
        StampShape::Bomb => Color32::from_rgb(50, 50, 55),
        StampShape::Sun => Color32::from_rgb(230, 110, 0),
        StampShape::Flower => Color32::from_rgb(230, 70, 150),
        StampShape::Tree => Color32::from_rgb(34, 145, 55),
        StampShape::Cat => Color32::from_rgb(215, 115, 20),
        StampShape::Ghost => Color32::from_rgb(0, 150, 200),
        StampShape::Car => Color32::from_rgb(215, 45, 35),
        StampShape::Gift => Color32::from_rgb(155, 60, 180),
        StampShape::Lightning => Color32::from_rgb(225, 170, 0),
        StampShape::Moon => Color32::from_rgb(160, 160, 80),
        StampShape::Diamond => Color32::from_rgb(0, 180, 215),
        StampShape::ThumbsUp => Color32::from_rgb(30, 130, 210),
    }
}
