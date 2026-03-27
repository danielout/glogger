/// Tauri commands for chat functionality
use crate::chat_parser;
use crate::db::{DbPool, chat_commands};
use crate::db::queries::log_positions;
use crate::settings::SettingsManager;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{State, AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct ScanProgress {
    current: usize,
    total: usize,
    file_name: String,
}

#[tauri::command]
pub async fn scan_chat_logs(
    path: String,
    db_pool: State<'_, DbPool>,
    settings: State<'_, Arc<SettingsManager>>,
    app: AppHandle,
) -> Result<ScanResult, String> {
    let chat_logs_path = PathBuf::from(&path);
    let excluded_channels = settings.get().excluded_chat_channels;

    let log_files = chat_parser::get_chat_log_files(&chat_logs_path)
        .map_err(|e| format!("Failed to scan chat logs directory: {e}"))?;

    let total_files = log_files.len();
    let mut total_messages = 0;
    let mut files_processed = 0;

    for (index, log_file) in log_files.iter().enumerate() {
        let _ = app.emit("chat-scan-progress", ScanProgress {
            current: index + 1,
            total: total_files,
            file_name: log_file.file_name.clone(),
        });

        let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

        let file_path_str = log_file.file_path.to_string_lossy().to_string();
        let start_position = log_positions::get_position(&conn, &file_path_str)
            .map_err(|e| format!("Failed to get log position: {e}"))?;

        let player_name = if start_position == 0 {
            chat_parser::extract_player_name(&log_file.file_path)
                .map_err(|e| format!("Failed to extract player name: {e}"))?
        } else {
            None
        };

        let (messages, new_position) = chat_parser::read_chat_log(
            &log_file.file_path,
            start_position,
        ).map_err(|e| format!("Failed to read chat log: {e}"))?;

        if !messages.is_empty() {
            let inserted = chat_commands::insert_chat_messages(&conn, &messages, &log_file.file_name, &excluded_channels)
                .map_err(|e| format!("Failed to insert messages: {e}"))?;
            total_messages += inserted;
        }

        files_processed += 1;

        let metadata = serde_json::json!({
            "file_name": log_file.file_name,
            "file_date": log_file.file_date.format("%Y-%m-%d").to_string()
        }).to_string();

        log_positions::update_position(
            &conn,
            &file_path_str,
            "chat",
            new_position,
            player_name.as_deref(),
            Some(&metadata),
        ).map_err(|e| format!("Failed to update log position: {e}"))?;
    }

    Ok(ScanResult {
        files_processed,
        messages_imported: total_messages,
    })
}

#[derive(serde::Serialize)]
pub struct ScanResult {
    pub files_processed: usize,
    pub messages_imported: usize,
}

#[tauri::command]
pub async fn scan_chat_log_file(
    path: String,
    db_pool: State<'_, DbPool>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<ScanResult, String> {
    let log_path = PathBuf::from(&path);
    let excluded_channels = settings.get().excluded_chat_channels;

    if !log_path.exists() {
        return Err(format!("Chat log file not found: {}", path));
    }

    let file_name = log_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file name")?
        .to_string();

    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    let file_path_str = log_path.to_string_lossy().to_string();
    let start_position = log_positions::get_position(&conn, &file_path_str)
        .map_err(|e| format!("Failed to get log position: {e}"))?;

    let player_name = if start_position == 0 {
        chat_parser::extract_player_name(&log_path)
            .map_err(|e| format!("Failed to extract player name: {e}"))?
    } else {
        None
    };

    let (messages, new_position) = chat_parser::read_chat_log(
        &log_path,
        start_position,
    ).map_err(|e| format!("Failed to read chat log: {e}"))?;

    let mut messages_imported = 0;
    if !messages.is_empty() {
        messages_imported = chat_commands::insert_chat_messages(&conn, &messages, &file_name, &excluded_channels)
            .map_err(|e| format!("Failed to insert messages: {e}"))?;
    }

    let file_date = chat_parser::parse_chat_log_filename(&file_name);
    let metadata = file_date.map(|d| {
        serde_json::json!({
            "file_name": file_name,
            "file_date": d.format("%Y-%m-%d").to_string()
        }).to_string()
    });

    log_positions::update_position(
        &conn,
        &file_path_str,
        "chat",
        new_position,
        player_name.as_deref(),
        metadata.as_deref(),
    ).map_err(|e| format!("Failed to update log position: {e}"))?;

    Ok(ScanResult {
        files_processed: 1,
        messages_imported,
    })
}

#[tauri::command]
pub async fn get_chat_messages(
    channel: Option<String>,
    sender: Option<String>,
    search_text: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    has_item_links: Option<bool>,
    item_name: Option<String>,
    tell_partner: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    db_pool: State<'_, DbPool>,
) -> Result<Vec<chat_commands::ChatMessageRow>, String> {
    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    let filter = chat_commands::ChatMessageFilter {
        channel,
        sender,
        search_text,
        start_time,
        end_time,
        has_item_links,
        item_name,
        tell_partner,
        limit: limit.unwrap_or(100),
        offset: offset.unwrap_or(0),
    };

    chat_commands::get_chat_messages(&conn, &filter)
        .map_err(|e| format!("Failed to get messages: {e}"))
}

#[tauri::command]
pub async fn get_chat_channels(
    db_pool: State<'_, DbPool>,
) -> Result<Vec<String>, String> {
    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    chat_commands::get_channels(&conn)
        .map_err(|e| format!("Failed to get channels: {e}"))
}

#[tauri::command]
pub async fn get_chat_channel_stats(
    db_pool: State<'_, DbPool>,
) -> Result<Vec<ChannelStat>, String> {
    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    let stats = chat_commands::get_channel_stats(&conn)
        .map_err(|e| format!("Failed to get channel stats: {e}"))?;

    Ok(stats
        .into_iter()
        .map(|(channel, count)| ChannelStat { channel, count })
        .collect())
}

#[derive(serde::Serialize)]
pub struct ChannelStat {
    pub channel: String,
    pub count: i64,
}

#[tauri::command]
pub async fn tail_chat_log(
    chat_log_file: String,
    db_pool: State<'_, DbPool>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<Vec<chat_commands::ChatMessageRow>, String> {
    let log_path = PathBuf::from(&chat_log_file);
    let excluded_channels = settings.get().excluded_chat_channels;

    if !log_path.exists() {
        return Err(format!("Chat log file not found: {}", chat_log_file));
    }

    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    let file_path_str = log_path.to_string_lossy().to_string();
    let start_position = log_positions::get_position(&conn, &file_path_str)
        .map_err(|e| format!("Failed to get log position: {e}"))?;

    let (messages, new_position) = chat_parser::read_chat_log(
        &log_path,
        start_position,
    ).map_err(|e| format!("Failed to read chat log: {e}"))?;

    if messages.is_empty() {
        return Ok(Vec::new());
    }

    let file_name = log_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file name")?;

    let player_name = if start_position == 0 {
        chat_parser::extract_player_name(&log_path)
            .map_err(|e| format!("Failed to extract player name: {e}"))?
    } else {
        None
    };

    chat_commands::insert_chat_messages(&conn, &messages, file_name, &excluded_channels)
        .map_err(|e| format!("Failed to insert messages: {e}"))?;

    let file_date = chat_parser::parse_chat_log_filename(file_name)
        .ok_or("Invalid chat log filename format")?;

    let metadata = serde_json::json!({
        "file_name": file_name,
        "file_date": file_date.format("%Y-%m-%d").to_string()
    }).to_string();

    log_positions::update_position(
        &conn,
        &file_path_str,
        "chat",
        new_position,
        player_name.as_deref(),
        Some(&metadata),
    ).map_err(|e| format!("Failed to update log position: {e}"))?;

    // Convert to ChatMessageRow for return
    let rows: Vec<chat_commands::ChatMessageRow> = messages
        .iter()
        .enumerate()
        .map(|(i, msg)| chat_commands::ChatMessageRow {
            id: i as i64, // Temporary ID
            timestamp: msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            channel: msg.channel.clone(),
            sender: msg.sender.clone(),
            message: msg.message.clone(),
            is_system: msg.is_system,
            from_player: msg.from_player,
            item_links: msg.item_links.iter().map(|link| chat_commands::ChatItemLinkRow {
                raw_text: link.raw_text.clone(),
                item_name: link.item_name.clone(),
                item_id: None,
            }).collect(),
        })
        .collect();

    Ok(rows)
}

#[tauri::command]
pub async fn get_chat_stats(
    db_pool: State<'_, DbPool>,
) -> Result<chat_commands::ChatStats, String> {
    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    chat_commands::get_chat_stats(&conn)
        .map_err(|e| format!("Failed to get chat stats: {e}"))
}

#[tauri::command]
pub async fn get_tell_conversations(
    db_pool: State<'_, DbPool>,
) -> Result<Vec<ChannelStat>, String> {
    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    let conversations = chat_commands::get_tell_conversations(&conn)
        .map_err(|e| format!("Failed to get conversations: {e}"))?;

    Ok(conversations
        .into_iter()
        .map(|(channel, count)| ChannelStat { channel, count })
        .collect())
}

#[tauri::command]
pub async fn purge_chat_messages(
    days: u32,
    db_pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff_date.format("%Y-%m-%d %H:%M:%S").to_string();

    // Delete orphaned item links for messages that will be purged
    // (in case foreign_keys pragma wasn't active for older data)
    conn.execute(
        "DELETE FROM chat_item_links WHERE message_id IN (SELECT id FROM chat_messages WHERE timestamp < ?1)",
        [&cutoff_str],
    )
    .map_err(|e| format!("Failed to purge item links: {e}"))?;

    let deleted = conn.execute(
        "DELETE FROM chat_messages WHERE timestamp < ?1",
        [&cutoff_str],
    )
    .map_err(|e| format!("Failed to purge messages: {e}"))?;

    Ok(deleted)
}

#[tauri::command]
pub async fn delete_all_chat_messages(
    db_pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    // Delete item links first (in case foreign_keys pragma wasn't active for older data)
    conn.execute("DELETE FROM chat_item_links", [])
        .map_err(|e| format!("Failed to delete item links: {e}"))?;

    let deleted = conn.execute("DELETE FROM chat_messages", [])
        .map_err(|e| format!("Failed to delete all messages: {e}"))?;

    // Clear legacy chat log file tracking
    conn.execute("DELETE FROM chat_log_files", [])
        .map_err(|e| format!("Failed to delete chat log files: {e}"))?;

    // Reset file positions for chat logs so they can be re-imported
    conn.execute(
        "DELETE FROM log_file_positions WHERE file_type = 'chat'",
        [],
    )
    .map_err(|e| format!("Failed to reset file positions: {e}"))?;

    // Reclaim disk space
    conn.execute_batch("VACUUM;")
        .map_err(|e| format!("Failed to vacuum database: {e}"))?;

    Ok(deleted)
}

#[tauri::command]
pub async fn get_watch_rule_messages(
    rule_id: u64,
    limit: Option<i64>,
    offset: Option<i64>,
    db_pool: State<'_, DbPool>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<Vec<chat_commands::ChatMessageRow>, String> {
    let app_settings = settings.get();
    let rule = app_settings
        .watch_rules
        .iter()
        .find(|r| r.id == rule_id)
        .ok_or_else(|| format!("Watch rule {} not found", rule_id))?;

    let conn = db_pool.get().map_err(|e| format!("Database error: {e}"))?;

    chat_commands::get_watch_rule_messages(
        &conn,
        rule,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
        &app_settings.excluded_chat_channels,
    )
    .map_err(|e| format!("Failed to get watch rule messages: {e}"))
}
