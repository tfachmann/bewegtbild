use egui_video::{Player, PlayerState};

pub struct PlayingVideo {
    path_playing: String,
    player: Player,
}

impl PlayingVideo {
    pub fn new(player: Player, path_playing: String) -> Self {
        Self {
            path_playing,
            player,
        }
    }
}

pub struct VideoPlayer {
    video: Option<PlayingVideo>,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self { video: None }
    }

    pub fn init(&mut self, ctx: &egui::Context, video_path: &str) {
        let video_path = video_path.to_owned();
        let player = Player::new(ctx, &video_path).unwrap();
        self.video = Some(PlayingVideo::new(player, video_path.to_owned()));
    }

    pub fn start(&mut self) {
        if let Some(video) = self.video.as_mut() {
            video.player.start();
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, rect: egui::Rect) -> Option<egui::Response> {
        if let Some(video) = self.video.as_mut() {
            Some(video.player.ui_at(ui, rect))
        } else {
            None
        }
    }

    pub fn is_playing(&self) -> bool {
        self.video.is_some()
    }

    pub fn destroy(&mut self) {
        self.video = None
    }

    pub fn pause(&mut self) {
        if let Some(v) = self.video.as_mut() {
            v.player.pause();
        }
    }

    pub fn resume(&mut self) {
        if let Some(v) = self.video.as_mut() {
            v.player.resume();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.video
            .as_ref()
            .is_some_and(|v| matches!(v.player.player_state.get(), PlayerState::Paused))
    }

    pub fn step_frame(&mut self, forward: bool) {
        if let Some(v) = self.video.as_mut() {
            if !matches!(v.player.player_state.get(), PlayerState::Paused) {
                return;
            }
            if v.player.duration_ms <= 0 || v.player.framerate <= 0.0 {
                return;
            }
            let frame_ms = (1000.0 / v.player.framerate) as i64;
            let delta = if forward { frame_ms } else { -frame_ms };
            let target = (v.player.elapsed_ms() + delta).clamp(0, v.player.duration_ms);
            let frac = target as f32 / v.player.duration_ms as f32;
            v.player.seek(frac);
        }
    }

    pub fn restart(&mut self) {
        if let Some(v) = self.video.as_mut() {
            v.player.start();
        }
    }

    pub fn size(&self) -> Option<egui::Vec2> {
        self.video.as_ref().map(|video| video.player.size)
    }

    pub fn elapsed_ms(&self) -> Option<i64> {
        self.video.as_ref().map(|v| v.player.elapsed_ms())
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.video.as_ref().map(|v| v.player.duration_ms)
    }

    pub fn seek_fraction(&mut self, frac: f32) {
        if let Some(v) = self.video.as_mut() {
            v.player.seek(frac.clamp(0.0, 1.0));
        }
    }

    pub fn path(&self) -> Option<&str> {
        self.video.as_ref().map(|v| v.path_playing.as_str())
    }

    pub fn toggle_pause(&mut self) {
        if self.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }
}
