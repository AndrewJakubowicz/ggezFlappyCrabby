use crate::entity::Entity;
use crate::Sprite;
use ggez::nalgebra::{Point2, Vector2};
use std::collections::VecDeque;

const NUM_PIPES: u8 = 4;
const SEGMENTS: u8 = 4;
/// Count total segments. The pipe lengths and tops.
pub const TOTAL: u8 = SEGMENTS * 2 + 2;

const GAP: f32 = 45.0;
pub const PIPE_DV: f32 = 0.1;

pub fn pipe_position(t: f32) -> f32 {
    GAP + 10.0 + t.sin() * ((600.0 / 4.0) - GAP - 10.0)
}

fn create_pipe_bottom(
    sprite_base: Sprite,
    sprite_top: Sprite,
    x: f32,
    top: f32,
    total_dist: f32,
) -> Vec<Entity> {
    let top_height = sprite_top.height;

    let mut pipe_top = Entity::new().add_physics(false);
    pipe_top.sprite = Some(sprite_top);
    pipe_top.position = Point2::new(x, top);
    pipe_top.is_pipe = true;
    let pipe_top = pipe_top
        .scroller(total_dist)
        .set_velocity(ggez::nalgebra::Vector2::new(-0.7, 0.0));

    let segments = SEGMENTS;
    let mut p = (0..segments)
        .into_iter()
        .map(|i| {
            let mut pipe_bottom = Entity::new().add_physics(false);
            pipe_bottom.is_pipe = true;
            pipe_bottom.sprite = Some(sprite_base.clone());
            pipe_bottom.position =
                Point2::new(x, top + top_height + (sprite_base.height * (i as f32)));
            pipe_bottom
                .scroller(total_dist)
                .set_velocity(ggez::nalgebra::Vector2::new(-0.7, 0.0))
        })
        .collect::<Vec<Entity>>();
    p.push(pipe_top);
    p
}

pub fn create_pipes(
    sprite_base: Sprite,
    sprite_top: Sprite,
    x: f32,
) -> (Vec<Entity>, VecDeque<f32>) {
    let number_of_pipes = NUM_PIPES;
    let width = sprite_top.width;
    let space_width = width * 1.5;
    let total_dist = (width + space_width) * (number_of_pipes as f32);
    let mut dequeue = VecDeque::with_capacity(5);

    for i in 0..number_of_pipes {
        dequeue.push_back(i as f32 * PIPE_DV);
    }

    let gap = GAP;
    (
        (0..number_of_pipes)
            .into_iter()
            .flat_map(|i| {
                let top = pipe_position(*dequeue.get(i as usize).unwrap_or(&0.0));
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
            .collect(),
        dequeue,
    )
}

fn create_pipe_top(
    sprite_base: Sprite,
    sprite_top: Sprite,
    x: f32,
    top: f32,
    total_dist: f32,
) -> Vec<Entity> {
    let top_height = sprite_top.height;

    let mut pipe_top = Entity::new().add_physics(false);
    let mut sp_top = sprite_top;
    sp_top.scale.y = -1.0;
    pipe_top.sprite = Some(sp_top);
    pipe_top.position = Point2::new(x, top);
    pipe_top.is_pipe = true;
    let pipe_top = pipe_top
        .scroller(total_dist)
        .set_velocity(ggez::nalgebra::Vector2::new(-0.7, 0.0));

    let segments = SEGMENTS;
    let mut p = (0..segments)
        .into_iter()
        .map(|i| {
            let mut pipe_bottom = Entity::new().add_physics(false);
            pipe_bottom.sprite = Some(sprite_base.clone());
            pipe_bottom.is_pipe = true;
            pipe_bottom.position =
                Point2::new(x, top - top_height - (sprite_base.height * (i as f32)));
            pipe_bottom
                .scroller(total_dist)
                .set_velocity(ggez::nalgebra::Vector2::new(-0.7, 0.0))
        })
        .collect::<Vec<Entity>>();
    p.push(pipe_top);
    p
}
