//! Layers panel UI for managing layers, visibility, ordering, opacity, and operations.

use egui::{
    Color32, ColorImage, Context, Frame, RichText, Sense, Stroke, TextureHandle, TextureOptions,
    Ui, Vec2,
};
use paint_core::history::History;
use paint_core::layers::{LayerId, LayerStack};
use paint_core::palette::ColorRgba;
use std::collections::HashMap;

#[derive(Default)]
pub struct LayersView {
    pub is_open: bool,
    editing_layer_id: Option<LayerId>,
    rename_text: String,
    thumbnail_textures: HashMap<LayerId, (TextureHandle, u64)>,
}

impl std::fmt::Debug for LayersView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayersView")
            .field("is_open", &self.is_open)
            .field("editing_layer_id", &self.editing_layer_id)
            .finish()
    }
}

pub enum LayerAction {
    Modified,
    StackChanged,
    ImportImageRequested,
}

impl LayersView {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        stack: &mut LayerStack,
        history: &mut History,
        texture_version: u64,
        bg_color: ColorRgba,
    ) -> Option<LayerAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            ui.heading("Layers");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Count: {}", stack.layers.len()));
            });
        });

        ui.separator();

        // Opacity slider for active layer
        if let Some(active) = stack.active_layer_mut() {
            ui.horizontal(|ui| {
                ui.label("Opacity:");
                let mut op = active.opacity as i32;
                if ui
                    .add(egui::Slider::new(&mut op, 0..=100).suffix("%"))
                    .changed()
                {
                    active.opacity = op.clamp(0, 100) as u8;
                    action = Some(LayerAction::Modified);
                }
            });
        }

        ui.separator();

        // Action Toolbar
        ui.horizontal_wrapped(|ui| {
            if ui
                .button("+ New")
                .on_hover_text("Add New Transparent Layer (Ctrl+Shift+N)")
                .clicked()
            {
                let before = stack.clone();
                let count = stack.layers.len() + 1;
                stack.add_layer(format!("Layer {}", count));
                history.record_layer_stack_change(&before, stack);
                action = Some(LayerAction::StackChanged);
            }

            let can_delete = stack.layers.len() > 1;
            if ui
                .add_enabled(can_delete, egui::Button::new("🗑 Del"))
                .on_hover_text("Delete Active Layer")
                .clicked()
            {
                let before = stack.clone();
                if stack.delete_active_layer() {
                    history.record_layer_stack_change(&before, stack);
                    action = Some(LayerAction::StackChanged);
                }
            }

            if ui
                .button("⧉ Dup")
                .on_hover_text("Duplicate Active Layer")
                .clicked()
            {
                let before = stack.clone();
                if stack.duplicate_layer(stack.active_layer_id).is_some() {
                    history.record_layer_stack_change(&before, stack);
                    action = Some(LayerAction::StackChanged);
                }
            }

            if ui
                .button("📥 Import")
                .on_hover_text("Import Image as New Layer (Ctrl+Shift+I)")
                .clicked()
            {
                action = Some(LayerAction::ImportImageRequested);
            }

            let active_idx = stack.active_index().unwrap_or(0);
            let can_up = active_idx + 1 < stack.layers.len();
            let can_down = active_idx > 0;

            if ui
                .add_enabled(can_up, egui::Button::new("▲ Up"))
                .on_hover_text("Move Layer Up")
                .clicked()
            {
                let before = stack.clone();
                if stack.move_up(stack.active_layer_id) {
                    history.record_layer_stack_change(&before, stack);
                    action = Some(LayerAction::StackChanged);
                }
            }

            if ui
                .add_enabled(can_down, egui::Button::new("▼ Dn"))
                .on_hover_text("Move Layer Down")
                .clicked()
            {
                let before = stack.clone();
                if stack.move_down(stack.active_layer_id) {
                    history.record_layer_stack_change(&before, stack);
                    action = Some(LayerAction::StackChanged);
                }
            }

            let can_merge = active_idx > 0;
            if ui
                .add_enabled(can_merge, egui::Button::new("▼_ Merge"))
                .on_hover_text("Merge Active Layer Down (Ctrl+E)")
                .clicked()
            {
                let before = stack.clone();
                if stack.merge_down(stack.active_layer_id) {
                    history.record_layer_stack_change(&before, stack);
                    action = Some(LayerAction::StackChanged);
                }
            }

            let can_flatten = stack.layers.len() > 1;
            if ui
                .add_enabled(can_flatten, egui::Button::new("⊟ Flat"))
                .on_hover_text("Flatten Image")
                .clicked()
            {
                let before = stack.clone();
                stack.flatten(bg_color);
                history.record_layer_stack_change(&before, stack);
                action = Some(LayerAction::StackChanged);
            }
        });

        ui.separator();

        // Layer List (Rendered Top-to-Bottom: top layer at highest index displayed first)
        let ctx = ui.ctx().clone();
        let total_layers = stack.layers.len();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for display_idx in 0..total_layers {
                let idx = total_layers - 1 - display_idx;
                let layer_id = stack.layers[idx].id;
                let is_active = layer_id == stack.active_layer_id;

                let is_visible = stack.layers[idx].visible;
                let is_locked = stack.layers[idx].locked;
                let layer_name = stack.layers[idx].name.clone();

                let bg_color = if is_active {
                    Color32::from_rgb(0, 120, 215) // Windows XP/11 Active Accent Blue
                } else if idx.is_multiple_of(2) {
                    Color32::from_gray(245)
                } else {
                    Color32::from_gray(235)
                };

                let text_color = if is_active {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                };

                Frame::NONE
                    .fill(bg_color)
                    .stroke(Stroke::new(
                        1.0_f32,
                        if is_active {
                            Color32::from_rgb(0, 80, 180)
                        } else {
                            Color32::from_gray(200)
                        },
                    ))
                    .inner_margin(egui::Margin::symmetric(4, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Visibility toggle button (Eye)
                            let eye_label = if is_visible { "👁" } else { "⊘" };
                            let eye_btn = ui.button(eye_label).on_hover_text("Toggle Visibility");
                            if eye_btn.clicked() {
                                let before = stack.clone();
                                stack.layers[idx].visible = !stack.layers[idx].visible;
                                history.record_layer_stack_change(&before, stack);
                                action = Some(LayerAction::Modified);
                            }

                            // Lock toggle button (Lock)
                            let lock_label = if is_locked { "🔒" } else { "🔓" };
                            let lock_btn = ui.button(lock_label).on_hover_text("Toggle Layer Lock");
                            if lock_btn.clicked() {
                                let before = stack.clone();
                                stack.layers[idx].locked = !stack.layers[idx].locked;
                                history.record_layer_stack_change(&before, stack);
                                action = Some(LayerAction::Modified);
                            }

                            // Thumbnail preview
                            let thumbnail_size = Vec2::new(24.0, 24.0);
                            let tex_handle = self.get_or_create_thumbnail(
                                &ctx,
                                layer_id,
                                &stack.layers[idx].buffer,
                                texture_version,
                            );
                            ui.image((tex_handle.id(), thumbnail_size));

                            // Layer name / rename handling
                            if self.editing_layer_id == Some(layer_id) {
                                let edit = ui.text_edit_singleline(&mut self.rename_text);
                                if edit.lost_focus()
                                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    if !self.rename_text.trim().is_empty() {
                                        let before = stack.clone();
                                        stack.layers[idx].name =
                                            self.rename_text.trim().to_string();
                                        history.record_layer_stack_change(&before, stack);
                                        action = Some(LayerAction::Modified);
                                    }
                                    self.editing_layer_id = None;
                                }
                            } else {
                                let label_text =
                                    RichText::new(&layer_name).color(text_color).strong();
                                let response =
                                    ui.add(egui::Label::new(label_text).sense(Sense::click()));

                                if response.clicked() {
                                    stack.set_active_layer(layer_id);
                                    action = Some(LayerAction::Modified);
                                }

                                if response.double_clicked() {
                                    self.editing_layer_id = Some(layer_id);
                                    self.rename_text = layer_name;
                                }
                            }
                        });
                    });

                ui.add_space(2.0);
            }
        });

        action
    }

    fn get_or_create_thumbnail(
        &mut self,
        ctx: &Context,
        layer_id: LayerId,
        buffer: &paint_core::image::PixelBuffer,
        texture_version: u64,
    ) -> TextureHandle {
        if let Some((handle, ver)) = self.thumbnail_textures.get(&layer_id) {
            if *ver == texture_version {
                return handle.clone();
            }
        }

        let color_img = ColorImage::from_rgba_unmultiplied(
            [buffer.width() as usize, buffer.height() as usize],
            buffer.as_raw(),
        );
        let tex = ctx.load_texture(
            format!("layer_thumb_{}", layer_id.0),
            color_img,
            TextureOptions::NEAREST,
        );
        self.thumbnail_textures
            .insert(layer_id, (tex.clone(), texture_version));
        tex
    }
}
