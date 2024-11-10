#![warn(clippy::all, rust_2018_idioms)]

mod app;
use std::path::PathBuf;

pub use app::TemplateApp;

mod config;
pub use config::Config;
mod pdf;
mod slides;
mod video;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SizeEntry {
    Percent(usize),
}

impl SizeEntry {
    pub fn calc_size(&self, bbox_val: f32) -> f32 {
        match *self {
            SizeEntry::Percent(percent) => bbox_val * (percent as f32 / 100.0),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PosRequest {
    width: SizeEntry,
    height: SizeEntry,
}

impl PosRequest {
    pub fn by_bbox(&self, bbox_wh: (f32, f32)) -> (f32, f32) {
        (
            self.width.calc_size(bbox_wh.0),
            self.height.calc_size(bbox_wh.1),
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SizeRequest {
    Size(SizeEntry, SizeEntry),
    AutoWidth(SizeEntry),
    AutoHeight(SizeEntry),
}

impl SizeRequest {
    pub fn by_bbox(&self, video_dim: (f32, f32), bbox_wh: (f32, f32)) -> (f32, f32) {
        let (bbox_w, bbox_h) = bbox_wh;
        match self {
            SizeRequest::Size(w, h) => (w.calc_size(bbox_w), h.calc_size(bbox_h)),
            SizeRequest::AutoWidth(h) => {
                let (dim_w, dim_h) = video_dim;
                let ratio = dim_w / dim_h;
                let h_render = h.calc_size(bbox_h);
                let w_render = h_render * ratio;
                (w_render, h_render)
            }
            SizeRequest::AutoHeight(w) => {
                let (dim_w, dim_h) = video_dim;
                let ratio = dim_h / dim_w;
                let w_render = w.calc_size(bbox_w);
                let h_render = w_render * ratio;
                (w_render, h_render)
            }
        }
    }
}

impl Default for SizeRequest {
    fn default() -> Self {
        Self::AutoHeight(SizeEntry::Percent(30))
    }
}

#[derive(Clone)]
pub struct VideoEntry {
    pub video_path: PathBuf,
    // TODO: pos should _not_ be of type SizeRequest
    pub pos: PosRequest,
    pub size: SizeRequest,
}
