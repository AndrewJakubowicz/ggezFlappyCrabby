use crate::atlas::Sprite;
use crate::pipe::{PipeTracker, pipe_velocity};
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::{Point2, Vector2};
use ggez::Context;
use ggez::GameResult;

const DEBUG: bool = false;


#[derive(Debug, PartialEq, Eq, Clone)]
/// The current state of the game.
pub enum PlayState {
    StartScreen,
    Play,
    Dead { time: std::time::Duration },
}


pub struct Scroll {
    jump_distance: f32,
}

#[derive(PartialEq, Eq, Clone)]
pub enum ScoringPipe {
    Dormant,
    ReadyToScore,
    Scored,
}

pub struct PipeEntity {
    pub sprite: Sprite,
    pub position: Point2<f32>,
    pub scroller: Scroll,
    pub scoring_pipe: ScoringPipe,
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
    pub fn new(sprite: Sprite, position: (f32, f32), scroller : f32) -> Self {

        Self {
            sprite,
            position: Point2::new(position.0, position.1),
            scroller: Scroll { jump_distance: scroller },
            scoring_pipe: ScoringPipe::Dormant,
        }
    }

    pub fn new_pipe(sprite: Sprite, x: f32, y: f32, scroll : f32) -> Self {
        Self::new(sprite, (x, y), scroll)
    }

    // Panics if there isn't a sprite.
    pub fn get_rect(&self) -> graphics::Rect {
        let mut rect = self.sprite.get_bound_box();
        rect.move_to(self.position.clone());

        rect
    }

    pub fn is_ready_to_score(&self) -> bool {
        ScoringPipe::ReadyToScore == self.scoring_pipe
    }


    fn recycle_passed_pipes(&mut self, pipe_tracker: &mut PipeTracker) {
        let right_pos = &self.sprite.width + self.position.x;
        if right_pos >= 0.0 {
            return ;
        }

        self.position.y += pipe_tracker.get_pipe_difference();

        self.resetScoredPipes();
        let scroll = &self.scroller;
        self.position.x += scroll.jump_distance;
    }

    fn resetScoredPipes(&mut self) {
        if self.scoring_pipe == ScoringPipe::Scored {
            self.scoring_pipe = ScoringPipe::ReadyToScore;
        }
    }
}

impl PipeEntity {
    pub fn update(
        &mut self,
        pipe_tracker: &mut PipeTracker,
        state: &PlayState,
    ) {
        if PlayState::StartScreen == *state {
          return ;
        }

        // Moves the pipes towards the crab !
        let speed = pipe_velocity();
        self.position += Vector2::new(speed, 0.0);
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
        if self.position.x  > 20.0 {
            return false;
        }

        if self.is_ready_to_score() && PlayState::is_playing(&play_state) {
            self.scoring_pipe = ScoringPipe::Scored;

            return true;
        }

        false
    }
}
