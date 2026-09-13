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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_idle_at_given_position() {
        let player = Player::new(Vector2::new(10.0, 20.0), 100.0);

        assert_eq!(player.position, Vector2::new(10.0, 20.0));
        assert_eq!(player.state, PlayerState::Idle);
    }

    #[test]
    fn update_without_click_stays_idle_in_place() {
        let mut player = Player::new(Vector2::new(10.0, 20.0), 100.0);

        player.update(1.0, None);

        assert_eq!(player.position, Vector2::new(10.0, 20.0));
        assert_eq!(player.state, PlayerState::Idle);
    }

    #[test]
    fn click_starts_walking_towards_target() {
        let mut player = Player::new(Vector2::new(0.0, 0.0), 100.0);

        player.update(0.0, Some(Vector2::new(100.0, 0.0)));

        assert_eq!(
            player.state,
            PlayerState::Walking {
                target: Vector2::new(100.0, 0.0)
            }
        );
    }

    #[test]
    fn walking_moves_towards_target_by_speed_times_dt() {
        let mut player = Player::new(Vector2::new(0.0, 0.0), 100.0);
        player.update(0.0, Some(Vector2::new(100.0, 0.0)));

        player.update(0.1, None);

        assert_eq!(player.position, Vector2::new(10.0, 0.0));
        assert_eq!(
            player.state,
            PlayerState::Walking {
                target: Vector2::new(100.0, 0.0)
            }
        );
    }

    #[test]
    fn reaching_target_switches_back_to_idle_without_overshoot() {
        let mut player = Player::new(Vector2::new(0.0, 0.0), 100.0);
        player.update(0.0, Some(Vector2::new(50.0, 0.0)));

        // A big dt would overshoot the target if not clamped.
        player.update(1.0, None);

        assert_eq!(player.position, Vector2::new(50.0, 0.0));
        assert_eq!(player.state, PlayerState::Idle);
    }

    #[test]
    fn click_while_walking_retargets_immediately() {
        let mut player = Player::new(Vector2::new(0.0, 0.0), 100.0);
        player.update(0.0, Some(Vector2::new(100.0, 0.0)));

        player.update(0.0, Some(Vector2::new(0.0, 50.0)));

        assert_eq!(
            player.state,
            PlayerState::Walking {
                target: Vector2::new(0.0, 50.0)
            }
        );
    }
}
