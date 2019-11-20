use atlas::Sprite;
use ggez::nalgebra::{Point2, Vector2};
use ggez::{
    event::EventHandler,
    graphics::{spritebatch::SpriteBatch, Text},
    Context,
    GameResult,
    graphics,
    event
};
mod entity;
use entity::Entity;
mod atlas;
mod pipe;
mod crab;
mod audio;
mod window;
use entity::PlayState;
use pipe::{create_pipes, PipeTracker};
use audio::Player;
use std::time::Duration;
use crate::entity::{PlayerEntity, TileEntity};
use crate::crab::create_player2;

const NUMBER_OF_TILES: u8 = 14;
const RESTART_AFTER: Duration = std::time::Duration::from_secs(1);

struct GameState {
    /// Array of entities.
    /// Drawn in order.
    pipes: Vec<Box<Entity>>,
    tiles: Vec<Box<TileEntity>>,
    player: Box<PlayerEntity>,
    /// The sprite batch of all the images
    sprite_batch: SpriteBatch,
    /// The struct that moves the pipes around :)
    /// Can use any function over time between 0 and 600/16
    pipe_tracker: PipeTracker,
    play_state: PlayState,
    atlas: atlas::Atlas,
    score: i128,
    best_score: i128,
    sound_player: audio::Player,
}

impl GameState {
    /// Creates a new GameState
    /// Panics if can't access the sprite image resource.
    fn new(ctx: &mut Context, sprite_batch: SpriteBatch) -> Self {
        let mut pipe_tracker = pipe::PipeTracker::new();
        let sprites =
            atlas::Atlas::parse_atlas_json(std::path::Path::new("resources/texture_atlas.json"));
        let sound_player = Player::new(ctx);

        Self {
            pipes: GameState::create_start_entities(&sprites, &mut pipe_tracker),
            player: create_player2(&sprites),
            tiles : create_tiles(&sprites),
            sprite_batch,
            pipe_tracker,
            play_state: PlayState::StartScreen,
            atlas: sprites,
            score: 0,
            best_score: 0,
            sound_player
        }
    }

    /// The last entity *must* be the player.
    fn create_start_entities(
        sprites: &atlas::Atlas,
        pipe_tracker: &mut PipeTracker,
    ) -> Vec<Box<Entity>> {
        let pipes = create_pipes(
            sprites.create_sprite("pipe_bottom.png"),
            sprites.create_sprite("pipe_top.png"),
            pipe_tracker,
            200.0,
        );
        pipes
    }

    fn restart(&mut self) {
        self.sound_player.begin();
        let mut pt = PipeTracker::new();
        self.pipes = GameState::create_start_entities(&self.atlas, &mut pt);
        self.player = create_player2(&self.atlas);
        self.pipe_tracker = pt;
        self.play_state = PlayState::StartScreen;
        self.swap_scores();
        self.score = 0;
    }

    fn swap_scores(&mut self) {
        if self.score > self.best_score {
            self.best_score = self.score;
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let state = self.play_state.clone();
        self.handle_after_losing(ctx, state);
        let state = self.player.update(ctx, &self.play_state);
        if !self.play_state.is_playing() && state == PlayState::Play {
            self.play_state = PlayState::Play;
        }
        for i in 0..self.pipes.len() {
            self.pipes[i].update(ctx, &mut self.pipe_tracker, &self.play_state);
        }
        // TODO: Another loop to check if player is dead.
        update_it(self, ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb(112, 216, 255));

        self.player.draw(&mut self.sprite_batch)?;
        for i in 0..self.tiles.len() {
            self.tiles[i].draw(ctx, &mut self.sprite_batch);
        }
        for i in 0..self.pipes.len() {
            self.pipes[i].draw(ctx, &mut self.sprite_batch)?;
        }

        let p = graphics::DrawParam::new().scale(Vector2::new(4.0, 4.0));
        {
            graphics::draw(ctx, &mut self.sprite_batch, p)?;
            self.sprite_batch.clear();
        }

        draw_scores(self.score, self.best_score, ctx);

        graphics::present(ctx)?;
        std::thread::yield_now();
        Ok(())
    }
}

fn update_it(game: &mut GameState, ctx: &mut Context) {
    let player = &game.player;
    let pipes = &mut game.pipes;
    let tiles = &mut game.tiles;

    // Check player against others.

    for i in 0..pipes.len() {
        if pipes[i].set_scored(&game.play_state) {
            game.score += 1;
            game.sound_player.score();
        }

        // if hits a pipe  || hits ground
        if (player.overlaps(&pipes[i]) || hits_ground(player)) && game.play_state.is_playing() {
            game.sound_player.ouch();
            game.play_state.set_dead(ggez::timer::time_since_start(ctx));
        }
    }
}

fn hits_ground(player: &Box<PlayerEntity>) -> bool {
    player.position.y > 135.0
}

fn main() {
    let resource_dir = std::path::PathBuf::from("./resources");

    let cb = window::build_window(resource_dir);

    let (ctx, event_loop) = &mut cb.build().expect("Failed to build ggez!");

    let batch = create_batch_sprite(ctx);

    let mut state = GameState::new(ctx, batch);

    state.sound_player.begin();
    event::run(ctx, event_loop, &mut state).unwrap();
}

fn create_batch_sprite(ctx: &mut Context) -> SpriteBatch {
    let image = graphics::Image::new(ctx, "/texture_atlas.png").unwrap();
    let mut batch = graphics::spritebatch::SpriteBatch::new(image);
    batch.set_filter(graphics::FilterMode::Nearest);
    batch
}

fn create_tile_scroll(sprite: Sprite, x: f32) -> Box<TileEntity> {
    let tile = entity::TileEntity::new(sprite, (x, 145.0));
    // floor tiles do not need to move... do they ?!
    // let tile = tile.scroller(jump).set_velocity((-1.0, 0.0));

    Box::new(tile)
}

fn create_tiles(sprites: &atlas::Atlas) -> Vec<Box<TileEntity>> {
    let floor_tile = sprites.create_sprite("floor_tile.png");
    let width = floor_tile.width;

    (0..NUMBER_OF_TILES)
        .into_iter()
        .map(|i| create_tile_scroll(floor_tile.clone(), (i as f32) * width))
        .collect()
}

impl GameState {
    fn handle_after_losing(&mut self, ctx: &mut Context, state: PlayState) {
        match state {
            PlayState::Dead { time } => {
                if (ggez::timer::time_since_start(ctx) - time) > RESTART_AFTER {
                    self.restart()
                }
            }
            _ => {}
        }
    }
}

impl PlayState {
    fn is_playing(&self) -> bool {
        *self == PlayState::Play
    }

    fn set_dead (&mut self, time : std::time::Duration) {
        *self = PlayState::Dead {
            time
        }
    }

    fn is_not_dead (&self) -> bool {
        *self == PlayState::Play || *self == PlayState::StartScreen
    }
}

fn draw_scores(score : i128, best_score: i128, ctx: &mut Context) {
    let fps_display = Text::new(format!(
        "Best Score: {}   Current Score: {}",
        best_score, score
    ));

    graphics::draw(
        ctx,
        &fps_display,
        (Point2::new(10.0, 10.0), graphics::WHITE),
    );
}

