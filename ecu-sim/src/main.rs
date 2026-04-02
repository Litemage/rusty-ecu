mod stub;
mod ui;

use ui::ECUSimApp;

fn main() -> Result<(), eframe::Error> {
    // Run the simulation app
    let native_opts = eframe::NativeOptions::default();
    eframe::run_native(
        "ECU Simulator",
        native_opts,
        Box::new(|cc| Ok(Box::new(ECUSimApp::new(cc)))),
    )
}
