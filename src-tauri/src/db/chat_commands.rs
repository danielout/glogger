use super::DbConnection;
/// Database commands for chat message operations
use crate::chat_parser::ChatMessage;
use crate::settings::{ConditionMatch, WatchCondition, WatchRule};
use rusqlite::{params, OptionalExtension, Result};

/// Insert a batch of chat messages into the database.
/// Messages on excluded channels are silently skipped — they must never be stored.
pub fn insert_chat_messages(
    conn: &DbConnection,
    messages: &[ChatMessage],
    log_file: &str,
    excluded_channels: &[String],
) -> Result<usize> {
    let mut inserted = 0;

    for msg in messages {
        // Never store messages from excluded channels
        if let Some(ref channel) = msg.channel {
            if excluded_channels.iter().any(|c| c == channel) {
                continue;
            }
        }

        // Insert the message - use INSERT OR IGNORE to handle duplicates gracefully
        let rows_affected = conn.execute(
            "INSERT OR IGNORE INTO chat_messages (timestamp, channel, sender, message, is_system, log_file, from_player)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                msg.channel,
                msg.sender,
                msg.message,
                msg.is_system,
                log_file,
                msg.from_player
            ],
        )?;

        // Skip if this was a duplicate (no rows inserted)
        if rows_affected == 0 {
            continue;
        }

        // Get the ID of the inserted message
        let message_id = conn.last_insert_rowid();

        // Insert item links if any
        for link in &msg.item_links {
            // Look up the item in the game data by name
            let item_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM items WHERE name = ?1 COLLATE NOCASE",
                    params![&link.item_name],
                    |row| row.get(0),
                )
                .optional()?;

            conn.execute(
                "INSERT INTO chat_item_links (message_id, raw_text, item_name, item_id)
                 VALUES (?1, ?2, ?3, ?4)",
                params![message_id, &link.raw_text, &link.item_name, item_id],
            )?;
        }

        inserted += 1;
    }

    Ok(inserted)
}

/// Get chat messages with optional filters
#[derive(Debug, Clone)]
pub struct ChatMessageFilter {
    pub channel: Option<String>,
    pub sender: Option<String>,
    pub search_text: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub has_item_links: Option<bool>,
    pub item_name: Option<String>,
    pub tell_partner: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for ChatMessageFilter {
    fn default() -> Self {
        Self {
            channel: None,
            sender: None,
            search_text: None,
            start_time: None,
            end_time: None,
            has_item_links: None,
            item_name: None,
            tell_partner: None,
            limit: 100,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatMessageRow {
    pub id: i64,
    pub timestamp: String,
    pub channel: Option<String>,
    pub sender: Option<String>,
    pub message: String,
    pub is_system: bool,
    pub from_player: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub item_links: Vec<ChatItemLinkRow>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatItemLinkRow {
    pub raw_text: String,
    pub item_name: String,
    pub item_id: Option<i64>,
}

pub fn get_chat_messages(
    conn: &DbConnection,
    filter: &ChatMessageFilter,
) -> Result<Vec<ChatMessageRow>> {
    // Build the query using a consistent approach for all filter combos
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut param_idx = 1;

    // Use FTS join if search_text is provided
    let use_fts = filter.search_text.is_some();

    // Tell partner filter: automatically sets channel to Tell and sender to partner
    if let Some(partner) = &filter.tell_partner {
        conditions.push(format!("cm.channel = ?{}", param_idx));
        params.push(Box::new("Tell".to_string()));
        param_idx += 1;

        conditions.push(format!("cm.sender = ?{}", param_idx));
        params.push(Box::new(partner.clone()));
        param_idx += 1;
    } else {
        if let Some(channel) = &filter.channel {
            conditions.push(format!("cm.channel = ?{}", param_idx));
            params.push(Box::new(channel.clone()));
            param_idx += 1;
        }

        if let Some(sender) = &filter.sender {
            conditions.push(format!("cm.sender = ?{}", param_idx));
            params.push(Box::new(sender.clone()));
            param_idx += 1;
        }
    }

    if let Some(start_time) = &filter.start_time {
        conditions.push(format!("cm.timestamp >= ?{}", param_idx));
        params.push(Box::new(start_time.clone()));
        param_idx += 1;
    }

    if let Some(end_time) = &filter.end_time {
        conditions.push(format!("cm.timestamp <= ?{}", param_idx));
        params.push(Box::new(end_time.clone()));
        param_idx += 1;
    }

    if let Some(has_links) = filter.has_item_links {
        if has_links {
            conditions.push(
                "EXISTS (SELECT 1 FROM chat_item_links cil WHERE cil.message_id = cm.id)"
                    .to_string(),
            );
        } else {
            conditions.push(
                "NOT EXISTS (SELECT 1 FROM chat_item_links cil WHERE cil.message_id = cm.id)"
                    .to_string(),
            );
        }
    }

    if let Some(item_name) = &filter.item_name {
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM chat_item_links cil WHERE cil.message_id = cm.id AND cil.item_name LIKE ?{})",
            param_idx
        ));
        params.push(Box::new(format!("%{}%", item_name)));
        param_idx += 1;
    }

    // Build the full query
    let from_clause = if use_fts {
        let search_text = filter.search_text.as_ref().unwrap();
        conditions.insert(0, format!("fts MATCH ?{}", param_idx));
        params.push(Box::new(search_text.clone()));
        // param_idx not needed after this
        "FROM chat_messages cm JOIN chat_messages_fts fts ON cm.id = fts.rowid".to_string()
    } else {
        "FROM chat_messages cm".to_string()
    };

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT cm.id, cm.timestamp, cm.channel, cm.sender, cm.message, cm.is_system, cm.from_player \
         {} {} ORDER BY cm.timestamp DESC LIMIT {} OFFSET {}",
        from_clause, where_clause, filter.limit, filter.offset
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(ChatMessageRow {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            channel: row.get(2)?,
            sender: row.get(3)?,
            message: row.get(4)?,
            is_system: row.get(5)?,
            from_player: row.get(6)?,
            item_links: Vec::new(),
        })
    })?;

    let mut messages = Vec::new();
    for row in rows {
        let mut msg = row?;
        msg.item_links = get_item_links_for_message(conn, msg.id)?;
        messages.push(msg);
    }

    Ok(messages)
}

/// Get item links for a specific message
fn get_item_links_for_message(
    conn: &DbConnection,
    message_id: i64,
) -> Result<Vec<ChatItemLinkRow>> {
    let mut stmt = conn.prepare(
        "SELECT raw_text, item_name, item_id
         FROM chat_item_links
         WHERE message_id = ?1
         ORDER BY id",
    )?;

    let rows = stmt.query_map([message_id], |row| {
        Ok(ChatItemLinkRow {
            raw_text: row.get(0)?,
            item_name: row.get(1)?,
            item_id: row.get(2)?,
        })
    })?;

    let mut links = Vec::new();
    for row in rows {
        links.push(row?);
    }

    Ok(links)
}

/// Get unique channels
pub fn get_channels(conn: &DbConnection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT channel FROM chat_messages
         WHERE channel IS NOT NULL
         ORDER BY channel",
    )?;

    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut channels = Vec::new();
    for row in rows {
        channels.push(row?);
    }

    Ok(channels)
}

/// Get message count by channel
pub fn get_channel_stats(conn: &DbConnection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT channel, COUNT(*) as count
         FROM chat_messages
         WHERE channel IS NOT NULL
         GROUP BY channel
         ORDER BY count DESC",
    )?;

    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut stats = Vec::new();
    for row in rows {
        stats.push(row?);
    }

    Ok(stats)
}

/// Get overall chat statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatStats {
    pub total_messages: i64,
    pub channel_count: i64,
    pub oldest_message: String,
    pub newest_message: String,
    pub database_size_bytes: i64,
    pub messages_size_bytes: i64,
    pub item_links_count: i64,
}

pub fn get_chat_stats(conn: &DbConnection) -> Result<ChatStats> {
    let total_messages: i64 = conn
        .query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0))
        .unwrap_or(0);

    let channel_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT channel) FROM chat_messages WHERE channel IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let oldest_message: String = conn
        .query_row(
            "SELECT timestamp FROM chat_messages ORDER BY timestamp ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "N/A".to_string());

    let newest_message: String = conn
        .query_row(
            "SELECT timestamp FROM chat_messages ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "N/A".to_string());

    let item_links_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM chat_item_links", [], |row| row.get(0))
        .unwrap_or(0);

    // Sum the size of chat-related tables and indexes only
    let messages_size_bytes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(pgsize), 0) FROM dbstat WHERE name LIKE 'chat_%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let database_size_bytes: i64 = conn
        .query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(ChatStats {
        total_messages,
        channel_count,
        oldest_message,
        newest_message,
        database_size_bytes,
        messages_size_bytes,
        item_links_count,
    })
}

/// Get list of unique conversation partners from Tell messages
pub fn get_tell_conversations(conn: &DbConnection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT sender, COUNT(*) as count
         FROM chat_messages
         WHERE channel = 'Tell' AND sender IS NOT NULL
         GROUP BY sender
         ORDER BY MAX(timestamp) DESC",
    )?;

    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut conversations = Vec::new();
    for row in rows {
        conversations.push(row?);
    }

    Ok(conversations)
}

/// Query chat messages that match a watch rule's conditions.
///
/// Builds a SQL query from the rule's conditions:
/// - ContainsText: matches message body OR item link names (case-insensitive)
/// - ContainsItemLink: matches item link names only
/// - FromSender: exact sender match (case-insensitive)
/// - Channel filter: restricts to specified channels
pub fn get_watch_rule_messages(
    conn: &DbConnection,
    rule: &WatchRule,
    limit: i64,
    offset: i64,
    excluded_channels: &[String],
) -> Result<Vec<ChatMessageRow>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut param_values: Vec<String> = Vec::new();
    let mut param_idx = 1;

    // Exclude messages from excluded channels (they shouldn't be in the DB,
    // but may exist from before the channel was excluded)
    if !excluded_channels.is_empty() {
        let placeholders: Vec<String> = excluded_channels
            .iter()
            .map(|_| {
                let p = format!("?{}", param_idx);
                param_idx += 1;
                p
            })
            .collect();
        conditions.push(format!(
            "(cm.channel IS NULL OR cm.channel NOT IN ({}))",
            placeholders.join(", ")
        ));
        for ch in excluded_channels {
            param_values.push(ch.clone());
        }
    }

    // Channel filter
    if let Some(ref channels) = rule.channels {
        if !channels.is_empty() {
            let placeholders: Vec<String> = channels
                .iter()
                .map(|_| {
                    let p = format!("?{}", param_idx);
                    param_idx += 1;
                    p
                })
                .collect();
            conditions.push(format!("cm.channel IN ({})", placeholders.join(", ")));
            for ch in channels {
                param_values.push(ch.clone());
            }
        }
    }

    // Build watch conditions
    let mut watch_conditions: Vec<String> = Vec::new();
    for condition in &rule.conditions {
        match condition {
            WatchCondition::ContainsText(text) => {
                let like_param = format!("%{}%", text);
                // Match in message body OR in item link names
                watch_conditions.push(format!(
                    "(cm.message LIKE ?{} COLLATE NOCASE OR EXISTS (\
                        SELECT 1 FROM chat_item_links cil \
                        WHERE cil.message_id = cm.id AND cil.item_name LIKE ?{} COLLATE NOCASE\
                    ))",
                    param_idx,
                    param_idx + 1
                ));
                param_values.push(like_param.clone());
                param_values.push(like_param);
                param_idx += 2;
            }
            WatchCondition::ContainsItemLink(item_name) => {
                let like_param = format!("%{}%", item_name);
                watch_conditions.push(format!(
                    "EXISTS (\
                        SELECT 1 FROM chat_item_links cil \
                        WHERE cil.message_id = cm.id AND cil.item_name LIKE ?{} COLLATE NOCASE\
                    )",
                    param_idx
                ));
                param_values.push(like_param);
                param_idx += 1;
            }
            WatchCondition::FromSender(sender) => {
                watch_conditions.push(format!("cm.sender = ?{} COLLATE NOCASE", param_idx));
                param_values.push(sender.clone());
                param_idx += 1;
            }
        }
    }

    // Join watch conditions with AND or OR based on match_mode
    if !watch_conditions.is_empty() {
        let joiner = match rule.match_mode {
            ConditionMatch::All => " AND ",
            ConditionMatch::Any => " OR ",
        };
        conditions.push(format!("({})", watch_conditions.join(joiner)));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT cm.id, cm.timestamp, cm.channel, cm.sender, cm.message, cm.is_system, cm.from_player \
         FROM chat_messages cm{} ORDER BY cm.timestamp DESC LIMIT {} OFFSET {}",
        where_clause, limit, offset
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = param_values
        .iter()
        .map(|p| p as &dyn rusqlite::ToSql)
        .collect();

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(ChatMessageRow {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            channel: row.get(2)?,
            sender: row.get(3)?,
            message: row.get(4)?,
            is_system: row.get(5)?,
            from_player: row.get(6)?,
            item_links: Vec::new(),
        })
    })?;

    let mut messages = Vec::new();
    for row in rows {
        let mut msg = row?;
        msg.item_links = get_item_links_for_message(conn, msg.id)?;
        messages.push(msg);
    }

    Ok(messages)
}
