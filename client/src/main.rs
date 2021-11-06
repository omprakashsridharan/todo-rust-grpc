use eframe::{egui, epi, run_native, NativeOptions};

#[derive(Default)]
pub struct TodoApp {}

fn main() {
    let app = TodoApp::default();
    let native_options = NativeOptions::default();
    run_native(Box::new(app), native_options);
}

impl epi::App for TodoApp {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                })
            })
        });
    }

    fn name(&self) -> &str {
        "Todo gRPC"
    }
}
