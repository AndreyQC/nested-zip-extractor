fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Nested ZIP Extractor",
        native_options,
        Box::new(|_cc| Box::new(nested_zip_extractor::app::NestedZipApp::default())),
    ).expect("Ошибка запуска приложения");
} 