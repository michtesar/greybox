use raylib::prelude::*;

use greybox::player::Player;
use greybox::view::View;

fn main() {
    let scene_resolution = Vector2 { x: 800.0, y: 600.0 };
    let (mut rl, thread) = raylib::init()
        .size(scene_resolution.x as i32, scene_resolution.y as i32)
        .title("Greybox Demo")
        .vsync()
        .resizable()
        .build();

    rl.set_target_fps(120);

    let mut player = Player::new(Vector2 { x: 100.0, y: 100.0 }, 100.0);

    while !rl.window_should_close() {
        let view = View::new(
            Vector2 {
                x: rl.get_screen_width() as f32,
                y: rl.get_screen_height() as f32,
            },
            scene_resolution,
        );
        let dt = rl.get_frame_time();

        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let scene_point = view.screen_to_scene(rl.get_mouse_position());
        let player_target = if clicked { scene_point } else { None };

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        player.update(dt, player_target);
        player.draw(&mut d);
    }
}
