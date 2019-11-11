use crate::entity::Entity;
use crate::Sprite;
use ggez::nalgebra::{Point2, Vector2};
use noise::NoiseFn;
use noise::Perlin;
use std::collections::VecDeque;

const NUM_PIPES: usize = 4;
const SEGMENTS: usize = 4;
/// Count total segments. The pipe lengths and tops.
pub const TOTAL: usize = (SEGMENTS * 2) + 2;

const PIPE_SPEED: f32 = 1.0;
/// Distance between pipes relative to their width.
const SPACE_MULTIPLIER: f32 = 1.5;

const VERTICAL_GAP: f32 = 57.0;
pub const VERTICAL_GAP_DEVIANCE: f32 = 0.6;

#[derive(Debug)]
pub struct PipeTracker {
    pipes_seen: usize,
    top: VecDeque<f32>,
    time: f32,
    random_fn: Perlin,
}

impl PipeTracker {
    pub fn new() -> Self {
        PipeTracker {
            pipes_seen: 1,
            top: VecDeque::new(),
            time: 0.0,
            random_fn: Perlin::new(),
        }
    }

    fn get_pipe_top(&mut self) -> f32 {
        self.pipes_seen += 1;
        let noise = (self.random_fn.get([self.time as f64, self.time as f64]) as f32 + 1.0);
        (VERTICAL_GAP + 5.0) + noise * ((600.0 / 4.0) - (VERTICAL_GAP * 2.0))
    }

    fn init_get_pipe_top(&mut self) -> f32 {
        self.time += VERTICAL_GAP_DEVIANCE;
        let result = self.get_pipe_top();
        self.pipes_seen = 0;
        self.top.push_back(result);
        result
    }

    // Returns the direction pipe has to move.
    pub fn get_pipe_difference(&mut self) -> f32 {
        // need to go back by the number of integers of other pipes.

        let last_pos = self.top.front().expect("Pipe wasn't placed!").clone();
        let now_pos = self.get_pipe_top();

        if (self.pipes_seen == 10) {
            self.top.pop_front();
            self.top.push_back(now_pos);
            self.pipes_seen = 0;
            self.time += VERTICAL_GAP_DEVIANCE;
        }
        now_pos - last_pos
    }
}

fn create_pipe_bottom(
    sprite_base: Sprite,
    sprite_top: Sprite,
    x: f32,
    top: f32,
    total_dist: f32,
) -> Vec<Entity> {
    let top_height = sprite_top.height;

    let mut pipe_tip = create_pipe_tip(sprite_top, x, top).scroller(total_dist);
    let segments = SEGMENTS;
    let mut p = (0..segments)
        .into_iter()
        .map(|i| make_pipe_body(&sprite_base, x, top, total_dist, top_height, 1.0 *(i as f32)))
        .collect::<Vec<Entity>>();

    p.push(pipe_tip);
    p
}

pub fn create_pipes(
    sprite_base: Sprite,
    sprite_top: Sprite,
    pipe_tracker: &mut PipeTracker,
    x: f32,
) -> Vec<Entity> {
    let number_of_pipes = NUM_PIPES;
    let width = sprite_top.width;
    let space_width = width * SPACE_MULTIPLIER;
    let total_dist = (width + space_width) * (number_of_pipes as f32);

    let gap = VERTICAL_GAP;
    (0..number_of_pipes)
        .into_iter()
        .flat_map(|i| {
            let top = pipe_tracker.init_get_pipe_top();
            let mut bottom = create_pipe_bottom(
                sprite_base.clone(),
                sprite_top.clone(),
                x + (space_width + width) * (i as f32),
                top,
                total_dist,
            );
            bottom.extend(create_pipe_top(
                sprite_base.clone(),
                sprite_top.clone(),
                x + (space_width + width) * (i as f32),
                top - gap,
                total_dist,
            ));
            bottom
        })
        .collect()
}

fn create_pipe_top(
    sprite_base: Sprite,
    sprite_top: Sprite,
    x: f32,
    top: f32,
    total_dist: f32,
) -> Vec<Entity> {
    use crate::entity::ScoringPipe;
    let top_height = -1.0 * sprite_top.height;
    let mut sprite_top = sprite_top;
    sprite_top.scale.y = -1.0;

    let mut pipe_tip = create_pipe_tip(sprite_top, x, top).scroller(total_dist);
    pipe_tip.scoring_pipe = Some(ScoringPipe::ReadyToScore);

    let segments = SEGMENTS;
    let mut p = (0..segments)
        .into_iter()
        .map(|i|
            make_pipe_body(&sprite_base, x, top, total_dist, top_height, -1.0 *(i as f32))
       )
        .collect::<Vec<Entity>>();

    p.push(pipe_tip);
    p
}

fn pipe_velocity() -> Vector2<f32> {
    ggez::nalgebra::Vector2::new(-PIPE_SPEED, 0.0)
}

fn make_pipe_body(sprite_base: &Sprite, x: f32, top: f32, total_dist: f32, top_height: f32, i: f32) -> Entity {
    let mut pipe_body = Entity::new().add_physics(false);
    pipe_body.sprite = Some(sprite_base.clone());
    pipe_body.is_pipe = true;
    pipe_body.position =
        Point2::new(x, top + top_height + (sprite_base.height * i));
    pipe_body
        .scroller(total_dist)
        .set_velocity(pipe_velocity())
}

fn create_pipe_tip(sprite_top: Sprite, x: f32, top: f32) -> Entity {
    let mut pipe_tip = Entity::new().add_physics(false);
    pipe_tip.sprite = Some(sprite_top);
    pipe_tip.position = Point2::new(x, top);
    pipe_tip.is_pipe = true;
    pipe_tip.set_velocity(pipe_velocity())
}
