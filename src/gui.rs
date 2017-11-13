extern crate sfml;

use self::sfml::graphics::*;
use self::sfml::window::*;
use self::sfml::system::*;

struct Settings {
    frames_per_row: u32,
    frame_gap: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            frames_per_row: 5,
            frame_gap: 4,
        }
    }
}

fn draw_frames(settings: &Settings, target: &mut RenderWindow) {
    let Vector2u { x: tw, y: th } = target.size();
    let frame_size = (tw - settings.frame_gap) / settings.frames_per_row;
    let mut shape = RectangleShape::with_size(Vector2f::new(frame_size as f32, frame_size as f32));
    for i in 0..settings.frames_per_row {
        let x = (i * (frame_size + settings.frame_gap)) as f32;
        shape.set_position((x, 0.));
        target.draw(&shape);
    }
}

pub fn run() {
    let mut window = RenderWindow::new(
        VideoMode::desktop_mode(),
        "Tagger",
        Style::NONE,
        &Default::default(),
    );
    window.set_framerate_limit(60);

    let settings = Settings::default();

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if let Event::Closed = event {
                window.close();
            }
        }
        window.clear(&Color::BLACK);
        draw_frames(&settings, &mut window);
        window.display();
    }
}
