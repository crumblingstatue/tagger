extern crate sfml;

use self::sfml::graphics::*;
use self::sfml::window::*;

pub fn run() {
    let mut window = RenderWindow::new((1024, 768), "Tagger", Style::CLOSE, &Default::default());
    window.set_framerate_limit(60);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if let Event::Closed = event {
                window.close();
            }
        }
        window.clear(&Color::BLACK);
        window.display();
    }
}
