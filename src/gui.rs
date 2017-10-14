use gdk_pixbuf::{InterpType, Pixbuf};
use gtk;
use gtk::{Box, Entry, EntryBuffer, Grid, Image, Label, Orientation, Window, WindowType};
use gtk::prelude::*;
use infix::parse_infix;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tagger_map::TaggerMap;
use tagmap::{MatchRule, MatchingEntries};

const SHOW_AT_ONCE: usize = 10;

fn update_grid(
    grid: &Grid,
    entries: MatchingEntries<String, String>,
    offset: usize,
    map: Rc<RefCell<TaggerMap>>,
    window: &Window,
) {
    for (i, (k, v)) in entries
        .skip(offset * SHOW_AT_ONCE)
        .take(SHOW_AT_ONCE)
        .enumerate()
    {
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
                    let filename_entry = Entry::new_with_buffer(&EntryBuffer::new(Some(k)));
                    filename_entry.connect_key_press_event({
                        let src = k.clone();
                        let map = Rc::clone(&map);
                        let window = window.clone();

                        move |entry, event| {
                            use gdk::enums::key;
                            let key = event.get_keyval();

                            if key == key::Return {
                                let dst = entry.get_text().unwrap();
                                let mut map = map.borrow_mut();
                                if map.tag_map.entries.get(&dst).is_some() {
                                    use gtk::{ButtonsType, MessageDialog, MessageType,
                                              DIALOG_MODAL};
                                    let msg = format!(
                                        "A file with the name \"{}\" already exists.",
                                        &dst
                                    );
                                    let dialog = MessageDialog::new(
                                        Some(&window),
                                        DIALOG_MODAL,
                                        MessageType::Info,
                                        ButtonsType::Ok,
                                        &msg,
                                    );
                                    dialog.run();
                                    dialog.destroy();
                                    return Inhibit(false);
                                }
                                let value = map.tag_map
                                    .entries
                                    .remove(&src)
                                    .expect("Didn't find entry.");
                                map.tag_map.entries.insert(dst.clone(), value);
                                ::std::fs::rename(&src, &dst).unwrap();
                                map.save_to_file(::LIST_DEFAULT_FILENAME).unwrap();
                            }

                            Inhibit(false)
                        }
                    });
                    b.add(&filename_entry);
                    let tag_entry = Entry::new_with_buffer(&EntryBuffer::new(Some(&v.join(" "))));
                    tag_entry.connect_key_press_event({
                        let map_key = k.clone();
                        let map = Rc::clone(&map);

                        move |entry, event| {
                            use gdk::enums::key;
                            let key = event.get_keyval();

                            if key == key::Return {
                                let text = entry.get_text().unwrap();
                                let mut map = map.borrow_mut();
                                *map.tag_map.entries.get_mut(&map_key).unwrap() =
                                    text.split_whitespace().map(|s| s.to_owned()).collect();
                                map.save_to_file(::LIST_DEFAULT_FILENAME).unwrap();
                            }

                            Inhibit(false)
                        }
                    });
                    b.add(&tag_entry);
                    slot.insert(b.clone());
                    b
                }
            }
        });
        grid.attach(&b, (i % 5) as i32, (i / 5) as i32, 1, 1);
        // We have to force a redraw in this stupid convoluted way, just so we
        // can show new images as they finished loading.
        grid.queue_draw();
        window.show_all();
        while gtk::events_pending() {
            gtk::main_iteration_do(true);
        }
    }
}

pub fn run(tagger_map: Rc<RefCell<TaggerMap>>) {
    use gdk::enums::key;
    use std::cell::Cell;

    let rule = Rc::new(RefCell::new(MatchRule::Rules(vec![])));

    gtk::init().unwrap();
    let window = Window::new(WindowType::Toplevel);
    let label = Label::new(Some("Filter"));
    let entry = Entry::new_with_buffer(&EntryBuffer::new(None));
    let h_box = Box::new(Orientation::Horizontal, 8);
    let page_counter = Rc::new(Cell::new(0));

    h_box.add(&label);
    h_box.add(&entry);
    let grid = Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    entry.connect_key_press_event({
        let rule = Rc::clone(&rule);
        let window = window.clone();
        let grid = grid.clone();
        let tagger_map = Rc::clone(&tagger_map);
        let page_counter = Rc::clone(&page_counter);

        move |entry, event| {
            let key = event.get_keyval();

            if key == key::Return {
                if let Some(text) = entry.get_text() {
                    match parse_infix(&text) {
                        Ok(parsed_rule) => {
                            *rule.borrow_mut() = parsed_rule;
                            grid.remove_row(0);
                            grid.remove_row(0);
                            page_counter.set(0);
                            update_grid(
                                &grid,
                                tagger_map.borrow().tag_map.matching_entries(&rule.borrow()),
                                0,
                                Rc::clone(&tagger_map),
                                &window,
                            );
                            window.show_all();
                        }
                        Err(e) => println!("{}", e),
                    }
                }
            }
            Inhibit(false)
        }
    });
    let v_box = Box::new(Orientation::Vertical, 8);
    v_box.add(&h_box);
    v_box.add(&grid);
    window.add(&v_box);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.connect_key_press_event({
        let tagger_map = Rc::clone(&tagger_map);
        let grid = grid.clone();
        let rule = Rc::clone(&rule);

        move |window, event| {
            use std::cmp;

            let key = event.get_keyval();
            if key == key::Page_Down {
                let map = tagger_map.borrow();
                grid.remove_row(0);
                grid.remove_row(0);
                let rule = rule.borrow();
                let entries = map.tag_map.matching_entries(&rule);
                let max_offset = {
                    let n_images = entries.clone().count();
                    let mut n_pages = n_images / SHOW_AT_ONCE;
                    if n_images % SHOW_AT_ONCE != 0 {
                        n_pages += 1;
                    }
                    if n_pages > 0 {
                        n_pages - 1
                    } else {
                        0
                    }
                };
                page_counter.set(cmp::min(page_counter.get() + 1, max_offset));
                update_grid(
                    &grid,
                    entries,
                    page_counter.get(),
                    Rc::clone(&tagger_map),
                    window,
                );
                window.show_all();
            } else if key == key::Page_Up {
                grid.remove_row(0);
                grid.remove_row(0);
                page_counter.set(cmp::max(page_counter.get(), 1) - 1);
                update_grid(
                    &grid,
                    tagger_map.borrow().tag_map.matching_entries(&rule.borrow()),
                    page_counter.get(),
                    Rc::clone(&tagger_map),
                    window,
                );
                window.show_all();
            }
            Inhibit(false)
        }
    });
    update_grid(
        &grid,
        tagger_map.borrow().tag_map.matching_entries(&rule.borrow()),
        0,
        Rc::clone(&tagger_map),
        &window,
    );
    window.show_all();
    gtk::main();
}
