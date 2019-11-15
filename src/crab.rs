use atlas::Sprite;
use ggez::nalgebra::{Point2, Vector2};
use rand::distributions::OpenClosed01;
use rand::{thread_rng, Rng};
use entity::{PlayState, ScoringPipe};
use ggez::graphics::Rect;
use std::time::Duration;
use crate::{atlas, entity};
use crate::entity::Entity;

pub fn create_player(sprites: &atlas::Atlas) -> Entity {
    let crab0 = sprites.create_sprite("crab0.png");
    let sprite = crab0.clone();
    let crab1 = sprites.create_sprite("crab1.png");
    let player_sprites = vec![crab0, crab1];
    let mut player = entity::Entity::new().add_physics(true);
    player.sprite = Some(sprite);
    player.is_player = true;
    player.position = Point2::new(40.0, entity::SCREEN_TOP);
    player.player_sprites = Some(player_sprites);
    player
}
