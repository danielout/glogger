use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::parsers::parse_skill_update;

#[tauri::command]
pub async fn start_watching(path: String, app: AppHandle) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    tokio::spawn(async move {
        let mut file = File::open(&path).expect("could not open log file");
        file.seek(SeekFrom::End(0)).expect("could not seek to end");
        let mut reader = BufReader::new(file);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Ok(_) => {
                    let l = line.trim_end().to_string();
                    if let Some(update) = parse_skill_update(&l) {
                        app.emit("skill-update", update).ok();
                    }
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn parse_log(path: String, app: AppHandle) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    tokio::spawn(async move {
        let file = File::open(&path).expect("could not open log file");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let l = l.trim_end().to_string();
                    if let Some(update) = parse_skill_update(&l) {
                        app.emit("skill-update", update).ok();
                    }
                }
                Err(e) => eprintln!("Read error: {}", e),
            }
        }
    });

    Ok(())
}