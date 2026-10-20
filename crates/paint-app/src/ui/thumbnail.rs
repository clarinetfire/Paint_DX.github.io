//! Floating Thumbnail window showing 100% full view of the canvas.

use egui::{ColorImage, Context, TextureHandle, TextureOptions, Vec2, Window};
use paint_core::image::PixelBuffer;

#[derive(Default)]
pub struct ThumbnailView {
    pub is_open: bool,
    texture: Option<TextureHandle>,
    last_version: u64,
}

impl std::fmt::Debug for ThumbnailView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThumbnailView")
            .field("is_open", &self.is_open)
            .field("last_version", &self.last_version)
            .finish()
    }
}

impl ThumbnailView {
    pub fn ui(&mut self, ctx: &Context, buffer: &PixelBuffer, texture_version: u64) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;

        Window::new("Thumbnail")
            .open(&mut is_open)
            .resizable(true)
            .default_size(Vec2::new(160.0, 120.0))
            .show(ctx, |ui| {
                if self.texture.is_none() || self.last_version != texture_version {
                    let color_img = ColorImage::from_rgba_unmultiplied(
                        [buffer.width() as usize, buffer.height() as usize],
                        buffer.as_raw(),
                    );
                    self.texture =
                        Some(ctx.load_texture("thumbnail_tex", color_img, TextureOptions::NEAREST));
                    self.last_version = texture_version;
                }

                if let Some(tex) = &self.texture {
                    let aspect = buffer.width() as f32 / buffer.height() as f32;
                    let avail_w = ui.available_width().max(40.0);
                    let display_size = Vec2::new(avail_w, avail_w / aspect);
                    ui.image((tex.id(), display_size));
                }
            });

        self.is_open = is_open;
    }
}
