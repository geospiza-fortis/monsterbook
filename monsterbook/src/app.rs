use super::assets;
use super::book::get_color;
use super::layout::WIN_HD;
use super::pipeline::{self, imsave};
use super::session::Session;
use super::vision::Image;
use eframe::{egui, epi};
use rfd::FileDialog;
use std::fs;
use std::sync::mpsc::Receiver;
use std::thread;

#[derive(Default)]
pub struct App {
    session: Session,
    picked_path: Option<String>,
    ingest_in_progress: Option<Receiver<(Session, Vec<String>)>>,
    messages: Vec<String>,
    stitched: Option<(Image, String)>,
    stitch_in_progress: Option<Receiver<(Image, String)>>,
    cards_per_row: u32,
    stitch_counter: u32,
    tex_mngr: TexMngr,
}

impl App {
    fn ingest_bytes(&mut self, name: &str, bytes: &[u8]) {
        match self.session.add_screenshot(bytes) {
            Ok(page_id) => self.messages.push(format!("{}: page {}", name, page_id)),
            Err(err) => self.messages.push(format!("{}: {}", name, err)),
        }
    }

    fn paste_clipboard(&mut self) {
        let image = arboard::Clipboard::new().and_then(|mut cb| cb.get_image());
        match image {
            Ok(data) => {
                let result = self.session.add_bitmap(
                    data.width as u32,
                    data.height as u32,
                    data.bytes.into_owned(),
                );
                match result {
                    Ok(page_id) => self.messages.push(format!("clipboard: page {}", page_id)),
                    Err(err) => self.messages.push(format!("clipboard: {}", err)),
                }
            }
            Err(_) => self.messages.push("clipboard: no image".into()),
        }
    }
}

impl<'a> epi::App for App {
    fn name(&self) -> &str {
        "Monsterbook Stitcher"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if self.cards_per_row == 0 {
            self.cards_per_row = 30;
        }

        if let Some(receiver) = &self.ingest_in_progress {
            if let Ok((session, mut messages)) = receiver.try_recv() {
                self.ingest_in_progress = None;
                self.session.merge(session);
                self.messages.append(&mut messages);
            }
        }

        if let Some(receiver) = &self.stitch_in_progress {
            if let Ok(data) = receiver.try_recv() {
                self.stitch_in_progress = None;
                self.stitched = Some(data);
            }
        }

        // drag-and-drop ingestion: dropped files carry bytes (web) or a path
        // (native)
        let dropped: Vec<egui::DroppedFile> = ctx.input().raw.dropped_files.clone();
        for file in dropped {
            if let Some(bytes) = &file.bytes {
                self.ingest_bytes(&file.name.clone(), bytes.clone().as_ref());
            } else if let Some(path) = &file.path {
                let name = path.display().to_string();
                match fs::read(path) {
                    Ok(bytes) => self.ingest_bytes(&name, &bytes),
                    Err(err) => self.messages.push(format!("{}: {}", name, err)),
                }
            }
        }

        // clipboard paste ingestion
        let paste = {
            let input = ctx.input();
            input.modifiers.command && input.key_pressed(egui::Key::V)
        };
        if paste {
            self.paste_clipboard();
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
            ui.label("Drop screenshots here, paste with Ctrl+V, or open a directory.");
            ui.horizontal(|ui| {
                if ui.button("Open directory...").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.picked_path = Some(path.display().to_string());
                        let (sender, receiver) = std::sync::mpsc::channel();
                        self.ingest_in_progress = Some(receiver);
                        thread::spawn(move || {
                            let mut session = Session::new();
                            let mut messages = Vec::new();
                            let mut paths: Vec<_> = fs::read_dir(&path)
                                .map(|dir| dir.filter_map(|e| e.ok().map(|e| e.path())).collect())
                                .unwrap_or_default();
                            paths.sort();
                            for path in paths {
                                let name = path.display().to_string();
                                let result = fs::read(&path)
                                    .map_err(|e| e.to_string())
                                    .and_then(|bytes| {
                                        session
                                            .add_screenshot(&bytes)
                                            .map_err(|e| e.to_string())
                                    });
                                match result {
                                    Ok(page_id) => {
                                        messages.push(format!("{}: page {}", name, page_id))
                                    }
                                    Err(err) => messages.push(format!("{}: {}", name, err)),
                                }
                            }
                            sender.send((session, messages)).unwrap();
                        });
                    }
                }
                if let Some(picked_path) = &self.picked_path {
                    ui.label(picked_path);
                }
            });

            if self.ingest_in_progress.is_some() {
                ui.label("ingesting images, please wait...");
            }

            // per-page progress: one colored square per page, dimmed when
            // the page has not been ingested. page 22 (gold_2) is excluded
            // from missing() (no reference exists) but shown for
            // completeness.
            let missing = self.session.missing();
            ui.horizontal_wrapped(|ui| {
                for page in assets::BOOK.pages.iter() {
                    let rgba = get_color(&page.tab_color);
                    let ingested = self.session.pages().contains_key(&page.page_id);
                    let color = if ingested {
                        egui::Color32::from_rgb(rgba[0], rgba[1], rgba[2])
                    } else {
                        egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], 40)
                    };
                    ui.colored_label(color, "■")
                        .on_hover_text(format!("page {} ({})", page.page_id, page.tab_color));
                }
                ui.label(format!("{} missing", missing.len()));
            });

            if let Some(message) = self.messages.last() {
                ui.label(message);
            }

            ui.horizontal(|ui| {
                ui.label("Cards per row");
                ui.add(egui::Slider::new(&mut self.cards_per_row, 10..=100));
            });
            let has_pages = !self.session.pages().is_empty();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(has_pages, egui::Button::new("Generate stitched image..."))
                    .clicked()
                {
                    let (sender, receiver) = std::sync::mpsc::channel();
                    self.stitch_in_progress = Some(receiver);
                    // clone the ingested pages to move them into the thread
                    let pages: Vec<(usize, Image)> = self
                        .session
                        .pages()
                        .iter()
                        .map(|(&id, page)| (id, page.clone()))
                        .collect();
                    let cards_per_row = self.cards_per_row;
                    self.stitch_counter += 1;
                    let tag = format!("stitch-{}-{}", self.stitch_counter, cards_per_row);
                    thread::spawn(move || {
                        let iter = pages.iter().map(|(id, page)| (*id, page));
                        if let Some(image) =
                            pipeline::stitch_page_cards(iter, cards_per_row, &WIN_HD)
                        {
                            sender.send((image, tag)).unwrap();
                        }
                    });
                }
                if ui
                    .add_enabled(has_pages, egui::Button::new("Transcribe..."))
                    .clicked()
                {
                    if let Some(path) = FileDialog::new().add_filter("json", &["json"]).save_file()
                    {
                        let entries = self.session.transcribe();
                        let doc = serde_json::json!({ "data": entries });
                        if let Ok(text) = serde_json::to_string_pretty(&doc) {
                            let _ = fs::write(&path, text);
                        }
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

            if self.stitch_in_progress.is_some() {
                ui.label("stitching, please wait...");
            }
            if let Some((stitched, path)) = &self.stitched {
                let image = decode_image(stitched.clone()).unwrap();
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
