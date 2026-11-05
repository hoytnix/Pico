use crate::models::BuildError;
use image::imageops::FilterType;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize, String> {
    if !src.exists() {
        return Ok(0);
    }
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;
    let mut copied = 0usize;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let rel = p.strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(p, &target).map_err(|e| e.to_string())?;
            copied += 1;
        }
    }
    Ok(copied)
}

pub fn generate_webp_variants_for_file(
    src_path: &Path,
    out_dir_for_image: &Path,
    rel_display: &str,
) -> Result<usize, BuildError> {
    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    // SVG, GIF, AVIF are copied unchanged in MVP (§6.7)
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
        return Ok(0);
    }

    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let img = image::open(src_path).map_err(|e| BuildError {
        file: rel_display.to_string(),
        line: 1,
        message: format!("Failed to decode image '{}': {}", rel_display, e),
    })?;

    fs::create_dir_all(out_dir_for_image).map_err(|e| BuildError {
        file: rel_display.to_string(),
        line: 1,
        message: format!("Failed to create output image folder: {}", e),
    })?;

    // 1. Hero variant: max 1200px wide (Lanczos3)
    let hero_img = if img.width() > 1200 {
        img.resize(1200, u32::MAX, FilterType::Lanczos3)
    } else {
        img.clone()
    };
    let hero_path = out_dir_for_image.join(format!("{}-hero.webp", stem));
    hero_img
        .save_with_format(&hero_path, image::ImageFormat::WebP)
        .map_err(|e| BuildError {
            file: rel_display.to_string(),
            line: 1,
            message: format!("Failed to write WebP hero variant for '{}': {}", rel_display, e),
        })?;

    // Also save `<stem>.webp` so direct `hero.webp` links work seamlessly
    let direct_webp = out_dir_for_image.join(format!("{}.webp", stem));
    if !direct_webp.exists() {
        let _ = hero_img.save_with_format(&direct_webp, image::ImageFormat::WebP);
    }

    // 2. Card thumb variant: max 600px wide (Lanczos3)
    let thumb_img = if img.width() > 600 {
        img.resize(600, u32::MAX, FilterType::Lanczos3)
    } else {
        img
    };
    let thumb_path = out_dir_for_image.join(format!("{}-thumb.webp", stem));
    thumb_img
        .save_with_format(&thumb_path, image::ImageFormat::WebP)
        .map_err(|e| BuildError {
            file: rel_display.to_string(),
            line: 1,
            message: format!(
                "Failed to write WebP thumbnail variant for '{}': {}",
                rel_display, e
            ),
        })?;

    Ok(2)
}
