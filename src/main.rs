use eframe::NativeOptions;

mod db;
mod backend;
mod app;
mod user;
mod group;
mod expenses;
mod notification;

use app::MyApp;


fn main() -> eframe::Result {

    let options = NativeOptions::default();
    eframe::run_native("SplitMoney", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))))
}
