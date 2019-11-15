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
use entity::{PlayState, ScoringPipe};
use pipe::{create_pipes, PipeTracker};
use audio::{Player};
use ggez::graphics::Rect;
use std::time::Duration;
use crab::create_player;

const NUMBER_OF_TILES: u8 = 14;
const RESTART_AFTER: Duration = std::time::Duration::from_secs(1);

struct GameState {
    /// Array of entities.
    /// Drawn in order.
    entities: Vec<Entity>,
    /// The sprite batch of all the images
    sprite_batch: SpriteBatch,
    /// The struct that moves the pipes around :)
    /// Can use any function over time between 0 and 600/16
    pt: PipeTracker,
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
        let mut sound_player = Player::new(ctx);

        sound_player.begin();

        Self {
            entities: GameState::create_start_entities(&sprites, &mut pipe_tracker),
            sprite_batch: spritebatch,
            pt: pipe_tracker,
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
    ) -> Vec<Entity> {
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
        self.pt = pt;
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
        self.handle_loosing(ctx, state);
        for i in 0..self.entities.len() {
            let (result, state) = self.entities[i].update(ctx, &mut self.pt, &self.play_state);
            result?;
            if self.play_state != PlayState::Play && state == PlayState::Play {
                self.play_state = PlayState::Play;
                break;
            }
        }
        // TODO: Another loop to check if player is dead.
        {
            if let Some((player, other)) = self.entities.split_last_mut() {
                /*
                if player.position.y > 1000.0 {
                    if self.play_state == PlayState::Play {
                        if let Some(p) = &mut player.physics {
                            p.velocity.y = -100.0;
                        }
                        self.play_state = PlayState::Dead {
                            time: ggez::timer::time_since_start(ctx),
                        };
                    }
                }
                */
                // Check player against others.
                let mut player_rect = player.get_bounds();
                player_rect.move_to(player.position.clone());
                for i in 0..other.len() {
                    {
                        let mut scored = false;
                        if let Some(ScoringPipe::ReadyToScore) = other[i].scoring_pipe {
                            if other[i].position.x < 20.0 {
                                scored = true;
                            }
                        }
                        if scored && PlayState::is_playing(&self.play_state) {
                            other[i].scoring_pipe = Some(ScoringPipe::Scored);
                            self.score += 1;
                            let pitch: f32 = thread_rng().sample(OpenClosed01);
                            self.sound_player.score_sound.set_pitch(1.0 + pitch);
                            self.sound_player.score();
                        }
                    }
                    if other[i].sprite.is_none() {
                        continue;
                    }
                    let mut other_rect = other[i].get_bounds();
                    other_rect.move_to(other[i].position.clone());
                    if other_rect.overlaps(&player_rect) {
                        if PlayState::is_playing(&self.play_state) {
                            self.sound_player.ouch();
                            self.play_state = PlayState::Dead {
                                time: ggez::timer::time_since_start(ctx),
                            };
                        }
                    }
                }
            }
        }
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

        let fps_display = Text::new(format!(
            "Best Score: {}   Current Score: {}",
            self.best_score, self.score
        ));

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

    let mut state = GameState::new(ctx, batch);

    event::run(ctx, event_loop, &mut state).unwrap();
}

fn create_tile_scroll(sprite: Sprite, x: f32, jump: f32) -> Entity {
    let mut tile = entity::Entity::new().add_physics(false);
    tile.sprite = Some(sprite);
    tile.position = Point2::new(x, 145.0);
    tile.scroller(jump)
        .set_velocity(ggez::nalgebra::Vector2::new(-1.0, 0.0))
}

fn create_tiles(sprite: Sprite) -> Vec<Entity> {
    let width = sprite.width;
    let total_dist = width * (NUMBER_OF_TILES as f32);
    (0..NUMBER_OF_TILES)
        .into_iter()
        .map(|i| create_tile_scroll(sprite.clone(), (i as f32) * width, total_dist))
        .collect()
}

impl GameState {
    fn handle_loosing(&mut self, ctx: &mut Context, state: PlayState) {
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
