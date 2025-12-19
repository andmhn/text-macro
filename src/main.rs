use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

fn main() -> gtk::glib::ExitCode {
    let application = gtk::Application::builder()
        .application_id("com.github.andmhn.cr")
        .build();
    application.connect_activate(|app| activate(app));
    application.run()
}

fn activate(app: &gtk::Application) {
    load_css();
    let sm = StateManager::new(State {
        path: String::new(),
        content: String::new(),
        times: 2,
    });

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Macro")
        .default_width(690)
        .default_height(420)
        .build();
    create_editor_ui(&window, sm.clone());
    window.present();
}

fn load_css() {
    let css = ".scrolled_text_area {\n
                    border-style: solid;\n
                    border-width: 2px;\n
                    border-color: rgba(233, 84, 32, 0.4);\n
                    border-radius: 5px;\n
                    padding: 1px;\n
                }\n";
    let provider = gtk::CssProvider::new();
    provider.load_from_data(css);
    let display = gtk::gdk::Display::default();
    match display {
        Some(display) => {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        None => {}
    }
}

fn create_editor_ui(window: &gtk::ApplicationWindow, sm: StateManager) {
    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    window.set_child(Some(&grid));

    // TOP BAR ------------------------------------------------------
    let top_bar = gtk::Box::new(gtk4::Orientation::Horizontal, 5);
    let file_entry = gtk::Entry::builder()
        .placeholder_text("Enter the file path here...")
        .hexpand(true)
        .build();
    let sm_clone = sm.clone();
    let file_entry_buffer = file_entry.buffer();
    file_entry.connect_activate(move |_| {
        sm_clone.set_path(file_entry_buffer.text().as_str());
    });
    top_bar.append(&file_entry);

    let sm_clone = sm.clone();
    let picker_btn = gtk::Button::with_label("Select File");
    let wn_clone = window.clone();
    let fb = file_entry.buffer();
    picker_btn.connect_clicked(move |_| {
        let file_dialog = gtk::FileDialog::new();
        let fb_clone = fb.clone();
        let sm_clone = sm_clone.clone();
        file_dialog.open(
            Some(&wn_clone),
            gtk::gio::Cancellable::NONE,
            move |result| match result {
                Ok(file) => {
                    let path = file.path().expect("Failed to get path");
                    println!("Selected file: {:?}", path);
                    fb_clone.set_text(path.display().to_string());
                    sm_clone.set_path(path.display().to_string().as_str());
                }
                Err(err) => {
                    eprintln!("Error selecting file: {}", err);
                }
            },
        );
    });
    top_bar.append(&picker_btn);

    // TEXT AREA ----------------------------------------------------
    let spacer = gtk::Box::new(gtk4::Orientation::Horizontal, 0);

    let description = gtk::Label::builder()
        .label("Text to Input/Repeat:")
        .halign(gtk::Align::Start)
        .build();
    let text_area = gtk::TextView::builder().margin_start(3).build();
    let scrolled_text_area = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&text_area)
        .build();
    scrolled_text_area.add_css_class("scrolled_text_area");

    // ACTION BAR ---------------------------------------------------
    let action_bar = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let n_label = gtk::Label::builder().name("times ").build();

    let adj = gtk::Adjustment::new(2.0, 0.0, 999.0, 1.0, 10.0, 0.0);
    let times_input = gtk::SpinButton::builder()
        .adjustment(&adj)
        .climb_rate(1.0)
        .digits(0)
        .build();
    let sm_clone = sm.clone();
    times_input.connect_value_changed(move |n| {
        sm_clone.set_times(n.value_as_int() as u32);
    });

    let repeat_btn = gtk::Button::builder().label("Repeat Text").build();
    let bfr = text_area.buffer().clone();
    let sm_clone = sm.clone();
    repeat_btn.connect_clicked(move |_| {
        let str_prev = bfr.text(&bfr.start_iter(), &bfr.end_iter(), true);
        for _ in 1..sm_clone.get_times() {
            bfr.insert(&mut bfr.end_iter(), str_prev.as_str());
        }
    });

    let spacer2 = gtk::Box::builder().hexpand(true).build();
    let append_btn = gtk::Button::builder().label("Append Text to File").build();
    append_btn.connect_clicked(|_| {}); // FIXME

    action_bar.append(&n_label);
    action_bar.append(&times_input);
    action_bar.append(&repeat_btn);
    action_bar.append(&spacer2);
    action_bar.append(&append_btn);

    let status_area = gtk::Label::builder() // TODO: Use Toast
        .label("Welcome!")
        .halign(gtk::Align::Start)
        .build();

    grid.attach(&top_bar, 0, 0, 1, 1);
    grid.attach(&spacer, 0, 1, 1, 1);
    grid.attach(&description, 0, 2, 1, 1);
    grid.attach(&scrolled_text_area, 0, 3, 1, 1);
    grid.attach(&action_bar, 0, 4, 1, 1);
    grid.attach(&status_area, 0, 5, 1, 1);
}

//==========================================================
// STATE
//==========================================================
struct State {
    path: String,
    content: String,
    times: u32,
}

#[derive(Clone)]
struct StateManager {
    state: Rc<RefCell<State>>,
}

impl StateManager {
    fn new(state: State) -> Self {
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }

    fn get_times(&self) -> u32 {
        self.state.borrow().times
    }
    fn set_times(&self, t: u32) {
        self.state.borrow_mut().times = t;
    }

    fn get_path(&self) -> String {
        self.state.borrow().path.clone()
    }
    fn set_path(&self, s: &str) {
        let path = &mut self.state.borrow_mut().path;
        path.clear();
        path.push_str(s);
    }

    // fn get_content(&self) -> Ref<'_, String> {
    //     Ref::map(self.state.borrow(), |s| &s.content)
    //     // use immedietly, ex:
    //     // println!("Path: {}", *state.get_content());
    // }

    // fn repeat_content(&self, n: u8) {
    //     let content = &mut self.state.borrow_mut().content;
    //     let str_prev = content.clone();
    //     for _ in 1..n {
    //         content.push_str(str_prev.as_str());
    //     }
    // }
}
