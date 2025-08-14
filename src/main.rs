use eframe::NativeOptions;

mod db;
mod backend;
mod app;
mod user;

use app::MyApp;


fn main() -> eframe::Result {

    let options = NativeOptions::default();
    eframe::run_native("SplitMoney", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))))
}
