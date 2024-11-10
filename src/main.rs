#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use bewegtbild::Config;
use clap::{Parser, Subcommand};
use notify::{Event, RecursiveMode, Watcher};
use std::fs;
use std::path::PathBuf;
use std::thread;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Write your configuration file on the fly.
    Config {
        #[clap(help = "PDF file to view")]
        pdf_path: PathBuf,

        #[clap(short, long, help = "Configuration file with video annotations")]
        config_path: PathBuf,
    },
    /// Start the presentation mode.
    View {
        #[clap(help = "PDF file to view")]
        pdf_path: PathBuf,

        #[clap(short, long, help = "Configuration file with video annotations")]
        config: Option<PathBuf>,
    },
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use bewegtbild::VideoEntry;
    use notify::event::ModifyKind;
    use std::{collections::HashMap, sync::mpsc};

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let args = Args::parse();
    match args.cmd {
        Commands::Config {
            pdf_path,
            config_path,
        } => {
            let config_path_abs = std::path::absolute(config_path.clone()).unwrap();
            let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
            let (ui_tx, ui_rx) = mpsc::channel::<HashMap<usize, Vec<VideoEntry>>>();

            let config_path_abs = config_path_abs.clone();
            thread::spawn(move || {
                let mut watcher =
                    notify::recommended_watcher(tx).expect("Failed to create watcher");
                watcher
                    .watch(
                        &config_path_abs.parent().unwrap(),
                        RecursiveMode::NonRecursive,
                    )
                    .expect("Failed to watch file");
                for res in rx {
                    match res {
                        Ok(event) => {
                            if event.paths.contains(&config_path_abs) {
                                if let notify::EventKind::Modify(ModifyKind::Data(
                                    notify::event::DataChange::Any,
                                )) = event.kind
                                {
                                    println!("CONFIG FILE HAS BEEN CHANGED");
                                    // Send signal to UI thread
                                    let config: Result<Config, serde_json::Error> =
                                        serde_json::from_str(
                                            &fs::read_to_string(config_path_abs.clone())
                                                .expect("Could not read config file."),
                                        );
                                    if let Ok(config) = config {
                                        ui_tx.send(config.slides_map()).unwrap();
                                    }
                                    //.expect("The format of the config file is wrong.");
                                }
                            }
                        }
                        Err(e) => println!("watch error: {:?}", e),
                    }
                }
            });
            let pdf_path = pdf_path;
            let config: Config = serde_json::from_str(
                &fs::read_to_string(config_path).expect("Could not read config file."),
            )
            .expect("The format of the config file is wrong.");
            println!("{:?}", config);

            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 300.0])
                    .with_min_inner_size([300.0, 220.0])
                    .with_icon(
                        // NOTE: Adding an icon is optional
                        eframe::icon_data::from_png_bytes(
                            &include_bytes!("../assets/icon-256.png")[..],
                        )
                        .expect("Failed to load icon"),
                    ),
                ..Default::default()
            };
            eframe::run_native(
                "bewegtbild",
                native_options,
                Box::new(|cc| {
                    Ok(Box::new(bewegtbild::TemplateApp::new(
                        cc,
                        pdf_path,
                        config.slides_map(),
                        Some(ui_rx),
                    )))
                }),
            )
        }
        Commands::View { pdf_path, config } => {
            let pdf_path = pdf_path;
            let config = match config {
                Some(config_path) => serde_json::from_str(
                    &fs::read_to_string(config_path).expect("Could not read config file."),
                )
                .expect("The format of the config file is wrong."),
                None => Config::default(),
            };
            println!("{:?}", config);

            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 300.0])
                    .with_min_inner_size([300.0, 220.0])
                    .with_icon(
                        // NOTE: Adding an icon is optional
                        eframe::icon_data::from_png_bytes(
                            &include_bytes!("../assets/icon-256.png")[..],
                        )
                        .expect("Failed to load icon"),
                    ),
                ..Default::default()
            };
            eframe::run_native(
                "bewegtbild",
                native_options,
                Box::new(|cc| {
                    Ok(Box::new(bewegtbild::TemplateApp::new(
                        cc,
                        pdf_path,
                        config.slides_map(),
                        None,
                    )))
                }),
            )
        }
    }
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(eframe_template::TemplateApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
