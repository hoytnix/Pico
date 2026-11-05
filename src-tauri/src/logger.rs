use log::{Level, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct PicoFileLogger {
    file: Mutex<Option<File>>,
    log_path: PathBuf,
}

impl PicoFileLogger {
    pub fn new(log_path: PathBuf) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        Self {
            file: Mutex::new(file),
            log_path,
        }
    }

    pub fn write_line(&self, line: &str) {
        eprint!("{}", line);
        let mut guard = self.file.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            if let Ok(f) = OpenOptions::new().create(true).append(true).open(&self.log_path) {
                *guard = Some(f);
            }
        }
        if let Some(file) = guard.as_mut() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}

impl log::Log for PicoFileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();
        // Keep hyper and h2 internal polling noise restrained to info/warn/error
        if target.starts_with("h2") || target.starts_with("hyper") {
            metadata.level() <= Level::Info
        } else {
            true
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let line = format!(
                "{} [{:<5}] [{}] {}\n",
                now,
                record.level().as_str(),
                record.target(),
                record.args()
            );
            self.write_line(&line);
        }
    }

    fn flush(&self) {
        let mut guard = self.file.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(file) = guard.as_mut() {
            let _ = file.flush();
        }
    }
}

pub fn log_file_path() -> PathBuf {
    let base = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join(".pico.log")
}

pub fn init_logger() {
    let path = log_file_path();
    let logger = PicoFileLogger::new(path.clone());

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let banner = format!(
        "\n================================================================================\n\
         {} [INFO ] [pico::main] === Pico Application Starting ===\n\
         {} [INFO ] [pico::main] Log initialized at: {}\n\
         {} [INFO ] [pico::main] Process ID: {}, OS: {}\n\
         ================================================================================\n",
        now,
        now,
        path.to_string_lossy(),
        now,
        std::process::id(),
        std::env::consts::OS
    );
    logger.write_line(&banner);

    let _ = log::set_boxed_logger(Box::new(logger));
    log::set_max_level(log::LevelFilter::Trace);

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };

        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };

        log::error!(target: "pico::panic", "CRITICAL PANIC at {}: {}", location, payload);

        default_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_log_file_path_resolution() {
        let p = log_file_path();
        let s = p.to_string_lossy();
        assert!(s.ends_with(".pico.log"), "Path should end with .pico.log, got: {}", s);
    }

    #[test]
    fn test_pico_file_logger_writes_and_flushes() {
        let dir = tempdir().unwrap();
        let log_file = dir.path().join(".pico.log");
        let logger = PicoFileLogger::new(log_file.clone());

        logger.write_line("Test log entry line 1\n");
        logger.write_line("Test log entry line 2\n");

        let content = std::fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("Test log entry line 1"));
        assert!(content.contains("Test log entry line 2"));
    }
}
