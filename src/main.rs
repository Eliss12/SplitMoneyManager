use eframe::NativeOptions;

mod db;
mod backend;
mod app;
mod user;
mod group;
mod expenses;
mod notification;

use app::MyApp;


fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 450.0])
            .with_min_inner_size([600.0, 400.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "SplitMoney",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_pixels_per_point(1.5);
            Ok(Box::new(MyApp::default()))
        }),
    )
}
