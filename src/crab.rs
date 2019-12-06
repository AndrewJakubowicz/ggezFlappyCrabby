use crate::{atlas};
use crate::entity::{PlayState, PipeEntity};
use crate::atlas::Sprite;
use ggez::nalgebra::{Point2, Vector2};
use ggez::{Context, graphics, GameResult};
use ggez::graphics::spritebatch::SpriteBatch;

const GRAVITY: f32 = 0.25;
const JUMP_IMPULSE: f32 = 2.75;
pub const SCREEN_TOP: f32 = -16.0;

/// The physics on the entity.
pub struct Physics {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
    pub gravity: bool,
}

impl Physics {
    pub fn new (with_gravity: bool) -> Self {
        Self {
            velocity: Vector2::new(0.0, 0.0),
            acceleration: Vector2::new(0.0, 0.0),
            gravity: with_gravity,
        }
    }
}

pub fn create_player(sprites: &atlas::Atlas) -> Box<PlayerEntity> {
    let crab0 = sprites.create_sprite("crab0.png");
    let sprite = crab0.clone();
    let crab1 = sprites.create_sprite("crab1.png");
    let player_sprites = vec![crab0, crab1];
    let player = PlayerEntity::new(sprite, (40.0, SCREEN_TOP), player_sprites);

    Box::new(player)
}
pub struct PlayerEntity {
    pub sprite: Sprite,
    pub position: Point2<f32>,
    pub player_sprites: Vec<Sprite>,
    can_jump: bool,
    pub physics: Physics,
}

impl PlayerEntity {
    pub fn update(
        &mut self,
        ctx: &mut Context,
        state: &PlayState,
    ) -> PlayState {
        let physics = &mut self.physics;
        physics.acceleration = if physics.gravity {
            Vector2::new(0.0, GRAVITY)
        } else {
            Vector2::new(0.0, 0.0)
        };


        let mut state = state.clone();
        if state.is_not_dead()
        {
            use ggez::event::KeyCode;
            use ggez::input::keyboard;
            if !keyboard::pressed_keys(ctx).contains(&KeyCode::Space) && !self.can_jump {
                self.can_jump = true;
            }

            if keyboard::is_key_pressed(ctx, KeyCode::Space) && self.can_jump {
                let physics = &mut self.physics;
                PlayerEntity::jump(physics);

                // exit start screen state.
                if state == PlayState::StartScreen {
                    state = PlayState::Play;
                }
            }
        }

        // Self jumping script on the start screen.
        if state == PlayState::StartScreen {
            self.auto_jump()
        }

        self.change_player_position(ctx);

        // Crab should not go above the top of the screen easily.
        self.prevent_going_off();
        state
    }

    fn change_player_position(&mut self, ctx: &mut Context) {
        let delta = ggez::timer::delta(ctx).as_nanos() as f32;
        let physics = &mut self.physics;
        physics.acceleration.scale(1.0 / delta);
        physics.velocity += physics.acceleration;
        physics.velocity.scale(1.0 / delta);
        // moves all the entities on the board.
        self.position += physics.velocity;
    }

    pub fn new(sprite: Sprite, position: (f32, f32), player_sprites: Vec<Sprite>) -> Self {
        Self {
            sprite,
            position: Point2::new(position.0, position.1),
            physics: Physics::new(true),
            can_jump: true,
            player_sprites,
        }
    }
    pub fn overlaps(&self, other : &PipeEntity) -> bool {
        let player_rect = self.get_bounds();
        let other_rect = other.get_rect();

        other_rect.overlaps(&player_rect)
    }
    pub fn get_bounds(&self) -> graphics::Rect {
        let mut rect = self.sprite.get_bound_box();
        rect.move_to(self.position.clone());

        rect
    }
    fn prevent_going_off(&mut self) -> () {
        self.position.y = if self.position.y < SCREEN_TOP {
            SCREEN_TOP
        } else {
            self.position.y
        }
    }
    fn auto_jump(&mut self) -> () {
        let physics = &mut self.physics;
        if self.position.y > 600.0 / 8.0 {
            PlayerEntity::jump(physics);
        }
    }
    pub fn draw(&mut self, batch: &mut SpriteBatch) -> GameResult {
        self.draw_player(batch);

        Ok(())
    }
    fn draw_player(&mut self, batch: &mut SpriteBatch) {
        let s = &mut self.player_sprites;
        let p = &self.physics;
        // need velocity to map to these rotations between -0.2 and 0.2!
        let angle = rescale_range(p.velocity.y, -7.0, 7.0, -0.6, 0.6);
        let x = if p.velocity.y >= 0.0 {
            &mut s[1]
        } else {
            &mut s[0]
        };
        batch.add(
            x.add_draw_param(self.position.clone())
                .offset(Point2::new(0.5, 0.5))
                .rotation(angle),
        );
    }

    fn jump(physics: &mut Physics) {
        physics.acceleration = Vector2::new(0.0, -GRAVITY);
        physics.velocity = Vector2::new(0.0, -JUMP_IMPULSE);
    }
}


/// Returns an f32 scaled [oldMin, oldMax] into the range [newMin, newMax]
/// Thanks https://stackoverflow.com/a/5295202/6421793
fn rescale_range(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    use ggez::nalgebra::clamp;
    let old_range = old_max - old_min;
    let new_range = new_max - new_min;
    (((clamp(value, old_min, old_max) - old_min) * new_range) / old_range) + new_min
}
