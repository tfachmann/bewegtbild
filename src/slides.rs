use egui::ColorImage;

use crate::pdf::PdfRenderer;
use crate::video::VideoPlayer;
use std::collections::HashMap;

struct ImageState {
    needs_redraw: bool,
}

impl Default for ImageState {
    fn default() -> Self {
        ImageState {
            needs_redraw: false,
        }
    }
}

pub struct SlidesCache {
    slides: Slides,
    window_width: i32,
    window_height: i32,
    current_page_idx: usize,

    needs_redraw: bool,

    rendered_slides: HashMap<usize, (ColorImage, ImageState)>,

    video_player: VideoPlayer,
}

impl SlidesCache {
    pub fn new(slides: Slides, window_width: i32, window_height: i32) -> Self {
        Self {
            slides,
            window_width,
            window_height,
            current_page_idx: 0,
            needs_redraw: true,
            rendered_slides: HashMap::default(),
            video_player: VideoPlayer::new(),
        }
    }

    pub fn num_pages(&self) -> usize {
        self.slides.pdf_renderer.num_pages
    }

    pub fn change_size(&mut self, window_width: i32, window_height: i32) {
        // this will outdate the cache, and trigger a re-generation
        if self.window_width != window_width || self.window_height != window_height {
            self.window_width = window_width;
            self.window_height = window_height;
            // trigger regeneration
            self.needs_redraw = true;
            self.rendered_slides
                .values_mut()
                .for_each(|(_, state)| state.needs_redraw = true);
        }
    }

    fn update_img(&mut self) -> Option<ColorImage> {
        let img_res =
            self.slides
                .change_page(self.current_page_idx, self.window_width, self.window_height);
        if let Some(img) = &img_res {
            self.rendered_slides
                .insert(self.current_page_idx, (img.clone(), Default::default()));
        }
        img_res
    }

    pub fn get_page(&mut self, page_idx: usize) -> Option<ColorImage> {
        // changes the current page index
        // (should prioritize the generation of this page)
        // returns the (cached) and rendered page

        if !self.needs_redraw && page_idx == self.current_page_idx {
            return None;
        }

        self.current_page_idx = page_idx;
        match self.rendered_slides.get(&page_idx) {
            // use cache (img available, and no need to redraw)
            Some((
                img,
                ImageState {
                    needs_redraw: false,
                },
            )) => Some(img.clone()),
            // update (img available, but needs to redraw)
            Some((_, ImageState { needs_redraw: true })) => self.update_img(),
            // update (no img available)
            None => self.update_img(),
        }
    }

    pub fn handle_video(&mut self, page_idx: usize, ctx: &egui::Context, ui: &mut egui::Ui) {
        let video_path_01 = "./test.mp4";
        let video_path_02 = "./test_large.mp4";
        let video_path_03 = "./test.gif";
        // video => slide number matching
        match page_idx {
            0 | 1 | 2 | 3 | 6 | 7 => {
                // do not restart video if it is already playing
                // (videos can generally be spread over multiple slides)
                if !self.video_player.is_path_playing(video_path_01) {
                    self.video_player.init(ctx, &video_path_01);
                    self.video_player.start();
                }
            }
            4 => {
                if !self.video_player.is_path_playing(video_path_02) {
                    self.video_player.init(ctx, &video_path_02);
                    self.video_player.start();
                }
            }
            9 | 10 => {
                if !self.video_player.is_path_playing(video_path_03) {
                    self.video_player.init(ctx, &video_path_03);
                    self.video_player.start();
                }
            }
            _ => self.video_player.destroy(),
        }

        let rect = egui::Rect {
            min: egui::pos2(200.0, 300.0),
            max: egui::pos2(600.0, 800.0),
        };
        self.video_player.render(ui, rect);
        println!("{:?}", self.video_player.size())
    }
}

pub struct Slides {
    /// Instance of the PDF Renderer
    pdf_renderer: PdfRenderer,
}

impl Slides {
    pub fn new(pdf_renderer: PdfRenderer) -> Self {
        Self { pdf_renderer }
    }

    pub fn num_pages(&self) -> usize {
        self.pdf_renderer.num_pages
    }

    pub fn change_page(
        &mut self,
        page_idx: usize,
        window_width: i32,
        window_height: i32,
    ) -> Option<ColorImage> {
        self.pdf_renderer.set_size((window_width, window_height));
        self.pdf_renderer.render_page(page_idx)
    }
}
