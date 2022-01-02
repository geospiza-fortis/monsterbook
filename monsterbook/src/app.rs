use eframe::{egui, epi};
use rfd::FileDialog;

#[derive(Default)]
pub struct App {
    picked_path: Option<String>,
}

impl<'a> epi::App for App {
    fn name(&self) -> &str {
        "Monsterbook Stitcher"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let Self { picked_path } = self;

        egui::TopBottomPanel::bottom("info").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.hyperlink_to("made by geospiza", "https://geospiza.me");
                ui.label("|");
                ui.hyperlink_to(
                    "source code",
                    "https://github.com/geospiza-fortis/monsterbook",
                );
                ui.label("|");
                ui.label(format!("version {}", env!("CARGO_PKG_VERSION")));
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open directory...").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.picked_path = Some(path.display().to_string());
                    }
                }
                if let Some(picked_path) = &self.picked_path {
                    ui.monospace(picked_path);
                }
            });
        });
        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}
