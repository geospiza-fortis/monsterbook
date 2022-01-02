use super::crop::Image;
use super::utils;
use eframe::{egui, epi};
use rfd::FileDialog;
use std::sync::mpsc::Receiver;
use std::thread;

#[derive(Default)]
pub struct App {
    picked_path: Option<String>,
    cropped: Option<Vec<Image>>,
    crop_in_progress: Option<Receiver<Vec<Image>>>,
}

impl<'a> epi::App for App {
    fn name(&self) -> &str {
        "Monsterbook Stitcher"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let Self {
            picked_path,
            cropped,
            crop_in_progress,
        } = self;

        if let Some(receiver) = &self.crop_in_progress {
            if let Ok(data) = receiver.try_recv() {
                self.crop_in_progress = None;
                self.cropped = Some(data);
                frame.request_repaint();
            }
        }

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
                        // can I run something like this in a thread?
                        let (sender, receiver) = std::sync::mpsc::channel();
                        self.crop_in_progress = Some(receiver);
                        // will this thread die by itself?
                        thread::spawn(move || {
                            sender
                                .send(utils::get_cropped_images(&path).unwrap())
                                .unwrap();
                        });
                    }
                }
                if let Some(picked_path) = &self.picked_path {
                    ui.label(picked_path);
                }
            });
            if self.crop_in_progress.is_some() {
                ui.label("cropping images, please wait...");
            }
            if let Some(cropped) = &self.cropped {
                ui.label(format!("{} images", cropped.len()));
            }
        });
        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}
