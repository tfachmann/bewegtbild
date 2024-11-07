use egui_video::Player;

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

    pub fn is_path_playing(&self, video_path: &str) -> bool {
        self.video
            .as_ref()
            .is_some_and(|video| video.path_playing == video_path)
    }

    pub fn destroy(&mut self) {
        self.video = None
    }

    pub fn size(&self) -> Option<egui::Vec2> {
        if let Some(video) = self.video.as_ref() {
            Some(video.player.size)
        } else {
            None
        }
    }
}
