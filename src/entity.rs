use crate::atlas::Sprite;
use crate::pipe::PipeTracker;
use ggez::event::EventHandler;
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::Image;
use ggez::nalgebra::{Point2, Vector2};
use ggez::Context;
use ggez::GameResult;

const DEBUG: bool = true;

const GRAVITY: f32 = 0.28;
const JUMP_IMPULSE: f32 = 2.75;

#[derive(Debug, PartialEq, Eq, Clone)]
/// The current state of the game.
pub enum PlayState {
    StartScreen,
    Play,
    Dead,
}

/// The physics on the entity.
struct Physics {
    vel: Vector2<f32>,
    acc: Vector2<f32>,
    gravity: bool,
}

struct Scroll {
    jump_distance: f32,
}

pub struct Entity {
    pub sprite: Option<Sprite>,
    pub position: Point2<f32>,
    pub is_player: bool,
    can_jump: bool,
    physics: Option<Physics>,
    scroller: Option<Scroll>,
    pub is_pipe: bool,
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
            scroller: None,
            is_pipe: false,
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

    pub fn scroller(mut self, dist: f32) -> Self {
        self.scroller = Some(Scroll {
            jump_distance: dist,
        });
        self
    }

    pub fn set_velocity(mut self, vel: Vector2<f32>) -> Self {
        if let Some(p) = &mut self.physics {
            p.vel = vel;
        }
        self
    }
}

impl Entity {
    pub fn update(
        &mut self,
        ctx: &mut Context,
        pt: &mut PipeTracker,
        state: &PlayState,
    ) -> (GameResult, PlayState) {
        let delta = ggez::timer::delta(ctx).as_nanos() as f32;
        let mut state = state.clone();

        if let Some(physics) = &mut self.physics {
            physics.acc = if physics.gravity {
                Vector2::new(0.0, GRAVITY)
            } else {
                Vector2::new(0.0, 0.0)
            };
        }

        if self.is_player
            && self.physics.is_some()
            && (state == PlayState::StartScreen || state == PlayState::Play)
        {
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
                if state == PlayState::StartScreen {
                    state = PlayState::Play;
                }
                self.can_jump = false;
            }
        }

        // Self jumping script on the start screen.
        if self.is_player && self.physics.is_some() && state == PlayState::StartScreen {
            if let Some(physics) = &mut self.physics {
                if self.position.y > 600.0 / 8.0 {
                    physics.acc = Vector2::new(0.0, -GRAVITY);
                    physics.vel = Vector2::new(0.0, -JUMP_IMPULSE);
                }
            }
        }

        if let Some(physics) = &mut self.physics {
            if state == PlayState::Play || (state == PlayState::StartScreen && self.is_player) {
                physics.acc.scale(1.0 / delta);

                physics.vel += physics.acc;
                physics.vel.scale(1.0 / delta);
                self.position += physics.vel;

                // prevent falling off the left side of the screen.
                if let Some(scroll) = &self.scroller {
                    if let Some(sprite) = &self.sprite {
                        let right_pos = sprite.width + self.position.x;
                        if right_pos < 0.0 {
                            if (self.is_pipe) {
                                let diff = pt.get_pipe_difference();
                                self.position.y += diff;
                            }
                            self.position.x += scroll.jump_distance;
                        }
                    }
                }
            }
        }

        (Ok(()), state)
    }

    pub fn draw(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult {
        if let Some(s) = &mut self.sprite {
            s.add_draw_param(self.position.clone(), batch);
            if DEBUG {
                let mesh = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(1.0),
                    s.aabb(),
                    graphics::BLACK,
                )?;
                graphics::draw(ctx, &mesh, s.draw_params(self.position.clone()))?;
            }
        }
        Ok(())
    }
}
