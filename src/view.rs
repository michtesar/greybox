use raylib::ffi::Vector2;

/*
* AffineTransformationView
*
* Similary transformation view for scene to screen
* and screen to scene. It is asymetrical transformation
* as unprejected input can go out of the viewport, which
* is considered as invalid location (altough the algoritm)
* can deliver the coordinates but in negative space.
*
* We take use virtual resolution to fit the view port of the
* scene space to the screen space by fit (object-fit in CSS)
* where we can have at least one bars on the sides (pillarbox)
* or in vertical order (please fix this I forgot the name for it).
*/
#[derive(Debug)]
struct View {
    scale: f32,
    offset: Vector2,
}

impl View {
    pub fn new(window_resolution: Vector2) -> Self {
        // TODO: I am not sure whether this should come from elsewhere.
        let scene_resolution = Vector2 { x: 640.0, y: 480.0 };

        // Calculate the scale factors
        let x = window_resolution.x / scene_resolution.x;
        let y = window_resolution.y / scene_resolution.y;
        let scale = x.min(y);
        let offset = Vector2 {
            x: (window_resolution.x - (scene_resolution.x * scale)) / 2.0,
            y: (window_resolution.y - (scene_resolution.y * scale)) / 2.0,
        };
        View { scale, offset }
    }
}
