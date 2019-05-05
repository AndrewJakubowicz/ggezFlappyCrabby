use crate::atlas::{Sprite};
use ggez::event::EventHandler;
use ggez::graphics::Image;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::Point2;
use ggez::Context;
use ggez::GameResult;
pub struct Entity {
    pub sprite: Option<Sprite>,
}

/// Everything that can be interacted with is an entity.
/// The player is an entity, as well as the pipes.
impl Entity {
    pub fn new() -> Self {
        Self { sprite: None }
    }
}

impl Entity {
    pub fn update(&mut self, ctx: &mut Context) -> GameResult {
        let time = ggez::timer::time_since_start(ctx).as_millis();
        if let Some(s) = &mut self.sprite {
            s.set_position(Point2::new(
                ((time as f64 / 1_000.0).sin() * 30.0 + 90.0) as f32,
                ((time as f64 / 1_100.0).sin() * 50.0 + 60.0) as f32,
            ));
        }
        Ok(())
    }

    pub fn draw(&mut self, _ctx: &mut Context, batch: &mut SpriteBatch) -> GameResult {
        if let Some(s) = &mut self.sprite {
            s.add_draw_param(batch);
        }
        Ok(())
    }
}
