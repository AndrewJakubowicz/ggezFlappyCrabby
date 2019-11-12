use ggez::{
    audio::Source,
    Context
};
use ggez::audio::SoundSource;

pub struct Player {
    pub score_sound: ggez::audio::Source,
    pub ouch_sound: ggez::audio::Source,
    pub begin_sound: ggez::audio::Source,
}

impl Player {
    pub fn new(ctx: &mut Context) -> Self {
        let score_sound = ggez::audio::Source::new(ctx, "/score_point.wav").unwrap();
        let ouch_sound = ggez::audio::Source::new(ctx, "/ouch.wav").unwrap();
        let mut begin_sound =ggez::audio::Source::new(ctx, "/begin_game.wav").unwrap();

        Self {
            score_sound,
            ouch_sound,
            begin_sound,
        }
    }

    pub fn begin(&mut self){
        self.begin_sound.play_detached();
    }

    pub fn ouch(&mut self){
        self.ouch_sound.play_detached();
    }
    pub fn score(&mut self){
        self.score_sound.play_detached();
    }
}