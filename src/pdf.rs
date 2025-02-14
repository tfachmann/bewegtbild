use std::{fs, path::PathBuf};

use egui::ColorImage;
use image::DynamicImage;
use pdfium_render::prelude::*;

pub struct PdfRenderer {
    /// Instance to pdf rendering
    pdfium: Pdfium,
    /// Bytes of the loaded document
    document_bytes: Vec<u8>,
    /// Path of the loaded document
    path: PathBuf,
    /// Quick access to the number of pages
    pub num_pages: usize,
    /// Quick access to the rendering config
    pub render_config: PdfRenderConfig,
}

fn load_and_calc_pages(pdfium: &Pdfium, path: &PathBuf) -> (Vec<u8>, usize) {
    let password = None;
    let document_bytes = fs::read(path)
        .unwrap_or_else(|_| panic!("Could not load pdf document at {}", path.to_string_lossy()));
    let document = pdfium
        .load_pdf_from_byte_slice(&document_bytes, password)
        .expect("Loading documents failed, TODO: change function signature.");
    let num_pages = document.pages().len() as usize;
    drop(document);
    (document_bytes, num_pages)
}

impl PdfRenderer {
    pub fn new(render_config: PdfRenderConfig, pdf_path: PathBuf) -> Self {
        #[cfg(not(feature = "static"))]
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .expect("Could not access Pdfium bindings."),
        );
        #[cfg(feature = "static")]
        let pdfium = Pdfium::new(Pdfium::bind_to_statically_linked_library().unwrap());
        println!("Loading PDF document...");
        let (document_bytes, num_pages) = load_and_calc_pages(&pdfium, &pdf_path);

        Self {
            pdfium,
            document_bytes,
            path: pdf_path,
            num_pages,
            render_config,
        }
    }

    fn document(&self) -> PdfDocument<'_> {
        let password = None;
        self.pdfium
            .load_pdf_from_byte_slice(&self.document_bytes, password)
            .unwrap_or_else(|_| {
                panic!(
                    "Document `{}` seems to be corrupted.",
                    self.path.to_string_lossy()
                )
            })
    }

    /// Renders the page at the given index.
    fn image_by_page(&self, page_idx: usize) -> Option<DynamicImage> {
        let document = self.document();
        let pages = document.pages().iter().collect::<Vec<_>>();
        let image = pages
            .get(page_idx)?
            .render_with_config(&self.render_config)
            .ok()?
            .as_image();
        Some(image)
    }

    pub fn load_document(&mut self, path: PathBuf) {
        let (bytes, num_pages) = load_and_calc_pages(&self.pdfium, &path);
        self.document_bytes = bytes;
        self.num_pages = num_pages;
        self.path = path.to_owned();
    }

    pub fn set_size(&mut self, size: (i32, i32)) {
        self.render_config = PdfRenderConfig::new()
            .set_target_width(size.0)
            .set_maximum_height(size.1);
    }

    pub fn render_page(&self, page_idx: usize) -> Option<ColorImage> {
        println!("Rendering Page {}", page_idx);
        let rgba_image = self.image_by_page(page_idx)?.to_rgba8();
        let size = [rgba_image.width() as usize, rgba_image.height() as usize];
        let color_image =
            ColorImage::from_rgba_unmultiplied(size, rgba_image.as_flat_samples().as_slice());
        Some(color_image)
    }
}
