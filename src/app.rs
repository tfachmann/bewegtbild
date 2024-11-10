use std::path::PathBuf;

use egui::{ColorImage, TextureHandle};
use pdfium_render::prelude::PdfRenderConfig;
use std::collections::HashMap;
use std::sync::mpsc;

use crate::{
    pdf::PdfRenderer,
    slides::{Slides, SlidesCache},
    VideoEntry,
};

fn is_num(key: &egui::Key) -> bool {
    use egui::Key;
    match key {
        Key::Num0
        | Key::Num1
        | Key::Num2
        | Key::Num3
        | Key::Num4
        | Key::Num5
        | Key::Num6
        | Key::Num7
        | Key::Num8
        | Key::Num9 => true,
        _ => false,
    }
}

fn key_to_num(key: &egui::Key) -> Option<usize> {
    use egui::Key;
    match key {
        Key::Num0 => Some(0),
        Key::Num1 => Some(1),
        Key::Num2 => Some(2),
        Key::Num3 => Some(3),
        Key::Num4 => Some(4),
        Key::Num5 => Some(5),
        Key::Num6 => Some(6),
        Key::Num7 => Some(7),
        Key::Num8 => Some(8),
        Key::Num9 => Some(9),
        _ => None,
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    // Example stuff:
    slides: SlidesCache,
    texture: TextureHandle,
    config_changed_rx: Option<mpsc::Receiver<HashMap<usize, Vec<VideoEntry>>>>,

    requested_page_idx: usize,

    key_stack: Vec<egui::Key>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        pdf_path: PathBuf,
        video_map: HashMap<usize, Vec<VideoEntry>>,
        config_changed_rx: Option<mpsc::Receiver<HashMap<usize, Vec<VideoEntry>>>>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let pdf_renderer = PdfRenderer::new(PdfRenderConfig::new(), pdf_path);

        Self {
            slides: SlidesCache::new(Slides::new(pdf_renderer), 100, 100, video_map),
            texture: cc.egui_ctx.load_texture(
                "slides_page",
                ColorImage::example(),
                Default::default(),
            ),
            requested_page_idx: 0,
            key_stack: Vec::new(),
            config_changed_rx,
        }
    }

    fn stack_as_num(&self) -> Option<usize> {
        if self.key_stack.is_empty() || !self.key_stack.iter().all(|key| is_num(key)) {
            None
        } else {
            Some(
                self.key_stack
                    .iter()
                    .map(|key| key_to_num(key).unwrap())
                    .fold(0, |acc, x| acc * 10 + x),
            )
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input(|i| {
                // next slide
                if i.key_pressed(egui::Key::ArrowRight)
                    || i.key_pressed(egui::Key::L)
                    || i.key_pressed(egui::Key::N)
                    || i.key_pressed(egui::Key::Space)
                {
                    if self.requested_page_idx < self.slides.num_pages() - 1 {
                        self.requested_page_idx += 1;
                    }
                }
                // previous slide
                if i.key_pressed(egui::Key::ArrowLeft)
                    || i.key_pressed(egui::Key::H)
                    || i.key_pressed(egui::Key::P)
                {
                    self.requested_page_idx = self.requested_page_idx.saturating_sub(1);
                }
                // jump to slide (or last slide)
                if i.modifiers.shift_only() && i.key_pressed(egui::Key::G) {
                    if let Some(num) = self.stack_as_num() {
                        self.requested_page_idx = if num >= self.slides.num_pages() - 1 {
                            self.slides.num_pages() - 1
                        } else {
                            num
                        }
                    } else {
                        self.requested_page_idx = self.slides.num_pages() - 1
                    }
                    self.key_stack.clear();
                }
                // jump to slide
                if i.key_pressed(egui::Key::Enter) {
                    if let Some(num) = self.stack_as_num() {
                        self.requested_page_idx = if num >= self.slides.num_pages() - 1 {
                            self.slides.num_pages() - 1
                        } else {
                            num
                        }
                    }
                    self.key_stack.clear();
                }
                // cancel key stack
                if i.key_pressed(egui::Key::Escape) {
                    self.key_stack.clear();
                }
                // number pressed
                if i.key_pressed(egui::Key::Num0)
                    || i.key_pressed(egui::Key::Num1)
                    || i.key_pressed(egui::Key::Num2)
                    || i.key_pressed(egui::Key::Num3)
                    || i.key_pressed(egui::Key::Num4)
                    || i.key_pressed(egui::Key::Num5)
                    || i.key_pressed(egui::Key::Num6)
                    || i.key_pressed(egui::Key::Num7)
                    || i.key_pressed(egui::Key::Num8)
                    || i.key_pressed(egui::Key::Num9)
                {
                    let pressed_key = i.events.iter().find_map(|ev| {
                        if let egui::Event::Key { key, .. } = ev {
                            Some(key)
                        } else {
                            None
                        }
                    });
                    if let Some(key) = pressed_key {
                        self.key_stack.push(key.clone())
                    }
                }
            });

            if let Some(config_changed_rx) = &self.config_changed_rx {
                if let Ok(new_video_map) = config_changed_rx.try_recv() {
                    println!("Config changed from UI");
                    self.slides.change_video_map(new_video_map);
                }
                // necessary to register changes to the config
                ctx.request_repaint();
            }

            let size = ctx.input(|i: &egui::InputState| i.screen_rect());
            let width = size.max.x;
            let height = size.max.y;
            self.slides.change_size(width as i32, height as i32);

            if let Some(img) = self.slides.get_page(self.requested_page_idx) {
                self.texture.set(img, Default::default());
            }

            let slide_size = self.texture.size_vec2();
            let sized_texture = egui::load::SizedTexture::new(self.texture.id(), slide_size);
            let available_rect = ui.available_rect_before_wrap();
            let slide_pos = available_rect.center() - 0.5 * slide_size;
            let img_rect = egui::Rect::from_min_size(slide_pos, slide_size);
            ui.put(
                img_rect,
                egui::Image::new(sized_texture).fit_to_exact_size(slide_size),
            );
            self.slides
                .handle_video(self.requested_page_idx, slide_pos, slide_size, ctx, ui);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
