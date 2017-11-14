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
    fail_texture: Texture,
    frame_size: u32,
}

impl State {
    fn new(window_width: u32) -> Self {
        let frames_per_row = 5;
        let frame_gap = 2;
        Self {
            frames_per_row,
            frame_gap,
            y_offset: 0.0,
            font: Font::from_memory(include_bytes!("../Vera.ttf")).unwrap(),
            fail_texture: Texture::from_memory(include_bytes!("../fail.png"), &Default::default())
                .unwrap(),
            frame_size: (window_width - frame_gap) / frames_per_row,
        }
    }
}

fn draw_frames<'a, I: IntoIterator<Item = &'a mut Frame>>(
    state: &State,
    frames: I,
    target: &mut RenderWindow,
) {
    let Vector2u { y: th, .. } = target.size();
    let frame_size = state.frame_size;
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
    for (i, frame) in frames.enumerate() {
        let i = i as u32;
        let column = i % state.frames_per_row;
        let row = i / state.frames_per_row;
        let x = (column * (frame_size + state.frame_gap)) as f32;
        let y =
            (row * (frame_size + state.frame_gap)) as f32 - (state.y_offset % frame_size as f32);
        {
            let mut sprite = Sprite::with_texture(
                frame
                    .texture_lazy(frame_size)
                    .unwrap_or(&state.fail_texture),
            );
            sprite.set_position((x, y));
            target.draw(&sprite);
        }
        let mut text = Text::new(&frame.name, &state.font, 8);
        text.set_position((x, y));
        text.set_fill_color(&Color::BLACK);
        target.draw(&text);
    }
}

// Frame containing image and other data
struct Frame {
    name: String,
    tags: Vec<String>,
    texture: Option<Texture>,
    load_fail: bool,
}

fn load_thumbnail(path: &str, size: u32) -> Option<Texture> {
    let orig = Texture::from_file(path)?;
    let mut rt = RenderTexture::new(size, size, false).unwrap();
    let mut spr = Sprite::with_texture(&orig);
    let xscale = size as f32 / orig.size().x as f32;
    let yscale = size as f32 / orig.size().y as f32;
    spr.set_scale((xscale, yscale));
    rt.clear(&Color::WHITE);
    rt.draw(&spr);
    rt.display();
    Some(rt.texture().to_owned())
}

impl Frame {
    fn texture_lazy(&mut self, size: u32) -> Option<&Texture> {
        if self.load_fail {
            return None;
        }
        let name = &self.name;
        match self.texture {
            Some(ref texture) => Some(texture),
            None => {
                let th = match load_thumbnail(name, size) {
                    Some(th) => th,
                    None => {
                        self.load_fail = true;
                        return None;
                    }
                };
                self.texture = Some(th);
                self.texture.as_ref()
            }
        }
    }
}

fn construct_frameset(tagger_map: &TaggerMap, rule: &str) -> Result<Vec<Frame>, infix::ParseError> {
    let rules = infix::parse_infix(rule)?;
    let entries = tagger_map.tag_map.matching_entries(&rules);
    let mut frameset = Vec::new();
    for (name, tags) in entries {
        frameset.push(Frame {
            name: name.clone(),
            tags: tags.to_owned(),
            texture: None,
            load_fail: false,
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

    let mut state = State::new(window.size().x);
    let mut frameset = construct_frameset(tagger_map, "").unwrap();

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed { code, .. } => if code == Key::PageDown {
                    state.y_offset += window.size().y as f32;
                } else if code == Key::PageUp {
                    state.y_offset -= window.size().y as f32;
                },
                Event::MouseButtonPressed { button, x, y } => if button == mouse::Button::Left {
                    let frame_x = x as u32 / (state.frame_size + state.frame_gap);
                    let frame_y =
                        (y as u32 + state.y_offset as u32) / (state.frame_size + state.frame_gap);
                    let frame_index = frame_y * state.frames_per_row + frame_x;
                    open_in_image_viewer(&frameset[frame_index as usize].name);
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
        draw_frames(&state, &mut frameset, &mut window);
        window.display();
    }
}

fn open_in_image_viewer(name: &str) {
    use std::process::Command;
    Command::new("viewnior").arg(name).spawn().unwrap();
}
