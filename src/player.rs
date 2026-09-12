use raylib::{ffi::Rectangle, prelude::*};

pub struct Player {
    position: Vector2,
    speed: f32,
    state: PlayerState,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum PlayerState {
    Idle,
    Walking { target: Vector2 },
}

impl Player {
    pub fn new(position: Vector2, speed: f32) -> Self {
        Player {
            position,
            speed,
            state: PlayerState::Idle,
        }
    }

    pub fn update(&mut self, dt: f32, click: Option<Vector2>) {
        if let Some(target) = click {
            self.state = PlayerState::Walking { target }
        }

        match self.state {
            PlayerState::Idle => {} // nothing to do waiting for the input
            PlayerState::Walking { target } => {
                self.position = self.position.move_towards(target, self.speed * dt);
                if self.position == target {
                    self.state = PlayerState::Idle;
                }
            }
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle<'_>) {
        let height = 80.0;
        let width = 40.0;

        let player_rect = Rectangle::new(self.position.x, self.position.y, width, height);
        let player_origin = Vector2::new(width / 2.0, height);
        let player_rotation = 0.0;
        let player_color = Color::GREEN;

        d.draw_rectangle_pro(player_rect, player_origin, player_rotation, player_color);
    }
}
