use gtk;
use gtk::prelude::*;
use gtk::{Box, Entry, EntryBuffer, Grid, Image, Orientation, Window, WindowType};
use gdk_pixbuf::{InterpType, Pixbuf};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use tagger_map::TaggerMap;

const SHOW_AT_ONCE: usize = 10;

fn update_grid(grid: &Grid, tagger_map: &TaggerMap, offset: usize) {
    for (i, (k, v)) in tagger_map.tag_map
        .entries
        .iter()
        .skip(offset * SHOW_AT_ONCE)
        .take(SHOW_AT_ONCE)
        .enumerate() {
        thread_local!(static CACHE: RefCell<HashMap<String, Box>> = RefCell::new(HashMap::new()));
        let b = CACHE.with(|cache| {
            use std::collections::hash_map::Entry as HashEntry;

            match cache.borrow_mut().entry(k.clone()) {
                HashEntry::Occupied(slot) => slot.get().clone(),
                HashEntry::Vacant(slot) => {
                    let image = match Pixbuf::new_from_file(k) {
                        Ok(buf) => {
                            let scaled = buf.scale_simple(192, 192, InterpType::Bilinear)
                                .expect("Failed to scale image");
                            Image::new_from_pixbuf(Some(&scaled))
                        }
                        Err(e) => {
                            println!("Error: Failed to load image: {}", e);
                            Image::new()
                        }
                    };
                    let b = Box::new(Orientation::Vertical, 2);
                    b.add(&image);
                    b.add(&Entry::new_with_buffer(&EntryBuffer::new(Some(&k))));
                    b.add(&Entry::new_with_buffer(&EntryBuffer::new(Some(&v.join(" ")))));
                    slot.insert(b.clone());
                    b
                }
            }
        });
        grid.attach(&b, (i % 5) as i32, (i / 5) as i32, 1, 1);
    }
}

pub fn run(tagger_map: Rc<RefCell<TaggerMap>>) {
    gtk::init().unwrap();
    let window = Window::new(WindowType::Toplevel);
    let grid = Rc::new(Grid::new());
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    window.add(&*grid);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.connect_key_press_event({
        use std::cell::Cell;

        let tagger_map = tagger_map.clone();
        let grid = grid.clone();
        let counter = Cell::new(0);

        move |window, event| {
            use gdk::enums::key;
            use std::cmp;

            let key = event.get_keyval();
            if key == key::Page_Down {
                let map = tagger_map.borrow();
                grid.remove_row(0);
                grid.remove_row(0);
                let max_offset = (map.tag_map.entries.len() / SHOW_AT_ONCE) - 1;
                counter.set(cmp::min(counter.get(), max_offset) + 1);
                update_grid(&grid, &map, counter.get());
                window.show_all();
            } else if key == key::Page_Up {
                grid.remove_row(0);
                grid.remove_row(0);
                counter.set(cmp::max(counter.get(), 1) - 1);
                update_grid(&grid, &tagger_map.borrow(), counter.get());
                window.show_all();
            }
            Inhibit(false)
        }
    });
    update_grid(&grid, &tagger_map.borrow(), 0);
    window.show_all();
    gtk::main();
}
