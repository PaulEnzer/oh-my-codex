use std::env;

use macroquad::prelude::Conf;

pub const SCREEN_WIDTH: f32 = 1280.0;
pub const SCREEN_HEIGHT: f32 = 360.0;
pub const GROUND_Y: f32 = 300.0;
pub const PLAYER_X: f32 = 120.0;
pub const BASE_SPEED: f32 = 380.0;
pub const MAX_SPEED_BONUS: f32 = 320.0;
pub const SCORE_RATE: f32 = 18.0;
pub const STAR_COUNT: usize = 28;

fn resolve_window_dimension(env_key: &str, fallback: f32, minimum: i32) -> i32 {
    env::var(env_key)
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .filter(|value| *value >= minimum)
        .unwrap_or(fallback as i32)
}

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Rust Dino".to_owned(),
        window_width: resolve_window_dimension("DINO_WINDOW_WIDTH", SCREEN_WIDTH, 320),
        window_height: resolve_window_dimension("DINO_WINDOW_HEIGHT", SCREEN_HEIGHT, 90),
        high_dpi: false,
        sample_count: 1,
        window_resizable: false,
        ..Default::default()
    }
}
