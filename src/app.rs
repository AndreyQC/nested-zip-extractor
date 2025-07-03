use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use crate::extractor::{extract_zip_recursive, ExtractionProgress};
use eframe::App;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct NestedZipApp {
    zip_path: Option<PathBuf>,
    target_dir: Option<PathBuf>,
    progress: Arc<Mutex<ExtractionProgress>>,
    status: String,
    error: Option<String>,
    extracting: bool,
}

impl Default for NestedZipApp {
    fn default() -> Self {
        Self {
            zip_path: None,
            target_dir: None,
            progress: Arc::new(Mutex::new(ExtractionProgress::default())),
            status: "Ожидание действий пользователя".to_string(),
            error: None,
            extracting: false,
        }
    }
}

impl App for NestedZipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nested ZIP Extractor");
            ui.separator();

            if ui.button("Выбрать ZIP-файл").clicked() {
                if let Some(path) = FileDialog::new().add_filter("ZIP", &["zip"]).pick_file() {
                    self.zip_path = Some(path);
                }
            }
            if let Some(path) = &self.zip_path {
                ui.label(format!("Выбранный ZIP: {}", path.display()));
            }

            if ui.button("Выбрать папку для распаковки").clicked() {
                if let Some(dir) = FileDialog::new().pick_folder() {
                    self.target_dir = Some(dir);
                }
            }
            if let Some(dir) = &self.target_dir {
                ui.label(format!("Папка назначения: {}", dir.display()));
            }

            ui.separator();
            let (percent, extracted, total, done) = {
                let progress = self.progress.lock().unwrap();
                (progress.percent(), progress.extracted, progress.total, progress.done)
            };
            if self.extracting {
                // Вывод прогресса в консоль
                println!("Прогресс: {}/{} ({}%)", extracted, total, percent);
                ui.label(format!("Извлечение: {}/{} ({}%)", extracted, total, percent));
                ui.add(egui::ProgressBar::new(percent as f32 / 100.0).show_percentage());
                if done {
                    self.extracting = false;
                    self.status = "Извлечение завершено".to_string();
                }
            } else if ui.button("Извлечь").clicked() {
                if let (Some(zip), Some(target)) = (self.zip_path.clone(), self.target_dir.clone()) {
                    self.status = "Извлечение...".to_string();
                    self.error = None;
                    self.extracting = true;
                    let progress = self.progress.clone();
                    // Сброс прогресса
                    {
                        let mut p = progress.lock().unwrap();
                        *p = crate::extractor::ExtractionProgress::default();
                    }
                    std::thread::spawn(move || {
                        let res = extract_zip_recursive(&zip, &target, &progress);
                        let mut p = progress.lock().unwrap();
                        p.done = true;
                        if let Err(e) = res {
                            p.error = Some(format!("Ошибка: {e}"));
                        }
                    });
                } else {
                    self.error = Some("Выберите ZIP-файл и папку назначения".to_string());
                }
            }

            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, err);
            }
            ui.label(&self.status);
        });
        ctx.request_repaint();
    }
} 