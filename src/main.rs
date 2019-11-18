use atlas::Sprite;
use ggez::nalgebra::{Point2, Vector2};
use ggez::{
    audio::SoundSource,
    event::EventHandler,
    graphics::{spritebatch::SpriteBatch, Text},
    Context,
    GameResult,
    graphics,
    event
};
use rand::distributions::OpenClosed01;
use rand::{thread_rng, Rng};
mod entity;
use entity::Entity;
mod atlas;
mod pipe;
mod crab;
mod audio;
mod window;
use entity::{PlayState, ScoringPipe};
use pipe::{create_pipes, PipeTracker};
use audio::Player;
use std::time::Duration;
use crab::create_player;
use crate::entity::GameEntity;

const NUMBER_OF_TILES: u8 = 14;
const RESTART_AFTER: Duration = std::time::Duration::from_secs(1);

struct GameState {
    /// Array of entities.
    /// Drawn in order.
    entities: Vec<Box<Entity>>,
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
    fn new(ctx: &mut Context, spritebatch: SpriteBatch) -> Self {
        let mut pipe_tracker = pipe::PipeTracker::new();
        let sprites =
            atlas::Atlas::parse_atlas_json(std::path::Path::new("resources/texture_atlas.json"));
        let sound_player = Player::new(ctx);

        Self {
            entities: GameState::create_start_entities(&sprites, &mut pipe_tracker),
            sprite_batch: spritebatch,
            pipe_tracker: pipe_tracker,
            play_state: PlayState::StartScreen,
            atlas: sprites,
            score: 0,
            best_score: 0,
            sound_player: sound_player
        }
    }

    /// The last entity *must* be the player.
    fn create_start_entities(
        sprites: &atlas::Atlas,
        pipe_tracker: &mut PipeTracker,
    ) -> Vec<Box<Entity>> {
        let floor_tile = sprites.create_sprite("floor_tile.png");
        let player = create_player(sprites);
        let mut entities = create_tiles(floor_tile);
        let pipes = create_pipes(
            sprites.create_sprite("pipe_bottom.png"),
            sprites.create_sprite("pipe_top.png"),
            pipe_tracker,
            200.0,
        );
        entities.extend(pipes);
        entities.extend(vec![player]);
        entities
    }

    fn restart(&mut self) {
        self.sound_player.begin();
        let mut pt = PipeTracker::new();
        self.entities = GameState::create_start_entities(&self.atlas, &mut pt);
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
        for i in 0..self.entities.len() {
            let (result, state) = self.entities[i].update(ctx, &mut self.pipe_tracker, &self.play_state);
            result?;
            if self.play_state != PlayState::Play && state == PlayState::Play {
                self.play_state = PlayState::Play;
                break;
            }
        }
        // TODO: Another loop to check if player is dead.
        update_it(self, ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb(112, 216, 255));

        for i in 0..self.entities.len() {
            self.entities[i].draw(ctx, &mut self.sprite_batch)?;
        }

        let p = graphics::DrawParam::new().scale(Vector2::new(4.0, 4.0));
        {
            graphics::draw(ctx, &mut self.sprite_batch, p);
            self.sprite_batch.clear();
        }

        draw_scores(self.score, self.best_score, ctx);

        graphics::present(ctx)?;
        std::thread::yield_now();
        Ok(())
    }
}

fn update_it(game: &mut GameState, ctx: &mut Context) {
    if let Some((player, other)) = game.entities.split_last_mut() {
        /*
        if player.position.y > 1000.0 {
            if g.play_state == PlayState::Play {
                if let Some(p) = &mut player.physics {
                    p.velocity.y = -100.0;
                }
                g.play_state = PlayState::Dead {
                    time: ggez::timer::time_since_start(ctx),
                };
            }
        }
        */
        // Check player against others.

        for i in 0..other.len() {

            if other[i].set_score(&game.play_state) {
                game.score += 1;
                let pitch: f32 = thread_rng().sample(OpenClosed01);
                game.sound_player.score_sound.set_pitch(1.0 + pitch);
                game.sound_player.score();
            }

            if player.overlaps(&other[i]) && PlayState::is_playing(&game.play_state) {
                game.sound_player.ouch();
                game.play_state = PlayState::Dead {
                    time: ggez::timer::time_since_start(ctx),
                };
            }
        }
    }
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

fn create_tile_scroll(sprite: Sprite, x: f32, jump: f32) -> Box<Entity> {
    let mut tile = entity::Entity::new(false, sprite);
    tile.position = Point2::new(x, 145.0);
    let tile = tile.scroller(jump)
        .set_velocity(ggez::nalgebra::Vector2::new(-1.0, 0.0));
    Box::new(tile)
}

fn create_tiles(sprite: Sprite) -> Vec<Box<Entity>> {
    let width = sprite.width;
    let total_dist = width * (NUMBER_OF_TILES as f32);
    (0..NUMBER_OF_TILES)
        .into_iter()
        .map(|i| create_tile_scroll(sprite.clone(), (i as f32) * width, total_dist))
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
    fn is_playing(play_state : &PlayState) -> bool {
        *play_state == PlayState::Play
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

