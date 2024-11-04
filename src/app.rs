use egui::{ColorImage, TextureHandle};
use pdfium_render::prelude::PdfRenderConfig;

use crate::{
    pdf::PdfRenderer,
    slides::{Slides, SlidesCache},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    // Example stuff:
    slides: SlidesCache,
    texture: TextureHandle,

    requested_page_idx: usize,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Self {
            slides: SlidesCache::new(
                Slides::new(PdfRenderer::new(
                    PdfRenderConfig::new(),
                    "./test.pdf".into(),
                )),
                100,
                100,
            ),
            texture: cc.egui_ctx.load_texture(
                "slides_page",
                ColorImage::example(),
                Default::default(),
            ),
            requested_page_idx: 0,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::L) {
                    println!("ArrowRight Pressed!");
                    self.requested_page_idx += 1;
                }
                if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::H) {
                    println!("ArrowRight Pressed!");
                    self.requested_page_idx = self.requested_page_idx.saturating_sub(1);
                }
            });

            let size = ctx.input(|i: &egui::InputState| i.screen_rect());
            let width = size.max.x;
            let height = size.max.y;
            println!("window size: ({}, {})", width, height);
            self.slides.change_size(width as i32, height as i32);

            if let Some(img) = self.slides.get_page(self.requested_page_idx) {
                self.texture.set(img, Default::default());
            }

            let size = self.texture.size_vec2();
            let sized_texture = egui::load::SizedTexture::new(self.texture.id(), size);
            ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
