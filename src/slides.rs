use egui::ColorImage;

use crate::pdf::PdfRenderer;
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
        }
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
