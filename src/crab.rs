use ggez::nalgebra::Point2;
use crate::{atlas, entity};
use crate::entity::{Entity, GameEntity};

pub fn create_player(sprites: &atlas::Atlas) -> Box<Entity> {
    let crab0 = sprites.create_sprite("crab0.png");
    let sprite = crab0.clone();
    let crab1 = sprites.create_sprite("crab1.png");
    let player_sprites = vec![crab0, crab1];
    let mut player = entity::Entity::new(true, sprite, 40.0, entity::SCREEN_TOP);
    player.is_player = true;
    player.player_sprites = Some(player_sprites);

    Box::new(player)
}
