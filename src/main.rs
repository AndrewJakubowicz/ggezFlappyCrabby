use ggez::nalgebra::{Point2, Vector2};
use ggez::{
    conf::Conf,
    event::EventHandler,
    graphics::{spritebatch::SpriteBatch, Drawable, Text},
    *,
};
mod entity;
use entity::Entity;
mod atlas;

use std::{cell::RefCell, rc::Rc};

struct GameState {
    /// Array of entities.
    /// Drawn in order.
    entities: Vec<Entity>,
    /// The sprite batch of all the images
    spritebatch: SpriteBatch,
}

impl GameState {
    /// Creates a new GameState
    /// Panics if can't access the sprite image resource.
    fn new(spritebatch: SpriteBatch) -> Self {
        Self {
            entities: vec![],
            spritebatch,
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        for i in 0..self.entities.len() {
            self.entities[i].update(ctx)?;
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb(240, 230, 255));

        for i in 0..self.entities.len() {
            self.entities[i].draw(ctx, &mut self.spritebatch)?;
        }

        let p = graphics::DrawParam::new()
            .dest(Point2::new(0.0, 0.0))
            .color(graphics::WHITE);
        {
            graphics::draw(ctx, &mut self.spritebatch, p);
            self.spritebatch.clear();
        }

        let fps = timer::fps(ctx);
        let fps_display = Text::new(format!("FPS: {}", fps));

        graphics::draw(
            ctx,
            &fps_display,
            (Point2::new(10.0, 10.0), graphics::WHITE),
        )?;

        graphics::present(ctx)?;
        std::thread::yield_now();
        Ok(())
    }
}

fn main() {
    let resource_dir = std::path::PathBuf::from("./resources");

    let cb =
        ggez::ContextBuilder::new("FlappyCrab", "youCodeThings").add_resource_path(resource_dir);

    let (ctx, event_loop) = &mut cb.build().expect("Failed to build ggez!");

    let image = graphics::Image::new(ctx, "/texture_atlas.png").unwrap();
    let mut batch = graphics::spritebatch::SpriteBatch::new(image);
    batch.set_filter(graphics::FilterMode::Nearest);

    let mut state = GameState::new(batch);

    let sprites =
        atlas::Atlas::parse_atlas_json(std::path::Path::new("resources/texture_atlas.json"));
    let crab0 = sprites.create_sprite("crab0.png");

    let mut player = entity::Entity::new().add_physics(true);
    player.sprite = Some(crab0);
    player.is_player = true;

    state.entities = vec![player];

    event::run(ctx, event_loop, &mut state).unwrap();
}
