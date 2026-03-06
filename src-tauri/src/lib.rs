use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

// The event payload Vue will receive
#[derive(serde::Serialize, Clone)]
struct SkillUpdate {
    raw_line: String,
    skill_type: String,
    level: u32,
    xp: u32,
    tnl: u32,
}

// Called from Vue to start watching a log file
#[tauri::command]
async fn start_watching(path: String, app: AppHandle) -> Result<(), String> {
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
                    // No new data, wait and try again
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Ok(_) => {
                    let l = line.trim_end().to_string();
                    if let Some(update) = parse_skill_update(&l) {
                        // Emit the event to Vue
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

fn parse_skill_update(line: &str) -> Option<SkillUpdate> {
    // Looking for: ProcessUpdateSkill({type=Geology,raw=46,bonus=4,xp=348,tnl=2000,max=50}
    if !line.contains("ProcessUpdateSkill") {
        return None;
    }

    // Pull out the fields with simple string parsing for now
    let skill_type = extract_field(line, "type=")?;
    let level: u32  = extract_field(line, "raw=")?.parse().ok()?;
    let xp: u32     = extract_field(line, "xp=")?.parse().ok()?;
    let tnl: u32    = extract_field(line, "tnl=")?.parse().ok()?;

    Some(SkillUpdate {
        raw_line: line.to_string(),
        skill_type,
        level,
        xp,
        tnl,
    })
}

// Extracts the value after `key` up to the next comma or `}`
fn extract_field(line: &str, key: &str) -> Option<String> {
    let start = line.find(key)? + key.len();
    let rest = &line[start..];
    let end = rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())  
        .invoke_handler(tauri::generate_handler![start_watching])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}