use crate::config::GROUND_Y;
use ::rand::Rng;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    SmallCactus,
    LargeCactus,
    CactusCluster,
    BirdLow,
    BirdHigh,
}

#[derive(Debug, Clone, Copy)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub x: f32,
    y_override: Option<f32>,
    flap_phase: f32,
}

impl Obstacle {
    pub fn new(kind: ObstacleKind, x: f32) -> Self {
        Self {
            kind,
            x,
            y_override: None,
            flap_phase: 0.0,
        }
    }

    pub fn new_at_y(kind: ObstacleKind, x: f32, y: f32) -> Self {
        Self {
            kind,
            x,
            y_override: Some(y),
            flap_phase: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, speed: f32) {
        self.x -= speed * dt;
        self.flap_phase += dt * 12.0;
    }

    pub fn is_offscreen(&self) -> bool {
        self.x + self.size().0 < -40.0
    }

    pub fn bounds(&self) -> Rect {
        let (width, height) = self.size();
        Rect::new(self.x, self.y(), width, height)
    }

    pub fn collision_box(&self) -> Rect {
        let bounds = self.bounds();
        match self.kind {
            ObstacleKind::SmallCactus => Rect::new(
                bounds.x + 3.0,
                bounds.y + 2.0,
                bounds.w - 6.0,
                bounds.h - 4.0,
            ),
            ObstacleKind::LargeCactus => Rect::new(
                bounds.x + 3.0,
                bounds.y + 2.0,
                bounds.w - 6.0,
                bounds.h - 5.0,
            ),
            ObstacleKind::CactusCluster => Rect::new(
                bounds.x + 5.0,
                bounds.y + 4.0,
                bounds.w - 10.0,
                bounds.h - 6.0,
            ),
            ObstacleKind::BirdLow | ObstacleKind::BirdHigh => Rect::new(
                bounds.x + 6.0,
                bounds.y + 7.0,
                bounds.w - 12.0,
                bounds.h - 12.0,
            ),
        }
    }

    pub fn draw(&self, cactus_texture: Option<&Texture2D>, star_texture: Option<&Texture2D>) {
        match self.kind {
            ObstacleKind::SmallCactus | ObstacleKind::LargeCactus | ObstacleKind::CactusCluster => {
                if let Some(texture) = cactus_texture {
                    self.draw_textured_cactus(texture);
                } else {
                    self.draw_fallback_cactus();
                }
            }
            ObstacleKind::BirdLow | ObstacleKind::BirdHigh => {
                if let Some(texture) = star_texture {
                    self.draw_textured_star(texture);
                } else {
                    self.draw_fallback_star();
                }
            }
        }
    }

    pub fn size(&self) -> (f32, f32) {
        match self.kind {
            ObstacleKind::SmallCactus => (28.0, 54.0),
            ObstacleKind::LargeCactus => (34.0, 72.0),
            ObstacleKind::CactusCluster => (64.0, 52.0),
            ObstacleKind::BirdLow | ObstacleKind::BirdHigh => (48.0, 30.0),
        }
    }

    pub fn y(&self) -> f32 {
        if let Some(y) = self.y_override {
            return y;
        }

        match self.kind {
            ObstacleKind::BirdLow => GROUND_Y - 64.0,
            ObstacleKind::BirdHigh => GROUND_Y - 108.0,
            _ => GROUND_Y - self.size().1,
        }
    }

    fn draw_textured_cactus(&self, texture: &Texture2D) {
        let bounds = self.bounds();
        let aspect = texture.width() / texture.height().max(1.0);
        let (scale_w, scale_h) = match self.kind {
            ObstacleKind::SmallCactus => (1.05, 1.08),
            ObstacleKind::LargeCactus => (1.10, 1.16),
            ObstacleKind::CactusCluster => (1.40, 1.05),
            _ => (1.0, 1.0),
        };
        let draw_h = bounds.h * scale_h;
        let draw_w = draw_h * aspect * scale_w;
        let x = bounds.x + (bounds.w - draw_w) * 0.5;
        let y = bounds.y + bounds.h - draw_h;

        draw_texture_ex(
            texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );
    }

    fn draw_textured_star(&self, texture: &Texture2D) {
        let bounds = self.bounds();
        let aspect = texture.width() / texture.height().max(1.0);
        let pulse = 0.96 + self.flap_phase.sin().abs() * 0.16;
        let draw_h = bounds.h * 1.45 * pulse;
        let draw_w = draw_h * aspect;
        let x = bounds.x + (bounds.w - draw_w) * 0.5;
        let y = bounds.y + (bounds.h - draw_h) * 0.55;

        draw_texture_ex(
            texture,
            x,
            y,
            Color::from_rgba(255, 255, 255, 245),
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                rotation: self.flap_phase * 0.03,
                pivot: Some(vec2(x + draw_w * 0.5, y + draw_h * 0.5)),
                ..Default::default()
            },
        );
    }

    fn draw_fallback_cactus(&self) {
        let bounds = self.bounds();
        let body = Color::from_rgba(95, 224, 165, 255);
        let mid = Color::from_rgba(48, 156, 112, 255);
        let dark = Color::from_rgba(22, 82, 65, 255);
        let glow = Color::from_rgba(166, 255, 214, 110);
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, body);
        draw_rectangle(
            bounds.x + 2.0,
            bounds.y + 3.0,
            bounds.w - 4.0,
            bounds.h * 0.18,
            glow,
        );
        draw_rectangle(
            bounds.x + bounds.w * 0.14,
            bounds.y,
            bounds.w * 0.18,
            bounds.h,
            mid,
        );
        draw_rectangle(
            bounds.x + bounds.w * 0.56,
            bounds.y + 4.0,
            bounds.w * 0.14,
            bounds.h - 8.0,
            mid,
        );
        draw_rectangle(
            bounds.x + bounds.w * 0.26,
            bounds.y + bounds.h * 0.18,
            bounds.w * 0.16,
            bounds.h * 0.24,
            body,
        );
        draw_rectangle(
            bounds.x + bounds.w * 0.62,
            bounds.y + bounds.h * 0.2,
            bounds.w * 0.14,
            bounds.h * 0.22,
            body,
        );
        draw_line(
            bounds.x + 2.0,
            bounds.y + 5.0,
            bounds.x + 2.0,
            bounds.y + bounds.h - 6.0,
            2.0,
            dark,
        );
        draw_line(
            bounds.x + bounds.w - 2.0,
            bounds.y + 5.0,
            bounds.x + bounds.w - 2.0,
            bounds.y + bounds.h - 6.0,
            2.0,
            dark,
        );
        draw_line(
            bounds.x + bounds.w * 0.5,
            bounds.y + 4.0,
            bounds.x + bounds.w * 0.5,
            bounds.y + bounds.h - 5.0,
            1.2,
            dark,
        );
        for offset in [0.24_f32, 0.42, 0.68] {
            let px = bounds.x + bounds.w * offset;
            draw_circle(px, bounds.y + bounds.h * 0.22, 1.4, dark);
            draw_circle(px - 2.0, bounds.y + bounds.h * 0.52, 1.2, dark);
        }
    }

    fn draw_fallback_star(&self) {
        let bounds = self.bounds();
        let glow = Color::from_rgba(255, 222, 103, 75);
        let core = Color::from_rgba(255, 232, 125, 255);
        let pulse = self.flap_phase.sin().abs();
        draw_circle(
            bounds.x + bounds.w * 0.5,
            bounds.y + bounds.h * 0.45,
            18.0 + pulse * 4.0,
            glow,
        );
        draw_poly(
            bounds.x + bounds.w * 0.5,
            bounds.y + bounds.h * 0.45,
            5,
            14.0 + pulse * 2.0,
            self.flap_phase * 1.4,
            core,
        );
    }
}

pub fn choose_obstacle_kind<R: Rng + ?Sized>(rng: &mut R, score: u32) -> ObstacleKind {
    let bird_allowed = score >= 180;
    let roll = rng.gen_range(0..100);

    if bird_allowed && roll >= 74 {
        if roll % 2 == 0 {
            ObstacleKind::BirdLow
        } else {
            ObstacleKind::BirdHigh
        }
    } else if roll >= 48 {
        ObstacleKind::CactusCluster
    } else if roll >= 20 {
        ObstacleKind::LargeCactus
    } else {
        ObstacleKind::SmallCactus
    }
}

pub fn next_spawn_delay<R: Rng + ?Sized>(rng: &mut R, speed: f32) -> f32 {
    let normalized_speed = ((speed - 360.0) / 260.0).clamp(0.0, 1.0);
    let min_delay = 0.95 - normalized_speed * 0.18;
    let max_delay = 1.75 - normalized_speed * 0.28;
    rng.gen_range(min_delay..max_delay.max(min_delay + 0.08))
}

pub fn minimum_spawn_gap(speed: f32) -> f32 {
    let normalized_speed = ((speed - 360.0) / 260.0).clamp(0.0, 1.0);
    320.0 - normalized_speed * 70.0
}

#[cfg(test)]
mod tests {
    use super::{choose_obstacle_kind, minimum_spawn_gap, next_spawn_delay, ObstacleKind};
    use ::rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn birds_do_not_spawn_before_threshold() {
        let mut rng = StdRng::seed_from_u64(4);
        for _ in 0..20 {
            let kind = choose_obstacle_kind(&mut rng, 100);
            assert!(!matches!(
                kind,
                ObstacleKind::BirdLow | ObstacleKind::BirdHigh
            ));
        }
    }

    #[test]
    fn spawn_delay_tightens_as_speed_increases() {
        let mut slow_rng = StdRng::seed_from_u64(7);
        let mut fast_rng = StdRng::seed_from_u64(7);

        let slow = next_spawn_delay(&mut slow_rng, 380.0);
        let fast = next_spawn_delay(&mut fast_rng, 620.0);

        assert!(fast < slow);
    }

    #[test]
    fn minimum_spawn_gap_shrinks_gradually_with_speed() {
        assert!(minimum_spawn_gap(380.0) > minimum_spawn_gap(620.0));
        assert!(minimum_spawn_gap(380.0) >= 250.0);
    }
}
