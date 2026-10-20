use crate::ui::toolbox::{stamp_color, ToolboxView};
use egui::{
    Color32, ColorImage, PointerButton, Pos2, Rect as EguiRect, Sense, Stroke, TextureHandle,
    TextureOptions, Ui, Vec2,
};
use paint_core::frames::AnimationDocument;
use paint_core::geometry::{snap_to_square, Point, Rect};
use paint_core::history::History;
use paint_core::image::PixelBuffer;
use paint_core::layers::LayerStack;
use paint_core::onion_skin::{composite_with_onion_skin, OnionSkinConfig};
use paint_core::palette::ColorRgba;
use paint_core::tools::airbrush::apply_airbrush_spray;
use paint_core::tools::brush::{apply_brush_stamp, draw_brush_stroke};
use paint_core::tools::eraser::{
    apply_color_eraser_stamp, apply_eraser_stamp, draw_color_eraser_stroke, draw_eraser_stroke,
    draw_transparent_eraser_stroke,
};
use paint_core::tools::fill::flood_fill_tolerance;
use paint_core::tools::line::{constrained_line_end, draw_line};
use paint_core::tools::mixer::{apply_mixer_effect, MixerEffect};
use paint_core::tools::pencil::{draw_pencil_point_sized, draw_pencil_stroke_sized};
use paint_core::tools::selection::{
    commit_floating_selection, lift_freeform_selection, lift_rectangular_selection,
    stamp_floating_selection, FloatingSelection, SelectionMode,
};
use paint_core::tools::shapes::{
    draw_curve_cubic, draw_ellipse, draw_polygon, draw_rectangle, draw_rounded_rectangle,
};
use paint_core::tools::text::{draw_text_rotated, render_text_to_buffer, FontFamilyChoice};
use paint_core::tools::wacky::{
    apply_stamp_with_options, apply_wacky_brush_point_with_options,
    draw_wacky_brush_stroke_with_options, pattern_flood_fill_with_options, BrushAngleMode,
    BrushSpacing, PatternFillOptions, SeedMode, StampOptions, StampSpacing, StampTint,
    WackyBrushOptions, WackyBrushShape, WackyBrushState,
};
use paint_core::tools::{FillStyle, ToolKind};
use paint_core::transform::*;

fn draw_canvas_checkerboard(painter: &egui::Painter, rect: EguiRect, grid_size: f32) {
    let cols = (rect.width() / grid_size).ceil() as usize;
    let rows = (rect.height() / grid_size).ceil() as usize;
    for r in 0..rows {
        for c in 0..cols {
            let color = if (r + c) % 2 == 0 {
                Color32::from_gray(245)
            } else {
                Color32::from_gray(215)
            };
            let min = Pos2::new(
                (rect.min.x + c as f32 * grid_size).min(rect.max.x),
                (rect.min.y + r as f32 * grid_size).min(rect.max.y),
            );
            let max = Pos2::new(
                (min.x + grid_size).min(rect.max.x),
                (min.y + grid_size).min(rect.max.y),
            );
            painter.rect_filled(EguiRect::from_min_max(min, max), 0.0, color);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymmetryMode {
    #[default]
    None,
    Horizontal,
    Vertical,
    Quad,
}

impl SymmetryMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "Symmetry: Off",
            Self::Horizontal => "Symmetry: Horizontal (Left/Right)",
            Self::Vertical => "Symmetry: Vertical (Top/Bottom)",
            Self::Quad => "Symmetry: 4-Way (Quad)",
        }
    }
}

pub fn get_mirrored_points(pt: Point, width: i32, height: i32, mode: SymmetryMode) -> Vec<Point> {
    match mode {
        SymmetryMode::None => vec![pt],
        SymmetryMode::Horizontal => {
            let p_mirror = Point::new(width - 1 - pt.x, pt.y);
            vec![pt, p_mirror]
        }
        SymmetryMode::Vertical => {
            let p_mirror = Point::new(pt.x, height - 1 - pt.y);
            vec![pt, p_mirror]
        }
        SymmetryMode::Quad => {
            let p_h = Point::new(width - 1 - pt.x, pt.y);
            let p_v = Point::new(pt.x, height - 1 - pt.y);
            let p_hv = Point::new(width - 1 - pt.x, height - 1 - pt.y);
            vec![pt, p_h, p_v, p_hv]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionHandle {
    TopLeft,
    TopCenter,
    TopRight,
    RightCenter,
    BottomRight,
    BottomCenter,
    BottomLeft,
    LeftCenter,
    Rotate,
}

pub struct CanvasView {
    pub zoom: f32,
    pub show_grid: bool,
    pub symmetry_mode: SymmetryMode,
    pub cursor_canvas_pos: Option<Point>,

    // Texture cache & cached composite
    texture: Option<TextureHandle>,
    texture_version: u64,
    last_uploaded_version: u64,
    pub cached_composite: PixelBuffer,
    last_composite_version: u64,
    last_onion_config: Option<(usize, OnionSkinConfig)>,

    // Active continuous drag/drawing state
    is_drawing: bool,
    active_button: Option<PointerButton>,
    draw_start_pt: Option<Point>,
    prev_draw_pt: Option<Point>,
    current_mouse_pt: Option<Point>,
    spray_step: u32,
    pub wacky_state: WackyBrushState,

    // Multi-step curve tool state
    curve_p0: Option<Point>,
    curve_p3: Option<Point>,
    curve_p1: Option<Point>,
    curve_step: u8, // 0: none, 1: base line drawn, 2: 1st control point, 3: 2nd control point

    // Multi-step polygon tool state
    polygon_vertices: Vec<Point>,
    is_drawing_polygon: bool,

    // Freeform lasso points
    lasso_points: Vec<Point>,

    // Active floating selection
    pub floating_selection: Option<FloatingSelection>,
    pub selection_texture: Option<TextureHandle>,
    is_dragging_selection: bool,
    selection_drag_start_origin: Option<Point>,
    selection_drag_mouse_start: Option<Point>,

    // Dynamic selection scale and rotation
    active_selection_handle: Option<SelectionHandle>,
    selection_transform_initial_sel: Option<FloatingSelection>,
    selection_rotate_start_angle: f32,
    selection_rotate_accum_deg: f32,

    // Move layer / Hand tool state
    is_dragging_layer: bool,
    layer_drag_mouse_start: Option<Point>,
    layer_drag_start_offset: Option<(i32, i32)>,

    layer_drag_before_stack: Option<LayerStack>,

    // Text tool state
    pub active_text_pos: Option<Point>,
    pub text_input_buf: String,
    pub text_font_size: u32,
    pub text_font_family: FontFamilyChoice,
    pub text_rotation: u16,
    pub text_is_opaque: bool,
    pub text_focus_requested: bool,
    text_preview_texture: Option<TextureHandle>,
    text_preview_size: (u32, u32),
    text_preview_cache_key: Option<(
        String,
        FontFamilyChoice,
        u32,
        u16,
        bool,
        ColorRgba,
        ColorRgba,
    )>,
    is_dragging_text: bool,
    text_drag_mouse_start: Option<Point>,
    text_drag_start_pos: Option<Point>,

    // Stroke undo tracking
    stroke_before_buf: Option<PixelBuffer>,
}

impl std::fmt::Debug for CanvasView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasView")
            .field("zoom", &self.zoom)
            .field("show_grid", &self.show_grid)
            .field("symmetry_mode", &self.symmetry_mode)
            .field("cursor_canvas_pos", &self.cursor_canvas_pos)
            .field("texture_version", &self.texture_version)
            .field("is_drawing", &self.is_drawing)
            .finish()
    }
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            show_grid: false,
            symmetry_mode: SymmetryMode::None,
            cursor_canvas_pos: None,
            texture: None,
            texture_version: 0,
            last_uploaded_version: u64::MAX,
            cached_composite: PixelBuffer::new(640, 480, ColorRgba::TRANSPARENT),
            last_composite_version: u64::MAX,
            last_onion_config: None,
            is_drawing: false,
            active_button: None,
            draw_start_pt: None,
            prev_draw_pt: None,
            current_mouse_pt: None,
            spray_step: 0,
            wacky_state: WackyBrushState::default(),
            curve_p0: None,
            curve_p3: None,
            curve_p1: None,
            curve_step: 0,
            polygon_vertices: Vec::new(),
            is_drawing_polygon: false,
            lasso_points: Vec::new(),
            floating_selection: None,
            selection_texture: None,
            is_dragging_selection: false,
            selection_drag_start_origin: None,
            selection_drag_mouse_start: None,
            active_selection_handle: None,
            selection_transform_initial_sel: None,
            selection_rotate_start_angle: 0.0,
            selection_rotate_accum_deg: 0.0,
            is_dragging_layer: false,
            layer_drag_mouse_start: None,
            layer_drag_start_offset: None,
            layer_drag_before_stack: None,
            active_text_pos: None,
            text_input_buf: "Text".to_string(),
            text_font_size: 16,
            text_font_family: FontFamilyChoice::Pixel8x8,
            text_rotation: 0,
            text_is_opaque: false,
            text_focus_requested: false,
            text_preview_texture: None,
            text_preview_size: (0, 0),
            text_preview_cache_key: None,
            is_dragging_text: false,
            text_drag_mouse_start: None,
            text_drag_start_pos: None,
            stroke_before_buf: None,
        }
    }
}

impl CanvasView {
    pub fn texture_version(&self) -> u64 {
        self.texture_version
    }

    pub fn mark_dirty(&mut self) {
        self.texture_version = self.texture_version.wrapping_add(1);
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 64.0);
    }

    pub fn has_floating_selection(&self) -> bool {
        self.floating_selection.is_some()
    }

    /// Checks if a canvas point is paintable given active floating selection / stencil masking.
    pub fn is_pixel_paintable(&self, pt: Point) -> bool {
        if let Some(ref sel) = self.floating_selection {
            let rect = sel.current_rect();
            if !rect.contains(pt) {
                return false;
            }
            if let Some(ref mask) = sel.mask {
                let lx = pt.x - rect.min.x;
                let ly = pt.y - rect.min.y;
                let idx = (ly * sel.width() as i32 + lx) as usize;
                return mask.get(idx).copied().unwrap_or(false);
            }
            true
        } else {
            true
        }
    }

    pub fn select_all(&mut self, stack: &mut LayerStack, bg_color: ColorRgba) {
        let w = stack.width;
        let h = stack.height;
        if w == 0 || h == 0 {
            return;
        }
        if let Some(layer) = stack.active_layer_mut() {
            if layer.locked || !layer.visible {
                return;
            }
            self.floating_selection = lift_rectangular_selection(
                &mut layer.buffer,
                Point::new(0, 0),
                Point::new(w as i32 - 1, h as i32 - 1),
                bg_color,
            );
            self.mark_dirty();
        }
    }

    pub fn clear_selection(&mut self) {
        self.floating_selection = None;
        self.selection_texture = None;
        self.mark_dirty();
    }

    pub fn copy_selection_or_canvas(&self, stack: &LayerStack) -> PixelBuffer {
        if let Some(ref sel) = self.floating_selection {
            sel.buffer.clone()
        } else if let Some(layer) = stack.active_layer() {
            layer.buffer.clone()
        } else {
            stack.composite()
        }
    }

    pub fn cut_selection_or_canvas(
        &mut self,
        stack: &mut LayerStack,
        bg_color: ColorRgba,
    ) -> PixelBuffer {
        if let Some(sel) = self.floating_selection.take() {
            self.mark_dirty();
            sel.buffer
        } else if let Some(layer) = stack.active_layer_mut() {
            if layer.locked || !layer.visible {
                return layer.buffer.clone();
            }
            let copy = layer.buffer.clone();
            layer.buffer.clear(bg_color);
            self.mark_dirty();
            copy
        } else {
            PixelBuffer::new(stack.width, stack.height, bg_color)
        }
    }

    pub fn paste_as_selection(
        &mut self,
        stack: &mut LayerStack,
        pasted: PixelBuffer,
        toolbox: &ToolboxView,
        bg_color: ColorRgba,
    ) {
        self.commit_any_selection(stack, toolbox, bg_color);
        let w = pasted.width() as i32;
        let h = pasted.height() as i32;
        self.floating_selection = Some(FloatingSelection {
            original_rect: Rect::from_points(Point::new(0, 0), Point::new(w - 1, h - 1)),
            current_origin: Point::new(0, 0),
            buffer: pasted,
            mask: None,
        });
        self.mark_dirty();
    }

    pub fn apply_flip_horizontal(&mut self, stack: &mut LayerStack, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = flip_horizontal(&sel.buffer);
        } else {
            stack.flip_horizontal(all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_flip_vertical(&mut self, stack: &mut LayerStack, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = flip_vertical(&sel.buffer);
        } else {
            stack.flip_vertical(all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_rotate_90(&mut self, stack: &mut LayerStack, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = rotate_90(&sel.buffer);
        } else {
            stack.rotate_90(all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_rotate_180(&mut self, stack: &mut LayerStack, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = rotate_180(&sel.buffer);
        } else {
            stack.rotate_180(all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_rotate_270(&mut self, stack: &mut LayerStack, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = rotate_270(&sel.buffer);
        } else {
            stack.rotate_270(all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_invert_colors(&mut self, stack: &mut LayerStack, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            invert_colors(&mut sel.buffer);
        } else {
            stack.invert_colors(all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_mixer_effect(
        &mut self,
        stack: &mut LayerStack,
        effect: MixerEffect,
        seed: u64,
        all_layers: bool,
    ) {
        if effect == MixerEffect::Spin {
            self.apply_rotate_90(stack, all_layers);
            return;
        }

        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = apply_mixer_effect(&sel.buffer, effect, seed);
        } else if all_layers {
            for layer in &mut stack.layers {
                layer.buffer = apply_mixer_effect(&layer.buffer, effect, seed);
            }
        } else if let Some(layer) = stack.active_layer_mut() {
            layer.buffer = apply_mixer_effect(&layer.buffer, effect, seed);
        }
        self.mark_dirty();
    }

    pub fn apply_stretch(&mut self, stack: &mut LayerStack, sx: u32, sy: u32, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = stretch(&sel.buffer, sx, sy);
        } else {
            stack.stretch(sx, sy, all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_skew(
        &mut self,
        stack: &mut LayerStack,
        skx: i32,
        sky: i32,
        fill: ColorRgba,
        all_layers: bool,
    ) {
        if let Some(ref mut sel) = self.floating_selection {
            sel.buffer = skew(&sel.buffer, skx, sky, fill);
        } else {
            stack.skew(skx, sky, fill, all_layers);
        }
        self.mark_dirty();
    }

    pub fn apply_resize_canvas(
        &mut self,
        stack: &mut LayerStack,
        new_w: u32,
        new_h: u32,
        fill: ColorRgba,
    ) {
        stack.resize(new_w, new_h, fill);
        self.mark_dirty();
    }

    pub fn apply_monochrome(&mut self, stack: &mut LayerStack, all_layers: bool) {
        if let Some(ref mut sel) = self.floating_selection {
            convert_to_black_and_white(&mut sel.buffer);
        } else if all_layers {
            for layer in &mut stack.layers {
                convert_to_black_and_white(&mut layer.buffer);
            }
        } else if let Some(layer) = stack.active_layer_mut() {
            convert_to_black_and_white(&mut layer.buffer);
        }
        self.mark_dirty();
    }

    pub fn commit_any_selection(
        &mut self,
        stack: &mut LayerStack,
        toolbox: &ToolboxView,
        bg_color: ColorRgba,
    ) {
        self.selection_texture = None;
        if let Some(sel) = self.floating_selection.take() {
            if let Some(layer) = stack.active_layer_mut() {
                commit_floating_selection(&mut layer.buffer, sel, toolbox.selection_mode, bg_color);
            }
            self.mark_dirty();
        }
    }

    pub fn commit_any_text(
        &mut self,
        stack: &mut LayerStack,
        history: &mut History,
        _toolbox: &ToolboxView,
        fg_color: ColorRgba,
        bg_color: ColorRgba,
    ) {
        self.text_preview_texture = None;
        self.text_preview_cache_key = None;
        self.is_dragging_text = false;
        if let Some(text_pos) = self.active_text_pos.take() {
            if !self.text_input_buf.trim().is_empty() {
                let active_layer_id = stack.active_layer_id;
                if let Some(layer) = stack.active_layer_mut() {
                    let before = layer.buffer.clone();
                    let mode = if self.text_is_opaque {
                        SelectionMode::Opaque
                    } else {
                        SelectionMode::Transparent
                    };
                    draw_text_rotated(
                        &mut layer.buffer,
                        text_pos,
                        &self.text_input_buf,
                        self.text_font_family,
                        self.text_font_size,
                        mode,
                        fg_color,
                        bg_color,
                        self.text_rotation,
                    );
                    history.record_layer_pixel_change(active_layer_id, &before, &layer.buffer);
                }
            }
            self.mark_dirty();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        stack: &mut LayerStack,
        history: &mut History,
        toolbox: &ToolboxView,
        fg_color: &mut ColorRgba,
        bg_color: &mut ColorRgba,
        onion_skin: Option<(&AnimationDocument, &OnionSkinConfig)>,
    ) {
        // Handle Ctrl + Mouse wheel zooming
        let (ctrl_pressed, scroll_delta_y) = ui.input(|i| (i.modifiers.ctrl, i.raw_scroll_delta.y));

        if ctrl_pressed && scroll_delta_y.abs() > 0.0 {
            let zoom_levels = [
                0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0,
            ];
            let current_idx = zoom_levels
                .iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| {
                    (a - self.zoom)
                        .abs()
                        .partial_cmp(&(b - self.zoom).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(1);

            if scroll_delta_y > 0.0 && current_idx + 1 < zoom_levels.len() {
                self.zoom = zoom_levels[current_idx + 1];
            } else if scroll_delta_y < 0.0 && current_idx > 0 {
                self.zoom = zoom_levels[current_idx - 1];
            }
        }

        let is_shift_down = ui.input(|i| i.modifiers.shift);
        let _is_ctrl_down = ui.input(|i| i.modifiers.ctrl);

        // Check onion skin configuration state
        let is_onion_active = onion_skin
            .as_ref()
            .map(|(_, cfg)| cfg.enabled)
            .unwrap_or(false);

        let current_onion_state = if is_onion_active {
            onion_skin.map(|(doc, cfg)| (doc.active_frame_index, (*cfg).clone()))
        } else {
            None
        };

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let avail_size = ui.available_size();
                let canvas_width = stack.width as f32 * self.zoom;
                let canvas_height = stack.height as f32 * self.zoom;
                let canvas_size = Vec2::new(canvas_width, canvas_height);

                let pad_x = ((avail_size.x - canvas_width) / 2.0).max(0.0);
                let pad_y = ((avail_size.y - canvas_height) / 2.0).max(0.0);

                if pad_y > 0.0 {
                    ui.add_space(pad_y);
                }

                let (rect, response) = ui
                    .horizontal(|ui| {
                        if pad_x > 0.0 {
                            ui.add_space(pad_x);
                        }
                        ui.allocate_exact_size(canvas_size, Sense::click_and_drag())
                    })
                    .inner;

                // Mouse coordinate tracking
                let is_actively_dragging =
                    self.is_drawing || self.is_dragging_layer || self.is_dragging_selection;
                let pointer_pos = if is_actively_dragging {
                    ui.input(|i| i.pointer.latest_pos().or(i.pointer.interact_pos()))
                        .or_else(|| response.interact_pointer_pos())
                } else {
                    response.hover_pos()
                };

                if let Some(pos) = pointer_pos {
                    let rel_x = ((pos.x - rect.min.x) / self.zoom).floor() as i32;
                    let rel_y = ((pos.y - rect.min.y) / self.zoom).floor() as i32;

                    if rel_x >= 0
                        && rel_x < stack.width as i32
                        && rel_y >= 0
                        && rel_y < stack.height as i32
                    {
                        let pt = Point::new(rel_x, rel_y);
                        self.cursor_canvas_pos = Some(pt);
                        self.current_mouse_pt = Some(pt);
                    } else {
                        self.cursor_canvas_pos = None;
                        if is_actively_dragging {
                            self.current_mouse_pt = Some(Point::new(rel_x, rel_y));
                        }
                    }
                } else {
                    self.cursor_canvas_pos = None;
                    if !is_actively_dragging {
                        self.current_mouse_pt = None;
                    }
                }

                // Handle interactive input
                let is_primary_down = response.is_pointer_button_down_on();
                let is_secondary_down =
                    ui.input(|i| i.pointer.button_down(PointerButton::Secondary));

                let active_btn = if is_primary_down {
                    Some(PointerButton::Primary)
                } else if is_secondary_down && response.hovered() {
                    Some(PointerButton::Secondary)
                } else {
                    None
                };

                let active_color = match self.active_button.or(active_btn) {
                    Some(PointerButton::Secondary) => *bg_color,
                    _ => *fg_color,
                };

                let active_layer_id = stack.active_layer_id;
                let layer_is_usable = stack
                    .active_layer()
                    .map(|l| l.visible && !l.locked)
                    .unwrap_or(false);

                // Curve cancel on Escape or tool switch
                if self.curve_step > 0
                    && (ui.input(|i| i.key_pressed(egui::Key::Escape))
                        || toolbox.active_tool != ToolKind::Curve)
                {
                    self.curve_step = 0;
                    self.curve_p0 = None;
                    self.curve_p1 = None;
                    self.curve_p3 = None;
                    self.is_drawing = false;
                    self.stroke_before_buf = None;
                    self.draw_start_pt = None;
                    self.mark_dirty();
                }

                // Polygon cancel on Escape or tool switch
                if self.is_drawing_polygon
                    && (ui.input(|i| i.key_pressed(egui::Key::Escape))
                        || toolbox.active_tool != ToolKind::Polygon)
                {
                    self.is_drawing_polygon = false;
                    self.polygon_vertices.clear();
                    self.stroke_before_buf = None;
                }

                // Text commit on tool switch
                if self.active_text_pos.is_some() && toolbox.active_tool != ToolKind::Text {
                    self.commit_any_text(stack, history, toolbox, *fg_color, *bg_color);
                }

                // Polygon complete on double click or Enter key
                if self.is_drawing_polygon
                    && (response.double_clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    && self.polygon_vertices.len() >= 3
                {
                    if let Some(layer) = stack.active_layer_mut() {
                        let before = self
                            .stroke_before_buf
                            .take()
                            .unwrap_or_else(|| layer.buffer.clone());
                        draw_polygon(
                            &mut layer.buffer,
                            &self.polygon_vertices,
                            toolbox.fill_style,
                            toolbox.line_width,
                            *fg_color,
                            *bg_color,
                        );
                        history.record_layer_pixel_change(active_layer_id, &before, &layer.buffer);
                    }
                    self.is_drawing_polygon = false;
                    self.polygon_vertices.clear();
                    self.mark_dirty();
                }

                // Mouse down / initial click
                if !self.is_drawing
                    && !self.is_dragging_layer
                    && !self.is_dragging_selection
                    && active_btn.is_some()
                {
                    if let Some(pt) = self.current_mouse_pt {
                        match toolbox.active_tool {
                            ToolKind::PickColor => {
                                if let Some(c) = self.cached_composite.get_pixel_i32(pt.x, pt.y) {
                                    if active_btn == Some(PointerButton::Primary) {
                                        *fg_color = c;
                                    } else {
                                        *bg_color = c;
                                    }
                                }
                            }
                            ToolKind::Magnifier => {
                                if active_btn == Some(PointerButton::Primary) {
                                    self.set_zoom(toolbox.magnifier_zoom);
                                } else {
                                    self.set_zoom(1.0);
                                }
                            }
                            ToolKind::Fill if layer_is_usable => {
                                self.commit_any_selection(stack, toolbox, *bg_color);
                                let (doc_w, doc_h) = (stack.width as i32, stack.height as i32);
                                if let Some(layer) = stack.active_layer_mut() {
                                    let before = layer.buffer.clone();
                                    let pts =
                                        get_mirrored_points(pt, doc_w, doc_h, self.symmetry_mode);
                                    for p in pts {
                                        flood_fill_tolerance(
                                            &mut layer.buffer,
                                            p,
                                            active_color,
                                            toolbox.fill_tolerance,
                                        );
                                    }
                                    history.record_layer_pixel_change(
                                        active_layer_id,
                                        &before,
                                        &layer.buffer,
                                    );
                                    self.mark_dirty();
                                }
                            }
                            ToolKind::PatternFill if layer_is_usable => {
                                let clip_rect = if self.floating_selection.is_some() {
                                    self.floating_selection.as_ref().map(|s| s.current_rect())
                                } else {
                                    self.commit_any_selection(stack, toolbox, *bg_color);
                                    None
                                };
                                let pattern_opt = PatternFillOptions {
                                    scale: if toolbox.precision_mode {
                                        toolbox.pattern_scale
                                    } else {
                                        1
                                    },
                                    origin: if toolbox.precision_mode {
                                        Point::new(
                                            toolbox.pattern_offset_x,
                                            toolbox.pattern_offset_y,
                                        )
                                    } else {
                                        Point::new(0, 0)
                                    },
                                    clip_rect,
                                };
                                let (doc_w, doc_h) = (stack.width as i32, stack.height as i32);
                                if let Some(layer) = stack.active_layer_mut() {
                                    let before = layer.buffer.clone();
                                    let pts =
                                        get_mirrored_points(pt, doc_w, doc_h, self.symmetry_mode);
                                    for p in pts {
                                        pattern_flood_fill_with_options(
                                            &mut layer.buffer,
                                            p,
                                            toolbox.fill_pattern,
                                            toolbox.fill_tolerance,
                                            active_color,
                                            *bg_color,
                                            &pattern_opt,
                                        );
                                    }
                                    history.record_layer_pixel_change(
                                        active_layer_id,
                                        &before,
                                        &layer.buffer,
                                    );
                                    self.mark_dirty();
                                }
                            }
                            ToolKind::Move if layer_is_usable => {
                                self.commit_any_selection(stack, toolbox, *bg_color);
                                self.commit_any_text(stack, history, toolbox, *fg_color, *bg_color);
                                if let Some(layer) = stack.active_layer() {
                                    self.is_dragging_layer = true;
                                    self.layer_drag_mouse_start = Some(pt);
                                    self.layer_drag_start_offset =
                                        Some((layer.offset_x, layer.offset_y));
                                    self.layer_drag_before_stack = Some(stack.clone());
                                }
                            }
                            ToolKind::Text if layer_is_usable => {
                                let (tw, th) = self.text_preview_size;
                                let is_hit_text = if let Some(tpos) = self.active_text_pos {
                                    let bounds = Rect::from_coords(
                                        tpos.x,
                                        tpos.y,
                                        tpos.x + tw.max(20) as i32,
                                        tpos.y + th.max(16) as i32,
                                    );
                                    bounds.contains(pt)
                                } else {
                                    false
                                };

                                if is_hit_text {
                                    self.is_dragging_text = true;
                                    self.text_drag_mouse_start = Some(pt);
                                    self.text_drag_start_pos = self.active_text_pos;
                                } else {
                                    self.commit_any_selection(stack, toolbox, *bg_color);
                                    self.commit_any_text(
                                        stack, history, toolbox, *fg_color, *bg_color,
                                    );
                                    self.active_text_pos = Some(pt);
                                    self.text_input_buf = "Text".to_string();
                                    self.text_font_size = toolbox.font_size;
                                    self.text_font_family = toolbox.font_family;
                                    self.text_rotation = toolbox.text_rotation;
                                    self.text_is_opaque =
                                        toolbox.selection_mode == SelectionMode::Opaque;
                                    self.text_focus_requested = true;
                                }
                            }
                            ToolKind::Select | ToolKind::FreeformSelect if layer_is_usable => {
                                // 1. Check if an active handle (Scale or Rotation) is clicked on floating selection
                                let handle_hit = if let (Some(ref sel), Some(screen_p)) =
                                    (&self.floating_selection, pointer_pos)
                                {
                                    let sel_min = Pos2::new(
                                        rect.min.x + sel.current_origin.x as f32 * self.zoom,
                                        rect.min.y + sel.current_origin.y as f32 * self.zoom,
                                    );
                                    let sel_max = Pos2::new(
                                        sel_min.x + sel.buffer.width() as f32 * self.zoom,
                                        sel_min.y + sel.buffer.height() as f32 * self.zoom,
                                    );
                                    let sel_rect = EguiRect::from_min_max(sel_min, sel_max);
                                    let hit_radius = 8.0_f32;

                                    let rot_pos = Pos2::new(
                                        sel_rect.center_top().x,
                                        sel_rect.center_top().y - 18.0,
                                    );
                                    if screen_p.distance(rot_pos) <= hit_radius + 2.0 {
                                        Some(SelectionHandle::Rotate)
                                    } else if screen_p.distance(sel_rect.left_top()) <= hit_radius {
                                        Some(SelectionHandle::TopLeft)
                                    } else if screen_p.distance(sel_rect.center_top()) <= hit_radius
                                    {
                                        Some(SelectionHandle::TopCenter)
                                    } else if screen_p.distance(sel_rect.right_top()) <= hit_radius
                                    {
                                        Some(SelectionHandle::TopRight)
                                    } else if screen_p.distance(sel_rect.right_center())
                                        <= hit_radius
                                    {
                                        Some(SelectionHandle::RightCenter)
                                    } else if screen_p.distance(sel_rect.right_bottom())
                                        <= hit_radius
                                    {
                                        Some(SelectionHandle::BottomRight)
                                    } else if screen_p.distance(sel_rect.center_bottom())
                                        <= hit_radius
                                    {
                                        Some(SelectionHandle::BottomCenter)
                                    } else if screen_p.distance(sel_rect.left_bottom())
                                        <= hit_radius
                                    {
                                        Some(SelectionHandle::BottomLeft)
                                    } else if screen_p.distance(sel_rect.left_center())
                                        <= hit_radius
                                    {
                                        Some(SelectionHandle::LeftCenter)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                if let Some(handle) = handle_hit {
                                    if let Some(ref sel) = self.floating_selection {
                                        if let Some(layer) = stack.active_layer_mut() {
                                            self.stroke_before_buf = Some(layer.buffer.clone());
                                        }
                                        self.is_dragging_selection = true;
                                        self.active_selection_handle = Some(handle);
                                        self.selection_transform_initial_sel = Some(sel.clone());
                                        self.selection_drag_start_origin = Some(sel.current_origin);
                                        self.selection_drag_mouse_start = Some(pt);

                                        if handle == SelectionHandle::Rotate {
                                            let center_x = sel.current_origin.x as f32
                                                + sel.buffer.width() as f32 / 2.0;
                                            let center_y = sel.current_origin.y as f32
                                                + sel.buffer.height() as f32 / 2.0;
                                            self.selection_rotate_start_angle = (pt.y as f32
                                                - center_y)
                                                .atan2(pt.x as f32 - center_x);
                                            self.selection_rotate_accum_deg = 0.0;
                                        }
                                    }
                                } else {
                                    let sel_hit = self.floating_selection.as_ref().map(|s| {
                                        (s.current_rect().contains(pt), s.current_origin, s.clone())
                                    });
                                    if let Some((is_hit, sel_origin, sel_clone)) = sel_hit {
                                        if is_hit {
                                            if let Some(layer) = stack.active_layer_mut() {
                                                self.stroke_before_buf = Some(layer.buffer.clone());
                                                if ui.input(|i| i.modifiers.ctrl) {
                                                    stamp_floating_selection(
                                                        &mut layer.buffer,
                                                        &sel_clone,
                                                        sel_origin,
                                                        toolbox.selection_mode,
                                                        *bg_color,
                                                    );
                                                    self.mark_dirty();
                                                }
                                            }
                                            self.is_dragging_selection = true;
                                            self.active_selection_handle = None;
                                            self.selection_drag_start_origin = Some(sel_origin);
                                            self.selection_drag_mouse_start = Some(pt);
                                        } else {
                                            self.commit_any_selection(stack, toolbox, *bg_color);
                                            if let Some(layer) = stack.active_layer_mut() {
                                                let before = layer.buffer.clone();
                                                history.record_layer_pixel_change(
                                                    active_layer_id,
                                                    &before,
                                                    &layer.buffer,
                                                );
                                            }

                                            if let Some(layer) = stack.active_layer() {
                                                self.stroke_before_buf = Some(layer.buffer.clone());
                                            }
                                            self.is_drawing = true;
                                            self.active_button = active_btn;
                                            self.draw_start_pt = Some(pt);
                                            self.lasso_points = vec![pt];
                                        }
                                    } else {
                                        if let Some(layer) = stack.active_layer() {
                                            self.stroke_before_buf = Some(layer.buffer.clone());
                                        }
                                        self.is_drawing = true;
                                        self.active_button = active_btn;
                                        self.draw_start_pt = Some(pt);
                                        self.lasso_points = vec![pt];
                                    }
                                }
                            }

                            ToolKind::Curve if layer_is_usable => {
                                self.commit_any_selection(stack, toolbox, *bg_color);
                                if self.curve_step == 0 {
                                    if let Some(layer) = stack.active_layer() {
                                        self.stroke_before_buf = Some(layer.buffer.clone());
                                    }
                                    self.is_drawing = true;
                                    self.active_button = active_btn;
                                    self.draw_start_pt = Some(pt);
                                } else {
                                    self.is_drawing = true;
                                    self.active_button = active_btn;
                                    self.draw_start_pt = Some(pt);
                                }
                            }
                            ToolKind::Polygon if layer_is_usable => {
                                self.commit_any_selection(stack, toolbox, *bg_color);
                                if !self.is_drawing_polygon {
                                    if let Some(layer) = stack.active_layer() {
                                        self.stroke_before_buf = Some(layer.buffer.clone());
                                    }
                                    self.is_drawing_polygon = true;
                                    self.polygon_vertices = vec![pt];
                                } else {
                                    let start = self.polygon_vertices[0];
                                    if (pt.x - start.x).abs() <= 6
                                        && (pt.y - start.y).abs() <= 6
                                        && self.polygon_vertices.len() >= 3
                                    {
                                        let (doc_w, doc_h) =
                                            (stack.width as i32, stack.height as i32);
                                        if let Some(layer) = stack.active_layer_mut() {
                                            let before = self
                                                .stroke_before_buf
                                                .take()
                                                .unwrap_or_else(|| layer.buffer.clone());
                                            let symmetry = self.symmetry_mode;
                                            let branch_count = match symmetry {
                                                SymmetryMode::None => 1,
                                                SymmetryMode::Horizontal
                                                | SymmetryMode::Vertical => 2,
                                                SymmetryMode::Quad => 4,
                                            };
                                            for b in 0..branch_count {
                                                let branch_verts: Vec<Point> = self
                                                    .polygon_vertices
                                                    .iter()
                                                    .map(|v| {
                                                        let pts = get_mirrored_points(
                                                            *v, doc_w, doc_h, symmetry,
                                                        );
                                                        pts[b]
                                                    })
                                                    .collect();
                                                draw_polygon(
                                                    &mut layer.buffer,
                                                    &branch_verts,
                                                    toolbox.fill_style,
                                                    toolbox.line_width,
                                                    *fg_color,
                                                    *bg_color,
                                                );
                                            }
                                            history.record_layer_pixel_change(
                                                active_layer_id,
                                                &before,
                                                &layer.buffer,
                                            );
                                        }
                                        self.is_drawing_polygon = false;
                                        self.polygon_vertices.clear();
                                        self.mark_dirty();
                                    } else if self.polygon_vertices.last() != Some(&pt) {
                                        self.polygon_vertices.push(pt);
                                    }
                                }
                            }
                            _ if layer_is_usable => {
                                self.commit_any_selection(stack, toolbox, *bg_color);
                                let (doc_w, doc_h) = (stack.width as i32, stack.height as i32);
                                let symmetry = self.symmetry_mode;
                                if let Some(layer) = stack.active_layer_mut() {
                                    self.stroke_before_buf = Some(layer.buffer.clone());
                                    self.is_drawing = true;
                                    self.active_button = active_btn;
                                    self.draw_start_pt = Some(pt);
                                    self.prev_draw_pt = Some(pt);
                                    if toolbox.active_tool == ToolKind::WackyBrush {
                                        self.wacky_state.reset_stroke();
                                    }

                                    let pts = get_mirrored_points(pt, doc_w, doc_h, symmetry);

                                    match toolbox.active_tool {
                                        ToolKind::Pencil => {
                                            for p in pts {
                                                draw_pencil_point_sized(
                                                    &mut layer.buffer,
                                                    p,
                                                    toolbox.pencil_size,
                                                    active_color,
                                                );
                                            }
                                            self.mark_dirty();
                                        }
                                        ToolKind::Brush => {
                                            for p in pts {
                                                apply_brush_stamp(
                                                    &mut layer.buffer,
                                                    p,
                                                    toolbox.brush_shape,
                                                    active_color,
                                                );
                                            }
                                            self.mark_dirty();
                                        }
                                        ToolKind::Eraser => {
                                            for p in pts {
                                                if toolbox.erase_to_transparent {
                                                    if self.active_button
                                                        == Some(PointerButton::Secondary)
                                                    {
                                                        apply_color_eraser_stamp(
                                                            &mut layer.buffer,
                                                            p,
                                                            toolbox.eraser_size,
                                                            *fg_color,
                                                            ColorRgba::rgba(0, 0, 0, 0),
                                                        );
                                                    } else {
                                                        apply_eraser_stamp(
                                                            &mut layer.buffer,
                                                            p,
                                                            toolbox.eraser_size,
                                                            ColorRgba::rgba(0, 0, 0, 0),
                                                        );
                                                    }
                                                } else if self.active_button
                                                    == Some(PointerButton::Secondary)
                                                {
                                                    apply_color_eraser_stamp(
                                                        &mut layer.buffer,
                                                        p,
                                                        toolbox.eraser_size,
                                                        *fg_color,
                                                        *bg_color,
                                                    );
                                                } else {
                                                    apply_eraser_stamp(
                                                        &mut layer.buffer,
                                                        p,
                                                        toolbox.eraser_size,
                                                        *bg_color,
                                                    );
                                                }
                                            }
                                            self.mark_dirty();
                                        }
                                        ToolKind::Airbrush => {
                                            self.spray_step = self.spray_step.wrapping_add(1);
                                            for p in pts {
                                                apply_airbrush_spray(
                                                    &mut layer.buffer,
                                                    p,
                                                    toolbox.airbrush_size,
                                                    active_color,
                                                    self.spray_step,
                                                );
                                            }
                                            self.mark_dirty();
                                        }
                                        ToolKind::WackyBrush => {
                                            let spacing = if toolbox.wacky_continuous {
                                                BrushSpacing::Continuous
                                            } else if toolbox.precision_mode {
                                                BrushSpacing::Custom(toolbox.wacky_spacing)
                                            } else {
                                                BrushSpacing::Discrete
                                            };
                                            let angle_mode = if toolbox.precision_mode {
                                                if toolbox.wacky_auto_tangent {
                                                    BrushAngleMode::FollowStroke
                                                } else {
                                                    BrushAngleMode::Fixed(toolbox.wacky_angle)
                                                }
                                            } else {
                                                BrushAngleMode::Fixed(0.0)
                                            };
                                            let seed_mode = if toolbox.precision_mode
                                                && toolbox.wacky_seed_locked
                                            {
                                                SeedMode::Locked(toolbox.wacky_seed_value)
                                            } else {
                                                SeedMode::Dynamic
                                            };
                                            let options = WackyBrushOptions {
                                                spacing,
                                                scatter_radius: if toolbox.precision_mode {
                                                    toolbox.wacky_scatter_radius
                                                } else {
                                                    0
                                                },
                                                particle_density: if toolbox.precision_mode {
                                                    toolbox.wacky_density
                                                } else {
                                                    1.0
                                                },
                                                burst_count_override: None,
                                                angle_mode,
                                                seed_mode,
                                            };
                                            for p in pts {
                                                if self.is_pixel_paintable(p) {
                                                    apply_wacky_brush_point_with_options(
                                                        &mut layer.buffer,
                                                        p,
                                                        toolbox.wacky_brush_shape,
                                                        toolbox.wacky_brush_size,
                                                        active_color,
                                                        *bg_color,
                                                        &mut self.wacky_state,
                                                        &options,
                                                    );
                                                }
                                            }
                                            self.mark_dirty();
                                            if toolbox.wacky_brush_shape
                                                == WackyBrushShape::LeakyPen
                                            {
                                                ui.ctx().request_repaint();
                                            }
                                        }
                                        ToolKind::Stamp => {
                                            let scale = if toolbox.precision_mode {
                                                (toolbox.stamp_fine_scale
                                                    * toolbox.stamp_scale as f32)
                                                    .round()
                                                    .max(1.0)
                                                    as u32
                                            } else {
                                                toolbox.stamp_scale
                                            };
                                            let rot = if toolbox.precision_mode {
                                                toolbox.stamp_free_rotation
                                            } else {
                                                toolbox.stamp_rotation as f32
                                            };
                                            let tint = match toolbox.stamp_tint_mode {
                                                1 => StampTint::Monochrome(active_color),
                                                2 => StampTint::TwoTone {
                                                    fg: active_color,
                                                    bg: *bg_color,
                                                },
                                                3 => StampTint::Outlined {
                                                    fill: active_color,
                                                    outline: *bg_color,
                                                },
                                                _ => StampTint::OriginalPalette,
                                            };
                                            let spacing = if toolbox.precision_mode
                                                && toolbox.stamp_continuous_trail
                                            {
                                                StampSpacing::Interval(toolbox.stamp_trail_spacing)
                                            } else {
                                                StampSpacing::Single
                                            };
                                            let stamp_opt = StampOptions {
                                                scale,
                                                rotation_deg: rot,
                                                spacing,
                                                tint,
                                            };
                                            for p in pts {
                                                if self.is_pixel_paintable(p) {
                                                    apply_stamp_with_options(
                                                        &mut layer.buffer,
                                                        p,
                                                        toolbox.stamp_shape,
                                                        &stamp_opt,
                                                    );
                                                }
                                            }
                                            self.mark_dirty();
                                        }
                                        ToolKind::ElectricMixer => {
                                            let scope = if toolbox.mixer_scope_selection {
                                                if let Some(ref sel) = self.floating_selection {
                                                    if let Some(ref mask) = sel.mask {
                                                        paint_core::tools::mixer::MixerScope::Masked {
                                                            bounds: sel.current_rect(),
                                                            mask: mask.clone(),
                                                        }
                                                    } else {
                                                        paint_core::tools::mixer::MixerScope::BoundingBox(
                                                            sel.current_rect(),
                                                        )
                                                    }
                                                } else {
                                                    paint_core::tools::mixer::MixerScope::EntireCanvas
                                                }
                                            } else {
                                                paint_core::tools::mixer::MixerScope::EntireCanvas
                                            };

                                            let mixer_opt =
                                                paint_core::tools::mixer::MixerOptions {
                                                    seed: self.wacky_state.step,
                                                    scope,
                                                    params:
                                                        paint_core::tools::mixer::MixerParams::default(
                                                        ),
                                                };
                                            layer.buffer =
                                                paint_core::tools::mixer::apply_mixer_effect_with_options(
                                                    &layer.buffer,
                                                    toolbox.active_mixer_effect,
                                                    &mixer_opt,
                                                );
                                            self.wacky_state.step =
                                                self.wacky_state.step.wrapping_add(1013904223);
                                            self.mark_dirty();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Mouse drag & active drawing
                if self.is_dragging_text {
                    if let (Some(mouse_start), Some(start_pos), Some(curr_pt)) = (
                        self.text_drag_mouse_start,
                        self.text_drag_start_pos,
                        self.current_mouse_pt,
                    ) {
                        let delta_x = curr_pt.x - mouse_start.x;
                        let delta_y = curr_pt.y - mouse_start.y;
                        self.active_text_pos =
                            Some(Point::new(start_pos.x + delta_x, start_pos.y + delta_y));
                        self.mark_dirty();
                    }
                    if !is_primary_down {
                        self.is_dragging_text = false;
                    }
                }

                if self.is_dragging_layer {
                    if let (Some(mouse_start), Some((start_ox, start_oy)), Some(curr_pt)) = (
                        self.layer_drag_mouse_start,
                        self.layer_drag_start_offset,
                        self.current_mouse_pt,
                    ) {
                        let delta_x = curr_pt.x - mouse_start.x;
                        let delta_y = curr_pt.y - mouse_start.y;
                        if let Some(layer) = stack.active_layer_mut() {
                            if layer.offset_x != start_ox + delta_x
                                || layer.offset_y != start_oy + delta_y
                            {
                                layer.offset_x = start_ox + delta_x;
                                layer.offset_y = start_oy + delta_y;
                                self.mark_dirty();
                            }
                        }
                    }
                    ui.ctx().request_repaint();

                    if !ui.input(|i| i.pointer.primary_down() || i.pointer.secondary_down()) {
                        if let Some(before) = self.layer_drag_before_stack.take() {
                            if before != *stack {
                                history.record_layer_stack_change(&before, stack);
                            }
                        }
                        self.is_dragging_layer = false;
                        self.layer_drag_mouse_start = None;
                        self.layer_drag_start_offset = None;
                        self.mark_dirty();
                    }
                } else if self.is_dragging_selection {
                    if let (Some(ref mut sel), Some(mouse_start), Some(curr_pt)) = (
                        &mut self.floating_selection,
                        self.selection_drag_mouse_start,
                        self.current_mouse_pt,
                    ) {
                        if let Some(handle) = self.active_selection_handle {
                            if let Some(ref init_sel) = self.selection_transform_initial_sel {
                                match handle {
                                    SelectionHandle::Rotate => {
                                        let center_x = init_sel.current_origin.x as f32
                                            + init_sel.buffer.width() as f32 / 2.0;
                                        let center_y = init_sel.current_origin.y as f32
                                            + init_sel.buffer.height() as f32 / 2.0;
                                        let current_angle = (curr_pt.y as f32 - center_y)
                                            .atan2(curr_pt.x as f32 - center_x);
                                        let mut delta_deg = (current_angle
                                            - self.selection_rotate_start_angle)
                                            .to_degrees();

                                        // Snap to 15 degrees if shift held
                                        if is_shift_down {
                                            delta_deg = (delta_deg / 15.0).round() * 15.0;
                                        }

                                        let rotated_buf = rotate_arbitrary(
                                            &init_sel.buffer,
                                            delta_deg,
                                            ColorRgba::TRANSPARENT,
                                        );
                                        let new_w = rotated_buf.width() as i32;
                                        let new_h = rotated_buf.height() as i32;

                                        sel.current_origin = Point::new(
                                            (center_x - new_w as f32 / 2.0).round() as i32,
                                            (center_y - new_h as f32 / 2.0).round() as i32,
                                        );
                                        sel.buffer = rotated_buf;
                                        sel.mask = None; // rotated buffer contains transparent alpha
                                        self.mark_dirty();
                                    }
                                    _ => {
                                        // Dynamic Scale via 8 handles
                                        let orig_w = init_sel.buffer.width() as i32;
                                        let orig_h = init_sel.buffer.height() as i32;
                                        let init_x0 = init_sel.current_origin.x;
                                        let init_y0 = init_sel.current_origin.y;
                                        let init_x1 = init_x0 + orig_w;
                                        let init_y1 = init_y0 + orig_h;

                                        let delta_x = curr_pt.x - mouse_start.x;
                                        let delta_y = curr_pt.y - mouse_start.y;

                                        let (mut new_x0, mut new_y0, mut new_x1, mut new_y1) =
                                            (init_x0, init_y0, init_x1, init_y1);

                                        match handle {
                                            SelectionHandle::TopLeft => {
                                                new_x0 += delta_x;
                                                new_y0 += delta_y;
                                            }
                                            SelectionHandle::TopCenter => {
                                                new_y0 += delta_y;
                                            }
                                            SelectionHandle::TopRight => {
                                                new_x1 += delta_x;
                                                new_y0 += delta_y;
                                            }
                                            SelectionHandle::RightCenter => {
                                                new_x1 += delta_x;
                                            }
                                            SelectionHandle::BottomRight => {
                                                new_x1 += delta_x;
                                                new_y1 += delta_y;
                                            }
                                            SelectionHandle::BottomCenter => {
                                                new_y1 += delta_y;
                                            }
                                            SelectionHandle::BottomLeft => {
                                                new_x0 += delta_x;
                                                new_y1 += delta_y;
                                            }
                                            SelectionHandle::LeftCenter => {
                                                new_x0 += delta_x;
                                            }
                                            SelectionHandle::Rotate => {}
                                        }

                                        let mut w = (new_x1 - new_x0).max(2);
                                        let mut h = (new_y1 - new_y0).max(2);

                                        if is_shift_down {
                                            // Proportional scale
                                            let scale = (w as f32 / orig_w as f32)
                                                .max(h as f32 / orig_h as f32);
                                            w = ((orig_w as f32 * scale).round() as i32).max(2);
                                            h = ((orig_h as f32 * scale).round() as i32).max(2);
                                        }

                                        let pct_x =
                                            ((w as u64 * 100) / orig_w as u64).max(1) as u32;
                                        let pct_y =
                                            ((h as u64 * 100) / orig_h as u64).max(1) as u32;

                                        sel.current_origin = Point::new(new_x0, new_y0);
                                        sel.buffer = stretch(&init_sel.buffer, pct_x, pct_y);
                                        if let Some(ref orig_mask) = init_sel.mask {
                                            // Stretch 1-bit mask by nearest neighbor
                                            let mut new_mask = Vec::with_capacity((w * h) as usize);
                                            for my in 0..h {
                                                let src_my = ((my as u64 * orig_h as u64)
                                                    / h as u64)
                                                    .min((orig_h - 1) as u64)
                                                    as usize;
                                                for mx in 0..w {
                                                    let src_mx = ((mx as u64 * orig_w as u64)
                                                        / w as u64)
                                                        .min((orig_w - 1) as u64)
                                                        as usize;
                                                    let m_val = orig_mask
                                                        .get(src_my * orig_w as usize + src_mx)
                                                        .copied()
                                                        .unwrap_or(true);
                                                    new_mask.push(m_val);
                                                }
                                            }
                                            sel.mask = Some(new_mask);
                                        }
                                        self.mark_dirty();
                                    }
                                }
                            }
                        } else if let Some(origin_start) = self.selection_drag_start_origin {
                            // Standard translation
                            let delta_x = curr_pt.x - mouse_start.x;
                            let delta_y = curr_pt.y - mouse_start.y;
                            sel.current_origin =
                                Point::new(origin_start.x + delta_x, origin_start.y + delta_y);

                            if is_shift_down {
                                if let Some(layer) = stack.active_layer_mut() {
                                    stamp_floating_selection(
                                        &mut layer.buffer,
                                        sel,
                                        sel.current_origin,
                                        toolbox.selection_mode,
                                        *bg_color,
                                    );
                                    self.mark_dirty();
                                }
                            }
                        }
                    }

                    if !ui.input(|i| i.pointer.primary_down() || i.pointer.secondary_down()) {
                        if let (Some(before), Some(layer)) =
                            (self.stroke_before_buf.take(), stack.active_layer())
                        {
                            history.record_layer_pixel_change(
                                active_layer_id,
                                &before,
                                &layer.buffer,
                            );
                        }
                        self.is_dragging_selection = false;
                        self.active_selection_handle = None;
                        self.selection_transform_initial_sel = None;
                        self.selection_drag_start_origin = None;
                        self.selection_drag_mouse_start = None;
                    }
                } else if self.is_drawing {
                    let (doc_w, doc_h) = (stack.width as i32, stack.height as i32);
                    let symmetry = self.symmetry_mode;
                    if let (Some(curr_pt), Some(layer)) =
                        (self.current_mouse_pt, stack.active_layer_mut())
                    {
                        match toolbox.active_tool {
                            ToolKind::Pencil => {
                                if let Some(prev_pt) = self.prev_draw_pt {
                                    if prev_pt != curr_pt {
                                        let prev_pts =
                                            get_mirrored_points(prev_pt, doc_w, doc_h, symmetry);
                                        let curr_pts =
                                            get_mirrored_points(curr_pt, doc_w, doc_h, symmetry);
                                        for (p0, p1) in prev_pts.into_iter().zip(curr_pts) {
                                            draw_pencil_stroke_sized(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                toolbox.pencil_size,
                                                active_color,
                                            );
                                        }
                                        self.prev_draw_pt = Some(curr_pt);
                                        self.mark_dirty();
                                    }
                                }
                            }
                            ToolKind::Brush => {
                                if let Some(prev_pt) = self.prev_draw_pt {
                                    if prev_pt != curr_pt {
                                        let prev_pts =
                                            get_mirrored_points(prev_pt, doc_w, doc_h, symmetry);
                                        let curr_pts =
                                            get_mirrored_points(curr_pt, doc_w, doc_h, symmetry);
                                        for (p0, p1) in prev_pts.into_iter().zip(curr_pts) {
                                            draw_brush_stroke(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                toolbox.brush_shape,
                                                active_color,
                                            );
                                        }
                                        self.prev_draw_pt = Some(curr_pt);
                                        self.mark_dirty();
                                    }
                                }
                            }
                            ToolKind::Eraser => {
                                if let Some(prev_pt) = self.prev_draw_pt {
                                    let prev_pts =
                                        get_mirrored_points(prev_pt, doc_w, doc_h, symmetry);
                                    let curr_pts =
                                        get_mirrored_points(curr_pt, doc_w, doc_h, symmetry);
                                    for (p0, p1) in prev_pts.into_iter().zip(curr_pts) {
                                        if toolbox.erase_to_transparent {
                                            if self.active_button == Some(PointerButton::Secondary)
                                            {
                                                draw_color_eraser_stroke(
                                                    &mut layer.buffer,
                                                    p0,
                                                    p1,
                                                    toolbox.eraser_size,
                                                    *fg_color,
                                                    ColorRgba::rgba(0, 0, 0, 0),
                                                );
                                            } else {
                                                draw_transparent_eraser_stroke(
                                                    &mut layer.buffer,
                                                    p0,
                                                    p1,
                                                    toolbox.eraser_size,
                                                );
                                            }
                                        } else if self.active_button
                                            == Some(PointerButton::Secondary)
                                        {
                                            draw_color_eraser_stroke(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                toolbox.eraser_size,
                                                *fg_color,
                                                *bg_color,
                                            );
                                        } else {
                                            draw_eraser_stroke(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                toolbox.eraser_size,
                                                *bg_color,
                                            );
                                        }
                                    }
                                    self.prev_draw_pt = Some(curr_pt);
                                    self.mark_dirty();
                                }
                            }
                            ToolKind::Airbrush => {
                                self.spray_step = self.spray_step.wrapping_add(1);
                                let pts = get_mirrored_points(curr_pt, doc_w, doc_h, symmetry);
                                for p in pts {
                                    apply_airbrush_spray(
                                        &mut layer.buffer,
                                        p,
                                        toolbox.airbrush_size,
                                        active_color,
                                        self.spray_step,
                                    );
                                }
                                self.prev_draw_pt = Some(curr_pt);
                                self.mark_dirty();
                                ui.ctx().request_repaint();
                            }
                            ToolKind::WackyBrush => {
                                self.wacky_state.hold_ticks =
                                    self.wacky_state.hold_ticks.saturating_add(1);
                                if let Some(prev_pt) = self.prev_draw_pt {
                                    let spacing = if toolbox.precision_mode {
                                        if toolbox.wacky_continuous {
                                            BrushSpacing::Continuous
                                        } else {
                                            BrushSpacing::Custom(toolbox.wacky_spacing)
                                        }
                                    } else {
                                        BrushSpacing::Discrete
                                    };
                                    let angle_mode = if toolbox.precision_mode {
                                        if toolbox.wacky_auto_tangent {
                                            BrushAngleMode::FollowStroke
                                        } else {
                                            BrushAngleMode::Fixed(toolbox.wacky_angle)
                                        }
                                    } else {
                                        BrushAngleMode::Fixed(0.0)
                                    };
                                    let seed_mode = if toolbox.precision_mode
                                        && toolbox.wacky_seed_locked
                                    {
                                        SeedMode::Locked(toolbox.wacky_seed_value)
                                    } else {
                                        SeedMode::Dynamic
                                    };
                                    let options = WackyBrushOptions {
                                        spacing,
                                        scatter_radius: if toolbox.precision_mode {
                                            toolbox.wacky_scatter_radius
                                        } else {
                                            0
                                        },
                                        particle_density: if toolbox.precision_mode {
                                            toolbox.wacky_density
                                        } else {
                                            1.0
                                        },
                                        burst_count_override: None,
                                        angle_mode,
                                        seed_mode,
                                    };

                                    if prev_pt != curr_pt {
                                        let prev_pts =
                                            get_mirrored_points(prev_pt, doc_w, doc_h, symmetry);
                                        let curr_pts =
                                            get_mirrored_points(curr_pt, doc_w, doc_h, symmetry);
                                        for (p0, p1) in prev_pts.into_iter().zip(curr_pts) {
                                            if self.is_pixel_paintable(p1) {
                                                draw_wacky_brush_stroke_with_options(
                                                    &mut layer.buffer,
                                                    p0,
                                                    p1,
                                                    toolbox.wacky_brush_shape,
                                                    toolbox.wacky_brush_size,
                                                    active_color,
                                                    *bg_color,
                                                    &mut self.wacky_state,
                                                    &options,
                                                );
                                            }
                                        }
                                        self.prev_draw_pt = Some(curr_pt);
                                        self.mark_dirty();
                                    } else if toolbox.wacky_brush_shape == WackyBrushShape::LeakyPen
                                    {
                                        let pts =
                                            get_mirrored_points(curr_pt, doc_w, doc_h, symmetry);
                                        for p in pts {
                                            if self.is_pixel_paintable(p) {
                                                apply_wacky_brush_point_with_options(
                                                    &mut layer.buffer,
                                                    p,
                                                    toolbox.wacky_brush_shape,
                                                    toolbox.wacky_brush_size,
                                                    active_color,
                                                    *bg_color,
                                                    &mut self.wacky_state,
                                                    &options,
                                                );
                                            }
                                        }
                                        self.mark_dirty();
                                    }
                                }
                                if toolbox.wacky_brush_shape == WackyBrushShape::LeakyPen {
                                    ui.ctx().request_repaint();
                                }
                            }
                            ToolKind::Stamp => {
                                if toolbox.precision_mode && toolbox.stamp_continuous_trail {
                                    if let Some(prev_pt) = self.prev_draw_pt {
                                        let dx = curr_pt.x - prev_pt.x;
                                        let dy = curr_pt.y - prev_pt.y;
                                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                                        let interval = (toolbox.stamp_trail_spacing.max(4)) as f32;
                                        if dist >= interval {
                                            let steps = (dist / interval).floor() as i32;
                                            let scale = if toolbox.precision_mode {
                                                (toolbox.stamp_fine_scale
                                                    * toolbox.stamp_scale as f32)
                                                    .round()
                                                    .max(1.0)
                                                    as u32
                                            } else {
                                                toolbox.stamp_scale
                                            };
                                            let rot = if toolbox.precision_mode {
                                                toolbox.stamp_free_rotation
                                            } else {
                                                toolbox.stamp_rotation as f32
                                            };
                                            let tint = match toolbox.stamp_tint_mode {
                                                1 => StampTint::Monochrome(active_color),
                                                2 => StampTint::TwoTone {
                                                    fg: active_color,
                                                    bg: *bg_color,
                                                },
                                                3 => StampTint::Outlined {
                                                    fill: active_color,
                                                    outline: *bg_color,
                                                },
                                                _ => StampTint::OriginalPalette,
                                            };
                                            let stamp_opt = StampOptions {
                                                scale,
                                                rotation_deg: rot,
                                                spacing: StampSpacing::Interval(
                                                    toolbox.stamp_trail_spacing,
                                                ),
                                                tint,
                                            };
                                            for step_i in 1..=steps {
                                                let t = step_i as f32 / steps as f32;
                                                let inter_x =
                                                    (prev_pt.x as f32 + t * dx as f32).round()
                                                        as i32;
                                                let inter_y =
                                                    (prev_pt.y as f32 + t * dy as f32).round()
                                                        as i32;
                                                let pt = Point::new(inter_x, inter_y);
                                                let pts = get_mirrored_points(
                                                    pt, doc_w, doc_h, symmetry,
                                                );
                                                for p in pts {
                                                    if self.is_pixel_paintable(p) {
                                                        apply_stamp_with_options(
                                                            &mut layer.buffer,
                                                            p,
                                                            toolbox.stamp_shape,
                                                            &stamp_opt,
                                                        );
                                                    }
                                                }
                                            }
                                            self.prev_draw_pt = Some(curr_pt);
                                            self.mark_dirty();
                                        }
                                    }
                                }
                            }
                            ToolKind::FreeformSelect
                                if self.lasso_points.last() != Some(&curr_pt) =>
                            {
                                self.lasso_points.push(curr_pt);
                            }
                            _ => {}
                        }
                    }

                    // Check mouse release
                    let btn_still_down = match self.active_button {
                        Some(PointerButton::Primary) => ui.input(|i| i.pointer.primary_down()),
                        Some(PointerButton::Secondary) => ui.input(|i| i.pointer.secondary_down()),
                        _ => false,
                    };

                    if !btn_still_down {
                        if let (Some(start_pt), Some(curr_pt)) =
                            (self.draw_start_pt, self.current_mouse_pt)
                        {
                            let final_pt = if is_shift_down {
                                match toolbox.active_tool {
                                    ToolKind::Line => constrained_line_end(start_pt, curr_pt, true),
                                    ToolKind::Rectangle
                                    | ToolKind::Ellipse
                                    | ToolKind::RoundedRectangle => {
                                        snap_to_square(start_pt, curr_pt)
                                    }
                                    _ => curr_pt,
                                }
                            } else {
                                curr_pt
                            };

                            let (doc_w, doc_h) = (stack.width as i32, stack.height as i32);
                            if let Some(layer) = stack.active_layer_mut() {
                                let symmetry = self.symmetry_mode;
                                let start_pts =
                                    get_mirrored_points(start_pt, doc_w, doc_h, symmetry);
                                let final_pts =
                                    get_mirrored_points(final_pt, doc_w, doc_h, symmetry);

                                match toolbox.active_tool {
                                    ToolKind::Line => {
                                        for (p0, p1) in start_pts.into_iter().zip(final_pts) {
                                            draw_line(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                toolbox.line_width,
                                                active_color,
                                            );
                                        }
                                        self.mark_dirty();
                                    }
                                    ToolKind::Rectangle => {
                                        for (p0, p1) in start_pts.into_iter().zip(final_pts) {
                                            draw_rectangle(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                toolbox.fill_style,
                                                toolbox.line_width,
                                                *fg_color,
                                                *bg_color,
                                            );
                                        }
                                        self.mark_dirty();
                                    }
                                    ToolKind::Ellipse => {
                                        for (p0, p1) in start_pts.into_iter().zip(final_pts) {
                                            draw_ellipse(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                toolbox.fill_style,
                                                toolbox.line_width,
                                                *fg_color,
                                                *bg_color,
                                            );
                                        }
                                        self.mark_dirty();
                                    }
                                    ToolKind::RoundedRectangle => {
                                        for (p0, p1) in start_pts.into_iter().zip(final_pts) {
                                            draw_rounded_rectangle(
                                                &mut layer.buffer,
                                                p0,
                                                p1,
                                                8,
                                                toolbox.fill_style,
                                                toolbox.line_width,
                                                *fg_color,
                                                *bg_color,
                                            );
                                        }
                                        self.mark_dirty();
                                    }
                                    ToolKind::Curve => {
                                        if self.curve_step == 0 {
                                            self.curve_p0 = Some(start_pt);
                                            self.curve_p3 = Some(final_pt);
                                            self.curve_step = 1;
                                            self.is_drawing = false;
                                            self.draw_start_pt = None;
                                            self.prev_draw_pt = None;
                                            self.active_button = None;
                                        } else if self.curve_step == 1 {
                                            self.curve_p1 = Some(final_pt);
                                            self.curve_step = 2;
                                            self.is_drawing = false;
                                            self.draw_start_pt = None;
                                            self.prev_draw_pt = None;
                                            self.active_button = None;
                                        } else if self.curve_step == 2 {
                                            if let (Some(p0), Some(p3), Some(p1)) =
                                                (self.curve_p0, self.curve_p3, self.curve_p1)
                                            {
                                                let p2 = final_pt;
                                                let p0_pts =
                                                    get_mirrored_points(p0, doc_w, doc_h, symmetry);
                                                let p1_pts =
                                                    get_mirrored_points(p1, doc_w, doc_h, symmetry);
                                                let p2_pts =
                                                    get_mirrored_points(p2, doc_w, doc_h, symmetry);
                                                let p3_pts =
                                                    get_mirrored_points(p3, doc_w, doc_h, symmetry);
                                                for i in 0..p0_pts.len() {
                                                    draw_curve_cubic(
                                                        &mut layer.buffer,
                                                        p0_pts[i],
                                                        p1_pts[i],
                                                        p2_pts[i],
                                                        p3_pts[i],
                                                        toolbox.line_width,
                                                        active_color,
                                                    );
                                                }
                                            }
                                            self.curve_step = 0;
                                            self.curve_p0 = None;
                                            self.curve_p1 = None;
                                            self.curve_p3 = None;
                                            if let (Some(before), Some(layer)) = (
                                                self.stroke_before_buf.take(),
                                                stack.active_layer(),
                                            ) {
                                                history.record_layer_pixel_change(
                                                    active_layer_id,
                                                    &before,
                                                    &layer.buffer,
                                                );
                                            }
                                            self.is_drawing = false;
                                            self.active_button = None;
                                            self.draw_start_pt = None;
                                            self.prev_draw_pt = None;
                                            self.mark_dirty();
                                        }
                                    }
                                    ToolKind::Select => {
                                        let dx = (final_pt.x - start_pt.x).abs();
                                        let dy = (final_pt.y - start_pt.y).abs();
                                        if dx >= 2 || dy >= 2 {
                                            self.floating_selection = lift_rectangular_selection(
                                                &mut layer.buffer,
                                                start_pt,
                                                final_pt,
                                                *bg_color,
                                            );
                                            self.mark_dirty();
                                        } else {
                                            self.floating_selection = None;
                                            self.stroke_before_buf = None;
                                        }
                                    }
                                    ToolKind::FreeformSelect => {
                                        if self.lasso_points.len() >= 3 {
                                            self.floating_selection = lift_freeform_selection(
                                                &mut layer.buffer,
                                                &self.lasso_points,
                                                *bg_color,
                                            );
                                            self.lasso_points.clear();
                                            self.mark_dirty();
                                        } else {
                                            self.lasso_points.clear();
                                            self.floating_selection = None;
                                            self.stroke_before_buf = None;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if toolbox.active_tool != ToolKind::Curve {
                            if let (Some(before), Some(layer)) =
                                (self.stroke_before_buf.take(), stack.active_layer())
                            {
                                if before != layer.buffer {
                                    history.record_layer_pixel_change(
                                        active_layer_id,
                                        &before,
                                        &layer.buffer,
                                    );
                                }
                            }
                            self.is_drawing = false;
                            self.active_button = None;
                            self.draw_start_pt = None;
                            self.prev_draw_pt = None;
                        }
                    }
                }

                // If drawing or dragging continuously, request immediate repaint for high refresh-rate smoothness
                if self.is_drawing || self.is_dragging_layer || self.is_dragging_selection {
                    ui.ctx().request_repaint();
                }

                // Update cached composite if version changed (e.g. after stroke) or onion skin changed
                if let Some((doc, cfg)) =
                    onion_skin.filter(|(_, cfg)| cfg.enabled && !self.is_drawing)
                {
                    let state_changed =
                        self.last_onion_config.as_ref() != current_onion_state.as_ref();
                    if state_changed
                        || self.last_composite_version != self.texture_version
                        || self.cached_composite.width() != stack.width
                        || self.cached_composite.height() != stack.height
                    {
                        self.cached_composite =
                            composite_with_onion_skin(doc, doc.active_frame_index, cfg);
                        self.last_composite_version = self.texture_version;
                        self.last_onion_config = current_onion_state;
                    }
                } else {
                    let had_onion = self.last_onion_config.is_some();
                    if had_onion
                        || self.last_composite_version != self.texture_version
                        || self.cached_composite.width() != stack.width
                        || self.cached_composite.height() != stack.height
                    {
                        self.cached_composite = stack.composite();
                        self.last_composite_version = self.texture_version;
                        self.last_onion_config = None;
                    }
                }

                // Update GPU canvas texture only when texture version has changed or texture not loaded
                let texture = if self.last_uploaded_version != self.texture_version
                    || self.texture.is_none()
                {
                    let color_image = ColorImage::from_rgba_unmultiplied(
                        [
                            self.cached_composite.width() as usize,
                            self.cached_composite.height() as usize,
                        ],
                        self.cached_composite.as_raw(),
                    );
                    let tex = self.texture.get_or_insert_with(|| {
                        ui.ctx().load_texture(
                            "canvas_texture",
                            color_image.clone(),
                            TextureOptions::NEAREST,
                        )
                    });
                    tex.set(color_image, TextureOptions::NEAREST);
                    self.last_uploaded_version = self.texture_version;
                    tex.clone()
                } else {
                    self.texture.clone().expect("texture exists")
                };

                // Update selection texture if a floating selection exists
                if let Some(ref sel) = self.floating_selection {
                    let w = sel.buffer.width() as usize;
                    let h = sel.buffer.height() as usize;
                    let mut pixels = Vec::with_capacity(w * h);
                    for y in 0..h {
                        for x in 0..w {
                            let is_masked = if let Some(ref mask) = sel.mask {
                                mask.get(y * w + x).copied().unwrap_or(false)
                            } else {
                                true
                            };

                            if !is_masked {
                                pixels.push(Color32::TRANSPARENT);
                            } else if let Some(c) = sel.buffer.get_pixel_i32(x as i32, y as i32) {
                                if toolbox.selection_mode == SelectionMode::Transparent
                                    && c == *bg_color
                                {
                                    pixels.push(Color32::TRANSPARENT);
                                } else {
                                    pixels
                                        .push(Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a));
                                }
                            } else {
                                pixels.push(Color32::TRANSPARENT);
                            }
                        }
                    }
                    let sel_image = ColorImage {
                        size: [w, h],
                        pixels,
                    };
                    if let Some(ref mut tex) = self.selection_texture {
                        tex.set(sel_image, TextureOptions::NEAREST);
                    } else {
                        self.selection_texture = Some(ui.ctx().load_texture(
                            "floating_selection_tex",
                            sel_image,
                            TextureOptions::NEAREST,
                        ));
                    }
                } else {
                    self.selection_texture = None;
                }

                // Draw background checkerboard
                draw_canvas_checkerboard(ui.painter(), rect, 12.0);

                // Draw canvas texture
                ui.painter().image(
                    texture.id(),
                    rect,
                    EguiRect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Render Symmetry guide lines
                self.render_symmetry_guides(ui.painter(), rect, stack.width, stack.height);

                // Live preview rendering for shape tools (Line, Rectangle, Ellipse, RoundedRectangle, Select)
                if self.is_drawing {
                    if let (Some(start_pt), Some(curr_pt)) =
                        (self.draw_start_pt, self.current_mouse_pt)
                    {
                        let final_pt = if is_shift_down {
                            match toolbox.active_tool {
                                ToolKind::Line => constrained_line_end(start_pt, curr_pt, true),
                                ToolKind::Rectangle
                                | ToolKind::Ellipse
                                | ToolKind::RoundedRectangle => snap_to_square(start_pt, curr_pt),
                                _ => curr_pt,
                            }
                        } else {
                            curr_pt
                        };

                        self.render_tool_preview(
                            ui.painter(),
                            rect,
                            toolbox.active_tool,
                            toolbox,
                            start_pt,
                            final_pt,
                            active_color,
                            *bg_color,
                            stack.width as i32,
                            stack.height as i32,
                        );
                    }
                }

                // Render Curve live preview
                self.render_curve_preview(
                    ui.painter(),
                    rect,
                    toolbox,
                    active_color,
                    stack.width as i32,
                    stack.height as i32,
                );

                // Render Polygon live preview
                self.render_polygon_preview(
                    ui.painter(),
                    rect,
                    toolbox,
                    *fg_color,
                    stack.width as i32,
                    stack.height as i32,
                );

                // Render Floating Selection on canvas
                if let (Some(ref sel), Some(ref tex)) =
                    (&self.floating_selection, &self.selection_texture)
                {
                    self.render_floating_selection(ui.painter(), rect, sel, tex);
                }

                // Render Text Tool Box on canvas
                if let Some(text_pos) = self.active_text_pos {
                    self.render_text_overlay(
                        ui, rect, text_pos, stack, history, toolbox, *fg_color, *bg_color,
                    );
                }

                // Render Pixel Grid when show_grid is active
                if self.show_grid && self.zoom >= 2.0 {
                    self.render_pixel_grid(ui.painter(), rect, stack.width, stack.height);
                }

                // Render cursor guide for Wacky Brush / Rubber Stamp
                if let Some(cursor_pt) = self.cursor_canvas_pos {
                    if !self.is_drawing {
                        let screen_pos = Pos2 {
                            x: rect.min.x + cursor_pt.x as f32 * self.zoom,
                            y: rect.min.y + cursor_pt.y as f32 * self.zoom,
                        };
                        if toolbox.active_tool == ToolKind::WackyBrush {
                            let r_screen = toolbox.wacky_brush_size as f32 * self.zoom.max(1.0);
                            ui.painter().circle_stroke(
                                screen_pos,
                                r_screen,
                                Stroke::new(
                                    1.0_f32,
                                    Color32::from_rgba_unmultiplied(100, 150, 255, 180),
                                ),
                            );

                            if toolbox.precision_mode {
                                if toolbox.wacky_scatter_radius > 0 {
                                    let r_scatter = (toolbox.wacky_brush_size
                                        + toolbox.wacky_scatter_radius)
                                        as f32
                                        * self.zoom.max(1.0);
                                    ui.painter().circle_stroke(
                                        screen_pos,
                                        r_scatter,
                                        Stroke::new(
                                            1.0_f32,
                                            Color32::from_rgba_unmultiplied(255, 180, 50, 140),
                                        ),
                                    );
                                }

                                if !toolbox.wacky_auto_tangent {
                                    let rad = toolbox.wacky_angle.to_radians();
                                    let len = r_screen + 10.0;
                                    let end_p = Pos2::new(
                                        screen_pos.x + rad.cos() * len,
                                        screen_pos.y + rad.sin() * len,
                                    );
                                    ui.painter().line_segment(
                                        [screen_pos, end_p],
                                        Stroke::new(1.5_f32, Color32::from_rgb(255, 80, 80)),
                                    );
                                    ui.painter().circle_filled(
                                        end_p,
                                        2.5,
                                        Color32::from_rgb(255, 80, 80),
                                    );
                                }
                            }
                        } else if toolbox.active_tool == ToolKind::ElectricMixer {
                            let r_screen = (14.0 * self.zoom).clamp(10.0, 28.0);
                            ui.painter().circle_stroke(
                                screen_pos,
                                r_screen,
                                Stroke::new(
                                    1.2_f32,
                                    Color32::from_rgba_unmultiplied(255, 215, 0, 190),
                                ),
                            );
                            ui.painter().text(
                                screen_pos,
                                egui::Align2::CENTER_CENTER,
                                "⚡",
                                egui::FontId::monospace(13.0),
                                Color32::from_rgb(255, 230, 0),
                            );

                            if toolbox.mixer_scope_selection && self.floating_selection.is_some() {
                                if let Some(ref sel) = self.floating_selection {
                                    let bounds = sel.current_rect();
                                    let min_p = Pos2 {
                                        x: rect.min.x + bounds.min.x as f32 * self.zoom,
                                        y: rect.min.y + bounds.min.y as f32 * self.zoom,
                                    };
                                    let max_p = Pos2 {
                                        x: rect.min.x + (bounds.max.x + 1) as f32 * self.zoom,
                                        y: rect.min.y + (bounds.max.y + 1) as f32 * self.zoom,
                                    };
                                    ui.painter().rect_stroke(
                                        EguiRect::from_min_max(min_p, max_p),
                                        0.0,
                                        Stroke::new(1.5_f32, Color32::from_rgb(255, 215, 0)),
                                        egui::StrokeKind::Outside,
                                    );
                                }
                            }
                        } else if toolbox.active_tool == ToolKind::Stamp {
                            let eff_scale = if toolbox.precision_mode {
                                (toolbox.stamp_fine_scale * toolbox.stamp_scale as f32).max(1.0)
                            } else {
                                toolbox.stamp_scale as f32
                            };
                            let scale_px = (eff_scale * self.zoom).max(1.0);
                            let rot_deg = if toolbox.precision_mode {
                                toolbox.stamp_free_rotation
                            } else {
                                toolbox.stamp_rotation as f32
                            };
                            let half = 6.0 * scale_px;
                            let stamp_box =
                                EguiRect::from_center_size(screen_pos, Vec2::splat(half * 2.0));
                            ui.painter().rect_stroke(
                                stamp_box,
                                0.0,
                                Stroke::new(
                                    1.0_f32,
                                    Color32::from_rgba_unmultiplied(255, 100, 150, 130),
                                ),
                                egui::StrokeKind::Outside,
                            );

                            let bitmap = toolbox.stamp_shape.bitmap();
                            let norm_tint = match toolbox.stamp_tint_mode {
                                1 => Color32::from_rgba_unmultiplied(
                                    active_color.r,
                                    active_color.g,
                                    active_color.b,
                                    190,
                                ),
                                2 => Color32::from_rgba_unmultiplied(
                                    active_color.r,
                                    active_color.g,
                                    active_color.b,
                                    190,
                                ),
                                3 => Color32::from_rgba_unmultiplied(
                                    active_color.r,
                                    active_color.g,
                                    active_color.b,
                                    190,
                                ),
                                _ => {
                                    let sc = stamp_color(toolbox.stamp_shape);
                                    Color32::from_rgba_unmultiplied(sc.r(), sc.g(), sc.b(), 190)
                                }
                            };

                            let rad = rot_deg.to_radians();
                            let cos_a = rad.cos();
                            let sin_a = rad.sin();

                            for (r, row_str) in bitmap.iter().enumerate() {
                                for (c, ch) in row_str.chars().enumerate() {
                                    if ch == 'X' {
                                        let norm_c = c as f32 + 0.5 - 6.0;
                                        let norm_r = r as f32 + 0.5 - 6.0;
                                        let rx = (norm_c * cos_a - norm_r * sin_a) * scale_px;
                                        let ry = (norm_c * sin_a + norm_r * cos_a) * scale_px;
                                        let px = screen_pos.x + rx;
                                        let py = screen_pos.y + ry;
                                        ui.painter().rect_filled(
                                            EguiRect::from_center_size(
                                                Pos2::new(px, py),
                                                Vec2::splat(scale_px.max(1.0)),
                                            ),
                                            0.0,
                                            norm_tint,
                                        );
                                    }
                                }
                            }
                        } else if toolbox.active_tool == ToolKind::PatternFill {
                            let tile_sz = (32.0 * toolbox.pattern_scale as f32 * self.zoom)
                                .clamp(16.0, 160.0);
                            let tile_box =
                                EguiRect::from_center_size(screen_pos, Vec2::splat(tile_sz));
                            ui.painter().rect_stroke(
                                tile_box,
                                0.0,
                                Stroke::new(
                                    1.0_f32,
                                    Color32::from_rgba_unmultiplied(50, 180, 220, 160),
                                ),
                                egui::StrokeKind::Outside,
                            );
                            ui.painter().text(
                                tile_box.left_top() + Vec2::new(3.0, 2.0),
                                egui::Align2::LEFT_TOP,
                                format!(
                                    "{}x (+{},+{})",
                                    toolbox.pattern_scale,
                                    toolbox.pattern_offset_x,
                                    toolbox.pattern_offset_y
                                ),
                                egui::FontId::monospace(10.0),
                                Color32::from_rgb(50, 180, 220),
                            );
                        }
                    }
                }
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_tool_preview(
        &self,
        painter: &egui::Painter,
        canvas_rect: EguiRect,
        tool: ToolKind,
        toolbox: &ToolboxView,
        p0: Point,
        p1: Point,
        fg_col: ColorRgba,
        bg_col: ColorRgba,
        doc_w: i32,
        doc_h: i32,
    ) {
        let to_screen = |p: Point| Pos2 {
            x: canvas_rect.min.x + p.x as f32 * self.zoom,
            y: canvas_rect.min.y + p.y as f32 * self.zoom,
        };

        let stroke_color = Color32::from_rgba_unmultiplied(fg_col.r, fg_col.g, fg_col.b, 200);
        let stroke = Stroke::new(toolbox.line_width as f32 * self.zoom.max(1.0), stroke_color);

        if tool == ToolKind::Select {
            let sp0 = to_screen(p0);
            let sp1 = to_screen(p1);
            let r = EguiRect::from_two_pos(sp0, sp1);
            let dash_stroke = Stroke::new(1.0_f32, Color32::BLACK);
            painter.rect_stroke(r, 0.0, dash_stroke, egui::StrokeKind::Outside);
            return;
        }

        let start_pts = get_mirrored_points(p0, doc_w, doc_h, self.symmetry_mode);
        let final_pts = get_mirrored_points(p1, doc_w, doc_h, self.symmetry_mode);

        for (curr_p0, curr_p1) in start_pts.into_iter().zip(final_pts) {
            let sp0 = to_screen(curr_p0);
            let sp1 = to_screen(curr_p1);

            match tool {
                ToolKind::Line => {
                    painter.line_segment([sp0, sp1], stroke);
                }
                ToolKind::Rectangle => {
                    let r = EguiRect::from_two_pos(sp0, sp1);
                    match toolbox.fill_style {
                        FillStyle::OutlineOnly => {
                            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Outside);
                        }
                        FillStyle::FilledWithOutline => {
                            let fill_color =
                                Color32::from_rgba_unmultiplied(bg_col.r, bg_col.g, bg_col.b, 180);
                            painter.rect_filled(r, 0.0, fill_color);
                            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Outside);
                        }
                        FillStyle::FillOnly => {
                            let fill_color =
                                Color32::from_rgba_unmultiplied(fg_col.r, fg_col.g, fg_col.b, 200);
                            painter.rect_filled(r, 0.0, fill_color);
                        }
                    }
                }
                ToolKind::Ellipse => {
                    let r = EguiRect::from_two_pos(sp0, sp1);
                    let center = r.center();
                    let rx = (r.width() / 2.0).max(0.5);
                    let ry = (r.height() / 2.0).max(0.5);
                    let num_segments = 64;
                    let points: Vec<Pos2> = (0..=num_segments)
                        .map(|i| {
                            let theta = (i as f32 / num_segments as f32) * std::f32::consts::TAU;
                            Pos2::new(center.x + rx * theta.cos(), center.y + ry * theta.sin())
                        })
                        .collect();

                    match toolbox.fill_style {
                        FillStyle::OutlineOnly => {
                            painter.add(egui::Shape::closed_line(points, stroke));
                        }
                        FillStyle::FilledWithOutline => {
                            let fill_color =
                                Color32::from_rgba_unmultiplied(bg_col.r, bg_col.g, bg_col.b, 180);
                            let fill_shape = egui::Shape::convex_polygon(
                                points.iter().take(num_segments).copied().collect(),
                                fill_color,
                                Stroke::NONE,
                            );
                            painter.add(fill_shape);
                            painter.add(egui::Shape::closed_line(points, stroke));
                        }
                        FillStyle::FillOnly => {
                            let fill_color =
                                Color32::from_rgba_unmultiplied(fg_col.r, fg_col.g, fg_col.b, 200);
                            let fill_shape = egui::Shape::convex_polygon(
                                points.iter().take(num_segments).copied().collect(),
                                fill_color,
                                Stroke::NONE,
                            );
                            painter.add(fill_shape);
                        }
                    }
                }
                ToolKind::RoundedRectangle => {
                    let r = EguiRect::from_two_pos(sp0, sp1);
                    match toolbox.fill_style {
                        FillStyle::OutlineOnly => {
                            painter.rect_stroke(
                                r,
                                8.0 * self.zoom,
                                stroke,
                                egui::StrokeKind::Outside,
                            );
                        }
                        FillStyle::FilledWithOutline => {
                            let fill_color =
                                Color32::from_rgba_unmultiplied(bg_col.r, bg_col.g, bg_col.b, 180);
                            painter.rect_filled(r, 8.0 * self.zoom, fill_color);
                            painter.rect_stroke(
                                r,
                                8.0 * self.zoom,
                                stroke,
                                egui::StrokeKind::Outside,
                            );
                        }
                        FillStyle::FillOnly => {
                            let fill_color =
                                Color32::from_rgba_unmultiplied(fg_col.r, fg_col.g, fg_col.b, 200);
                            painter.rect_filled(r, 8.0 * self.zoom, fill_color);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn render_floating_selection(
        &self,
        painter: &egui::Painter,
        canvas_rect: EguiRect,
        sel: &FloatingSelection,
        tex: &TextureHandle,
    ) {
        let sel_min = Pos2::new(
            canvas_rect.min.x + sel.current_origin.x as f32 * self.zoom,
            canvas_rect.min.y + sel.current_origin.y as f32 * self.zoom,
        );
        let sel_max = Pos2::new(
            sel_min.x + sel.buffer.width() as f32 * self.zoom,
            sel_min.y + sel.buffer.height() as f32 * self.zoom,
        );
        let sel_rect = EguiRect::from_min_max(sel_min, sel_max);

        // Draw floating selection image with crisp nearest-neighbor interpolation
        painter.image(
            tex.id(),
            sel_rect,
            EguiRect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        // Marching ants / selection rectangle border
        painter.rect_stroke(
            sel_rect,
            0.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 120, 215)),
            egui::StrokeKind::Outside,
        );

        // Selection resize handles
        let handle_size = 6.0;
        let corners = [
            sel_rect.left_top(),
            sel_rect.right_top(),
            sel_rect.left_bottom(),
            sel_rect.right_bottom(),
            sel_rect.center_top(),
            sel_rect.center_bottom(),
            sel_rect.left_center(),
            sel_rect.right_center(),
        ];
        for pt in corners {
            let handle_rect = EguiRect::from_center_size(pt, Vec2::splat(handle_size));
            painter.rect_filled(handle_rect, 0.0, Color32::WHITE);
            painter.rect_stroke(
                handle_rect,
                0.0,
                Stroke::new(1.0_f32, Color32::BLACK),
                egui::StrokeKind::Outside,
            );
        }

        // Selection rotation handle above top center
        let rotate_center = Pos2::new(sel_rect.center_top().x, sel_rect.center_top().y - 18.0);
        painter.line_segment(
            [sel_rect.center_top(), rotate_center],
            Stroke::new(1.0_f32, Color32::from_rgb(0, 120, 215)),
        );
        painter.circle_filled(rotate_center, 4.5, Color32::from_rgb(0, 120, 215));
        painter.circle_stroke(rotate_center, 4.5, Stroke::new(1.0_f32, Color32::WHITE));
    }

    #[allow(clippy::too_many_arguments)]
    fn render_text_overlay(
        &mut self,
        ui: &mut Ui,
        canvas_rect: EguiRect,
        text_pos: Point,
        stack: &mut LayerStack,
        history: &mut History,
        toolbox: &ToolboxView,
        fg_color: ColorRgba,
        bg_color: ColorRgba,
    ) {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.active_text_pos = None;
            self.text_preview_texture = None;
            return;
        }

        // Live rasterized preview using cached texture for buttery smooth 60fps performance
        let preview_text = if self.text_input_buf.is_empty() {
            " "
        } else {
            &self.text_input_buf
        };

        let mode = if self.text_is_opaque {
            SelectionMode::Opaque
        } else {
            SelectionMode::Transparent
        };

        let cache_key = (
            preview_text.to_string(),
            self.text_font_family,
            self.text_font_size,
            self.text_rotation,
            self.text_is_opaque,
            fg_color,
            bg_color,
        );

        if self.text_preview_cache_key.as_ref() != Some(&cache_key)
            || self.text_preview_texture.is_none()
        {
            let preview_buf = render_text_to_buffer(
                preview_text,
                self.text_font_family,
                self.text_font_size,
                mode,
                fg_color,
                bg_color,
            );

            let rotated = match (self.text_rotation % 360) / 90 {
                1 => rotate_90(&preview_buf),
                2 => rotate_180(&preview_buf),
                3 => rotate_270(&preview_buf),
                _ => preview_buf,
            };

            let (w, h) = (rotated.width() as usize, rotated.height() as usize);
            let mut pixels = Vec::with_capacity(w * h);
            for y in 0..h {
                for x in 0..w {
                    if let Some(p) = rotated.get_pixel_i32(x as i32, y as i32) {
                        pixels.push(Color32::from_rgba_unmultiplied(p.r, p.g, p.b, p.a));
                    } else {
                        pixels.push(Color32::TRANSPARENT);
                    }
                }
            }
            let color_img = ColorImage {
                size: [w, h],
                pixels,
            };
            if let Some(ref mut tex) = self.text_preview_texture {
                tex.set(color_img, TextureOptions::NEAREST);
            } else {
                self.text_preview_texture = Some(ui.ctx().load_texture(
                    "text_preview_tex",
                    color_img,
                    TextureOptions::NEAREST,
                ));
            }
            self.text_preview_size = (w as u32, h as u32);
            self.text_preview_cache_key = Some(cache_key);
        }

        let (preview_w, preview_h) = (
            self.text_preview_size.0 as f32,
            self.text_preview_size.1 as f32,
        );
        let origin_screen = Pos2::new(
            canvas_rect.min.x + text_pos.x as f32 * self.zoom,
            canvas_rect.min.y + text_pos.y as f32 * self.zoom,
        );
        let preview_rect = EguiRect::from_min_size(
            origin_screen,
            Vec2::new(preview_w * self.zoom, preview_h * self.zoom),
        );

        if let Some(ref tex) = self.text_preview_texture {
            ui.painter().image(
                tex.id(),
                preview_rect,
                EguiRect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // Draw selection marquee border around text preview
        ui.painter().rect_stroke(
            preview_rect,
            0.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 120, 215)),
            egui::StrokeKind::Outside,
        );

        let mut commit_requested = false;
        let mut cancel_requested = false;

        // Floating draggable dialog window
        let mut window_open = true;
        let default_x = (preview_rect.max.x + 12.0)
            .min(ui.ctx().screen_rect().width() - 320.0)
            .max(10.0);
        let default_y = (preview_rect.min.y)
            .min(ui.ctx().screen_rect().height() - 250.0)
            .max(40.0);

        egui::Window::new("📝 Text Box")
            .id(egui::Id::new("floating_text_box_window"))
            .default_pos(Pos2::new(default_x, default_y))
            .default_size(Vec2::new(290.0, 190.0))
            .resizable(true)
            .collapsible(false)
            .open(&mut window_open)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Font:").size(11.0));
                    egui::ComboBox::from_id_salt("floating_text_font_combo")
                        .selected_text(self.text_font_family.display_name())
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            for fam in FontFamilyChoice::ALL {
                                ui.selectable_value(
                                    &mut self.text_font_family,
                                    fam,
                                    fam.display_name(),
                                );
                            }
                        });

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Size:").size(11.0));
                    ui.add(
                        egui::DragValue::new(&mut self.text_font_size)
                            .range(8..=128)
                            .speed(1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Rotation:").size(11.0));
                    for &deg in &[0, 90, 180, 270] {
                        let is_sel = self.text_rotation == deg;
                        if ui.selectable_label(is_sel, format!("{}°", deg)).clicked() {
                            self.text_rotation = deg;
                        }
                    }
                    if ui.button("⟳ +90°").clicked() {
                        self.text_rotation = (self.text_rotation + 90) % 360;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Mode:").size(11.0));
                    if ui
                        .selectable_label(!self.text_is_opaque, "Transparent")
                        .clicked()
                    {
                        self.text_is_opaque = false;
                    }
                    if ui.selectable_label(self.text_is_opaque, "Opaque").clicked() {
                        self.text_is_opaque = true;
                    }

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Pos:").size(11.0));
                    if let Some(pos) = self.active_text_pos.as_mut() {
                        ui.add(egui::DragValue::new(&mut pos.x).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut pos.y).prefix("Y: "));
                    }
                });

                ui.separator();
                ui.label(egui::RichText::new("Text (multiline, dynamic):").size(10.0));
                let text_res = ui.add(
                    egui::TextEdit::multiline(&mut self.text_input_buf)
                        .desired_rows(3)
                        .desired_width(ui.available_width() - 8.0),
                );
                if self.text_focus_requested {
                    text_res.request_focus();
                    self.text_focus_requested = false;
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            egui::RichText::new("✓ Apply (Enter)")
                                .size(11.0)
                                .color(Color32::from_rgb(0, 140, 0)),
                        )
                        .clicked()
                        || (ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter)))
                    {
                        commit_requested = true;
                    }
                    if ui
                        .button(
                            egui::RichText::new("✗ Cancel (Esc)")
                                .size(11.0)
                                .color(Color32::from_rgb(180, 0, 0)),
                        )
                        .clicked()
                    {
                        cancel_requested = true;
                    }
                });
            });

        if !window_open || cancel_requested {
            self.active_text_pos = None;
            self.text_preview_texture = None;
            return;
        }

        if commit_requested {
            self.commit_any_text(stack, history, toolbox, fg_color, bg_color);
        }
    }

    fn render_pixel_grid(
        &self,
        painter: &egui::Painter,
        canvas_rect: EguiRect,
        width: u32,
        height: u32,
    ) {
        let clip_rect = painter.clip_rect().intersect(canvas_rect);
        if !clip_rect.is_positive() {
            return;
        }

        let start_x =
            (((clip_rect.min.x - canvas_rect.min.x) / self.zoom).floor() as i64).max(0) as u32;
        let end_x = (((clip_rect.max.x - canvas_rect.min.x) / self.zoom).ceil() as u32).min(width);
        let start_y =
            (((clip_rect.min.y - canvas_rect.min.y) / self.zoom).floor() as i64).max(0) as u32;
        let end_y = (((clip_rect.max.y - canvas_rect.min.y) / self.zoom).ceil() as u32).min(height);

        let alpha = if self.zoom >= 4.0 { 120 } else { 75 };
        let grid_stroke = Stroke::new(
            0.5_f32,
            Color32::from_rgba_unmultiplied(128, 128, 128, alpha),
        );

        for x in start_x..=end_x {
            let sx = canvas_rect.min.x + x as f32 * self.zoom;
            painter.line_segment(
                [
                    Pos2::new(sx, canvas_rect.min.y),
                    Pos2::new(sx, canvas_rect.max.y),
                ],
                grid_stroke,
            );
        }
        for y in start_y..=end_y {
            let sy = canvas_rect.min.y + y as f32 * self.zoom;
            painter.line_segment(
                [
                    Pos2::new(canvas_rect.min.x, sy),
                    Pos2::new(canvas_rect.max.x, sy),
                ],
                grid_stroke,
            );
        }
    }

    fn render_symmetry_guides(
        &self,
        painter: &egui::Painter,
        canvas_rect: EguiRect,
        width: u32,
        height: u32,
    ) {
        if self.symmetry_mode == SymmetryMode::None {
            return;
        }

        let guide_color = Color32::from_rgba_unmultiplied(0, 150, 255, 140);
        let guide_stroke = Stroke::new(1.0_f32, guide_color);

        if self.symmetry_mode == SymmetryMode::Horizontal
            || self.symmetry_mode == SymmetryMode::Quad
        {
            let center_x = canvas_rect.min.x + (width as f32 * self.zoom) / 2.0;
            painter.line_segment(
                [
                    Pos2::new(center_x, canvas_rect.min.y),
                    Pos2::new(center_x, canvas_rect.max.y),
                ],
                guide_stroke,
            );
        }

        if self.symmetry_mode == SymmetryMode::Vertical || self.symmetry_mode == SymmetryMode::Quad
        {
            let center_y = canvas_rect.min.y + (height as f32 * self.zoom) / 2.0;
            painter.line_segment(
                [
                    Pos2::new(canvas_rect.min.x, center_y),
                    Pos2::new(canvas_rect.max.x, center_y),
                ],
                guide_stroke,
            );
        }
    }

    fn render_polygon_preview(
        &self,
        painter: &egui::Painter,
        canvas_rect: EguiRect,
        toolbox: &ToolboxView,
        fg_col: ColorRgba,
        doc_w: i32,
        doc_h: i32,
    ) {
        if !self.is_drawing_polygon || self.polygon_vertices.is_empty() {
            return;
        }

        let to_screen = |p: Point| Pos2 {
            x: canvas_rect.min.x + p.x as f32 * self.zoom,
            y: canvas_rect.min.y + p.y as f32 * self.zoom,
        };

        let stroke_color = Color32::from_rgba_unmultiplied(fg_col.r, fg_col.g, fg_col.b, 200);
        let stroke = Stroke::new(toolbox.line_width as f32 * self.zoom.max(1.0), stroke_color);

        let symmetry = self.symmetry_mode;
        let branch_count = match symmetry {
            SymmetryMode::None => 1,
            SymmetryMode::Horizontal | SymmetryMode::Vertical => 2,
            SymmetryMode::Quad => 4,
        };

        for b in 0..branch_count {
            let branch_verts: Vec<Point> = self
                .polygon_vertices
                .iter()
                .map(|v| {
                    let pts = get_mirrored_points(*v, doc_w, doc_h, symmetry);
                    pts[b]
                })
                .collect();

            // Render committed polygon edges
            for i in 0..branch_verts.len().saturating_sub(1) {
                let sp0 = to_screen(branch_verts[i]);
                let sp1 = to_screen(branch_verts[i + 1]);
                painter.line_segment([sp0, sp1], stroke);
            }

            // Render rubber-band line from last vertex to current cursor
            if let (Some(last_pt), Some(curr_pt)) = (branch_verts.last(), self.cursor_canvas_pos) {
                let curr_pts = get_mirrored_points(curr_pt, doc_w, doc_h, symmetry);
                let sp_last = to_screen(*last_pt);
                let sp_curr = to_screen(curr_pts[b]);
                let rubber_stroke = Stroke::new(
                    toolbox.line_width as f32 * self.zoom.max(1.0),
                    Color32::from_rgba_unmultiplied(fg_col.r, fg_col.g, fg_col.b, 160),
                );
                painter.line_segment([sp_last, sp_curr], rubber_stroke);

                // Highlight start point if cursor is close and can close polygon
                let start_pt = branch_verts[0];
                if (curr_pts[b].x - start_pt.x).abs() <= 6
                    && (curr_pts[b].y - start_pt.y).abs() <= 6
                    && branch_verts.len() >= 3
                {
                    let sp_start = to_screen(start_pt);
                    painter.circle_stroke(
                        sp_start,
                        6.0 * self.zoom.max(1.0),
                        Stroke::new(1.5_f32, Color32::from_rgb(0, 200, 0)),
                    );
                }
            }
        }
    }

    fn render_curve_preview(
        &self,
        painter: &egui::Painter,
        canvas_rect: EguiRect,
        toolbox: &ToolboxView,
        fg_col: ColorRgba,
        doc_w: i32,
        doc_h: i32,
    ) {
        if toolbox.active_tool != ToolKind::Curve {
            return;
        }

        let to_screen = |p: Point| Pos2 {
            x: canvas_rect.min.x + (p.x as f32 + 0.5) * self.zoom,
            y: canvas_rect.min.y + (p.y as f32 + 0.5) * self.zoom,
        };

        let stroke_color = Color32::from_rgba_unmultiplied(fg_col.r, fg_col.g, fg_col.b, 220);
        let stroke = Stroke::new(toolbox.line_width as f32 * self.zoom.max(1.0), stroke_color);

        let curr_pt = self.current_mouse_pt;
        let symmetry = self.symmetry_mode;

        if self.curve_step == 0 {
            if self.is_drawing {
                if let (Some(p0), Some(p3)) = (self.draw_start_pt, curr_pt) {
                    let p0_pts = get_mirrored_points(p0, doc_w, doc_h, symmetry);
                    let p3_pts = get_mirrored_points(p3, doc_w, doc_h, symmetry);
                    for (mp0, mp3) in p0_pts.into_iter().zip(p3_pts) {
                        painter.line_segment([to_screen(mp0), to_screen(mp3)], stroke);
                    }
                }
            }
        } else if self.curve_step == 1 {
            if let (Some(p0), Some(p3)) = (self.curve_p0, self.curve_p3) {
                let p0_pts = get_mirrored_points(p0, doc_w, doc_h, symmetry);
                let p3_pts = get_mirrored_points(p3, doc_w, doc_h, symmetry);
                let ctrl_pts = if let (true, Some(pt)) = (self.is_drawing, curr_pt) {
                    get_mirrored_points(pt, doc_w, doc_h, symmetry)
                } else {
                    vec![]
                };

                for i in 0..p0_pts.len() {
                    let mp0 = p0_pts[i];
                    let mp3 = p3_pts[i];
                    let pts = if let Some(&mctrl) = ctrl_pts.get(i) {
                        paint_core::geometry::rasterize_bezier_quadratic(mp0, mctrl, mp3)
                    } else {
                        paint_core::geometry::bresenham_line(mp0, mp3)
                    };
                    if pts.len() >= 2 {
                        let screen_pts: Vec<Pos2> = pts.into_iter().map(to_screen).collect();
                        painter.add(egui::Shape::line(screen_pts, stroke));
                    }
                }
            }
        } else if self.curve_step == 2 {
            if let (Some(p0), Some(p3), Some(p1)) = (self.curve_p0, self.curve_p3, self.curve_p1) {
                let p0_pts = get_mirrored_points(p0, doc_w, doc_h, symmetry);
                let p1_pts = get_mirrored_points(p1, doc_w, doc_h, symmetry);
                let p3_pts = get_mirrored_points(p3, doc_w, doc_h, symmetry);
                let p2_raw = match (self.is_drawing, curr_pt) {
                    (true, Some(pt)) => pt,
                    _ => p1,
                };
                let p2_pts = get_mirrored_points(p2_raw, doc_w, doc_h, symmetry);

                for i in 0..p0_pts.len() {
                    let pts = paint_core::geometry::rasterize_bezier_cubic(
                        p0_pts[i], p1_pts[i], p2_pts[i], p3_pts[i],
                    );
                    if pts.len() >= 2 {
                        let screen_pts: Vec<Pos2> = pts.into_iter().map(to_screen).collect();
                        painter.add(egui::Shape::line(screen_pts, stroke));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetry_point_generation() {
        let pt = Point::new(10, 20);
        let w = 100;
        let h = 80;

        // None
        let none = get_mirrored_points(pt, w, h, SymmetryMode::None);
        assert_eq!(none, vec![Point::new(10, 20)]);

        // Horizontal (w - 1 - 10 = 89)
        let horiz = get_mirrored_points(pt, w, h, SymmetryMode::Horizontal);
        assert_eq!(horiz, vec![Point::new(10, 20), Point::new(89, 20)]);

        // Vertical (h - 1 - 20 = 59)
        let vert = get_mirrored_points(pt, w, h, SymmetryMode::Vertical);
        assert_eq!(vert, vec![Point::new(10, 20), Point::new(10, 59)]);

        // Quad
        let quad = get_mirrored_points(pt, w, h, SymmetryMode::Quad);
        assert_eq!(
            quad,
            vec![
                Point::new(10, 20),
                Point::new(89, 20),
                Point::new(10, 59),
                Point::new(89, 59)
            ]
        );
    }

    #[test]
    fn test_symmetry_point_on_center_axis() {
        let pt = Point::new(50, 40);
        let w = 101; // odd width, center is 50
        let h = 81; // odd height, center is 40

        let horiz = get_mirrored_points(pt, w, h, SymmetryMode::Horizontal);
        assert_eq!(horiz.len(), 2);
        assert_eq!(horiz[0], Point::new(50, 40));
        assert_eq!(horiz[1], Point::new(50, 40));

        let quad = get_mirrored_points(pt, w, h, SymmetryMode::Quad);
        assert_eq!(quad.len(), 4);
    }
}
