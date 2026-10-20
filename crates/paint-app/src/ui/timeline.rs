//! Animation timeline panel UI.
//!
//! Provides transport controls (play, pause, loop, FPS, durations),
//! layer-by-frame cels grid, onion-skin toggles and settings, and animation tags.

use egui::{vec2, Color32, Frame as EguiFrame, Rect, Stroke, Ui};
use paint_core::frames::{AnimTag, AnimationDocument};
use paint_core::layers::LayerId;
use paint_core::onion_skin::{OnionSkinConfig, OnionSkinTint};
use web_time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineAction {
    SelectFrame(usize),
    SelectLayer(LayerId),
    AddFrame { copy_from_prev: bool },
    DuplicateFrame(usize),
    DeleteFrame(usize),
    MoveFrame { from: usize, to: usize },
    SetFrameDuration { frame: usize, duration_ms: u32 },
    SetAllDurations(u32),
    SetFps(u32),
    TogglePlay,
    Stop,
    FirstFrame,
    PrevFrame,
    NextFrame,
    LastFrame,
    AddTag(AnimTag),
    DeleteTag(usize),
    SelectTag(Option<String>),
    ClearCel { frame: usize, layer_id: LayerId },
    ExportSequence,
    ExportSpriteSheet,
    ExportGif,
}

#[derive(Debug)]
pub struct TimelineView {
    pub is_open: bool,
    pub is_playing: bool,
    pub loop_playback: bool,
    pub onion_skin: OnionSkinConfig,
    pub onion_skin_popup_open: bool,
    pub tag_dialog_open: bool,
    pub new_tag_name: String,
    pub new_tag_from: usize,
    pub new_tag_to: usize,
    pub active_tag: Option<String>,
    pub last_tick: Option<Instant>,
    pub playback_accum_ms: f64,
}

impl Default for TimelineView {
    fn default() -> Self {
        Self {
            is_open: true,
            is_playing: false,
            loop_playback: true,
            onion_skin: OnionSkinConfig::default(),
            onion_skin_popup_open: false,
            tag_dialog_open: false,
            new_tag_name: "Walk".to_string(),
            new_tag_from: 0,
            new_tag_to: 0,
            active_tag: None,
            last_tick: None,
            playback_accum_ms: 0.0,
        }
    }
}

impl TimelineView {
    /// Renders the Timeline panel and returns any user actions triggered during this frame.
    pub fn ui(&mut self, ui: &mut Ui, doc: &AnimationDocument) -> Vec<TimelineAction> {
        let mut actions = Vec::new();

        ui.vertical(|ui| {
            // 1. Transport Bar
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 2.0);

                // Playback Navigation buttons
                if ui.button("⏮").on_hover_text("First Frame (Home)").clicked() {
                    actions.push(TimelineAction::FirstFrame);
                }
                if ui.button("◀").on_hover_text("Previous Frame (,)").clicked() {
                    actions.push(TimelineAction::PrevFrame);
                }

                let play_icon = if self.is_playing { "⏸" } else { "▶" };
                let play_btn = ui.button(play_icon);
                if play_btn.on_hover_text("Play / Pause (Space)").clicked() {
                    actions.push(TimelineAction::TogglePlay);
                }

                if ui.button("▶").on_hover_text("Next Frame (.)").clicked() {
                    actions.push(TimelineAction::NextFrame);
                }
                if ui.button("⏭").on_hover_text("Last Frame (End)").clicked() {
                    actions.push(TimelineAction::LastFrame);
                }

                ui.separator();

                // Loop button
                let loop_text = if self.loop_playback {
                    "🔁 Loop: On"
                } else {
                    "🔁 Loop: Off"
                };
                if ui.selectable_label(self.loop_playback, loop_text).clicked() {
                    self.loop_playback = !self.loop_playback;
                }

                ui.separator();

                // Current frame / total frames
                let active_f = doc.active_frame_index;
                let total_f = doc.frames.len();
                ui.label(format!("Frame {} / {}", active_f + 1, total_f));

                // Frame duration editor
                if let Some(frame) = doc.frames.get(active_f) {
                    let mut duration = frame.duration_ms;
                    ui.label("Duration:");
                    let drag = ui.add(
                        egui::DragValue::new(&mut duration)
                            .speed(10.0)
                            .range(10..=5000)
                            .suffix(" ms"),
                    );
                    if drag.changed() {
                        actions.push(TimelineAction::SetFrameDuration {
                            frame: active_f,
                            duration_ms: duration,
                        });
                    }
                }

                // FPS editor
                let mut fps = doc.fps;
                ui.label("FPS:");
                let fps_drag = ui.add(egui::DragValue::new(&mut fps).speed(1.0).range(1..=60));
                if fps_drag.changed() {
                    actions.push(TimelineAction::SetFps(fps));
                }

                ui.separator();

                // Onion skin toggle & settings
                let onion_btn = ui.selectable_label(self.onion_skin.enabled, "🧅 Onion Skin");
                if onion_btn.clicked() {
                    self.onion_skin.enabled = !self.onion_skin.enabled;
                }
                if ui
                    .button("⚙")
                    .on_hover_text("Onion Skin Settings")
                    .clicked()
                {
                    self.onion_skin_popup_open = !self.onion_skin_popup_open;
                }

                ui.separator();

                // Add Frame button
                if ui
                    .button("➕ New Frame")
                    .on_hover_text("Add blank frame")
                    .clicked()
                {
                    actions.push(TimelineAction::AddFrame {
                        copy_from_prev: false,
                    });
                }
                if ui
                    .button("📋 Duplicate Frame")
                    .on_hover_text("Duplicate current frame")
                    .clicked()
                {
                    actions.push(TimelineAction::DuplicateFrame(active_f));
                }

                let can_delete_frame = doc.frames.len() > 1;
                if ui
                    .add_enabled(can_delete_frame, egui::Button::new("🗑 Delete Frame"))
                    .on_hover_text(if can_delete_frame {
                        "Delete current frame"
                    } else {
                        "Cannot delete the only frame"
                    })
                    .clicked()
                {
                    actions.push(TimelineAction::DeleteFrame(active_f));
                }

                // Tag Manager button
                if ui.button("🏷 Add Tag...").clicked() {
                    self.tag_dialog_open = true;
                    self.new_tag_from = active_f;
                    self.new_tag_to = active_f.min(total_f.saturating_sub(1));
                }

                ui.separator();

                // Export buttons
                if ui
                    .button("📤 Export Sequence...")
                    .on_hover_text("Export animation as PNG sequence (Ctrl+Shift+E)")
                    .clicked()
                {
                    actions.push(TimelineAction::ExportSequence);
                }
                if ui
                    .button("📦 Export Sheet...")
                    .on_hover_text("Export animation as Sprite Sheet (Ctrl+Shift+X)")
                    .clicked()
                {
                    actions.push(TimelineAction::ExportSpriteSheet);
                }
                if ui
                    .button("🎞 Export GIF...")
                    .on_hover_text("Export animation as Animated GIF (Ctrl+Shift+G)")
                    .clicked()
                {
                    actions.push(TimelineAction::ExportGif);
                }
            });

            // 2. Onion Skin popup settings
            if self.onion_skin_popup_open {
                EguiFrame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Prev Frames:");
                        ui.add(egui::DragValue::new(&mut self.onion_skin.prev_frames).range(0..=5));
                        ui.label("Next Frames:");
                        ui.add(egui::DragValue::new(&mut self.onion_skin.next_frames).range(0..=5));
                        ui.label("Opacity:");
                        ui.add(
                            egui::DragValue::new(&mut self.onion_skin.opacity)
                                .range(5..=100)
                                .suffix("%"),
                        );

                        let is_tinted = self.onion_skin.tint_mode == OnionSkinTint::RedBlue;
                        let mut tint_check = is_tinted;
                        if ui.checkbox(&mut tint_check, "Red/Blue Tint").changed() {
                            self.onion_skin.tint_mode = if tint_check {
                                OnionSkinTint::RedBlue
                            } else {
                                OnionSkinTint::None
                            };
                        }
                    });
                });
            }

            // 3. Tag creation dialog inline
            if self.tag_dialog_open {
                EguiFrame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Tag Name:");
                        ui.text_edit_singleline(&mut self.new_tag_name);
                        ui.label("From (1-based):");
                        let mut from1 = self.new_tag_from + 1;
                        if ui
                            .add(egui::DragValue::new(&mut from1).range(1..=doc.frames.len()))
                            .changed()
                        {
                            self.new_tag_from = from1.saturating_sub(1);
                        }
                        ui.label("To (1-based):");
                        let mut to1 = self.new_tag_to + 1;
                        if ui
                            .add(egui::DragValue::new(&mut to1).range(1..=doc.frames.len()))
                            .changed()
                        {
                            self.new_tag_to = to1.saturating_sub(1);
                        }

                        if ui.button("Create").clicked() {
                            let tag = AnimTag::new(
                                self.new_tag_name.clone(),
                                self.new_tag_from,
                                self.new_tag_to,
                            );
                            actions.push(TimelineAction::AddTag(tag));
                            self.tag_dialog_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.tag_dialog_open = false;
                        }
                    });
                });
            }

            // 4. Tags List / Banner Chips
            if !doc.tags.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Tags:");
                    for (t_idx, tag) in doc.tags.iter().enumerate() {
                        let is_selected = self.active_tag.as_deref() == Some(&tag.name);
                        let chip_text =
                            format!("{} [{}-{}]", tag.name, tag.from_frame + 1, tag.to_frame + 1);
                        let chip = ui.selectable_label(is_selected, chip_text);

                        if chip.clicked() {
                            if is_selected {
                                self.active_tag = None;
                                actions.push(TimelineAction::SelectTag(None));
                            } else {
                                self.active_tag = Some(tag.name.clone());
                                actions.push(TimelineAction::SelectTag(Some(tag.name.clone())));
                                actions.push(TimelineAction::SelectFrame(tag.from_frame));
                            }
                        }

                        chip.context_menu(|ui| {
                            if ui.button("Delete Tag").clicked() {
                                actions.push(TimelineAction::DeleteTag(t_idx));
                                ui.close_menu();
                            }
                        });
                    }

                    if self.active_tag.is_some() && ui.button("Clear Tag Filter").clicked() {
                        self.active_tag = None;
                        actions.push(TimelineAction::SelectTag(None));
                    }
                });
            }

            ui.separator();

            // 5. Timeline Grid (Scrollable area)
            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let total_frames = doc.frames.len();
                    let layers_count = doc.layers.len();

                    let header_height = 24.0;
                    let row_height = 24.0;
                    let layer_col_width = 130.0;
                    let frame_col_width = 32.0;

                    let grid_w = layer_col_width + (total_frames as f32 * frame_col_width) + 40.0;
                    let grid_h = header_height + (layers_count as f32 * row_height) + 10.0;

                    let (response, painter) =
                        ui.allocate_painter(vec2(grid_w, grid_h), egui::Sense::click());
                    let origin = response.rect.min;

                    // Draw Frame Headers (Columns)
                    for f in 0..total_frames {
                        let col_x = origin.x + layer_col_width + (f as f32 * frame_col_width);
                        let col_rect = Rect::from_min_size(
                            egui::pos2(col_x, origin.y),
                            vec2(frame_col_width, header_height),
                        );

                        let is_active_frame = f == doc.active_frame_index;
                        let bg_color = if is_active_frame {
                            Color32::from_rgb(49, 106, 197) // Classic XP active highlight blue
                        } else {
                            Color32::from_rgb(230, 230, 235)
                        };

                        painter.rect_filled(col_rect, 2.0, bg_color);
                        painter.rect_stroke(
                            col_rect,
                            2.0,
                            Stroke::new(1.0_f32, Color32::from_rgb(180, 180, 190)),
                            egui::StrokeKind::Inside,
                        );

                        let text_color = if is_active_frame {
                            Color32::WHITE
                        } else {
                            Color32::BLACK
                        };
                        painter.text(
                            col_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}", f + 1),
                            egui::FontId::proportional(12.0),
                            text_color,
                        );

                        // Click header to select frame
                        if response.clicked() {
                            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                                if col_rect.contains(pos) {
                                    actions.push(TimelineAction::SelectFrame(f));
                                }
                            }
                        }
                    }

                    // Plus button at end of headers
                    let plus_x =
                        origin.x + layer_col_width + (total_frames as f32 * frame_col_width) + 4.0;
                    let plus_rect = Rect::from_min_size(
                        egui::pos2(plus_x, origin.y),
                        vec2(24.0, header_height),
                    );
                    painter.rect_filled(plus_rect, 2.0, Color32::from_rgb(220, 230, 245));
                    painter.rect_stroke(
                        plus_rect,
                        2.0,
                        Stroke::new(1.0_f32, Color32::from_rgb(150, 170, 200)),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        plus_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "+",
                        egui::FontId::proportional(14.0),
                        Color32::BLACK,
                    );
                    if response.clicked() {
                        if let Some(pos) = ui.ctx().pointer_interact_pos() {
                            if plus_rect.contains(pos) {
                                actions.push(TimelineAction::AddFrame {
                                    copy_from_prev: true,
                                });
                            }
                        }
                    }

                    // Draw Layer Rows (rendered from top of stack to bottom)
                    for (row_idx, meta) in doc.layers.iter().rev().enumerate() {
                        let row_y = origin.y + header_height + (row_idx as f32 * row_height);
                        let is_active_layer = meta.id == doc.active_layer_id;

                        // Layer label cell
                        let layer_rect = Rect::from_min_size(
                            egui::pos2(origin.x, row_y),
                            vec2(layer_col_width, row_height),
                        );
                        let layer_bg = if is_active_layer {
                            Color32::from_rgb(220, 235, 252)
                        } else {
                            Color32::from_rgb(245, 245, 248)
                        };
                        painter.rect_filled(layer_rect, 0.0, layer_bg);
                        painter.rect_stroke(
                            layer_rect,
                            0.0,
                            Stroke::new(0.5_f32, Color32::from_rgb(200, 200, 205)),
                            egui::StrokeKind::Inside,
                        );

                        let name_display = if meta.locked {
                            format!("🔒 {}", meta.name)
                        } else if !meta.visible {
                            format!("👁❌ {}", meta.name)
                        } else {
                            meta.name.clone()
                        };
                        painter.text(
                            egui::pos2(layer_rect.min.x + 6.0, layer_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            name_display,
                            egui::FontId::proportional(11.0),
                            Color32::from_rgb(30, 30, 30),
                        );

                        if response.clicked() {
                            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                                if layer_rect.contains(pos) {
                                    actions.push(TimelineAction::SelectLayer(meta.id));
                                }
                            }
                        }

                        // Draw Cels for this Layer across all frames
                        for f in 0..total_frames {
                            let cel_x = origin.x + layer_col_width + (f as f32 * frame_col_width);
                            let cel_rect = Rect::from_min_size(
                                egui::pos2(cel_x, row_y),
                                vec2(frame_col_width, row_height),
                            );

                            let is_active_cel = is_active_layer && f == doc.active_frame_index;
                            let cel_bg = if is_active_cel {
                                Color32::from_rgb(190, 220, 255)
                            } else if f == doc.active_frame_index {
                                Color32::from_rgb(235, 243, 255)
                            } else {
                                Color32::from_rgb(255, 255, 255)
                            };

                            painter.rect_filled(cel_rect, 0.0, cel_bg);
                            painter.rect_stroke(
                                cel_rect,
                                0.0,
                                Stroke::new(
                                    if is_active_cel { 1.5_f32 } else { 0.5_f32 },
                                    if is_active_cel {
                                        Color32::from_rgb(49, 106, 197)
                                    } else {
                                        Color32::from_rgb(215, 215, 220)
                                    },
                                ),
                                egui::StrokeKind::Inside,
                            );

                            // Cel content indicator
                            let is_empty = doc
                                .get_cel(f, meta.id)
                                .map(|c| c.is_empty())
                                .unwrap_or(true);

                            if !is_empty {
                                // Filled dot
                                painter.circle_filled(
                                    cel_rect.center(),
                                    4.5,
                                    Color32::from_rgb(60, 100, 160),
                                );
                            } else {
                                // Empty ring
                                painter.circle_stroke(
                                    cel_rect.center(),
                                    3.5,
                                    Stroke::new(1.0_f32, Color32::from_rgb(180, 180, 190)),
                                );
                            }

                            // Click cel to select frame and layer
                            if response.clicked() {
                                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                                    if cel_rect.contains(pos) {
                                        actions.push(TimelineAction::SelectFrame(f));
                                        actions.push(TimelineAction::SelectLayer(meta.id));
                                    }
                                }
                            }
                        }
                    }
                });
        });

        actions
    }
}
