use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::RefCell;
use std::rc::Rc;

use std::io::prelude::*;

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
        times: 2,
        status_area: gtk::Label::builder().halign(gtk::Align::Start).build(),
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
                    border-color: alpha(@theme_selected_bg_color, 0.5);\n
                    border-radius: 5px;\n
                    padding: 1px;\n
                }\n";
    let provider = gtk::CssProvider::new();
    provider.load_from_data(css);
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
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
    let top_bar = create_topbar(window, &sm);

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
    let action_bar = create_action_bar(&sm, text_area);

    grid.attach(&top_bar, 0, 0, 1, 1);
    grid.attach(&spacer, 0, 1, 1, 1);
    grid.attach(&description, 0, 2, 1, 1);
    grid.attach(&scrolled_text_area, 0, 3, 1, 1);
    grid.attach(&action_bar, 0, 4, 1, 1);
    grid.attach(&sm.state.borrow().status_area, 0, 5, 1, 1);
}

fn create_topbar(window: &gtk4::ApplicationWindow, sm: &StateManager) -> gtk4::Box {
    let top_bar = gtk::Box::new(gtk4::Orientation::Horizontal, 5);
    let file_entry = gtk::Entry::builder()
        .placeholder_text("Enter the file path here...")
        .hexpand(true)
        .build();
    let sm_clone = sm.clone();
    let file_entry_buffer = file_entry.buffer();
    file_entry.connect_changed(move |_| {
        sm_clone.set_path(file_entry_buffer.text().as_str());
    });
    top_bar.append(&file_entry);

    let picker_btn = gtk::Button::with_label("Select File");
    let sm_clone = sm.clone();
    let wn_clone = window.clone();
    let fb = file_entry.buffer();
    picker_btn.connect_clicked(move |_| handle_file_pick(&sm_clone, &wn_clone, &fb));
    top_bar.append(&picker_btn);
    top_bar
}

fn create_action_bar(sm: &StateManager, text_area: gtk4::TextView) -> gtk4::Box {
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
    repeat_btn.connect_clicked(move |_| handle_repeat(&sm_clone, &bfr));

    let spacer = gtk::Box::builder().hexpand(true).build();
    let append_btn = gtk::Button::builder().label("Append Text to File").build();
    let sm_clone = sm.clone();
    let bfr = text_area.buffer().clone();
    append_btn.connect_clicked(move |_| handle_append(&sm_clone, &bfr));

    action_bar.append(&n_label);
    action_bar.append(&times_input);
    action_bar.append(&repeat_btn);
    action_bar.append(&spacer);
    action_bar.append(&append_btn);
    action_bar
}

// ==================================================================
// EVENTS
// ==================================================================

fn handle_file_pick(sm: &StateManager, wn: &gtk4::ApplicationWindow, fb: &gtk4::EntryBuffer) {
    let file_dialog = gtk::FileDialog::new();
    let fb = fb.clone();
    let sm = sm.clone();
    file_dialog.open(
        Some(wn),
        gtk::gio::Cancellable::NONE,
        move |result| match result {
            Ok(file) => {
                let path = file.path().expect("Failed to get path");
                fb.set_text(path.display().to_string());
                sm.set_path(path.display().to_string().as_str());
                sm.log(format!("Selected file: {:?}", path));
            }
            Err(err) => {
                sm.log(format!("Error selecting file: {}", err));
            }
        },
    );
}

fn handle_repeat(sm: &StateManager, bfr: &gtk4::TextBuffer) {
    let str_prev = bfr.text(&bfr.start_iter(), &bfr.end_iter(), true);
    if str_prev.len() == 0 {
        sm.log("Skipping : Text area is empty");
        return;
    }
    let n = sm.get_times();
    for _ in 1..n {
        bfr.insert(&mut bfr.end_iter(), str_prev.as_str());
    }
    sm.log(format!("repeated {} times", n));
}

fn handle_append(sm: &StateManager, bfr: &gtk4::TextBuffer) {
    let path = sm.get_path();
    let text = bfr.text(&bfr.start_iter(), &bfr.end_iter(), true);

    let file = std::fs::OpenOptions::new()
        .append(true)
        .create(false)
        .open(&path);

    match file {
        Ok(mut file) => match file.write_all(text.as_bytes()) {
            Err(why) => sm.log(format!("couldn't append to {}: {}", path, why)),
            Ok(_) => sm.log(format!("successfully appended to {}", path)),
        },
        Err(why) => sm.log(format!("Couldn't Open File: {} : {}", path, why)),
    }
}

// ==========================================================
// STATE
// ==========================================================
struct State {
    path: String,
    times: u32,
    status_area: gtk::Label,
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

    fn log<S: Into<String>>(&self, text: S) {
        let text = text.into();
        self.state.borrow().status_area.set_text(text.as_str());
    }
}
