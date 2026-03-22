use crate::{
    config::{
        BASE_SPEED, GROUND_Y, MAX_SPEED_BONUS, SCORE_RATE, SCREEN_HEIGHT, SCREEN_WIDTH, STAR_COUNT,
    },
    github_gate::SupportMode,
    obstacle::{choose_obstacle_kind, minimum_spawn_gap, next_spawn_delay, Obstacle, ObstacleKind},
    persistence::HighScoreStore,
    player::Player,
};
use ::rand::{rngs::ThreadRng, thread_rng, Rng};
use macroquad::prelude::*;

enum Phase {
    Running,
    GameOver,
}

struct SkyStar {
    x: f32,
    y: f32,
    radius: f32,
    speed_factor: f32,
}

impl SkyStar {
    fn random(rng: &mut ThreadRng) -> Self {
        Self {
            x: rng.gen_range(0.0..SCREEN_WIDTH),
            y: rng.gen_range(18.0..SCREEN_HEIGHT * 0.55),
            radius: rng.gen_range(1.0..2.8),
            speed_factor: rng.gen_range(0.12..0.33),
        }
    }
}

struct GameAssets {
    player_stand: Option<Texture2D>,
    player_duck: Option<Texture2D>,
    player_jump_1: Option<Texture2D>,
    player_jump_2: Option<Texture2D>,
    player_jump_3: Option<Texture2D>,
    cactus: Option<Texture2D>,
    star_obstacle: Option<Texture2D>,
}

pub async fn run() {
    let mut game = Game::new().await;
    game.run().await;
}

struct Game {
    player: Player,
    assets: GameAssets,
    support_mode: SupportMode,
    obstacles: Vec<Obstacle>,
    sky_stars: Vec<SkyStar>,
    rng: ThreadRng,
    obstacle_timer: f32,
    star_burst_timer: f32,
    speed: f32,
    score: f32,
    best_score: u32,
    phase: Phase,
    ground_offset: f32,
    elapsed: f32,
    store: HighScoreStore,
    auto_close_after: Option<f32>,
}

impl Game {
    async fn new() -> Self {
        let mut rng = thread_rng();
        let store = HighScoreStore::from_env_or_default();
        let best_score = store.load();
        let support_mode = SupportMode::detect();
        let assets = GameAssets {
            player_stand: load_pixel_texture("assets/player_stand.png").await,
            player_duck: load_pixel_texture("assets/player_duck.png").await,
            player_jump_1: load_pixel_texture("assets/player_jump_1.png").await,
            player_jump_2: load_pixel_texture("assets/player_jump_2.png").await,
            player_jump_3: load_pixel_texture("assets/player_jump_3.png").await,
            cactus: load_pixel_texture("assets/obstacle_cactus.png").await,
            star_obstacle: load_pixel_texture("assets/obstacle_star.png").await,
        };

        Self {
            player: Player::new(),
            assets,
            support_mode,
            obstacles: Vec::new(),
            sky_stars: (0..STAR_COUNT).map(|_| SkyStar::random(&mut rng)).collect(),
            rng,
            obstacle_timer: 0.9,
            star_burst_timer: 2.8,
            speed: BASE_SPEED,
            score: 0.0,
            best_score,
            phase: Phase::Running,
            ground_offset: 0.0,
            elapsed: 0.0,
            store,
            auto_close_after: std::env::var("DINO_AUTOCLOSE_SECONDS")
                .ok()
                .and_then(|v| v.parse::<f32>().ok()),
        }
    }

    async fn run(&mut self) {
        loop {
            let dt = get_frame_time().min(1.0 / 30.0);
            if is_key_pressed(KeyCode::Escape) {
                self.persist_best_score();
                break;
            }

            self.elapsed += dt;
            if let Some(limit) = self.auto_close_after {
                if self.elapsed >= limit {
                    self.persist_best_score();
                    break;
                }
            }

            self.update(dt);
            self.draw();
            next_frame().await;
        }
    }

    fn update(&mut self, dt: f32) {
        let jump_pressed = is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::W)
            || is_mouse_button_pressed(MouseButton::Left);
        let duck_held = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);

        match self.phase {
            Phase::Running => {
                self.speed = BASE_SPEED + (self.score * 1.8).min(MAX_SPEED_BONUS);
                self.score += dt * SCORE_RATE;
                self.ground_offset = (self.ground_offset + self.speed * dt) % 46.0;
                if self.support_mode.is_penalty_mode() {
                    self.star_burst_timer -= dt;
                }

                self.update_sky_stars(dt);
                self.player.update(dt, jump_pressed, duck_held, false);
                self.update_obstacles(dt);
                self.spawn_obstacles(dt);
                self.check_collisions();
            }
            Phase::GameOver => {
                self.player.update(dt, false, false, true);
                if jump_pressed || is_key_pressed(KeyCode::Enter) {
                    self.restart();
                }
            }
        }
    }

    fn update_sky_stars(&mut self, dt: f32) {
        for star in &mut self.sky_stars {
            star.x -= self.speed * star.speed_factor * dt;
            if star.x < -8.0 {
                *star = SkyStar {
                    x: SCREEN_WIDTH + self.rng.gen_range(10.0..180.0),
                    ..SkyStar::random(&mut self.rng)
                };
            }
        }
    }

    fn update_obstacles(&mut self, dt: f32) {
        for obstacle in &mut self.obstacles {
            obstacle.update(dt, self.speed);
        }
        self.obstacles.retain(|obstacle| !obstacle.is_offscreen());
    }

    fn spawn_obstacles(&mut self, dt: f32) {
        self.obstacle_timer -= dt;
        if self.obstacle_timer > 0.0 {
            return;
        }

        if let Some(last_obstacle) = self.obstacles.last() {
            let last_right_edge = last_obstacle.x + last_obstacle.size().0;
            let required_gap = minimum_spawn_gap(self.speed);
            if last_right_edge > SCREEN_WIDTH - required_gap {
                self.obstacle_timer = 0.08;
                return;
            }
        }

        if self.support_mode.is_penalty_mode() && self.star_burst_timer <= 0.0 {
            self.spawn_star_burst();
            self.star_burst_timer = 3.2;
            self.obstacle_timer = 1.6;
            return;
        }

        let kind = choose_obstacle_kind(&mut self.rng, self.score as u32);
        self.obstacles
            .push(Obstacle::new(kind, SCREEN_WIDTH + 40.0));
        self.obstacle_timer = next_spawn_delay(&mut self.rng, self.speed);
    }

    fn spawn_star_burst(&mut self) {
        let columns = [SCREEN_WIDTH + 38.0, SCREEN_WIDTH + 92.0];
        let rows = [
            GROUND_Y - 182.0,
            GROUND_Y - 148.0,
            GROUND_Y - 114.0,
            GROUND_Y - 80.0,
            GROUND_Y - 46.0,
        ];

        for x in columns {
            for y in rows {
                self.obstacles
                    .push(Obstacle::new_at_y(ObstacleKind::BirdLow, x, y));
            }
        }
    }

    fn check_collisions(&mut self) {
        let player_box = self.player.collision_box();
        if self
            .obstacles
            .iter()
            .any(|obstacle| player_box.overlaps(&obstacle.collision_box()))
        {
            self.phase = Phase::GameOver;
            self.persist_best_score();
        }
    }

    fn restart(&mut self) {
        self.best_score = self.best_score.max(self.score as u32);
        self.score = 0.0;
        self.speed = BASE_SPEED;
        self.phase = Phase::Running;
        self.ground_offset = 0.0;
        self.obstacles.clear();
        self.obstacle_timer = 0.85;
        self.star_burst_timer = 2.8;
        self.player.reset();
    }

    fn persist_best_score(&mut self) {
        let run_score = self.score as u32;
        if let Ok(best) = self.store.store_if_higher(self.best_score, run_score) {
            self.best_score = best;
        }
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(6, 8, 14, 255));
        self.draw_sky();
        self.draw_ground();
        for obstacle in &self.obstacles {
            obstacle.draw(
                self.assets.cactus.as_ref(),
                self.assets.star_obstacle.as_ref(),
            );
        }
        self.player.draw(
            self.assets.player_stand.as_ref(),
            self.assets.player_duck.as_ref(),
            [
                self.assets.player_jump_1.as_ref(),
                self.assets.player_jump_2.as_ref(),
                self.assets.player_jump_3.as_ref(),
            ],
        );
        self.draw_ui();
        self.draw_support_overlay();
    }

    fn draw_sky(&self) {
        draw_circle(
            SCREEN_WIDTH - 140.0,
            72.0,
            26.0,
            Color::from_rgba(214, 220, 255, 255),
        );
        draw_circle(
            SCREEN_WIDTH - 128.0,
            68.0,
            24.0,
            Color::from_rgba(6, 8, 14, 255),
        );

        for star in &self.sky_stars {
            draw_circle(
                star.x,
                star.y,
                star.radius,
                Color::from_rgba(198, 208, 242, 185),
            );
        }

        if let Some((r, g, b, a)) = self.support_mode.scene_tint() {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_WIDTH,
                SCREEN_HEIGHT * 0.42,
                Color::from_rgba(r, g, b, a),
            );
        }

        draw_rectangle(
            0.0,
            GROUND_Y - 12.0,
            SCREEN_WIDTH,
            3.0,
            Color::from_rgba(36, 44, 66, 255),
        );
        draw_rectangle(
            0.0,
            GROUND_Y + 8.0,
            SCREEN_WIDTH,
            2.0,
            Color::from_rgba(17, 22, 35, 255),
        );
    }

    fn draw_ground(&self) {
        draw_rectangle(
            0.0,
            GROUND_Y,
            SCREEN_WIDTH,
            SCREEN_HEIGHT - GROUND_Y,
            Color::from_rgba(9, 11, 18, 255),
        );
        draw_line(
            0.0,
            GROUND_Y,
            SCREEN_WIDTH,
            GROUND_Y,
            2.0,
            Color::from_rgba(118, 128, 156, 255),
        );

        let dash_width = 22.0;
        let gap = 24.0;
        let mut x = -self.ground_offset;
        while x < SCREEN_WIDTH + dash_width {
            draw_rectangle(
                x,
                GROUND_Y + 16.0,
                dash_width,
                4.0,
                Color::from_rgba(64, 74, 98, 255),
            );
            x += dash_width + gap;
        }

        for ridge in 0..10 {
            let rx = ((ridge as f32 * 155.0) - self.ground_offset * 0.5)
                .rem_euclid(SCREEN_WIDTH + 120.0)
                - 60.0;
            draw_line(
                rx,
                GROUND_Y + 34.0,
                rx + 26.0,
                GROUND_Y + 28.0,
                2.0,
                Color::from_rgba(24, 29, 44, 255),
            );
        }
    }

    fn draw_ui(&self) {
        let current_score = self.score as u32;
        let displayed_best = self.best_score.max(current_score);

        let score_text = format!("HI {:05}  SCORE {:05}", displayed_best, current_score);
        draw_text(
            &score_text,
            SCREEN_WIDTH - 290.0,
            44.0,
            30.0,
            Color::from_rgba(220, 228, 246, 255),
        );

        draw_text(
            "SPACE / UP / W = JUMP   DOWN / S = DUCK   ESC = QUIT",
            28.0,
            36.0,
            22.0,
            Color::from_rgba(120, 132, 166, 255),
        );

        if matches!(self.phase, Phase::GameOver) {
            let panel_color = Color::from_rgba(3, 5, 10, 232);
            draw_rectangle(SCREEN_WIDTH * 0.5 - 300.0, 92.0, 600.0, 132.0, panel_color);

            let (title, subtitle) = if self.support_mode.is_penalty_mode() {
                (
                    "YOU DID NOT STAR THE REPOSITORY",
                    "You cannot keep this mode fair until you star the repo.",
                )
            } else {
                ("GAME OVER", "Press SPACE or ENTER to restart")
            };

            draw_text(
                title,
                SCREEN_WIDTH * 0.5 - 240.0,
                145.0,
                34.0,
                Color::from_rgba(247, 221, 221, 255),
            );
            draw_text(
                subtitle,
                SCREEN_WIDTH * 0.5 - 248.0,
                186.0,
                24.0,
                Color::from_rgba(214, 222, 241, 255),
            );
        }

        let collision = self.player.collision_box();
        if std::env::var("DINO_DEBUG_HITBOX").ok().as_deref() == Some("1") {
            draw_rectangle_lines(collision.x, collision.y, collision.w, collision.h, 1.0, RED);
            for obstacle in &self.obstacles {
                let rect = obstacle.collision_box();
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, ORANGE);
            }
        }
    }

    fn draw_support_overlay(&self) {
        let heading = self.support_mode.status_heading();
        let detail = self.support_mode.status_detail();

        draw_text(&heading, 28.0, 82.0, 22.0, WHITE);
        draw_text(
            &detail,
            28.0,
            108.0,
            18.0,
            Color::from_rgba(226, 231, 244, 255),
        );

        if matches!(self.support_mode, SupportMode::NotStarred { .. }) {
            draw_text(
                self.support_mode.repo_url(),
                28.0,
                132.0,
                16.0,
                Color::from_rgba(208, 214, 232, 255),
            );
        }
    }
}

async fn load_pixel_texture(path: &str) -> Option<Texture2D> {
    match load_texture(path).await {
        Ok(texture) => {
            texture.set_filter(FilterMode::Nearest);
            Some(texture)
        }
        Err(_) => None,
    }
}
