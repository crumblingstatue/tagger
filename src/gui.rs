extern crate sfml;

use self::sfml::graphics::*;
use self::sfml::window::*;
use self::sfml::system::*;
use tagger_map::TaggerMap;
use infix;

struct State {
    frames_per_row: u32,
    frame_gap: u32,
    y_offset: f32,
    font: Font,
}

impl Default for State {
    fn default() -> Self {
        Self {
            frames_per_row: 5,
            frame_gap: 4,
            y_offset: 0.0,
            font: Font::from_memory(include_bytes!("../Vera.ttf")).unwrap(),
        }
    }
}

fn draw_frames<'a, I: IntoIterator<Item = &'a Frame>>(
    state: &State,
    frames: I,
    target: &mut RenderWindow,
) {
    let Vector2u { x: tw, y: th } = target.size();
    let frame_size = (tw - state.frame_gap) / state.frames_per_row;
    let mut frames_per_column = th / frame_size;
    // Compensate for truncating division
    if th % frame_size != 0 {
        frames_per_column += 1;
    }
    // Since we can scroll, we can have another partially drawn frame per screen
    frames_per_column += 1;
    let frames_per_screen = (state.frames_per_row * frames_per_column) as usize;
    let row_offset = state.y_offset as u32 / frame_size;
    let skip = row_offset * state.frames_per_row;
    let frames = frames
        .into_iter()
        .skip(skip as usize)
        .take(frames_per_screen);
    let mut shape = RectangleShape::with_size(Vector2f::new(frame_size as f32, frame_size as f32));
    for (i, frame) in frames.enumerate() {
        let i = i as u32;
        let column = i % state.frames_per_row;
        let row = i / state.frames_per_row;
        let x = (column * (frame_size + state.frame_gap)) as f32;
        let y =
            (row * (frame_size + state.frame_gap)) as f32 - (state.y_offset % frame_size as f32);
        shape.set_position((x, y));
        target.draw(&shape);
        let mut text = Text::new(
            &format!("{}\n{}", frame.debug_n, frame.name),
            &state.font,
            8,
        );
        text.set_position((x, y));
        text.set_fill_color(&Color::BLACK);
        target.draw(&text);
    }
}

// Frame containing image and other data
struct Frame {
    name: String,
    tags: Vec<String>,
    debug_n: usize,
}

fn construct_frameset(tagger_map: &TaggerMap, rule: &str) -> Result<Vec<Frame>, infix::ParseError> {
    let rules = infix::parse_infix(rule)?;
    let entries = tagger_map.tag_map.matching_entries(&rules);
    let mut frameset = Vec::new();
    for (i, (name, tags)) in entries.enumerate() {
        frameset.push(Frame {
            name: name.clone(),
            tags: tags.to_owned(),
            debug_n: i,
        });
    }
    Ok(frameset)
}

pub fn run(tagger_map: &mut TaggerMap) {
    let mut window = RenderWindow::new(
        VideoMode::desktop_mode(),
        "Tagger",
        Style::NONE,
        &Default::default(),
    );
    window.set_framerate_limit(60);

    let mut state = State::default();
    let frameset = construct_frameset(tagger_map, "").unwrap();

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed { code, .. } => if code == Key::PageDown {
                    state.y_offset += window.size().y as f32;
                } else if code == Key::PageUp {
                    state.y_offset -= window.size().y as f32;
                },
                _ => {}
            }
        }
        let scroll_speed = 8.0;
        if Key::Down.is_pressed() {
            state.y_offset += scroll_speed;
        } else if Key::Up.is_pressed() {
            state.y_offset -= scroll_speed;
        }
        if state.y_offset < 0.0 {
            state.y_offset = 0.0;
        }
        window.clear(&Color::BLACK);
        draw_frames(&state, &frameset, &mut window);
        window.display();
    }
}
