use crate::atlas::Sprite;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::{Point2};
use crate::{NUMBER_OF_TILES, atlas};

pub struct TileEntity {
    pub sprite: Sprite,
    pub position: Point2<f32>,
}

impl TileEntity {
    pub fn draw(&mut self, batch: &mut SpriteBatch) {
        let s = &mut self.sprite;
        batch.add(s.add_draw_param(self.position.clone()));
    }

    pub fn new(sprite: Sprite, position: (f32, f32)) -> Self {
        Self {
            sprite,
            position: Point2::new(position.0, position.1),
        }
    }
}

fn create_tile_scroll(sprite: Sprite, x: f32) -> Box<TileEntity> {
    let tile = TileEntity::new(sprite, (x, 145.0));
    // floor tiles do not need to move... do they ?!
    // let tile = tile.scroller(jump).set_velocity((-1.0, 0.0));

    Box::new(tile)
}

pub fn create_tiles(sprites: &atlas::Atlas) -> Vec<Box<TileEntity>> {
    let floor_tile = sprites.create_sprite("floor_tile.png");
    let width = floor_tile.width;

    (0..NUMBER_OF_TILES)
        .into_iter()
        .map(|i| create_tile_scroll(floor_tile.clone(), (i as f32) * width))
        .collect()
}