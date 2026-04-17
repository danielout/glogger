//! Public data types for the survey tracker. These map closely to the v26
//! schema rows so persistence stays a thin layer.

use serde::{Deserialize, Serialize};

/// What kicked off a session. Stored as text in the DB.
///
/// - `Manual` — user clicked "Start Session" in the UI.
/// - `Crafting` — first survey-map crafting detected; session auto-ends when
///   `consumed_count >= crafted_count` and no `pending_loot` uses remain.
/// - `FirstUse` — first survey-map use detected with no active session;
///   session must be ended manually.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStartTrigger {
    Manual,
    Crafting,
    FirstUse,
}

impl SessionStartTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStartTrigger::Manual => "manual",
            SessionStartTrigger::Crafting => "crafting",
            SessionStartTrigger::FirstUse => "first_use",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "manual" => Some(SessionStartTrigger::Manual),
            "crafting" => Some(SessionStartTrigger::Crafting),
            "first_use" => Some(SessionStartTrigger::FirstUse),
            _ => None,
        }
    }
}

/// Behavioral kind of a survey map use. Mirrors `game_data::SurveyKind`
/// but exists separately here so the survey module can be self-contained
/// for serialization/persistence and so the DB-side string mapping is local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyUseKind {
    Basic,
    Motherlode,
    Multihit,
}

impl SurveyUseKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SurveyUseKind::Basic => "basic",
            SurveyUseKind::Motherlode => "motherlode",
            SurveyUseKind::Multihit => "multihit",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "basic" => Some(SurveyUseKind::Basic),
            "motherlode" => Some(SurveyUseKind::Motherlode),
            "multihit" => Some(SurveyUseKind::Multihit),
            _ => None,
        }
    }
}

impl From<crate::game_data::SurveyKind> for SurveyUseKind {
    fn from(k: crate::game_data::SurveyKind) -> Self {
        match k {
            crate::game_data::SurveyKind::Basic => SurveyUseKind::Basic,
            crate::game_data::SurveyKind::Motherlode => SurveyUseKind::Motherlode,
            crate::game_data::SurveyKind::Multihit => SurveyUseKind::Multihit,
        }
    }
}

/// Lifecycle status of a single `survey_uses` row.
///
/// - `PendingLoot` — survey map consumed; loot may still arrive (motherlode
///   mining cycle running, multihit window open).
/// - `Completed` — window closed cleanly with at least one loot row attached.
/// - `Aborted` — window closed without loot (motherlode despawned, multihit
///   timed out empty, etc.).
/// - `Unknown` — defensive default; shouldn't normally appear in production data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyUseStatus {
    PendingLoot,
    Completed,
    Aborted,
    Unknown,
}

impl SurveyUseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SurveyUseStatus::PendingLoot => "pending_loot",
            SurveyUseStatus::Completed => "completed",
            SurveyUseStatus::Aborted => "aborted",
            SurveyUseStatus::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending_loot" => Some(SurveyUseStatus::PendingLoot),
            "completed" => Some(SurveyUseStatus::Completed),
            "aborted" => Some(SurveyUseStatus::Aborted),
            "unknown" => Some(SurveyUseStatus::Unknown),
            _ => None,
        }
    }
}

/// A survey session header (one row from `survey_sessions`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveySession {
    pub id: i64,
    pub character_name: String,
    pub server_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub start_trigger: SessionStartTrigger,
    /// Only populated when `start_trigger == Crafting`. Used by the auto-end
    /// check (auto-end fires when `consumed_count >= crafted_count` AND no
    /// `pending_loot` uses remain).
    pub crafted_count: Option<u32>,
    pub consumed_count: u32,
    pub notes: Option<String>,
    /// User-facing name. Defaults to None (UI shows "Session #N").
    pub name: Option<String>,
    /// User-adjusted start/end overrides. When set, the UI uses these
    /// instead of `started_at`/`ended_at` for display and duration calcs.
    /// Clearable (set back to None to revert to computed bounds).
    pub user_started_at: Option<String>,
    pub user_ended_at: Option<String>,
    /// Timestamp of the first/last survey-map crafting detected in this session.
    pub first_craft_at: Option<String>,
    pub last_craft_at: Option<String>,
    /// Timestamp of the first/last loot attributed to this session.
    pub first_loot_at: Option<String>,
    pub last_loot_at: Option<String>,
}

/// A single survey-map use (one row from `survey_uses`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyUse {
    pub id: i64,
    /// Optional because a use can outlive its session — `ON DELETE SET NULL`.
    pub session_id: Option<i64>,
    pub character_name: String,
    pub server_name: String,
    pub used_at: String,
    pub map_internal_name: String,
    pub map_display_name: String,
    pub kind: SurveyUseKind,
    /// Live-tracked area at the time of use, falling back to the area parsed
    /// from `map_internal_name` when live area is unavailable.
    pub area: Option<String>,
    pub status: SurveyUseStatus,
    /// Denormalized convenience: total loot quantity attributed to this use.
    /// Updated by the aggregator as `item_transactions` rows are inserted.
    pub loot_qty: u32,
}
