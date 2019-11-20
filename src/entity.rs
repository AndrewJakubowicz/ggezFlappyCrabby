use crate::atlas::Sprite;
use crate::pipe::{PipeTracker, pipe_velocity};
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::{Point2, Vector2};
use ggez::Context;
use ggez::GameResult;

const DEBUG: bool = false;

const GRAVITY: f32 = 0.25;
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

pub struct TileEntity {
    pub sprite: Sprite,
    pub position: Point2<f32>,
}

impl TileEntity {
    pub fn draw(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) {
        let s = &mut self.sprite;
        batch.add(s.add_draw_param(self.position.clone()));
    }

    pub fn new(sprite: Sprite, position: (f32, f32)) -> Self {
        Self {
            sprite,
            position: Point2::new(position.0, position.1),
        }
    }
}

pub struct PipeEntity {
    pub sprite: Sprite,
    pub position: Point2<f32>,
    scroller: Option<Scroll>,
    pub scoring_pipe: Option<ScoringPipe>,
}

pub struct PlayerEntity {
    pub sprite: Sprite,
    pub position: Point2<f32>,
    pub player_sprites: Vec<Sprite>,
    can_jump: bool,
    pub physics: Physics,
}

impl PlayerEntity {
    pub fn update(
        &mut self,
        ctx: &mut Context,
        state: &PlayState,
    ) -> PlayState {
        let physics = &mut self.physics;
        physics.acceleration = if physics.gravity {
            Vector2::new(0.0, GRAVITY)
        } else {
            Vector2::new(0.0, 0.0)
        };


        let mut state = state.clone();
        if state.is_not_dead()
        {
            use ggez::event::KeyCode;
            use ggez::input::keyboard;
            if !keyboard::pressed_keys(ctx).contains(&KeyCode::Space) && !self.can_jump {
                self.can_jump = true;
            }

            if keyboard::is_key_pressed(ctx, KeyCode::Space) && self.can_jump {
                let physics = &mut self.physics;
                PlayerEntity::jump(physics);

                // exit start screen state.
                if state == PlayState::StartScreen {
                    state = PlayState::Play;
                }
            }
        }

        // Self jumping script on the start screen.
        if state == PlayState::StartScreen {
            self.auto_jump()
        }

        self.change_position(ctx);

        // clamp y to not go above the top of the screen easily.
        self.prevent_going_off();
        state
    }

    fn change_position(&mut self, ctx: &mut Context) {
        let delta = ggez::timer::delta(ctx).as_nanos() as f32;
        let physics = &mut self.physics;
        physics.acceleration.scale(1.0 / delta);
        physics.velocity += physics.acceleration;
        physics.velocity.scale(1.0 / delta);
        // moves all the entities on the board.
        self.position += physics.velocity;
    }

    pub fn new(sprite: Sprite, position: (f32, f32), player_sprites: Vec<Sprite>) -> Self {
        Self {
            sprite,
            position: Point2::new(position.0, position.1),
            physics: Physics::new(true),
            can_jump: true,
            player_sprites,
        }
    }
    pub fn overlaps(&self, other : &PipeEntity) -> bool {
        let player_rect = self.get_bounds();
        let other_rect = other.get_bounds();

        other_rect.overlaps(&player_rect)
    }
    pub fn get_bounds(&self) -> graphics::Rect {
        let mut rect = self.sprite.get_bound_box();
        rect.move_to(self.position.clone());

        rect
    }
    fn prevent_going_off(&mut self) -> () {
        self.position.y = if self.position.y < SCREEN_TOP {
            SCREEN_TOP
        } else {
            self.position.y
        }
    }
    fn auto_jump(&mut self) -> () {
        let physics = &mut self.physics;
        if self.position.y > 600.0 / 8.0 {
            PlayerEntity::jump(physics);
        }
    }
    pub fn draw(&mut self, batch: &mut SpriteBatch) -> GameResult {
        self.draw_player(batch);

        Ok(())
    }
    fn draw_player(&mut self, batch: &mut SpriteBatch) {
        let s = &mut self.player_sprites;
        let p = &self.physics;
        // need velocity to map to these rotations between -0.2 and 0.2!
        let angle = rescale_range(p.velocity.y, -7.0, 7.0, -0.6, 0.6);
        let x = if p.velocity.y >= 0.0 {
            &mut s[1]
        } else {
            &mut s[0]
        };
        batch.add(
            x.add_draw_param(self.position.clone())
                .offset(Point2::new(0.5, 0.5))
                .rotation(angle),
        );
    }

    fn jump(physics: &mut Physics) {
        physics.acceleration = Vector2::new(0.0, -GRAVITY);
        physics.velocity = Vector2::new(0.0, -JUMP_IMPULSE);
    }
}

/// Everything that can be interacted with is an entity.
/// The player is an entity, as well as the pipes.
pub trait GameEntity {
    fn update(&mut self, ctx: &mut Context, pipe_tracker: &mut PipeTracker, state: &PlayState) -> (GameResult, PlayState);
    fn draw(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult;
    fn overlaps(&self, other : &Self) -> bool;
    fn set_score (&mut self, play_state : &PlayState) -> bool;
}

/// Everything that can be interacted with is an entity.
/// The player is an entity, as well as the pipes.
impl PipeEntity {
    pub fn new(sprite: Sprite, position: (f32, f32)) -> Self {
        Self {
            sprite,
            position: Point2::new(position.0, position.1),
            scroller: None,
            scoring_pipe: None,
        }
    }

    pub fn new_pipe(sprite: Sprite, x: f32, y: f32) -> Self {
        let mut pipe = Self::new(sprite, (x, y));

        pipe
    }

    // Panics if there isn't a sprite.
    pub fn get_bounds(&self) -> graphics::Rect {
        let mut rect = self.sprite.get_bound_box();
        rect.move_to(self.position.clone());

        rect
    }

    pub fn get_scored (&self) -> bool {
        if let Some(ScoringPipe::ReadyToScore) = self.scoring_pipe {
            return true;
        } else {
            return false;
        }
    }

    pub fn scroller(mut self, dist: f32) -> Self {
        self.scroller = Some(Scroll {
            jump_distance: dist,
        });
        self
    }

    fn recycle_passed_pipes(&mut self, pipe_tracker: &mut PipeTracker) {
        let right_pos = &self.sprite.width + self.position.x;
        if right_pos >= 0.0 {
            return ;
        }

        self.position.y += pipe_tracker.get_pipe_difference();

        if self.scoring_pipe.is_some() {
            self.scoring_pipe = Some(ScoringPipe::ReadyToScore);
        }
        let scroll = self.scroller.as_ref().unwrap();
        self.position.x += scroll.jump_distance;
    }
}

impl PipeEntity {
    pub fn update(
        &mut self,
        ctx: &mut Context,
        pipe_tracker: &mut PipeTracker,
        state: &PlayState,
    ) {
        if PlayState::StartScreen == *state {
          return ;
        }

        // Moves the pipes towards the crab !
        let speed = pipe_velocity();
        self.position += Vector2::new(speed, 0);
        // when the pipes go off the left side.
        // we put them back at right side to come again.
        self.recycle_passed_pipes(pipe_tracker);
    }

    pub fn draw(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult {
        self.draw_entity(ctx, batch)?;
        Ok(())
    }

    fn draw_entity(&mut self, ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult {
        let s = &mut self.sprite;
        batch.add(s.add_draw_param(self.position.clone()));

        if !DEBUG {
            return Ok(())
        }

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

        Ok(())
    }

    pub fn set_scored(&mut self, play_state : &PlayState) -> bool {
        if self.position.x >= 20.0 {
            return false;
        }

        if  self.get_scored() && PlayState::is_playing(&play_state) {
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
