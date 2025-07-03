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
}

impl ExtractionProgress {
    pub fn percent(&self) -> u32 {
        if self.total == 0 { 0 } else { ((self.extracted as f32 / self.total as f32) * 100.0) as u32 }
    }
}

pub fn extract_zip_recursive(zip_path: &Path, target_dir: &Path, progress: &Arc<Mutex<ExtractionProgress>>) -> Result<()> {
    let file = File::open(zip_path).with_context(|| format!("Не удалось открыть ZIP: {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file).with_context(|| "Не удалось прочитать архив как ZIP")?;
    let total = archive.len();
    {
        let mut p = progress.lock().unwrap();
        p.total += total;
    }
    let zip_name = zip_path.file_stem().unwrap_or_default().to_string_lossy();
    let out_dir = target_dir.join(&*zip_name);
    fs::create_dir_all(&out_dir).with_context(|| format!("Не удалось создать папку: {}", out_dir.display()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = out_dir.join(file.name());
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
            // Если это ZIP внутри, рекурсивно извлечь
            if outpath.extension().map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false) {
                extract_zip_recursive(&outpath, &out_dir, progress)?;
            }
        }
        let mut p = progress.lock().unwrap();
        p.extracted += 1;
    }
    Ok(())
} 