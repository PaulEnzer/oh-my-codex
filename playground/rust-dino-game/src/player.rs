use crate::config::{GROUND_Y, PLAYER_X};
use macroquad::prelude::*;

const RUN_WIDTH: f32 = 40.0;
const RUN_HEIGHT: f32 = 54.0;
const DUCK_WIDTH: f32 = 56.0;
const DUCK_HEIGHT: f32 = 32.0;
const JUMP_VELOCITY: f32 = -880.0;
const GRAVITY: f32 = 2550.0;
const MAX_FALL_SPEED: f32 = 1500.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Running,
    Jumping,
    Ducking,
    Crashed,
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    velocity_y: f32,
    pub y: f32,
    pub state: PlayerState,
    leg_cycle: f32,
    jump_anim_time: f32,
}

impl Player {
    pub fn new() -> Self {
        let mut player = Self {
            velocity_y: 0.0,
            y: 0.0,
            state: PlayerState::Running,
            leg_cycle: 0.0,
            jump_anim_time: 0.0,
        };
        player.reset();
        player
    }

    pub fn reset(&mut self) {
        self.velocity_y = 0.0;
        self.state = PlayerState::Running;
        self.leg_cycle = 0.0;
        self.jump_anim_time = 0.0;
        self.y = GROUND_Y - RUN_HEIGHT;
    }

    pub fn is_grounded(&self) -> bool {
        (self.bottom() - GROUND_Y).abs() < 0.5
    }

    pub fn update(&mut self, dt: f32, jump_pressed: bool, duck_held: bool, game_over: bool) {
        if game_over {
            self.state = PlayerState::Crashed;
            self.jump_anim_time = 0.0;
            return;
        }

        if jump_pressed && self.is_grounded() {
            self.velocity_y = JUMP_VELOCITY;
            self.state = PlayerState::Jumping;
            self.jump_anim_time = 0.0;
        }

        if duck_held && self.is_grounded() && self.velocity_y.abs() < f32::EPSILON {
            self.state = PlayerState::Ducking;
            self.y = GROUND_Y - self.current_size().1;
        } else if self.is_grounded() && self.velocity_y.abs() < f32::EPSILON {
            self.state = PlayerState::Running;
            self.y = GROUND_Y - self.current_size().1;
        }

        self.velocity_y = (self.velocity_y + GRAVITY * dt).min(MAX_FALL_SPEED);
        self.y += self.velocity_y * dt;

        if self.bottom() >= GROUND_Y {
            self.velocity_y = 0.0;
            self.state = if duck_held {
                PlayerState::Ducking
            } else {
                PlayerState::Running
            };
            self.y = GROUND_Y - self.current_size().1;
            self.jump_anim_time = 0.0;
        } else {
            self.state = PlayerState::Jumping;
            self.jump_anim_time += dt;
        }

        self.leg_cycle += dt
            * if self.state == PlayerState::Ducking {
                13.0
            } else {
                10.0
            };
    }

    pub fn current_size(&self) -> (f32, f32) {
        match self.state {
            PlayerState::Ducking => (DUCK_WIDTH, DUCK_HEIGHT),
            _ => (RUN_WIDTH, RUN_HEIGHT),
        }
    }

    pub fn bounds(&self) -> Rect {
        let (width, height) = self.current_size();
        Rect::new(PLAYER_X, self.y, width, height)
    }

    pub fn collision_box(&self) -> Rect {
        let bounds = self.bounds();
        match self.state {
            PlayerState::Ducking => Rect::new(
                bounds.x + 8.0,
                bounds.y + 6.0,
                bounds.w - 16.0,
                bounds.h - 10.0,
            ),
            PlayerState::Crashed => Rect::new(
                bounds.x + 6.0,
                bounds.y + 6.0,
                bounds.w - 12.0,
                bounds.h - 12.0,
            ),
            _ => Rect::new(
                bounds.x + 7.0,
                bounds.y + 4.0,
                bounds.w - 14.0,
                bounds.h - 8.0,
            ),
        }
    }

    pub fn draw(
        &self,
        stand_texture: Option<&Texture2D>,
        duck_texture: Option<&Texture2D>,
        jump_textures: [Option<&Texture2D>; 3],
    ) {
        let active_texture = match self.state {
            PlayerState::Ducking => duck_texture.or(stand_texture),
            PlayerState::Jumping => self.current_jump_texture(jump_textures).or(stand_texture),
            _ => stand_texture,
        };

        if let Some(texture) = active_texture {
            self.draw_texture_sprite(texture, matches!(self.state, PlayerState::Ducking));
        } else {
            self.draw_fallback_sprite();
        }

        if self.state == PlayerState::Crashed {
            self.draw_crash_overlay();
        }
    }

    fn current_jump_texture<'a>(
        &self,
        jump_textures: [Option<&'a Texture2D>; 3],
    ) -> Option<&'a Texture2D> {
        if self.jump_anim_time < 0.14 {
            jump_textures[0]
        } else {
            jump_textures[1].or(jump_textures[2])
        }
    }

    fn draw_texture_sprite(&self, texture: &Texture2D, is_duck_texture: bool) {
        let bounds = self.bounds();
        let aspect = texture.width() / texture.height().max(1.0);
        let run_wave = self.leg_cycle.sin();
        let run_wave_fast = (self.leg_cycle * 2.0).sin();
        let mut target_height = if self.state == PlayerState::Ducking {
            bounds.h * if is_duck_texture { 1.24 } else { 0.94 }
        } else {
            bounds.h * 1.40
        };
        let mut target_width_scale = 1.0;
        let rotation;
        let tint;
        let mut x_offset = if self.state == PlayerState::Ducking {
            -0.16
        } else {
            -0.10
        };
        let y_offset;

        match self.state {
            PlayerState::Jumping => {
                let airtime_bob = (self.leg_cycle * 0.7).sin();
                let descent_progress = (self.velocity_y / MAX_FALL_SPEED).clamp(0.0, 1.0);
                target_height *= 1.04 - descent_progress * 0.02;
                target_width_scale = 1.00;
                rotation = -0.05 + airtime_bob * 0.02 + descent_progress * 0.04;
                x_offset = -0.13 + descent_progress * 0.01;
                y_offset = -2.5 + airtime_bob * 1.2 + descent_progress * 4.8;
                tint = Color::from_rgba(255, 255, 255, 252);
            }
            PlayerState::Crashed => {
                rotation = 0.18;
                y_offset = 2.0;
                tint = Color::from_rgba(255, 230, 230, 255);
            }
            PlayerState::Running => {
                tint = WHITE;
                target_height *= 1.0 + run_wave_fast.abs() * 0.02;
                target_width_scale = 1.0 + run_wave.abs() * 0.04;
                rotation = run_wave * 0.035;
                x_offset = -0.10 + run_wave * 0.01;
                y_offset = run_wave_fast.abs() * 1.7;
                self.draw_run_trail(bounds, run_wave, run_wave_fast);
            }
            PlayerState::Ducking => {
                tint = WHITE;
                let crouch_wave = (self.leg_cycle * 1.4).sin();
                if is_duck_texture {
                    target_width_scale = 1.55 + crouch_wave.abs() * 0.05;
                    rotation = -0.12 + crouch_wave * 0.02;
                    x_offset = -0.36;
                    y_offset = 5.4 + crouch_wave.abs() * 0.6;
                } else {
                    target_height *= 0.94;
                    target_width_scale = 1.34 + crouch_wave.abs() * 0.04;
                    rotation = -0.46 + crouch_wave * 0.02;
                    x_offset = -0.34;
                    y_offset = 6.5 + crouch_wave.abs() * 0.8;
                }
                self.draw_duck_trail(bounds, crouch_wave);
            }
        }

        let target_width = target_height * aspect * target_width_scale;
        let x = bounds.x + target_width * x_offset;
        let y = bounds.y + bounds.h - target_height + y_offset;

        draw_texture_ex(
            texture,
            x,
            y,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(target_width, target_height)),
                rotation,
                pivot: Some(vec2(x + target_width * 0.5, y + target_height * 0.62)),
                ..Default::default()
            },
        );
    }

    fn draw_run_trail(&self, bounds: Rect, run_wave: f32, run_wave_fast: f32) {
        let dust = Color::from_rgba(170, 179, 205, 90);
        let dust2 = Color::from_rgba(122, 132, 160, 70);
        let ground_y = bounds.y + bounds.h - 2.0;
        let back_x = bounds.x - 10.0 - run_wave * 2.5;
        draw_ellipse(
            back_x,
            ground_y,
            9.0 + run_wave_fast.abs() * 2.0,
            2.8,
            0.0,
            dust,
        );
        draw_ellipse(back_x - 12.0, ground_y + 1.0, 6.5, 2.2, 0.0, dust2);
    }

    fn draw_duck_trail(&self, bounds: Rect, crouch_wave: f32) {
        let streak = Color::from_rgba(140, 149, 178, 82);
        let dust = Color::from_rgba(184, 193, 217, 64);
        let y = bounds.y + bounds.h - 2.0;
        draw_line(
            bounds.x - 26.0,
            y,
            bounds.x - 4.0 + crouch_wave * 4.0,
            y - 2.0,
            2.4,
            streak,
        );
        draw_line(
            bounds.x - 38.0,
            y + 3.5,
            bounds.x - 10.0,
            y + 0.5,
            2.0,
            streak,
        );
        draw_ellipse(bounds.x - 14.0, y + 2.5, 13.0, 3.0, 0.0, dust);
        draw_ellipse(bounds.x - 28.0, y + 3.0, 9.0, 2.2, 0.0, dust);
    }

    fn draw_fallback_sprite(&self) {
        let bounds = self.bounds();
        let body = Color::from_rgba(245, 246, 252, 255);
        let shadow = Color::from_rgba(177, 181, 194, 255);
        let outline = Color::from_rgba(52, 57, 74, 255);
        let visor = Color::from_rgba(10, 12, 18, 255);

        if self.state == PlayerState::Ducking {
            draw_rectangle(
                bounds.x,
                bounds.y + 16.0,
                bounds.w - 2.0,
                bounds.h - 15.0,
                body,
            );
            draw_rectangle(
                bounds.x + bounds.w - 18.0,
                bounds.y + 10.0,
                15.0,
                10.0,
                body,
            );
            draw_rectangle(bounds.x + 5.0, bounds.y + bounds.h - 7.0, 20.0, 4.0, shadow);
            draw_rectangle(
                bounds.x + 26.0,
                bounds.y + bounds.h - 8.0,
                18.0,
                4.0,
                shadow,
            );
            draw_rectangle_lines(
                bounds.x,
                bounds.y + 16.0,
                bounds.w - 2.0,
                bounds.h - 15.0,
                2.0,
                outline,
            );
            draw_circle(bounds.x + bounds.w - 9.0, bounds.y + 14.0, 2.0, visor);
        } else {
            draw_rectangle(
                bounds.x + 12.0,
                bounds.y + 14.0,
                bounds.w - 16.0,
                bounds.h - 18.0,
                body,
            );
            draw_rectangle(bounds.x + bounds.w - 20.0, bounds.y + 2.0, 16.0, 18.0, body);
            draw_rectangle(
                bounds.x + 18.0,
                bounds.y + bounds.h - 15.0,
                6.0,
                15.0,
                shadow,
            );
            draw_rectangle(
                bounds.x + 30.0,
                bounds.y + bounds.h - 15.0,
                6.0,
                15.0,
                shadow,
            );
            draw_rectangle_lines(
                bounds.x + 12.0,
                bounds.y + 14.0,
                bounds.w - 16.0,
                bounds.h - 18.0,
                2.0,
                outline,
            );
            draw_circle(bounds.x + bounds.w - 9.0, bounds.y + 10.0, 2.0, visor);
        }
    }

    fn draw_crash_overlay(&self) {
        let bounds = self.bounds();
        let face_center = vec2(bounds.x + bounds.w * 0.62, bounds.y + bounds.h * 0.24);
        draw_line(
            face_center.x - 6.0,
            face_center.y - 4.0,
            face_center.x + 6.0,
            face_center.y + 4.0,
            2.0,
            RED,
        );
        draw_line(
            face_center.x - 6.0,
            face_center.y + 4.0,
            face_center.x + 6.0,
            face_center.y - 4.0,
            2.0,
            RED,
        );
    }

    fn bottom(&self) -> f32 {
        self.y + self.current_size().1
    }
}

#[cfg(test)]
mod tests {
    use super::{Player, PlayerState};

    #[test]
    fn ducking_reduces_hitbox_height() {
        let mut player = Player::new();
        player.update(0.0, false, true, false);

        assert_eq!(player.state, PlayerState::Ducking);
        assert!(player.bounds().h < 58.0);
    }

    #[test]
    fn jump_requires_ground_contact() {
        let mut player = Player::new();
        player.update(0.016, true, false, false);
        let first_y = player.y;

        player.update(0.016, true, false, false);

        assert!(player.y < first_y);
        assert_eq!(player.state, PlayerState::Jumping);
    }

    #[test]
    fn holding_duck_after_jump_stays_ducking_after_landing() {
        let mut player = Player::new();
        player.update(0.016, true, false, false);

        for _ in 0..120 {
            player.update(1.0 / 60.0, false, true, false);
        }

        assert_eq!(player.state, PlayerState::Ducking);
        let landed_y = player.y;

        player.update(1.0 / 60.0, false, true, false);

        assert_eq!(player.state, PlayerState::Ducking);
        assert!((player.y - landed_y).abs() < 0.001);
    }
}
