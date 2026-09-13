use raylib::ffi::Vector2;

/*
* AffineTransformationView
*
* Transforms coordinates between scene space and screen space.
*
* We use a virtual scene resolution and fit its viewport into the
* window (like `object-fit: contain` in CSS), preserving the scene's
* aspect ratio. Depending on the window's aspect ratio, this leaves
* bars on the sides (pillarbox) or on the top/bottom (letterbox).
*
* The transformation is asymmetrical: `scene_to_screen` always
* produces a valid screen coordinate, but `screen_to_scene` can be
* given a screen point that lands inside a pillarbox/letterbox bar.
* Such a point maps outside `scene_resolution`, so `screen_to_scene`
* validates the result and returns `None` instead of the raw
* out-of-bounds coordinate.
*/
#[derive(Debug)]
struct View {
    scale: f32,
    offset: Vector2,
    scene_resolution: Vector2,
}

impl View {
    pub fn new(window_resolution: Vector2, scene_resolution: Vector2) -> Self {
        // Calculate the scale factors
        let scale_x = window_resolution.x / scene_resolution.x;
        let scale_y = window_resolution.y / scene_resolution.y;
        let scale = scale_x.min(scale_y);
        let offset = Vector2 {
            x: (window_resolution.x - (scene_resolution.x * scale)) / 2.0,
            y: (window_resolution.y - (scene_resolution.y * scale)) / 2.0,
        };
        View {
            scale,
            offset,
            scene_resolution,
        }
    }

    pub fn scene_to_screen(&self, point: Vector2) -> Vector2 {
        Vector2 {
            x: (point.x * self.scale) + self.offset.x,
            y: (point.y * self.scale) + self.offset.y,
        }
    }

    pub fn screen_to_scene(&self, point: Vector2) -> Option<Vector2> {
        let point = Vector2 {
            x: (point.x - self.offset.x) / self.scale,
            y: (point.y - self.offset.y) / self.scale,
        };
        if point.x < 0.0 || point.x > self.scene_resolution.x {
            return None;
        }
        if point.y < 0.0 || point.y > self.scene_resolution.y {
            return None;
        }
        Some(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vector2_approx_eq(a: Vector2, b: Vector2) {
        const EPSILON: f32 = 1e-4;
        assert!(
            (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON,
            "expected {a:?} to be approximately equal to {b:?}"
        );
    }

    #[test]
    fn scene_to_screen_round_trip_no_pillarbox() {
        // Window is exactly 2x the scene resolution, so there is no
        // pillarbox/letterbox and scale/offset are trivial.
        let view = View::new(
            Vector2 {
                x: 1280.0,
                y: 960.0,
            },
            Vector2 { x: 640.0, y: 480.0 },
        );
        let scene_point = Vector2 { x: 320.0, y: 240.0 };

        let screen_point = view.scene_to_screen(scene_point);
        let round_tripped = view.screen_to_scene(screen_point);

        assert_eq!(round_tripped, Some(scene_point));
    }

    #[test]
    fn scene_to_screen_round_trip_with_pillarbox() {
        // Window is wider than the scene's aspect ratio, so we expect
        // bars on the left/right (pillarbox).
        let view = View::new(
            Vector2 { x: 800.0, y: 480.0 },
            Vector2 { x: 640.0, y: 480.0 },
        );
        let scene_point = Vector2 { x: 100.0, y: 400.0 };

        let screen_point = view.scene_to_screen(scene_point);
        let round_tripped = view.screen_to_scene(screen_point);

        assert_vector2_approx_eq(
            round_tripped.expect("point should be in bounds"),
            scene_point,
        );
    }

    #[test]
    fn scene_to_screen_round_trip_with_letterbox() {
        // Window is taller than the scene's aspect ratio, so we expect
        // bars on the top/bottom (letterbox).
        let view = View::new(
            Vector2 { x: 640.0, y: 960.0 },
            Vector2 { x: 640.0, y: 480.0 },
        );
        let scene_point = Vector2 { x: 500.0, y: 50.0 };

        let screen_point = view.scene_to_screen(scene_point);
        let round_tripped = view.screen_to_scene(screen_point);

        assert_vector2_approx_eq(
            round_tripped.expect("point should be in bounds"),
            scene_point,
        );
    }

    #[test]
    fn screen_to_scene_rejects_point_inside_pillarbox() {
        let view = View::new(
            Vector2 { x: 800.0, y: 480.0 },
            Vector2 { x: 640.0, y: 480.0 },
        );

        // x = 0 falls inside the left pillarbox bar, well before the
        // scene's viewport starts.
        let point_in_bar = Vector2 { x: 0.0, y: 240.0 };

        assert_eq!(view.screen_to_scene(point_in_bar), None);
    }

    #[test]
    fn screen_to_scene_accepts_scene_corners() {
        let view = View::new(
            Vector2 { x: 800.0, y: 480.0 },
            Vector2 { x: 640.0, y: 480.0 },
        );

        let top_left = view.scene_to_screen(Vector2 { x: 0.0, y: 0.0 });
        let bottom_right = view.scene_to_screen(Vector2 { x: 640.0, y: 480.0 });

        assert!(view.screen_to_scene(top_left).is_some());
        assert!(view.screen_to_scene(bottom_right).is_some());
    }
}
