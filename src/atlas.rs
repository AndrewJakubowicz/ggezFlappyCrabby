use ggez::graphics::{self, spritebatch::SpriteBatch};
use ggez::nalgebra::{Point2, Vector2};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub type RcSpriteBatch = Rc<RefCell<graphics::spritebatch::SpriteBatch>>;

#[derive(Deserialize, Debug)]
struct Meta {
    size: AtlasSize,
}

#[derive(Deserialize, Debug)]
struct AtlasSize {
    w: i32,
    h: i32,
}

#[derive(Deserialize, Debug, Clone)]
struct JsonRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Deserialize, Debug, Clone)]
struct SpriteData {
    filename: String,
    frame: JsonRect,
}

#[derive(Deserialize, Debug)]
pub struct Atlas {
    frames: Vec<SpriteData>,
    meta: Meta,
}

impl Atlas {
    pub fn parse_atlas_json(texture_atlas_file: &Path) -> Self {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(texture_atlas_file).expect("Couldn't find the texture_atlas file");
        let mut buf_reader = std::io::BufReader::new(file);
        serde_json::from_reader(buf_reader).expect("Couldn't create texture atlas")
    }

    /// Returns a sprite from the Atlas.
    pub fn create_sprite(&self, sprite_name: &str) -> Sprite {
        let width = self.meta.size.w as f32;
        let height = self.meta.size.h as f32;
        let atlas_rect = graphics::Rect::new(0.0, 0.0, width, height);

        if let Some(sprite_data) = self.frames.iter().find(|d| d.filename == sprite_name) {
            Sprite::new(graphics::Rect::fraction(
                sprite_data.frame.x as f32,
                sprite_data.frame.y as f32,
                sprite_data.frame.w as f32,
                sprite_data.frame.h as f32,
                &atlas_rect,
            ))
        } else {
            unimplemented!("Not handling failure to find sprite");
        }
    }
}

pub struct Sprite {
    /// The square that we want to cut out of the texture atlas.
    rect: graphics::Rect,
}

impl Sprite {
    pub fn new(rect: graphics::Rect) -> Self {
        Self { rect }
    }

    /// Adds a draw command to the sprite batch.
    pub fn add_draw_param(&mut self, pos: Point2<f32>, sprite_batch: &mut SpriteBatch) {
        let p = graphics::DrawParam::new()
            .src(self.rect.clone())
            .scale(Vector2::new(4.0, 4.0))
            .dest(pos);

        sprite_batch.add(p);
    }
}
