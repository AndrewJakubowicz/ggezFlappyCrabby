use ggez::{
    graphics::{spritebatch::SpriteBatch},
    Context,
};
use audio::Player;
use crate::crab::{PlayerEntity, create_player};
use crate::tile::{TileEntity, create_tiles};
use crate::entity::{PlayState, PipeEntity};
use crate::pipe::{create_pipes, PipeTracker};
use crate::{audio, atlas, pipe, RESTART_AFTER};

pub struct GameState {
    pub tiles_drawn: bool,
    pub pipes: Vec<Box<PipeEntity>>,
    pub tiles: Vec<Box<TileEntity>>,
    pub player: Box<PlayerEntity>,
    /// The sprite batch of all the images
    pub sprite_batch: SpriteBatch,
    /// The struct that moves the pipes around :)
    /// Can use any function over time between 0 and 600/16
    pub pipe_tracker: PipeTracker,
    pub play_state: PlayState,
    atlas: atlas::Atlas,
    pub score: i128,
    pub best_score: i128,
    pub sound_player: audio::Player,
}

impl GameState {
    pub fn handle_after_losing(&mut self, ctx: &mut Context, state: PlayState) {
        match state {
            PlayState::Dead { time } => {
                if (ggez::timer::time_since_start(ctx) - time) > RESTART_AFTER {
                    self.restart()
                }
            }
            _ => {}
        }
    }

    /// Creates a new GameState
    /// Panics if can't access the sprite image resource.
    pub fn new(ctx: &mut Context, sprite_batch: SpriteBatch) -> Self {
        let mut pipe_tracker = pipe::PipeTracker::new();
        let atlas =
            atlas::Atlas::parse_atlas_json(std::path::Path::new("resources/texture_atlas.json"));
        let sound_player = Player::new(ctx);

        Self {
            tiles_drawn: false,
            pipes: GameState::create_start_entities(&atlas, &mut pipe_tracker),
            player: create_player(&atlas),
            tiles : create_tiles(&atlas),
            sprite_batch,
            pipe_tracker,
            play_state: PlayState::StartScreen,
            atlas,
            score: 0,
            best_score: 0,
            sound_player
        }
    }

    /// The last entity *must* be the player.
    pub fn create_start_entities(
        sprites: &atlas::Atlas,
        pipe_tracker: &mut PipeTracker,
    ) -> Vec<Box<PipeEntity>> {
        let pipes = create_pipes(
            sprites.create_sprite("pipe_bottom.png"),
            sprites.create_sprite("pipe_top.png"),
            pipe_tracker,
            200.0,
        );
        pipes
    }

    pub fn restart(&mut self) {
        self.sound_player.begin();
        let mut pt = PipeTracker::new();
        self.pipes = GameState::create_start_entities(&self.atlas, &mut pt);
        self.player = create_player(&self.atlas);
        self.pipe_tracker = pt;
        self.play_state = PlayState::StartScreen;
        self.swap_scores();
        self.score = 0;
    }

    pub fn swap_scores(&mut self) {
        if self.score > self.best_score {
            self.best_score = self.score;
        }
    }
}

