//! Paint DX - Rust rewrite of classic XP Paint for Windows 11.

mod platform;
mod ui;

use eframe::egui;
use paint_core::image::PixelBuffer;
use paint_core::palette::WHITE;
use ui::canvas::CanvasView;
use ui::layers::LayersView;
use ui::palette::PaletteView;
use ui::timeline::TimelineView;
use ui::toolbox::ToolboxView;

pub struct PaintApp {
    buffer: PixelBuffer,
    canvas: CanvasView,
    toolbox: ToolboxView,
    palette: PaletteView,
    layers: LayersView,
    timeline: TimelineView,
    status_text: String,
}

impl Default for PaintApp {
    fn default() -> Self {
        Self {
            buffer: PixelBuffer::new(640, 480, WHITE),
            canvas: CanvasView::default(),
            toolbox: ToolboxView::default(),
            palette: PaletteView::default(),
            layers: LayersView::default(),
            timeline: TimelineView::default(),
            status_text: "For Help, click Help Topics on the Help Menu.".to_string(),
        }
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New\tCtrl+N").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Open...\tCtrl+O").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Save\tCtrl+S").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo\tCtrl+Z").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Repeat\tF4").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Cut\tCtrl+X").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Copy\tCtrl+C").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Paste\tCtrl+V").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.layers.is_open, "Layers Panel");
                    ui.checkbox(&mut self.timeline.is_open, "Timeline Panel");
                    ui.checkbox(&mut self.canvas.show_grid, "Grid\tCtrl+G");
                });

                ui.menu_button("Image", |ui| {
                    if ui.button("Flip/Rotate...\tCtrl+R").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Stretch/Skew...\tCtrl+W").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Invert Colors\tCtrl+I").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Attributes...\tCtrl+E").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Clear Image\tCtrl+Shift+N").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Colors", |ui| {
                    if ui.button("Edit Colors...").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Help Topics").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("About Paint DX").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_text);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{}x{}px",
                        self.buffer.width(),
                        self.buffer.height()
                    ));
                });
            });
        });

        // Bottom palette bar
        egui::TopBottomPanel::bottom("palette_panel").show(ctx, |ui| {
            self.palette.ui(ui);
        });

        // Optional bottom timeline panel
        if self.timeline.is_open {
            egui::TopBottomPanel::bottom("timeline_panel")
                .resizable(true)
                .default_height(100.0)
                .show(ctx, |ui| {
                    self.timeline.ui(ui);
                });
        }

        // Left toolbox panel
        egui::SidePanel::left("toolbox_panel")
            .resizable(false)
            .default_width(60.0)
            .show(ctx, |ui| {
                self.toolbox.ui(ui);
            });

        // Optional right layers panel
        if self.layers.is_open {
            egui::SidePanel::right("layers_panel")
                .resizable(true)
                .default_width(180.0)
                .show(ctx, |ui| {
                    self.layers.ui(ui);
                });
        }

        // Central canvas panel
        egui::CentralPanel::default().show(ctx, |ui| {
            self.canvas.ui(ui, &self.buffer);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("untitled - Paint DX")
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Paint DX",
        native_options,
        Box::new(|_cc| Ok(Box::new(PaintApp::default()))),
    )
}
