use crate::atlas::Sprite;
use crate::pipe::PipeTracker;
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::{Point2, Vector2};
use ggez::Context;
use ggez::GameResult;
use ggez::graphics::Rect;
use crate::GameState;
use ggez::audio::SoundSource;
use rand::Rng;
use rand::distributions::OpenClosed01;
use rand::thread_rng;

const DEBUG: bool = false;

const GRAVITY: f32 = 0.28;
const JUMP_IMPULSE: f32 = 2.75;
pub const SCREEN_TOP: f32 = -16.0;

#[derive(Debug, PartialEq, Eq, Clone)]
/// The current state of the game.
pub enum PlayState {
    StartScreen,
    Play,
    Dead { time: std::time::Duration },
}

/// The physics on the entity.
pub struct Physics {
    pub velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    gravity: bool,
}

impl Physics {
    fn new (with_gravity: bool) -> Self {
        Self {
            velocity: Vector2::new(0.0, 0.0),
            acceleration: Vector2::new(0.0, 0.0),
            gravity: with_gravity,
        }
    }
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
    pub sprite: Sprite,
    pub position: Point2<f32>,
    pub is_player: bool,
    can_jump: bool,
    pub physics: Physics,
    scroller: Option<Scroll>,
    pub is_pipe: bool,
    pub player_sprites: Option<Vec<Sprite>>,
    pub scoring_pipe: Option<ScoringPipe>,
}

pub struct PipeEntity {
    pub sprite: Sprite,
    pub position: Point2<f32>,
    pub is_player: bool,
    can_jump: bool,
    pub physics: Physics,
    scroller: Option<Scroll>,
    pub is_pipe: bool,
    pub player_sprites: Option<Vec<Sprite>>,
    pub scoring_pipe: Option<ScoringPipe>,
}
/// Everything that can be interacted with is an entity.
/// The player is an entity, as well as the pipes.
impl PipeEntity {
    pub fn new(with_gravity: bool, sprite: Sprite) -> Self {
        Self {
            sprite: sprite,
            position: Point2::new(0.0, 0.0),
            is_player: false,
            physics: Physics::new(with_gravity),
            can_jump: true,
            scroller: None,
            is_pipe: true,
            player_sprites: None,
            scoring_pipe: None,
        }
    }

    // Panics if there isn't a sprite.
    pub fn get_bounds(&self) -> graphics::Rect {
        self.sprite.get_bound_box()
    }

    pub fn scroller(mut self, dist: f32) -> Self {
        self.scroller = Some(Scroll {
            jump_distance: dist,
        });
        self
    }

    pub fn set_velocity(mut self, vel: Vector2<f32>) -> Self {
        self.physics.velocity = vel;

        self
    }
}
pub trait GameEntity {
    fn update(&mut self, ctx: &mut Context, pipe_tracker: &mut PipeTracker, state: &PlayState) -> (GameResult, PlayState);
    fn draw(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult;
    fn overlaps(&self, other : &Self) -> bool;
    fn set_score (&mut self, play_state : &PlayState) -> bool;
}

/// Everything that can be interacted with is an entity.
/// The player is an entity, as well as the pipes.
impl Entity {
    pub fn new(with_gravity: bool, sprite: Sprite) -> Self {
        Self {
            sprite: sprite,
            position: Point2::new(0.0, 0.0),
            is_player: false,
            physics: Physics::new(with_gravity),
            can_jump: true,
            scroller: None,
            is_pipe: false,
            player_sprites: None,
            scoring_pipe: None,
        }
    }

    // Panics if there isn't a sprite.
    pub fn get_bounds(&self) -> graphics::Rect {
        let mut rect = self.sprite.get_bound_box();
        rect.move_to(self.position.clone());

        rect
    }

    pub fn get_scored (&self) -> bool {
        let mut scored = false;
        if let Some(ScoringPipe::ReadyToScore) = self.scoring_pipe {
            if self.position.x < 20.0 {
                scored = true;
            }
        }
        scored
    }

    pub fn scroller(mut self, dist: f32) -> Self {
        self.scroller = Some(Scroll {
            jump_distance: dist,
        });
        self
    }

    pub fn set_velocity(mut self, vel: Vector2<f32>) -> Self {
        self.physics.velocity = vel;

        self
    }

    fn auto_jump(&mut self) -> () {
        let physics = &mut self.physics;
        if self.position.y > 600.0 / 8.0 {
            Entity::jump(physics);
        }
    }

    fn prevent_going_off(&mut self) -> () {
        self.position.y = if self.position.y < SCREEN_TOP {
            SCREEN_TOP
        } else {
            self.position.y
        }
    }
    fn jump(physics: &mut Physics) {
        physics.acceleration = Vector2::new(0.0, -GRAVITY);
        physics.velocity = Vector2::new(0.0, -JUMP_IMPULSE);
    }

    fn prevent_falling_off_left(&mut self, pipe_tracker: &mut PipeTracker) {
        if self.scroller.as_ref().is_none() {
            return ;
        }
        let scroll = self.scroller.as_ref().unwrap();
        let sprite = &self.sprite;
        let right_pos = sprite.width + self.position.x;
        if right_pos >= 0.0 {
            return ;
        }
        if self.is_pipe {
            self.position.y += pipe_tracker.get_pipe_difference();
        }
        if self.scoring_pipe.is_some() {
            self.scoring_pipe = Some(ScoringPipe::ReadyToScore);
        }
        self.position.x += scroll.jump_distance;
    }
}

impl Entity {
    pub fn update(
        &mut self,
        ctx: &mut Context,
        pipe_tracker: &mut PipeTracker,
        state: &PlayState,
    ) -> (GameResult, PlayState) {
        let delta = ggez::timer::delta(ctx).as_nanos() as f32;
        let mut state = state.clone();

        let physics = &mut self.physics;
        physics.acceleration = if physics.gravity {
            Vector2::new(0.0, GRAVITY)
        } else {
            Vector2::new(0.0, 0.0)
        };


        if self.is_player && (state == PlayState::StartScreen || state == PlayState::Play)
        {
            use ggez::event::KeyCode;
            use ggez::input::keyboard;
            if !keyboard::pressed_keys(ctx).contains(&KeyCode::Space) && !self.can_jump {
                self.can_jump = true;
            }

            if keyboard::is_key_pressed(ctx, KeyCode::Space) && self.can_jump {
                let physics = &mut self.physics;
                Entity::jump(physics);

                if state == PlayState::StartScreen {
                    state = PlayState::Play;
                }
                self.can_jump = false;
            }
        }

        // Self jumping script on the start screen.
        if self.is_player && state == PlayState::StartScreen {
            self.auto_jump()
        }

        if !(PlayState::StartScreen == state && self.is_pipe) {
            let physics = &mut self.physics;
            physics.acceleration.scale(1.0 / delta);

            physics.velocity += physics.acceleration;
            physics.velocity.scale(1.0 / delta);
            self.position += physics.velocity;

            // prevent falling off the left side of the screen.
            self.prevent_falling_off_left(pipe_tracker)
        }

        if self.is_player {
            // clamp y to not go above the top of the screen easily.
            self.prevent_going_off();
        }

        (Ok(()), state)
    }

    pub fn draw(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult {
        if self.player_sprites.is_some() {
            if let Some(s) = &mut self.player_sprites {
                let p = &self.physics;
                    // need velocity to map to these rotations between -0.2 and 0.2!
                    let angle = rescale_range(p.velocity.y, -7.0, 7.0, -0.6, 0.6);
                    let x = if p.velocity.y < 0.0 {
                        &mut s[0]
                    } else {
                        &mut s[1]
                    };
                    batch.add(
                        x.add_draw_param(self.position.clone())
                            .offset(Point2::new(0.5, 0.5))
                            .rotation(angle),
                    );

            }
        } else {
            let s = &mut self.sprite; {
                batch.add(s.add_draw_param(self.position.clone()));
                if DEBUG {
                    let rect = s.get_bound_box();
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

    pub fn overlaps(&self, other : &Self) -> bool {
        let player_rect = self.get_bounds();
        let other_rect = other.get_bounds();

        other_rect.overlaps(&player_rect)
    }

    pub fn set_scored(&mut self, play_state : &PlayState) -> bool {
        let scored = self.get_scored();

        if scored && PlayState::is_playing(&play_state) {
            self.scoring_pipe = Some(ScoringPipe::Scored);
            return true;
        }
        false
    }
}

/// Returns an f32 scaled [oldMin, oldMax] into the range [newMin, newMax]
/// Thanks https://stackoverflow.com/a/5295202/6421793
fn rescale_range(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    use ggez::nalgebra::clamp;
    let old_range = old_max - old_min;
    let new_range = new_max - new_min;
    (((clamp(value, old_min, old_max) - old_min) * new_range) / old_range) + new_min
}
