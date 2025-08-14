use eframe::NativeOptions;

mod models;
mod db;
mod backend;
mod app;

use app::MyApp;


fn main() -> eframe::Result {

    let options = NativeOptions::default();
    eframe::run_native("SplitMoney", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))))
}
