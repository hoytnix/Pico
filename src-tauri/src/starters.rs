use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub static EMBEDDED_STARTERS: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../starters");

pub fn resolve_starters_dir() -> Result<PathBuf, String> {
    let mut candidates = vec![
        PathBuf::from("starters"),
        PathBuf::from("../starters"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../starters"),
    ];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("starters"));
            candidates.push(exe_dir.join("../Resources/starters"));
            candidates.push(exe_dir.join("../lib/pico/starters"));
        }
    }

    for c in candidates {
        if c.is_dir() {
            return Ok(c);
        }
    }
    Err("Could not locate external starter templates directory on disk.".to_string())
}

fn extract_embedded_dir_recursive(
    dir: &include_dir::Dir<'_>,
    dest: &Path,
    prefix: &Path,
) -> std::io::Result<()> {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(sub_dir) => {
                let rel = sub_dir.path().strip_prefix(prefix).unwrap_or(sub_dir.path());
                fs::create_dir_all(dest.join(rel))?;
                extract_embedded_dir_recursive(sub_dir, dest, prefix)?;
            }
            include_dir::DirEntry::File(file) => {
                let rel = file.path().strip_prefix(prefix).unwrap_or(file.path());
                if let Some(parent) = dest.join(rel).parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(dest.join(rel), file.contents())?;
            }
        }
    }
    Ok(())
}

pub fn extract_embedded_starter(
    starter: &include_dir::Dir<'_>,
    dest: &Path,
    starter_key: &str,
) -> std::io::Result<()> {
    let prefix = Path::new(starter_key);
    extract_embedded_dir_recursive(starter, dest, prefix)
}

pub fn copy_starter_pack(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Could not create site directory: {}", e))?;
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
        }
    }
    Ok(())
}
