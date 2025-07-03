use std::fs::{self, File};
use std::io::{self};
use std::path::Path;
use zip::read::ZipArchive;
use anyhow::{Result, Context};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ExtractionProgress {
    pub total: usize,
    pub extracted: usize,
    pub done: bool,
    pub error: Option<String>,
    pub log: Vec<String>,
}

impl ExtractionProgress {
    pub fn percent(&self) -> u32 {
        if self.total == 0 { 0 } else { ((self.extracted as f32 / self.total as f32) * 100.0) as u32 }
    }
}

macro_rules! log_to {
    ($progress:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        let mut guard = $progress.lock().unwrap();
        guard.log.push(msg.clone());
        println!("{}", msg);
    }}
}

pub fn extract_zip_recursive(zip_path: &Path, target_dir: &Path, progress: &Arc<Mutex<ExtractionProgress>>, keep_original: bool) -> Result<()> {
    log_to!(progress, "Начинаю извлечение архива: {}", zip_path.display());
    
    let file = File::open(zip_path).with_context(|| format!("Не удалось открыть ZIP: {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file).with_context(|| "Не удалось прочитать архив как ZIP")?;
    let total = archive.len();
    
    {
        let mut p = progress.lock().unwrap();
        p.total += total;
    }
    
    let zip_name = zip_path.file_stem().unwrap_or_default().to_string_lossy();
    let out_dir = target_dir.join(&*zip_name);
    
    log_to!(progress, "Создаю директорию: {}", out_dir.display());
    fs::create_dir_all(&out_dir).with_context(|| format!("Не удалось создать папку: {}", out_dir.display()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = out_dir.join(file.name());
        
        if file.name().ends_with('/') {
            log_to!(progress, "Создаю директорию: {}", outpath.display());
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
            log_to!(progress, "Извлечен файл: {}", outpath.display());
            
            // Если это ZIP внутри, рекурсивно извлечь
            if outpath.extension().map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false) {
                log_to!(progress, "Обнаружен вложенный ZIP: {}", outpath.display());
                extract_zip_recursive(&outpath, &out_dir, progress, false)?;
                
                // Удаляем извлеченный вложенный ZIP файл после извлечения
                log_to!(progress, "Удаляю извлеченный вложенный ZIP файл: {}", outpath.display());
                if let Err(e) = fs::remove_file(&outpath) {
                    log_to!(progress, "Ошибка при удалении файла {}: {}", outpath.display(), e);
                }
            }
        }
        let mut p = progress.lock().unwrap();
        p.extracted += 1;
    }
    
    // Удаляем оригинальный ZIP файл только если keep_original = false
    if !keep_original {
        log_to!(progress, "Удаляю оригинальный ZIP файл: {}", zip_path.display());
        if let Err(e) = fs::remove_file(zip_path) {
            log_to!(progress, "Ошибка при удалении оригинального файла {}: {}", zip_path.display(), e);
        }
    } else {
        log_to!(progress, "Сохраняю оригинальный ZIP файл: {}", zip_path.display());
    }
    
    Ok(())
} 