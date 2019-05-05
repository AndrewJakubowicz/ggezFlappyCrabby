use crate::atlas::Sprite;
use ggez::event::EventHandler;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::Image;
use ggez::nalgebra::{Point2, Vector2};
use ggez::Context;
use ggez::GameResult;

const GRAVITY: f32 = 0.6;
const JUMP_IMPULSE: f32 = 8.0;

/// The physics on the entity.
struct Physics {
    vel: Vector2<f32>,
    acc: Vector2<f32>,
    gravity: bool,
}

pub struct Entity {
    pub sprite: Option<Sprite>,
    pub position: Point2<f32>,
    pub is_player: bool,
    can_jump: bool,
    physics: Option<Physics>,
}

/// Everything that can be interacted with is an entity.
/// The player is an entity, as well as the pipes.
impl Entity {
    pub fn new() -> Self {
        Self {
            sprite: None,
            position: Point2::new(0.0, 0.0),
            is_player: false,
            physics: None,
            can_jump: true,
        }
    }
    pub fn add_physics(mut self, with_gravity: bool) -> Self {
        self.physics = Some(Physics {
            vel: Vector2::new(0.0, 0.0),
            acc: Vector2::new(0.0, 0.0),
            gravity: with_gravity,
        });
        self
    }
}

impl Entity {
    pub fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(ctx).as_nanos() as f32;

        if let Some(physics) = &mut self.physics {
            physics.acc = if physics.gravity {
                Vector2::new(0.0, GRAVITY)
            } else {
                Vector2::new(0.0, 0.0)
            };
        }

        if self.is_player && self.physics.is_some() {
            use ggez::event::KeyCode;
            use ggez::input::keyboard;
            if !keyboard::pressed_keys(ctx).contains(&KeyCode::Space) && !self.can_jump {
                self.can_jump = true;
            }

            if keyboard::is_key_pressed(ctx, KeyCode::Space) && self.can_jump {
                if let Some(physics) = &mut self.physics {
                    physics.acc = Vector2::new(0.0, -GRAVITY);
                    physics.vel = Vector2::new(0.0, -JUMP_IMPULSE);
                }
                self.can_jump = false;
            }
        }

        if let Some(physics) = &mut self.physics {
            physics.acc.scale(1.0 / delta);

            physics.vel += physics.acc;
            physics.vel.scale(1.0 / delta);
            self.position += physics.vel;
        }

        Ok(())
    }

    pub fn draw(&mut self, _ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult {
        if let Some(s) = &mut self.sprite {
            s.add_draw_param(self.position.clone(), batch);
        }
        Ok(())
    }
}
