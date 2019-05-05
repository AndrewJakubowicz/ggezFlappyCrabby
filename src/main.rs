use atlas::Sprite;
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

        let p = graphics::DrawParam::new().scale(Vector2::new(4.0, 4.0));
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
    use ggez::conf::*;
    let resource_dir = std::path::PathBuf::from("./resources");

    let cb = ggez::ContextBuilder::new("FlappyCrab", "youCodeThings")
        .add_resource_path(resource_dir)
        .window_setup(
            WindowSetup::default()
                .title("Flappy Crab (/)(;,,;)(/)!!!")
                .samples(NumSamples::Zero)
                .vsync(true),
        )
        .window_mode(WindowMode::default().dimensions(800.0, 600.0));

    let (ctx, event_loop) = &mut cb.build().expect("Failed to build ggez!");

    let image = graphics::Image::new(ctx, "/texture_atlas.png").unwrap();
    let mut batch = graphics::spritebatch::SpriteBatch::new(image);
    batch.set_filter(graphics::FilterMode::Nearest);

    let mut state = GameState::new(batch);

    let sprites =
        atlas::Atlas::parse_atlas_json(std::path::Path::new("resources/texture_atlas.json"));
    let crab0 = sprites.create_sprite("crab0.png");
    let floor_tile = sprites.create_sprite("floor_tile.png");

    let player = create_player(crab0);

    let mut entities = create_tiles(floor_tile);
    entities.extend(vec![player]);

    state.entities = entities;

    event::run(ctx, event_loop, &mut state).unwrap();
}

fn create_player(sprite: Sprite) -> Entity {
    let mut player = entity::Entity::new().add_physics(true);
    player.sprite = Some(sprite);
    player.is_player = true;
    player.position = Point2::new(40.0, 0.0);
    player
}

fn create_tile_scroll(sprite: Sprite, x: f32, jump: f32) -> Entity {
    let mut tile = entity::Entity::new().add_physics(false);
    tile.sprite = Some(sprite);
    tile.position = Point2::new(x, 100.0);
    tile.scroller(jump)
        .set_velocity(ggez::nalgebra::Vector2::new(-1.0, 0.0))
}

fn create_tiles(sprite: Sprite) -> Vec<Entity> {
    let number_of_tiles = 14;
    let width = sprite.width;
    let total_dist = width * (number_of_tiles as f32);
    (0..number_of_tiles)
        .into_iter()
        .map(|i| create_tile_scroll(sprite.clone(), (i as f32) * (width as f32), total_dist))
        .collect()
}
