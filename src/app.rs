use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use crate::extractor::{extract_zip_recursive, ExtractionProgress};
use eframe::App;
use std::sync::{Arc, Mutex};

pub struct NestedZipApp {
    zip_path: Option<PathBuf>,
    target_dir: Option<PathBuf>,
    progress: Arc<Mutex<ExtractionProgress>>,
    status: String,
    error: Option<String>,
    extracting: bool,
    keep_original: bool,
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
            keep_original: true,
        }
    }
}

impl App for NestedZipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nested ZIP Extractor");
            ui.separator();
            ui.label("Иногда надо распаковать архивы в архивах. Например при анализе выгрузки объектов из Informatica Cloud. Та еще боль.");
            ui.label("Это приложение поможет вам распаковать архивы в архивах.");
            ui.label("Выберите ZIP-файл и папку для распаковки.");
            ui.label("Нажмите на кнопку \"Извлечь\" и подождите пока все файлы будут распакованы.");
            ui.label("Если вы хотите сохранить оригинальный ZIP файл, включите опцию \"Сохранить оригинальный ZIP файл\".");
            ui.label("Если вы хотите очистить лог операций, нажмите на кнопку \"Очистить лог\".");
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
            
            // Опция сохранения оригинального файла
            ui.checkbox(&mut self.keep_original, "Сохранить оригинальный ZIP файл");
            ui.label("Если включено, оригинальный ZIP файл останется в исходной папке");

            ui.separator();
            let (percent, extracted, total, done, log_messages) = {
                let progress = self.progress.lock().unwrap();
                (progress.percent(), progress.extracted, progress.total, progress.done, progress.log.clone())
            };
            if self.extracting {
                // Вывод прогресса в консоль
                println!("Прогресс: {}/{} ({}%)", extracted, total, percent);
                ui.label(format!("Извлечение: {}/{} ({}%)", extracted, total, percent));
                ui.add(egui::ProgressBar::new(percent as f32 / 100.0).show_percentage());
                
                // Показываем текущую операцию из последнего сообщения лога
                if let Some(last_message) = log_messages.last() {
                    ui.label(format!("Текущая операция: {}", last_message));
                }
                
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
                    let keep_original = self.keep_original;
                    // Сброс прогресса
                    {
                        let mut p = progress.lock().unwrap();
                        *p = crate::extractor::ExtractionProgress::default();
                    }
                    std::thread::spawn(move || {
                        let res = extract_zip_recursive(&zip, &target, &progress, keep_original);
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
            
            // Область для отображения лога операций
            ui.separator();
            ui.heading("Лог операций");
            
            // Кнопка очистки лога
            if ui.button("Очистить лог").clicked() {
                let mut progress = self.progress.lock().unwrap();
                progress.log.clear();
            }
            
            // Область прокрутки для лога
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (i, message) in log_messages.iter().enumerate() {
                        ui.label(format!("[{}] {}", i + 1, message));
                    }
                });
        });
        ctx.request_repaint();
    }
} 