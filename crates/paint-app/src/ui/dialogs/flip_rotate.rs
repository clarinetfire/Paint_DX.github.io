//! Flip and Rotate dialog.

use egui::{Context, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipRotateAction {
    FlipHorizontal,
    FlipVertical,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogChoice {
    Flip,
    Rotate,
}

#[derive(Debug)]
pub struct FlipRotateDialog {
    pub is_open: bool,
    pub choice: DialogChoice,
    pub flip_horizontal: bool,
    pub rotate_angle: u16, // 90, 180, 270
}

impl Default for FlipRotateDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            choice: DialogChoice::Flip,
            flip_horizontal: true,
            rotate_angle: 90,
        }
    }
}

impl FlipRotateDialog {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Shows the dialog. Returns `Some(action)` when OK is pressed.
    pub fn ui(&mut self, ctx: &Context) -> Option<FlipRotateAction> {
        if !self.is_open {
            return None;
        }

        let mut result = None;
        let mut close = false;

        Window::new("Flip and Rotate")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(260.0);

                ui.group(|ui| {
                    ui.radio_value(&mut self.choice, DialogChoice::Flip, "Flip");
                    ui.indent("flip_options", |ui| {
                        let is_flip = self.choice == DialogChoice::Flip;
                        ui.add_enabled_ui(is_flip, |ui| {
                            ui.radio_value(&mut self.flip_horizontal, true, "Flip horizontal");
                            ui.radio_value(&mut self.flip_horizontal, false, "Flip vertical");
                        });
                    });

                    ui.separator();

                    ui.radio_value(&mut self.choice, DialogChoice::Rotate, "Rotate by angle");
                    ui.indent("rotate_options", |ui| {
                        let is_rotate = self.choice == DialogChoice::Rotate;
                        ui.add_enabled_ui(is_rotate, |ui| {
                            ui.radio_value(&mut self.rotate_angle, 90, "90°");
                            ui.radio_value(&mut self.rotate_angle, 180, "180°");
                            ui.radio_value(&mut self.rotate_angle, 270, "270°");
                        });
                    });
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                        if ui.button("OK").clicked() {
                            let action = match self.choice {
                                DialogChoice::Flip => {
                                    if self.flip_horizontal {
                                        FlipRotateAction::FlipHorizontal
                                    } else {
                                        FlipRotateAction::FlipVertical
                                    }
                                }
                                DialogChoice::Rotate => match self.rotate_angle {
                                    180 => FlipRotateAction::Rotate180,
                                    270 => FlipRotateAction::Rotate270,
                                    _ => FlipRotateAction::Rotate90,
                                },
                            };
                            result = Some(action);
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
