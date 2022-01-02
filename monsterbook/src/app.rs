use super::crop::{imsave, Image};
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
    stitched: Option<(Image, String)>,
    stitch_in_progress: Option<Receiver<(Image, String)>>,
    cards_per_row: u32,
    tex_mngr: TexMngr,
}

impl<'a> epi::App for App {
    fn name(&self) -> &str {
        "Monsterbook Stitcher"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if self.cards_per_row == 0 {
            self.cards_per_row = 30;
        }

        if let Some(receiver) = &self.crop_in_progress {
            if let Ok(data) = receiver.try_recv() {
                self.crop_in_progress = None;
                self.cropped = Some(data);
            }
        }

        if let Some(receiver) = &self.stitch_in_progress {
            if let Ok(data) = receiver.try_recv() {
                self.stitch_in_progress = None;
                self.stitched = Some(data);
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
                        let (sender, receiver) = std::sync::mpsc::channel();
                        self.crop_in_progress = Some(receiver);
                        thread::spawn(move || {
                            let images = utils::get_cropped_images(&path).unwrap();
                            sender.send(images).unwrap();
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
            ui.horizontal(|ui| {
                ui.label("Cards per row");
                ui.add(egui::Slider::new(&mut self.cards_per_row, 10..=100));
            });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        self.cropped.is_some(),
                        egui::Button::new("Generate stitched image..."),
                    )
                    .clicked()
                {
                    if let Some(cropped) = &self.cropped {
                        let (sender, receiver) = std::sync::mpsc::channel();
                        self.stitch_in_progress = Some(receiver);
                        // we have to make clones in order to move the values
                        // into a thread
                        let cards_per_row = self.cards_per_row;
                        let path =
                            format!("{}/{}", self.picked_path.as_ref().unwrap(), cards_per_row);
                        let cloned = cropped.clone();
                        thread::spawn(move || {
                            // this path should be unique enough to update the current texture
                            let image = utils::stitch_cards(&cloned, cards_per_row);
                            sender.send((image, String::from(path))).unwrap();
                        });
                    }
                }
                if let Some((stitched, _)) = &self.stitched {
                    if ui.button("Save image").clicked() {
                        if let Some(path) =
                            FileDialog::new().add_filter("png", &["png"]).save_file()
                        {
                            imsave(&path, stitched).unwrap();
                        }
                    }
                }
            });

            if let Some((stitched, path)) = &self.stitched {
                let image = decode_image(stitched.clone()).unwrap();
                //let image = decode_image(self.cropped.as_ref().unwrap()[0].as_raw()).unwrap();
                if let Some(texture_id) = self.tex_mngr.texture(frame, &path, &image) {
                    let size = egui::Vec2::new(image.size[0] as f32, image.size[1] as f32);
                    ui.image(texture_id, size);
                }
            }
        });
        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}

#[derive(Default)]
struct TexMngr {
    loaded_url: String,
    texture_id: Option<egui::TextureId>,
}

impl TexMngr {
    fn texture(
        &mut self,
        frame: &epi::Frame,
        url: &str,
        image: &epi::Image,
    ) -> Option<egui::TextureId> {
        if self.loaded_url != url {
            if let Some(texture_id) = self.texture_id.take() {
                frame.free_texture(texture_id);
            }

            self.texture_id = Some(frame.alloc_texture(image.clone()));
            self.loaded_url = url.to_owned();
        }
        self.texture_id
    }
}

fn decode_image(image_buffer: Image) -> Option<epi::Image> {
    let size = [
        image_buffer.width() as usize,
        image_buffer.height() as usize,
    ];
    let pixels = image_buffer.into_vec();
    Some(epi::Image::from_rgba_unmultiplied(size, &pixels))
}
