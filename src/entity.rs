use crate::atlas::Sprite;
use crate::pipe::PipeTracker;
use ggez::event::EventHandler;
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::Image;
use ggez::nalgebra::{Point2, Vector2};
use ggez::Context;
use ggez::GameResult;

const DEBUG: bool = false;

const GRAVITY: f32 = 0.28;
const JUMP_IMPULSE: f32 = 2.75;

#[derive(Debug, PartialEq, Eq, Clone)]
/// The current state of the game.
pub enum PlayState {
    StartScreen,
    Play,
    Dead { time: std::time::Duration },
}

/// The physics on the entity.
pub struct Physics {
    pub vel: Vector2<f32>,
    acc: Vector2<f32>,
    gravity: bool,
}

struct Scroll {
    jump_distance: f32,
}

#[derive(PartialEq, Eq, Clone)]
pub enum ScoringPipe {
    ReadyToScore,
    Scored,
}

pub struct Entity {
    pub sprite: Option<Sprite>,
    pub position: Point2<f32>,
    pub is_player: bool,
    can_jump: bool,
    pub physics: Option<Physics>,
    scroller: Option<Scroll>,
    pub is_pipe: bool,
    pub player_sprites: Option<Vec<Sprite>>,
    pub scoring_pipe: Option<ScoringPipe>,
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
            player_sprites: None,
            scoring_pipe: None,
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

    // Panics if there isn't a sprite.
    pub fn get_bounds(&self) -> graphics::Rect {
        match &self.sprite {
            Some(sprite) => sprite.aabb(),
            None => unimplemented!("This is not implemented"),
        }
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
                    Entity::jump(physics);
                }
                if state == PlayState::StartScreen {
                    state = PlayState::Play;
                }
                self.can_jump = false;
            }
        }

        // Self jumping script on the start screen.
        if self.is_player && self.physics.is_some() && state == PlayState::StartScreen {
            self.auth_jump()
        }

        if let Some(physics) = &mut self.physics {
            if !(PlayState::StartScreen == state && self.is_pipe) {
                physics.acc.scale(1.0 / delta);

                physics.vel += physics.acc;
                physics.vel.scale(1.0 / delta);
                self.position += physics.vel;

                // prevent falling off the left side of the screen.
                if let Some(scroll) = &self.scroller {
                    if let Some(sprite) = &self.sprite {
                        let right_pos = sprite.width + self.position.x;
                        if right_pos < 0.0 {
                            if self.is_pipe {
                                let diff = pt.get_pipe_difference();
                                self.position.y += diff;
                            }
                            if self.scoring_pipe.is_some() {
                                self.scoring_pipe = Some(ScoringPipe::ReadyToScore);
                            }
                            self.position.x += scroll.jump_distance;
                        }
                    }
                }
            }

            if self.is_player {
                // clamp y to not go above the top of the screen easily.
                self.position.y = if self.position.y < -16.0 {
                    -16.0
                } else {
                    self.position.y
                }
            }
        }

        (Ok(()), state)
    }

    fn jump(physics: &mut Physics) {
        physics.acc = Vector2::new(0.0, -GRAVITY);
        physics.vel = Vector2::new(0.0, -JUMP_IMPULSE);
    }

    pub fn draw(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult {
        if self.player_sprites.is_some() && self.physics.is_some() {
            if let Some(s) = &mut self.player_sprites {
                if let Some(p) = &self.physics {
                    // need velocity to map to these rotations between -0.2 and 0.2!
                    let angle = rescale_range(p.vel.y, -7.0, 7.0, -0.6, 0.6);
                    if p.vel.y < 0.0 {
                        batch.add(
                            s[0].add_draw_param(self.position.clone())
                                .offset(Point2::new(0.5, 0.5))
                                .rotation(angle),
                        );
                    } else {
                        batch.add(
                            s[1].add_draw_param(self.position.clone())
                                .offset(Point2::new(0.5, 0.5))
                                .rotation(angle),
                        );
                    }
                }
            }
        } else {
            if let Some(s) = &mut self.sprite {
                batch.add(s.add_draw_param(self.position.clone()));
                if DEBUG {
                    let mut rect = s.aabb();
                    let mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(1.0),
                        rect,
                        graphics::BLACK,
                    )?;
                    let p = graphics::DrawParam::new()
                        .dest(self.position.clone() * 4.0)
                        .scale(Vector2::new(4.0, 4.0));
                    graphics::draw(ctx, &mesh, p)?;
                }
            }
        }
        Ok(())
    }

    fn auth_jump(&mut self) -> () {
        if let Some(physics) = &mut self.physics {
            if self.position.y > 600.0 / 8.0 {
                Entity::jump(physics);
            }
        }
    }
}

/// Returns an f32 scaled [oldMin, oldMax] into the range [newMin, newMax]
/// Thanks https://stackoverflow.com/a/5295202/6421793
fn rescale_range(value: f32, oldMin: f32, oldMax: f32, newMin: f32, newMax: f32) -> f32 {
    use ggez::nalgebra::clamp;
    let old_range = oldMax - oldMin;
    let new_range = newMax - newMin;
    (((clamp(value, oldMin, oldMax) - oldMin) * new_range) / old_range) + newMin
}
