use crate::{atlas, entity};
use crate::entity::PlayerEntity;

pub fn create_player2(sprites: &atlas::Atlas) -> Box<PlayerEntity> {
    let crab0 = sprites.create_sprite("crab0.png");
    let sprite = crab0.clone();
    let crab1 = sprites.create_sprite("crab1.png");
    let player_sprites = vec![crab0, crab1];
    let player = entity::PlayerEntity::new(sprite, (40.0, entity::SCREEN_TOP), player_sprites);

    Box::new(player)
}
