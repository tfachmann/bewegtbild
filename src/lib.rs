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
        match self {
            SizeEntry::Percent(percent) => bbox_val * (*percent as f32 / 100.0),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SizeRequest {
    pub width: SizeEntry,
    pub height: SizeEntry,
}

impl SizeRequest {
    pub fn by_bbox(&self, bbox_wh: (f32, f32)) -> (f32, f32) {
        (
            self.width.calc_size(bbox_wh.0),
            self.height.calc_size(bbox_wh.1),
        )
    }
}

impl Default for SizeRequest {
    fn default() -> Self {
        Self {
            width: SizeEntry::Percent(0),
            height: SizeEntry::Percent(0),
        }
    }
}

#[derive(Clone)]
pub struct VideoEntry {
    pub video_path: PathBuf,
    pub pos: SizeRequest,
    pub size: SizeRequest,
}
