use raylib::prelude::*;

use crate::player::Player;

pub mod player;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Greybox Demo")
        .vsync()
        .build();

    rl.set_target_fps(120);

    let mut player = Player::new(Vector2 { x: 100.0, y: 100.0 }, 100.0);

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        let player_target = if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            Some(rl.get_mouse_position())
        } else {
            None
        };

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        player.update(dt, player_target);
        player.draw(&mut d);
    }
}
