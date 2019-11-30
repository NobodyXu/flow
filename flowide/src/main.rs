#![deny(missing_docs)]
//! The `flowide` is a prototype of a native IDE for `flow` programs.

use std::env;
use std::env::args;
use std::sync::{Arc, Mutex};

use flowrlib::coordinator::{Coordinator, Submission};
use flowrlib::lib_manifest::LibraryManifest;
use flowrlib::loader::Loader;
use flowrlib::manifest::Manifest;
use flowrlib::provider::Provider;
use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use glib;
use gtk::{
    AboutDialog, AccelFlags, AccelGroup, Application, ApplicationWindow, Box, FileChooserAction, FileChooserDialog,
    FileFilter, Menu, MenuBar, MenuItem, ResponseType, ScrolledWindow, TextBuffer, TextView, WidgetExt, WindowPosition,
};
use gtk::prelude::*;
use log::info;
use provider::content::provider::MetaProvider;

use flowclib::compiler::compile;
use flowclib::compiler::loader;
use flowclib::deserializers::deserializer_helper;
use flowclib::generator::generate;
use flowclib::model::flow::Flow;
use flowclib::model::process::Process::FlowProcess;
use ide_runtime_client::IDERuntimeClient;
use runtime::runtime_client::RuntimeClient;
use runtime_context::RuntimeContext;
use ui_context::UIContext;

mod runtime_context;
mod ui_context;
mod ide_runtime_client;

/// upgrade weak reference or return
#[macro_export]
macro_rules! upgrade_weak {
    ($x:ident, $r:expr) => {{
        match $x.upgrade() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak!($x, ())
    };
}

fn resource(path: &str) -> String {
    format!("{}/resources/{}", env!("CARGO_MANIFEST_DIR"), path)
}

fn about_dialog() -> AboutDialog {
    let p = AboutDialog::new();
    p.set_program_name(env!("CARGO_PKG_NAME"));
    p.set_website_label(Some("Flow website"));
    p.set_website(Some(env!("CARGO_PKG_HOMEPAGE")));
    p.set_title(&format!("About {}", env!("CARGO_PKG_NAME")));
    p.set_version(Some(env!("CARGO_PKG_VERSION")));
    p.set_comments(Some(&format!("flowclib version: {}\nflowrlib version: {}",
                                 flowclib::info::version(), flowrlib::info::version())));
    println!("pwd {:?}", std::env::current_dir());
    if let Ok(image) = Pixbuf::new_from_file(resource("icons/png/128x128.png")) {
        p.set_logo(Some(&image));
    }

    //CARGO_PKG_DESCRIPTION
    //CARGO_PKG_REPOSITORY

    // AboutDialog has a secondary credits section
    p.set_authors(&[env!("CARGO_PKG_AUTHORS")]);

    p
}

fn run_flow() {
    println!("Run");
}

fn load_libs<'a>(loader: &'a mut Loader, provider: &dyn Provider, runtime_manifest: LibraryManifest) -> Result<String, String> {
    // Load this runtime's library of native (statically linked) implementations
    loader.add_lib(provider, runtime_manifest, "runtime").map_err(|e| e.to_string())?;

    // Load the native flowstdlib - before it maybe loaded from WASM
    loader.add_lib(provider, flowstdlib::get_manifest(), "flowstdlib").map_err(|e| e.to_string())?;

    Ok("Added the 'runtime' and 'flowstdlibs'".to_string())
}

fn load_from_uri(uri: &str, runtime_client: Arc<Mutex<dyn RuntimeClient>>, ui_context: UIContext) -> Result<String, String> {
    let mut loader = Loader::new();
    let provider = MetaProvider {};
    let runtime_manifest = runtime::manifest::create_runtime(runtime_client);

    load_libs(&mut loader, &provider, runtime_manifest).map_err(|e| e.to_string())?;

    let manifest = loader.load_manifest(&provider, uri)
        .map_err(|e| format!("Could not load the manifest: '{}'", e.to_string()))?;

    set_manifest_contents(&manifest, ui_context).unwrap(); // TODO

    let submission = Submission::new(manifest, 1, false, None);
    run_submission(submission);

    Ok(format!("Loaded Manifest from URI: {}", uri))
}

fn file_open_action(window: &ApplicationWindow, open: &MenuItem,
                    runtime_client: Arc<Mutex<IDERuntimeClient>>,
                    ui_context: UIContext) {
    let accepted_extensions = deserializer_helper::get_accepted_extensions();

    let window_weak = window.downgrade();
    open.connect_activate(move |_| {
        let window = upgrade_weak!(window_weak);
        let dialog = FileChooserDialog::new(Some("Choose a file"), Some(&window),
                                            FileChooserAction::Open);
        dialog.add_buttons(&[
            ("Open", ResponseType::Ok),
            ("Cancel", ResponseType::Cancel)
        ]);

        dialog.set_select_multiple(false);
        let filter = FileFilter::new();
        for extension in accepted_extensions {
            filter.add_pattern(&format!("*.{}", extension));
        }
        dialog.set_filter(&filter);
        dialog.run();
        let uris = dialog.get_uris();
        dialog.destroy();

        if let Some(uri) = uris.get(0) {
            load_from_uri(&uri.to_string(),
                          runtime_client.clone() as Arc<Mutex<dyn RuntimeClient>>,
                          ui_context.clone()).unwrap(); // TODO
        }
    });
}

fn menu_bar(window: &ApplicationWindow,
            runtime_client: Arc<Mutex<IDERuntimeClient>>,
            ui_context: UIContext) -> MenuBar {
    let menu = Menu::new();
    let accel_group = AccelGroup::new();
    window.add_accel_group(&accel_group);
    let menu_bar = MenuBar::new();

    let file = MenuItem::new_with_label("File");
    let open = MenuItem::new_with_label("Open");
    let about = MenuItem::new_with_label("About");
    let run = MenuItem::new_with_label("Run");
    let quit = MenuItem::new_with_label("Quit");

    menu.append(&open);
    menu.append(&about);
    menu.append(&run);
    menu.append(&quit);
    file.set_submenu(Some(&menu));
    menu_bar.append(&file);

    file_open_action(window, &open, runtime_client, ui_context);
    run.connect_activate(|_| run_flow()); // run the flow from run menu item

    let other_menu = Menu::new();
    let sub_other_menu = Menu::new();
    let other = MenuItem::new_with_label("Another");
    let sub_other = MenuItem::new_with_label("Sub another");
    let sub_other2 = MenuItem::new_with_label("Sub another 2");
    let sub_sub_other2 = MenuItem::new_with_label("Sub sub another 2");
    let sub_sub_other2_2 = MenuItem::new_with_label("Sub sub another2 2");

    sub_other_menu.append(&sub_sub_other2);
    sub_other_menu.append(&sub_sub_other2_2);
    sub_other2.set_submenu(Some(&sub_other_menu));
    other_menu.append(&sub_other);
    other_menu.append(&sub_other2);
    other.set_submenu(Some(&other_menu));
    menu_bar.append(&other);

    let window_weak = window.downgrade();
    quit.connect_activate(move |_| {
        let window = upgrade_weak!(window_weak);
        window.destroy();
    });

    // `Primary` is `Ctrl` on Windows and Linux, and `command` on macOS
    // It isn't available directly through gdk::ModifierType, since it has
    // different values on different platforms.
    let (key, modifier) = gtk::accelerator_parse("<Primary>Q");
    quit.add_accelerator("activate", &accel_group, key, modifier, AccelFlags::VISIBLE);

    let window_weak = window.downgrade();
    about.connect_activate(move |_| {
        let ad = about_dialog();
        let window = upgrade_weak!(window_weak);
        ad.set_transient_for(Some(&window));
        ad.run();
        ad.destroy();
    });

    menu_bar
}

fn args_view(buffer: &TextBuffer) -> TextView {
    let args_view = gtk::TextView::new();
    args_view.set_buffer(Some(buffer));
    args_view.set_size_request(-1, 1); // Want to fill width and be one line high :-(
    args_view
}

fn stdio(buffer: &TextBuffer) -> ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    let view = gtk::TextView::new();
    view.set_buffer(Some(buffer));
    view.set_editable(false);
    scroll.add(&view);
    scroll
}

fn manifest_viewer(buffer: &TextBuffer) -> ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    let view = gtk::TextView::new();
    view.set_buffer(Some(buffer));
    view.set_editable(false);
    scroll.add(&view);
    scroll
}

fn main_window(runtime_context: &RuntimeContext, ui_context: &UIContext) -> Box {
    let main = gtk::Box::new(gtk::Orientation::Vertical, 10);
    main.set_border_width(6);
    main.set_vexpand(true);
    main.set_hexpand(true);

    let args_view = args_view(&runtime_context.args);
    let stdout_view = stdio(&runtime_context.stdout);
    let stderr_view = stdio(&runtime_context.stderr);
    let manifest_view = manifest_viewer(&ui_context.manifest);

    main.pack_start(&manifest_view, true, true, 0);
    main.pack_start(&args_view, true, true, 0);
    main.pack_start(&stdout_view, true, true, 0);
    main.pack_start(&stderr_view, true, true, 0);

    main
}

fn build_ui(application: &gtk::Application, runtime_context: &RuntimeContext,
            ide_runtime_client: Arc<Mutex<IDERuntimeClient>>) {
    let ui_context = UIContext::new();
    let main_window = main_window(&runtime_context, &ui_context);

    let app_window = ApplicationWindow::new(application);

    app_window.set_title(env!("CARGO_PKG_NAME"));
    app_window.set_position(WindowPosition::Center);
    app_window.set_size_request(400, 400);

    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    v_box.pack_start(&menu_bar(&app_window, ide_runtime_client, ui_context), false, false, 0);
    v_box.pack_start(&main_window, true, true, 0);

    app_window.add(&v_box);

    app_window.show_all();
}

fn set_manifest_contents(manifest: &Manifest, ui_context: UIContext) -> Result<(), String> {
    let manifest_content = serde_json::to_string_pretty(&manifest)
        .map_err(|e| e.to_string())?;

    ui_context.manifest.set_text(&manifest_content);

    Ok(())
}

/*
    manifest_dir is used as a reference directory for relative paths to project files
*/
fn compile(flow: &Flow, debug_symbols: bool, manifest_dir: &str) -> Result<Manifest, String> {
    info!("Compiling Flow to Manifest");
    let tables = compile::compile(flow)
        .map_err(|e| format!("Could not compile flow: '{}'", e.to_string()))?;

    generate::create_manifest(&flow, debug_symbols, &manifest_dir, &tables)
        .map_err(|e| format!("Could create flow manifest: '{}'", e.to_string()))
}

fn load_flow(provider: &dyn Provider, url: &str) -> Result<Flow, String> {
    match loader::load_context(url, provider)
        .map_err(|e| format!("Could not load flow context: '{}'", e.to_string()))? {
        FlowProcess(flow) => Ok(flow),
        _ => Err("Process loaded was not of type 'Flow'".into())
    }
}

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
}

fn run_submission(submission: Submission) {
    let mut coordinator = Coordinator::new(1);

    info!("Submitting flow for execution");
    coordinator.submit(submission);
}

fn main() -> Result<(), String> {
    let (command_sender, command_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let ide_runtime_client_arc = Arc::new(Mutex::new(IDERuntimeClient::new(command_sender)));

    let application = Application::new(Some("net.mackenzie-serres.flow.ide"), Default::default())
        .expect("failed to initialize GTK application");

    let runtime_context = RuntimeContext::new();
    let context_clone = runtime_context.clone();
    let client_clone = ide_runtime_client_arc.clone();
    application.connect_activate(move |app|
        build_ui(app, &context_clone, client_clone.clone())
    );

    set_panic_hook();

//    let log_level_arg = get_log_level(&document);
//    init_logging(log_level_arg);

    // load a flow definition
//    let flow_lib_path = get_flow_lib_path(&document).map_err(|e| JsValue::from_str(&e.to_string()))?;
//    let flow = load_flow(&provider, "file:://Users/andrew/workspace/flow/flowide/src/hello_world.toml")
//        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // compile to manifest
//    manifest = compile(&flow, true, "/Users/andrew/workflow/flow")
//        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // or load a manifest from file

    // Attach the receiver to the default main context (None) and on every message process the command
    command_receiver.attach(None, move |command| {
        ide_runtime_client_arc.lock().unwrap().process_command(command, &runtime_context.clone());

        // Returning false here would close the receiver and have senders fail
        glib::Continue(true)
    });

    application.run(&args().collect::<Vec<_>>());

    Ok(())
}

// TODO Read flow lib path from env and add it as a setting with a dialog to edit it.