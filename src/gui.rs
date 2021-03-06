extern crate image;
extern crate sfml;

use self::sfml::graphics::*;
use self::sfml::window::*;
use self::sfml::system::*;
use self::image::{ImageBuffer, ImageResult, Rgba};
use std::sync::{Arc, Mutex};
use tagger_map::TaggerMap;
use infix;

struct State {
    frames_per_row: u32,
    y_offset: f32,
    font: Font,
    fail_texture: Texture,
    loading_texture: Texture,
    frame_size: u32,
}

impl State {
    fn new(window_width: u32) -> Self {
        let frames_per_row = 5;
        Self {
            frames_per_row,
            y_offset: 0.0,
            font: Font::from_memory(include_bytes!("../Vera.ttf")).unwrap(),
            fail_texture: Texture::from_memory(include_bytes!("../fail.png"), &Default::default())
                .unwrap(),
            loading_texture: Texture::from_memory(
                include_bytes!("../loading.png"),
                &Default::default(),
            ).unwrap(),
            frame_size: window_width / frames_per_row,
        }
    }
}

fn draw_frames(
    state: &State,
    frames: &mut [Frame],
    target: &mut RenderWindow,
    thumb_loader: &mut ThumbnailLoader,
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
    thumb_loader.write_to_frameset(frames);
    let frames = frames
        .into_iter()
        .enumerate()
        .skip(skip as usize)
        .take(frames_per_screen);
    for (on_screen_index, (index, frame)) in frames.enumerate() {
        let column = (on_screen_index as u32) % state.frames_per_row;
        let row = (on_screen_index as u32) / state.frames_per_row;
        let x = (column * frame_size) as f32;
        let y = (row * frame_size) as f32 - (state.y_offset % frame_size as f32);
        if !frame.load_fail && frame.texture.is_none() {
            thumb_loader.request(&frame.name, frame_size, index);
        }
        let mut sprite = Sprite::with_texture(if frame.load_fail {
            &state.fail_texture
        } else {
            match frame.texture {
                Some(ref t) => t,
                None => &state.loading_texture,
            }
        });
        sprite.set_position((x, y));
        if frame.selected {
            sprite.set_color(&Color::GREEN);
        }
        target.draw(&sprite);
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
    selected: bool,
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
            selected: false,
        });
    }
    Ok(frameset)
}

type RgbaBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

const BUSY_WITH_NONE: usize = ::std::usize::MAX;

/// Loads images on a separate thread, one at a time.
struct ThumbnailLoader {
    busy_with: usize,
    image_slot: Arc<Mutex<Option<ImageResult<RgbaBuf>>>>,
}

impl Default for ThumbnailLoader {
    fn default() -> Self {
        Self {
            busy_with: BUSY_WITH_NONE,
            image_slot: Default::default(),
        }
    }
}

impl ThumbnailLoader {
    fn request(&mut self, name: &str, size: u32, index: usize) {
        if self.busy_with == BUSY_WITH_NONE {
            self.busy_with = index;
            let image_slot = Arc::clone(&self.image_slot);
            let name = name.to_owned();
            ::std::thread::spawn(move || {
                use std::fs::File;
                use std::io::prelude::*;
                use self::image::FilterType;
                let mut f = File::open(name).unwrap();
                // Try to load file as efficiently as possible, using a single compact allocation.
                // We trust that `len` returned by metadata is correct.
                let len = f.metadata().unwrap().len() as usize;
                let mut buf = Vec::with_capacity(len as usize);
                unsafe {
                    // Set length for `read_exact` to fill.
                    buf.set_len(len);
                    // This should fill all the uninitialized buffer.
                    f.read_exact(&mut buf).unwrap();
                }
                // Because loading images is memory intensive, and we might load multiple images
                // in parallel, we eagerly drop some stuff in order to free up memory
                // as soon as possible.
                drop(f);
                let image_result = image::load_from_memory(&buf);
                drop(buf);
                let result =
                    image_result.map(|i| i.resize(size, size, FilterType::Triangle).to_rgba());
                *image_slot.lock().unwrap() = Some(result);
            });
        }
    }
    fn write_to_frameset(&mut self, frameset: &mut [Frame]) {
        if let Some(result) = self.image_slot.lock().unwrap().take() {
            match result {
                Ok(buf) => {
                    let (w, h) = buf.dimensions();
                    let mut tex = Texture::new(w, h).unwrap();
                    tex.update_from_pixels(&buf.into_raw(), w, h, 0, 0);
                    frameset[self.busy_with].texture = Some(tex);
                }
                Err(_) => frameset[self.busy_with].load_fail = true,
            }
            self.busy_with = BUSY_WITH_NONE;
        }
    }
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
    let mut thumb_loader = ThumbnailLoader::default();

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed { code, .. } => if code == Key::PageDown {
                    state.y_offset += window.size().y as f32;
                } else if code == Key::PageUp {
                    state.y_offset -= window.size().y as f32;
                } else if code == Key::Return {
                    let mut names: Vec<&str> = Vec::new();
                    for f in &frameset {
                        if f.selected {
                            names.push(&f.name);
                        }
                    }
                    open_in_image_viewer(&names);
                },
                Event::MouseButtonPressed { button, x, y } => if button == mouse::Button::Left {
                    let frame_x = x as u32 / state.frame_size;
                    let frame_y = (y as u32 + state.y_offset as u32) / state.frame_size;
                    let frame_index = frame_y * state.frames_per_row + frame_x;
                    let frame = &mut frameset[frame_index as usize];
                    if Key::LShift.is_pressed() {
                        frame.selected = !frame.selected;
                    } else {
                        open_in_image_viewer(&[&frame.name]);
                    }
                },
                Event::MouseWheelScrolled {
                    wheel: mouse::Wheel::Vertical,
                    delta,
                    ..
                } => {
                    state.y_offset -= delta * 32.0;
                }
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
        draw_frames(&state, &mut frameset, &mut window, &mut thumb_loader);
        window.display();
    }
}

fn open_in_image_viewer(names: &[&str]) {
    use std::process::Command;
    Command::new("viewnior").args(names).spawn().unwrap();
}
