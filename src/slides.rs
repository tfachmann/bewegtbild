use std::collections::HashMap;

use egui::ColorImage;

use crate::pdf::PdfRenderer;
use crate::video::VideoPlayer;
use crate::VideoEntry;

#[derive(Default)]
struct ImageState {
    needs_redraw: bool,
}

pub struct SlidesCache {
    slides: Slides,
    window_width: i32,
    window_height: i32,
    current_page_idx: usize,

    needs_redraw: bool,

    rendered_slides: HashMap<usize, (ColorImage, ImageState)>,

    video_entries: Vec<SlidesVideoEntry>,
}

struct SlidesVideoEntry {
    entry: VideoEntry,
    player: VideoPlayer,
}

impl SlidesCache {
    pub fn new(
        slides: Slides,
        window_width: i32,
        window_height: i32,
        video_entries: Vec<VideoEntry>, // TODO: Expect Vec<SlidesVideoEntry> directly
    ) -> Self {
        let video_entries = video_entries
            .into_iter()
            .map(|entry| SlidesVideoEntry {
                entry,
                player: VideoPlayer::new(),
            })
            .collect();
        Self {
            slides,
            window_width,
            window_height,
            current_page_idx: 0,
            needs_redraw: true,
            rendered_slides: HashMap::default(),
            video_entries,
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

    pub fn change_video_entries(&mut self, video_entries: Vec<VideoEntry>) {
        self.video_entries = video_entries
            .into_iter()
            .map(|entry| SlidesVideoEntry {
                entry,
                player: VideoPlayer::new(),
            })
            .collect();
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

    /// Handles the rendering of the videos of this slide to the given context and ui.
    ///
    /// Needs position and size of the slides to render the videos.
    pub fn handle_video(
        &mut self,
        page_idx: usize,
        slide_pos: egui::Pos2,
        slide_size: egui::Vec2,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        for SlidesVideoEntry { entry, player } in self.video_entries.iter_mut() {
            // video should not be rendered for this slide
            if !entry.slide_nums.contains(&page_idx) {
                if player.is_playing() {
                    // TODO: config it in a way where I could also choose not to destroy it
                    player.destroy();
                }
                // => destroy / pause / ...
                continue;
            }

            if !player.is_playing() {
                player.init(ctx, entry.video_path.to_str().unwrap());
                player.start();
            }

            // render video
            let slide_size = (slide_size.x, slide_size.y);
            let scaled_pos = entry.pos.by_bbox(slide_size);
            let scaled_pos = egui::vec2(scaled_pos.0, scaled_pos.1);
            let video_dim = player.size().unwrap();
            let video_dim = (video_dim.x, video_dim.y);
            let scaled_size = entry.size.by_bbox(video_dim, slide_size);
            let scaled_size = egui::vec2(scaled_size.0, scaled_size.1);
            let rect = egui::Rect {
                min: slide_pos + scaled_pos,
                max: slide_pos + scaled_pos + scaled_size,
            };
            // render to ui
            player.render(ui, rect);
        }
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
