// Pure parsing logic — no Tauri dependencies here

#[derive(serde::Serialize, Clone)]
pub struct SkillUpdate {
    pub skill_type: String,
    pub level: u32,
    pub xp: u32,
    pub tnl: u32,
    pub timestamp: String, // e.g. "00:08:37"
}

pub fn parse_skill_update(line: &str) -> Option<SkillUpdate> {
    if !line.contains("ProcessUpdateSkill") {
        return None;
    }

    let timestamp = parse_timestamp(line)?;
    let skill_type = extract_field(line, "type=")?;
    let level: u32  = extract_field(line, "raw=")?.parse().ok()?;
    let xp: u32     = extract_field(line, "xp=")?.parse().ok()?;
    let tnl: u32    = extract_field(line, "tnl=")?.parse().ok()?;

    Some(SkillUpdate { skill_type, level, xp, tnl, timestamp })
}

// Parses "[HH:MM:SS] " from the start of a line, returns "HH:MM:SS"
pub fn parse_timestamp(line: &str) -> Option<String> {
    let line = line.trim_start();
    if !line.starts_with('[') {
        return None;
    }
    let end = line.find(']')?;
    Some(line[1..end].to_string())
}

// Extracts the value after `key` up to the next comma or `}`
pub fn extract_field(line: &str, key: &str) -> Option<String> {
    let start = line.find(key)? + key.len();
    let rest = &line[start..];
    let end = rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len());
    Some(rest[..end].to_string())
}