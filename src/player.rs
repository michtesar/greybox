use raylib::prelude::*;

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
        todo!()
    }

    pub fn update(&mut self, dt: f32, click: Option<Vector2>) {
        todo!()
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle<'_>) {
        todo!()
    }
}
