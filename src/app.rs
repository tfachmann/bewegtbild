use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use egui::{ColorImage, TextureHandle, ViewportCommand};
use pdfium_render::prelude::PdfRenderConfig;
use std::sync::mpsc;

use crate::{
    pdf::PdfRenderer,
    slides::{Slides, SlidesCache},
    MonitorRect, VideoEntry,
};

const AUDIENCE_VIEWPORT_ID: &str = "audience";

fn is_num(key: &egui::Key) -> bool {
    use egui::Key;
    matches!(
        key,
        Key::Num0
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Num5
            | Key::Num6
            | Key::Num7
            | Key::Num8
            | Key::Num9
    )
}

fn format_ms(ms: u64) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
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

#[derive(Default)]
struct DeferredActions {
    cycle_audience_monitor: bool,
    toggle_audience_fullscreen: bool,
    toggle_presenter_mode: bool,
}

struct PlacementStep {
    target_monitor: usize,
    final_fullscreen: bool,
    countdown: u8,
}

pub struct TemplateApp {
    slides: SlidesCache,
    current_tex: TextureHandle,
    next_tex: TextureHandle,
    config_changed_rx: Option<mpsc::Receiver<Vec<VideoEntry>>>,

    requested_page_idx: usize,
    last_uploaded_curr: Option<usize>,
    last_uploaded_next: Option<usize>,

    key_stack: Vec<egui::Key>,

    monitors: Vec<MonitorRect>,
    audience_monitor_idx: Option<usize>,
    audience_fullscreen: bool,
    presenter_mode: bool,
    audience_window_alive: bool,
    placement: Option<PlacementStep>,

    seek_scrub_paused: HashSet<String>,

    start_time: Instant,
}

impl TemplateApp {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        pdf_path: PathBuf,
        config: Vec<VideoEntry>,
        config_changed_rx: Option<mpsc::Receiver<Vec<VideoEntry>>>,
        monitors: Vec<MonitorRect>,
        audience_monitor_idx: Option<usize>,
        presenter_mode: bool,
    ) -> Self {
        let pdf_renderer = PdfRenderer::new(PdfRenderConfig::new(), pdf_path);

        Self {
            slides: SlidesCache::new(Slides::new(pdf_renderer), 100, 100, config),
            current_tex: cc.egui_ctx.load_texture(
                "current_slide",
                ColorImage::example(),
                Default::default(),
            ),
            next_tex: cc.egui_ctx.load_texture(
                "next_slide",
                ColorImage::example(),
                Default::default(),
            ),
            requested_page_idx: 0,
            last_uploaded_curr: None,
            last_uploaded_next: None,
            key_stack: Vec::new(),
            config_changed_rx,
            monitors,
            audience_monitor_idx,
            audience_fullscreen: true,
            presenter_mode,
            audience_window_alive: false,
            placement: None,
            seek_scrub_paused: HashSet::new(),
            start_time: Instant::now(),
        }
    }

    fn start_placement(&mut self, target_monitor: usize, final_fullscreen: bool) {
        self.placement = Some(PlacementStep {
            target_monitor,
            final_fullscreen,
            countdown: 3,
        });
    }

    fn tick_placement(&mut self, ctx: &egui::Context) {
        let Some(step) = self.placement.as_mut() else {
            return;
        };
        let audience_id = egui::ViewportId::from_hash_of(AUDIENCE_VIEWPORT_ID);
        match step.countdown {
            3 => {
                ctx.send_viewport_cmd_to(audience_id, ViewportCommand::Fullscreen(false));
            }
            2 => {
                if let Some(m) = self.monitors.get(step.target_monitor).copied() {
                    ctx.send_viewport_cmd_to(
                        audience_id,
                        ViewportCommand::OuterPosition(egui::pos2(m.x as f32, m.y as f32)),
                    );
                }
            }
            1 => {
                if step.final_fullscreen {
                    ctx.send_viewport_cmd_to(audience_id, ViewportCommand::Fullscreen(true));
                }
            }
            _ => {}
        }
        step.countdown = step.countdown.saturating_sub(1);
        if step.countdown == 0 {
            self.placement = None;
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(80));
        }
    }

    fn stack_as_num(&self) -> Option<usize> {
        if self.key_stack.is_empty() || !self.key_stack.iter().all(is_num) {
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

    fn handle_input(&mut self, ctx: &egui::Context) -> DeferredActions {
        let mut actions = DeferredActions::default();
        ctx.input(|i| {
            let num_pages = self.slides.num_pages();
            if (i.key_pressed(egui::Key::ArrowRight)
                || (!i.modifiers.shift && i.key_pressed(egui::Key::L))
                || i.key_pressed(egui::Key::Space)
                || i.key_pressed(egui::Key::PageDown))
                && self.requested_page_idx < num_pages - 1
            {
                self.requested_page_idx += 1;
            }
            if i.key_pressed(egui::Key::ArrowLeft)
                || (!i.modifiers.shift && i.key_pressed(egui::Key::H))
                || i.key_pressed(egui::Key::PageUp)
            {
                self.requested_page_idx = self.requested_page_idx.saturating_sub(1);
            }
            if i.key_pressed(egui::Key::P) {
                self.slides.toggle_pause_current(self.requested_page_idx);
            }
            if i.modifiers.shift_only() && i.key_pressed(egui::Key::H) {
                self.slides
                    .step_frame_current(self.requested_page_idx, false);
            }
            if i.modifiers.shift_only() && i.key_pressed(egui::Key::L) {
                self.slides
                    .step_frame_current(self.requested_page_idx, true);
            }
            if i.modifiers.shift_only() && i.key_pressed(egui::Key::R) {
                self.slides.restart_current(self.requested_page_idx);
            }
            if i.modifiers.shift_only() && i.key_pressed(egui::Key::G) {
                if let Some(num) = self.stack_as_num() {
                    self.requested_page_idx = num.min(num_pages - 1);
                } else {
                    self.requested_page_idx = num_pages - 1;
                }
                self.key_stack.clear();
            }
            if i.key_pressed(egui::Key::Enter) {
                if let Some(num) = self.stack_as_num() {
                    self.requested_page_idx = num.min(num_pages - 1);
                }
                self.key_stack.clear();
            }
            if i.key_pressed(egui::Key::Escape) {
                self.key_stack.clear();
            }
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
                    self.key_stack.push(*key)
                }
            }
            if i.key_pressed(egui::Key::F2) {
                actions.toggle_presenter_mode = true;
            }
            if i.key_pressed(egui::Key::F5) {
                actions.cycle_audience_monitor = true;
            }
            if i.key_pressed(egui::Key::F11) {
                actions.toggle_audience_fullscreen = true;
            }
        });
        actions
    }

    fn apply_deferred_actions(&mut self, ctx: &egui::Context, actions: DeferredActions) {
        if actions.toggle_presenter_mode {
            self.presenter_mode = !self.presenter_mode;
            if !self.presenter_mode {
                let audience_id = egui::ViewportId::from_hash_of(AUDIENCE_VIEWPORT_ID);
                ctx.send_viewport_cmd_to(audience_id, ViewportCommand::Close);
                self.audience_window_alive = false;
                self.placement = None;
            }
        }
        if actions.cycle_audience_monitor && !self.monitors.is_empty() && self.presenter_mode {
            let next_idx = match self.audience_monitor_idx {
                Some(idx) => (idx + 1) % self.monitors.len(),
                None => 0,
            };
            self.audience_monitor_idx = Some(next_idx);
            eprintln!(
                "F5: switching audience to monitor {} ({:?})",
                next_idx, self.monitors[next_idx]
            );
            self.start_placement(next_idx, self.audience_fullscreen);
        }
        if actions.toggle_audience_fullscreen && self.presenter_mode {
            self.audience_fullscreen = !self.audience_fullscreen;
            let audience_id = egui::ViewportId::from_hash_of(AUDIENCE_VIEWPORT_ID);
            ctx.send_viewport_cmd_to(
                audience_id,
                ViewportCommand::Fullscreen(self.audience_fullscreen),
            );
        }
    }

    fn audience_pixel_size(&self, ctx: &egui::Context) -> (i32, i32) {
        // 1. live audience viewport
        let audience_id = egui::ViewportId::from_hash_of(AUDIENCE_VIEWPORT_ID);
        let (rect, ppp) = ctx.input_for(audience_id, |i| (i.screen_rect(), i.pixels_per_point()));
        let w = (rect.max.x - rect.min.x) * ppp;
        let h = (rect.max.y - rect.min.y) * ppp;
        if w > 1.0 && h > 1.0 {
            return (w as i32, h as i32);
        }
        // 2. fall back to display-info physical pixels of the target monitor
        if let Some(idx) = self.audience_monitor_idx {
            if let Some(m) = self.monitors.get(idx) {
                return (m.width as i32, m.height as i32);
            }
        }
        // 3. sane default
        (1920, 1080)
    }

    fn upload_textures_if_needed(&mut self, ctx: &egui::Context) {
        let (width, height) = if self.presenter_mode {
            self.audience_pixel_size(ctx)
        } else {
            let rect = ctx.input(|i| i.screen_rect());
            let ppp = ctx.input(|i| i.pixels_per_point());
            (
                ((rect.max.x - rect.min.x) * ppp) as i32,
                ((rect.max.y - rect.min.y) * ppp) as i32,
            )
        };
        self.slides.change_size(width, height);

        let curr_idx = self.requested_page_idx;
        let curr_changed = self.last_uploaded_curr != Some(curr_idx);
        if curr_changed {
            if let Some(img) = self.slides.get_page(curr_idx) {
                self.current_tex.set(img, Default::default());
                self.last_uploaded_curr = Some(curr_idx);
            }
        }

        if self.presenter_mode {
            let next_idx = curr_idx + 1;
            if next_idx < self.slides.num_pages() {
                let next_changed = self.last_uploaded_next != Some(next_idx);
                if next_changed {
                    if let Some(img) = self.slides.get_page(next_idx) {
                        self.next_tex.set(img, Default::default());
                        self.last_uploaded_next = Some(next_idx);
                    }
                    if let Some(img) = self.slides.get_page(curr_idx) {
                        self.current_tex.set(img, Default::default());
                    }
                }
            }
        }
    }

    fn draw_presenter(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let elapsed = self.start_time.elapsed();
        let mins = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;
        let num_pages = self.slides.num_pages();
        let curr_idx = self.requested_page_idx;
        let next_idx = curr_idx + 1;
        let has_next = next_idx < num_pages;

        ui.horizontal(|ui| {
            ui.heading(format!("{:02}:{:02}", mins, secs));
            if ui.button("⟲ reset").clicked() {
                self.start_time = Instant::now();
            }
            ui.label(format!("  slide {} / {}", curr_idx + 1, num_pages));
        });
        ui.separator();

        // Bottom video controls panel reserves its own space first;
        // remaining area is taken by the thumbnails below.
        egui::TopBottomPanel::bottom("video_controls")
            .resizable(false)
            .show_inside(ui, |ui| {
                self.draw_video_controls(ctx, ui, curr_idx);
            });

        let avail = ui.available_rect_before_wrap();
        let pad = 8.0;
        let inner_w = avail.width() - pad;
        let curr_w = inner_w * 0.65;
        let next_w = inner_w * 0.35 - pad;

        let curr_size = self.current_tex.size_vec2();
        let next_size = self.next_tex.size_vec2();

        ui.horizontal_top(|ui| {
            ui.allocate_ui(egui::vec2(curr_w, avail.height()), |ui| {
                ui.label("Current");
                let aspect = curr_size.y / curr_size.x;
                let w = ui.available_width();
                let h = (w * aspect).min(ui.available_height() - 20.0);
                let img = egui::Image::new(egui::load::SizedTexture::new(
                    self.current_tex.id(),
                    egui::vec2(w, h),
                ));
                ui.add(img);
            });
            ui.separator();
            ui.allocate_ui(egui::vec2(next_w, avail.height()), |ui| {
                if has_next {
                    ui.label("Next");
                    let aspect = next_size.y / next_size.x;
                    let w = ui.available_width();
                    let h = (w * aspect).min(ui.available_height() - 20.0);
                    let img = egui::Image::new(egui::load::SizedTexture::new(
                        self.next_tex.id(),
                        egui::vec2(w, h),
                    ));
                    ui.add(img);
                } else {
                    ui.label("Next: (end)");
                }
            });
        });
    }

    fn draw_video_controls(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, page_idx: usize) {
        let any_running = self.slides.any_current_video_running(page_idx);
        ui.horizontal(|ui| {
            ui.heading("Videos");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let pause_label = if any_running {
                    "⏸ pause all"
                } else {
                    "▶ resume all"
                };
                if ui.button(pause_label).clicked() {
                    self.slides.toggle_pause_current(page_idx);
                    ctx.request_repaint();
                }
                if ui.button("↺ restart all").clicked() {
                    self.slides.restart_current(page_idx);
                    ctx.request_repaint();
                }
            });
        });
        ui.separator();

        // Unified slider: average fraction across active videos; drag → seek all.
        let mut avg_elapsed_ms: i64 = 0;
        let mut avg_duration_ms: i64 = 0;
        let mut count: i64 = 0;
        self.slides.for_each_current_video_mut(page_idx, |player| {
            let d = player.duration_ms().unwrap_or(0);
            if d > 0 {
                avg_elapsed_ms += player.elapsed_ms().unwrap_or(0).clamp(0, d);
                avg_duration_ms += d;
                count += 1;
            }
        });
        const UNIFIED_KEY: &str = "*all*";
        let scrub = &mut self.seek_scrub_paused;

        if count > 0 {
            let avg_e = avg_elapsed_ms / count;
            let avg_d = avg_duration_ms / count;
            let mut unified_frac = if avg_d > 0 {
                avg_e as f32 / avg_d as f32
            } else {
                0.0
            };
            ui.horizontal(|ui| {
                ui.add_sized(egui::vec2(160.0, 20.0), egui::Label::new("all videos"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{} / {} (avg)",
                        format_ms(avg_e as u64),
                        format_ms(avg_d as u64),
                    ));
                    let slider_width = (ui.available_width() - 8.0).max(40.0);
                    ui.spacing_mut().slider_width = slider_width;
                    let resp = ui.add(
                        egui::Slider::new(&mut unified_frac, 0.0..=1.0)
                            .show_value(false)
                            .clamping(egui::SliderClamping::Always),
                    );
                    let held = resp.is_pointer_button_down_on();
                    if held && !scrub.contains(UNIFIED_KEY) {
                        // Pause every currently-playing video for the duration of the drag.
                        self.slides.for_each_current_video_mut(page_idx, |player| {
                            if !player.is_paused() {
                                let p = player.path().unwrap_or("").to_string();
                                player.pause();
                                scrub.insert(p);
                            }
                        });
                        scrub.insert(UNIFIED_KEY.to_string());
                    }
                    if resp.changed() {
                        self.slides.for_each_current_video_mut(page_idx, |player| {
                            player.seek_fraction(unified_frac);
                        });
                        ctx.request_repaint();
                        ctx.request_repaint_after(std::time::Duration::from_millis(150));
                    }
                    if !held && scrub.remove(UNIFIED_KEY) {
                        self.slides.for_each_current_video_mut(page_idx, |player| {
                            let p = player.path().unwrap_or("").to_string();
                            if scrub.remove(&p) {
                                player.resume();
                            }
                        });
                    }
                });
            });
            ui.separator();
        }

        let mut seek_requested = false;
        self.slides.for_each_current_video_mut(page_idx, |player| {
            let path = player.path().unwrap_or("(video)").to_string();
            let basename = std::path::Path::new(&path)
                .file_name()
                .map(|os| os.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let duration = player.duration_ms().unwrap_or(0).max(0);
            let elapsed = player.elapsed_ms().unwrap_or(0).clamp(0, duration);
            let mut frac = if duration > 0 {
                elapsed as f32 / duration as f32
            } else {
                0.0
            };
            let paused = player.is_paused();

            ui.horizontal(|ui| {
                ui.add_sized(
                    egui::vec2(160.0, 20.0),
                    egui::Label::new(basename).truncate(),
                );
                let btn_label = if paused { "▶" } else { "⏸" };
                if ui.button(btn_label).clicked() {
                    player.toggle_pause();
                }
                let time_text = format!(
                    "{} / {}",
                    format_ms(elapsed as u64),
                    format_ms(duration as u64),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(time_text);
                    let slider_width = (ui.available_width() - 8.0).max(40.0);
                    ui.spacing_mut().slider_width = slider_width;
                    let resp = ui.add(
                        egui::Slider::new(&mut frac, 0.0..=1.0)
                            .show_value(false)
                            .clamping(egui::SliderClamping::Always),
                    );
                    let held = resp.is_pointer_button_down_on();
                    if held && !paused && !scrub.contains(&path) {
                        player.pause();
                        scrub.insert(path.clone());
                    }
                    if resp.changed() && duration > 0 {
                        player.seek_fraction(frac);
                        seek_requested = true;
                    }
                    if !held && scrub.remove(&path) {
                        player.resume();
                    }
                });
            });
        });

        if seek_requested {
            ctx.request_repaint();
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
    }

    fn draw_audience(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let slide_size = self.current_tex.size_vec2();
        let available_rect = ui.available_rect_before_wrap();

        let avail_w = available_rect.width();
        let avail_h = available_rect.height();
        let scale = (avail_w / slide_size.x).min(avail_h / slide_size.y);
        let render_size = slide_size * scale;
        let slide_pos = available_rect.center() - 0.5 * render_size;
        let img_rect = egui::Rect::from_min_size(slide_pos, render_size);

        ui.painter().rect_filled(
            available_rect,
            egui::CornerRadius::ZERO,
            egui::Color32::BLACK,
        );
        ui.put(
            img_rect,
            egui::Image::new(egui::load::SizedTexture::new(
                self.current_tex.id(),
                render_size,
            ))
            .fit_to_exact_size(render_size),
        );
        self.slides
            .handle_video(self.requested_page_idx, slide_pos, render_size, ctx, ui);
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(config_changed_rx) = &self.config_changed_rx {
            if let Ok(new_video_config) = config_changed_rx.try_recv() {
                println!("Config changed from UI");
                self.slides.change_video_entries(new_video_config);
            }
            ctx.request_repaint();
        }

        let actions = self.handle_input(ctx);
        self.apply_deferred_actions(ctx, actions);

        self.upload_textures_if_needed(ctx);

        // audience viewport only when presenter mode is active
        if self.presenter_mode {
            let audience_id = egui::ViewportId::from_hash_of(AUDIENCE_VIEWPORT_ID);
            let mut builder = egui::ViewportBuilder::default()
                .with_title("bewegtbild — audience")
                .with_inner_size([960.0, 540.0]);
            if let Some(idx) = self.audience_monitor_idx {
                if let Some(m) = self.monitors.get(idx) {
                    builder = builder.with_position(egui::pos2(m.x as f32, m.y as f32));
                }
            }
            if !self.audience_window_alive {
                self.audience_window_alive = true;
                if let Some(idx) = self.audience_monitor_idx {
                    self.start_placement(idx, self.audience_fullscreen);
                }
            }
            self.tick_placement(ctx);

            ctx.show_viewport_immediate(audience_id, builder, |ctx, _class| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
                    .show(ctx, |ui| {
                        self.draw_audience(ctx, ui);
                    });
            });
        }

        // root window
        if self.presenter_mode {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.draw_presenter(ctx, ui);
            });
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        } else {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
                .show(ctx, |ui| {
                    self.draw_audience(ctx, ui);
                });
        }
    }
}
