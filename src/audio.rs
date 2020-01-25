use ggez::{
    audio::Source,
    Context
};
use ggez::audio::SoundSource;
use rand::distributions::OpenClosed01;
use rand::{thread_rng, Rng};


pub struct Player {
    pub score_sound: Source,
    pub ouch_sound: Source,
    pub begin_sound: Source,
}

impl Player {
    pub fn new(ctx: &mut Context) -> Self {
        let score_sound =  Source::new(ctx, "/score_point.wav").unwrap();
        let ouch_sound =  Source::new(ctx, "/ouch.wav").unwrap();
        let begin_sound = Source::new(ctx, "/begin_game.wav").unwrap();

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

    pub fn score(&mut self) {
        let pitch: f32 = thread_rng().sample(OpenClosed01);
        self.score_sound.set_pitch(1.0 + pitch);

        self.score_sound.play_detached();
    }
}