use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::parsers::parse_skill_update;
use crate::survey_parser::SurveyParser;

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
        let mut survey_parser = SurveyParser::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Ok(_) => {
                    let l = line.trim_end().to_string();
                    emit_events(&app, &l, &mut survey_parser);
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
        let mut survey_parser = SurveyParser::new();

        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let l = l.trim_end().to_string();
                    emit_events(&app, &l, &mut survey_parser);
                }
                Err(e) => eprintln!("Read error: {}", e),
            }
        }
    });

    Ok(())
}

/// Central dispatch: parse one line for all event types and emit any that fire
fn emit_events(app: &AppHandle, line: &str, survey_parser: &mut SurveyParser) {
    if let Some(update) = parse_skill_update(line) {
        app.emit("skill-update", update).ok();
    }
    for event in survey_parser.process_line(line) {
        app.emit("survey-event", event).ok();
    }
}