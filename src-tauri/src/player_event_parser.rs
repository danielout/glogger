use crate::parsers::parse_timestamp;
/// Player log event parser — foundational module for parsing all ProcessXxx events
/// from Player.log into structured, identity-resolved events.
///
/// This is a core system that features subscribe to. It maintains:
/// - Instance registry: maps instance IDs to item names/type IDs
/// - Stack tracking: current stack size per instance
/// - Interaction context: current NPC interaction for contextualizing events
/// - Pending delete buffer: 1-line lookahead to disambiguate storage/vendor/consumed
use std::collections::HashMap;

// ============================================================
// Event Types
// ============================================================

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum PlayerEvent {
    // === Item Events ===
    ItemAdded {
        timestamp: String,
        item_name: String,
        instance_id: u64,
        slot_index: i32,
        is_new: bool,
        /// Initial stack quantity. When a matching chat [Status] gain was in
        /// the buffer, this is the chat's authoritative quantity (e.g., 3 for
        /// "Fluorite x3 added to inventory"). Otherwise defaults to 1.
        /// Downstream consumers (GameStateManager, test harness) should use
        /// this as the transaction quantity instead of hardcoding 1.
        initial_quantity: u32,
        /// Source attribution derived from the parser's ActivityContext stack
        /// at the time of this event. See `ItemProvenance` docs.
        provenance: ItemProvenance,
    },
    ItemStackChanged {
        timestamp: String,
        instance_id: u64,
        item_name: Option<String>,
        item_type_id: u16,
        old_stack_size: u32,
        new_stack_size: u32,
        delta: i32,
        from_server: bool,
        provenance: ItemProvenance,
    },
    ItemDeleted {
        timestamp: String,
        instance_id: u64,
        item_name: Option<String>,
        context: DeleteContext,
    },

    // === Skill Events ===
    SkillsLoaded {
        timestamp: String,
        skills: Vec<SkillSnapshot>,
    },

    // === NPC Events ===
    InteractionStarted {
        timestamp: String,
        entity_id: u32,
        interaction_type: u32,
        npc_name: String,
    },
    FavorChanged {
        timestamp: String,
        npc_id: u32,
        npc_name: String,
        delta: f32,
        is_gift: bool,
    },

    // === Vendor Events ===
    VendorSold {
        timestamp: String,
        price: u32,
        item_name: String,
        instance_id: u64,
        is_buyback: bool,
    },
    VendorStackUpdated {
        timestamp: String,
        instance_id: u64,
        item_type_id: u16,
        new_stack_size: u32,
        price: u32,
    },

    // === Storage Events ===
    StorageDeposit {
        timestamp: String,
        npc_id: u32,
        vault_key: Option<String>,
        slot: i32,
        item_name: String,
        instance_id: u64,
    },
    StorageWithdrawal {
        timestamp: String,
        npc_id: u32,
        vault_key: Option<String>,
        instance_id: u64,
        quantity: u32,
        provenance: ItemProvenance,
    },

    // === Action Events ===
    DelayLoopStarted {
        timestamp: String,
        duration: f32,
        action_type: String,
        label: String,
        entity_id: u32,
        abort_condition: String,
    },

    // === Screen/Book Events ===
    ScreenText {
        timestamp: String,
        category: String,
        message: String,
    },
    BookOpened {
        timestamp: String,
        title: String,
        content: String,
        book_type: String,
    },

    // === Interaction Events ===
    InteractionEnded {
        timestamp: String,
        entity_id: i32,
    },

    // === Skill Bar Events ===
    ActiveSkillsChanged {
        timestamp: String,
        skill1: String,
        skill2: String,
    },

    // === Mount Events ===
    MountStateChanged {
        timestamp: String,
        entity_id: u32,
        is_mounting: bool,
    },

    // === Weather Events ===
    WeatherChanged {
        timestamp: String,
        weather_name: String,
        is_active: bool,
    },

    // === Recipe Events ===
    RecipeUpdated {
        timestamp: String,
        recipe_id: u32,
        completion_count: u32,
    },

    // === Combat Events ===
    CombatStateChanged {
        timestamp: String,
        in_combat: bool,
    },

    // === Vendor Gold Events ===
    VendorGoldChanged {
        timestamp: String,
        current_gold: u32,
        server_id: u64,
        max_gold: u32,
    },

    // === Attribute Events ===
    AttributesChanged {
        timestamp: String,
        entity_id: u32,
        attributes: Vec<AttributeValue>,
    },

    // === Login Snapshot Events ===
    AbilitiesLoaded {
        timestamp: String,
        skill1: String,
        skill2: String,
    },
    RecipesLoaded {
        timestamp: String,
    },
    EquipmentChanged {
        timestamp: String,
        entity_id: u32,
        appearance: String,
        equipment: Vec<EquipmentSlot>,
    },

    // === Effect Events ===
    EffectsAdded {
        timestamp: String,
        entity_id: u32,
        source_entity_id: u32,
        effect_ids: Vec<u32>,
        is_login_batch: bool,
    },
    // === Moon Phase Events ===
    MoonPhaseChanged {
        timestamp: String,
        phase: String,
    },

    // === Guild Events ===
    GuildInfoLoaded {
        timestamp: String,
        guild_id: u32,
        guild_name: String,
        motd: String,
    },

    // === Directed Goals Events ===
    DirectedGoalsLoaded {
        timestamp: String,
        goal_ids: Vec<u32>,
    },

    // === Player String Events ===
    PlayerStringUpdated {
        timestamp: String,
        key: String,
        value: String,
    },

    /// Signal-only: ProcessRemoveEffects prints opaque System.Int32[] so we can't
    /// extract which IDs were removed. We still emit the event so consumers know
    /// *something* changed.
    EffectsRemoved {
        timestamp: String,
        entity_id: u32,
    },
    EffectNameUpdated {
        timestamp: String,
        entity_id: u32,
        effect_instance_id: u32,
        display_name: String,
    },
    /// ProcessUpdateDescription — entity state change (garden plants, crafting items with timers, etc.)
    EntityDescriptionUpdated {
        timestamp: String,
        entity_id: u32,
        name: String,
        description: String,
        action: String,
        action_type: String,
        appearance: String,
        flags: u32,
    },
}

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum DeleteContext {
    StorageTransfer,
    VendorSale,
    #[allow(dead_code)]
    Consumed,
    Unknown,
}

/// Where did this item come from? Attached to every inventory gain event
/// (ItemAdded, ItemStackChanged, StorageWithdrawal) so downstream features
/// can aggregate by source (mining yield stats, survey loot tracking, etc.).
///
/// Attribution is best-effort: when the parser sees a clean match to exactly
/// one active `ActivityContext` at the event's timestamp, it emits
/// `Attributed`. When multiple contexts are active and none can be
/// confidently preferred, it emits `Uncertain` with the candidate list so
/// downstream code can decide whether/how to classify. When no context is
/// active, `UnknownSource` is used — this bucket is expected to be sizeable
/// because Player.log does not mark every gain with a surrounding activity.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum ItemProvenance {
    Attributed {
        source: ActivitySource,
        confidence: AttributionConfidence,
        /// A3 stitching link populated by feature aggregators (currently only
        /// the survey tracker). When set, this gain belongs to the named
        /// `survey_uses` row and downstream queries can group by it via
        /// `item_transactions.source_details->>'survey_use_id'`. The parser
        /// itself never sets this — it stays `None` until an aggregator that
        /// cares mutates it before the event reaches game-state persistence.
        #[serde(skip_serializing_if = "Option::is_none", default)]
        survey_use_id: Option<i64>,
    },
    Uncertain {
        candidates: Vec<ActivitySource>,
    },
    UnknownSource,
    /// Internal "don't attribute" signal — used for events that represent
    /// state reconciliation rather than real inventory gains (e.g., the
    /// first UpdateItemCode on a session-loaded item, session-load AddItems).
    /// Serialized as `{kind: "NotApplicable"}` so consumers can filter these
    /// out of gain-based aggregates.
    NotApplicable,
}

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum AttributionConfidence {
    /// Exactly one active context, emitted immediately.
    Confident,
    /// Multiple contexts active but one was very recently opened relative to
    /// the gain, or the best candidate was preferred by a tie-breaker.
    #[allow(dead_code)] // populated by future heuristics; no tie-breaker rules yet
    Probable,
    /// Fallback when we pick a best guess without strong signal.
    #[allow(dead_code)] // reserved for future item-type-to-loot-table matching
    Weak,
}

/// Database-ready representation of an `ItemProvenance`: three values that
/// populate the `source_kind`, `source_details`, and `confidence` columns on
/// `item_transactions`. All three may be `None` depending on the provenance
/// variant (see column documentation in migration v25).
pub struct ProvenanceColumns {
    pub source_kind: Option<String>,
    pub source_details: Option<String>,
    pub confidence: Option<String>,
}

impl ItemProvenance {
    /// Project this provenance into the three DB columns.
    ///
    /// Taxonomy for `source_kind`:
    /// - `"mining"`, `"survey_map_use"`, `"survey_map_craft"`,
    ///   `"general_craft"`, `"corpse_search"`, `"vendor_browsing"`,
    ///   `"storage_browsing"` — from `Attributed` variants (source kind
    ///   determines the string).
    /// - `"uncertain"` — multiple candidates, preserved in `source_details`.
    /// - `"unknown"` — `UnknownSource`.
    /// - `"not_applicable"` — session-load reloads, consumption/losses. Gain
    ///   aggregates should filter rows where source_kind = 'not_applicable'.
    ///
    /// `source_details` is a JSON blob with source-kind-specific fields
    /// (node_name, npc_name, etc.) — present whenever the source carries
    /// additional identifying info worth preserving.
    pub fn to_columns(&self) -> ProvenanceColumns {
        match self {
            ItemProvenance::Attributed {
                source,
                confidence,
                survey_use_id,
            } => {
                let (kind, details) = activity_source_to_kind_and_details(source);
                // If a survey_use_id is attached, splice it into the existing
                // details JSON (or create a fresh one). This is the A3
                // stitching link — `item_transactions.source_details->>'survey_use_id'`
                // is the grouping key for "all loot from this survey use".
                let details = match (details, survey_use_id) {
                    (Some(json), Some(use_id)) => Some(inject_survey_use_id(&json, *use_id)),
                    (None, Some(use_id)) => Some(format!(r#"{{"survey_use_id":{use_id}}}"#)),
                    (other, None) => other,
                };
                ProvenanceColumns {
                    source_kind: Some(kind.to_string()),
                    source_details: details,
                    confidence: Some(confidence_to_string(confidence).to_string()),
                }
            }
            ItemProvenance::Uncertain { candidates } => {
                // Serialize the full candidate list as JSON so downstream
                // classification/reclassification logic can access it later.
                let details = serde_json::to_string(candidates).ok();
                ProvenanceColumns {
                    source_kind: Some("uncertain".to_string()),
                    source_details: details,
                    confidence: None,
                }
            }
            ItemProvenance::UnknownSource => ProvenanceColumns {
                source_kind: Some("unknown".to_string()),
                source_details: None,
                confidence: None,
            },
            ItemProvenance::NotApplicable => ProvenanceColumns {
                source_kind: Some("not_applicable".to_string()),
                source_details: None,
                confidence: None,
            },
        }
    }
}

fn confidence_to_string(c: &AttributionConfidence) -> &'static str {
    match c {
        AttributionConfidence::Confident => "confident",
        AttributionConfidence::Probable => "probable",
        AttributionConfidence::Weak => "weak",
    }
}

/// Splice a `survey_use_id` field into an existing source-details JSON object.
///
/// Falls back to a minimal object containing only the survey_use_id if the
/// input isn't valid JSON (defensive — shouldn't happen since we control the
/// upstream serializer, but keeps a malformed details blob from corrupting
/// the stitching link).
fn inject_survey_use_id(details_json: &str, survey_use_id: i64) -> String {
    match serde_json::from_str::<serde_json::Value>(details_json) {
        Ok(serde_json::Value::Object(mut map)) => {
            map.insert(
                "survey_use_id".to_string(),
                serde_json::Value::from(survey_use_id),
            );
            serde_json::Value::Object(map).to_string()
        }
        _ => format!(r#"{{"survey_use_id":{survey_use_id}}}"#),
    }
}

/// Project an ActivitySource into a stable string kind and optional JSON
/// details blob. The JSON is flat and only includes fields that carry
/// identifying info (node names, NPC names, etc.).
fn activity_source_to_kind_and_details(source: &ActivitySource) -> (&'static str, Option<String>) {
    match source {
        ActivitySource::Mining {
            node_entity_id,
            node_name,
        } => {
            let details = serde_json::json!({
                "node_entity_id": node_entity_id,
                "node_name": node_name,
            });
            ("mining", Some(details.to_string()))
        }
        ActivitySource::SurveyMapUse {
            survey_map_internal_name,
        } => {
            let details = serde_json::json!({
                "survey_map_internal_name": survey_map_internal_name,
            });
            ("survey_map_use", Some(details.to_string()))
        }
        ActivitySource::SurveyMapCraft => ("survey_map_craft", None),
        ActivitySource::GeneralCraft { action_type, label } => {
            let details = serde_json::json!({
                "action_type": action_type,
                "label": label,
            });
            ("general_craft", Some(details.to_string()))
        }
        ActivitySource::CorpseSearch {
            entity_id,
            corpse_name,
        } => {
            let details = serde_json::json!({
                "entity_id": entity_id,
                "corpse_name": corpse_name,
            });
            ("corpse_search", Some(details.to_string()))
        }
        ActivitySource::VendorBrowsing {
            npc_entity_id,
            npc_name,
        } => {
            let details = serde_json::json!({
                "npc_entity_id": npc_entity_id,
                "npc_name": npc_name,
            });
            ("vendor_browsing", Some(details.to_string()))
        }
        ActivitySource::StorageBrowsing {
            vault_owner_entity_id,
            vault_name,
        } => {
            let details = serde_json::json!({
                "vault_owner_entity_id": vault_owner_entity_id,
                "vault_name": vault_name,
            });
            ("storage_browsing", Some(details.to_string()))
        }
    }
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct SkillSnapshot {
    pub skill_type: String,
    pub raw: u32,
    pub bonus: u32,
    pub xp: u32,
    pub tnl: i32,
    pub max: u32,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct AttributeValue {
    pub name: String,
    pub value: f32,
}

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct EquipmentSlot {
    pub slot: String,
    pub appearance_key: String,
}

// ============================================================
// Internal State Types
// ============================================================

#[derive(Clone, Debug)]
struct InstanceInfo {
    item_name: String,
    item_type_id: Option<u16>,
}

#[derive(Clone, Debug)]
struct PendingDelete {
    timestamp: String,
    instance_id: u64,
    item_name: Option<String>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct InteractionContext {
    entity_id: u32,
    npc_name: String,
    interaction_type: u32,
}

// ============================================================
// Activity Context — source attribution for item gains
// ============================================================
//
// Reconstructs what the player is currently *doing* from the log line stream.
// Each active context describes a potential source of inventory gains (mining
// a node, using a survey map, searching a corpse, browsing a vendor, etc.).
//
// Contexts are opened by reliable signals — primarily `ProcessDoDelayLoop`
// action_type + label, or screen-specific events like `ProcessTalkScreen`
// ("Search Corpse of X", …, Corpse). Contexts close on an explicit end signal
// (`ProcessEndInteraction`) or a timeout derived from the delay-loop duration.
//
// Signals that are *unreliable* across game versions and player environments —
// such as `ProcessStartInteraction` names (frequently empty for motherlode
// nodes) and `ProcessFirstEverInteraction` (fires inconsistently) — are only
// used as enrichment when present. Attribution must work even when they are
// absent.

/// Discriminated source of an inventory gain, reconstructed from player.log context.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum ActivitySource {
    /// Mining at a world node (motherlode or plain). node_name is present only
    /// when the game emitted it; many empty `StartInteraction` names occur in
    /// real logs and we must not rely on this being populated.
    Mining {
        node_entity_id: Option<u32>,
        node_name: Option<String>,
    },
    /// Using a survey map (primary source for survey loot).
    SurveyMapUse {
        survey_map_internal_name: Option<String>,
    },
    /// Crafting a new survey map via the Surveying skill.
    SurveyMapCraft,
    /// Other crafting/use actions detected via DoDelayLoop (cooking, brewing,
    /// distilling, using food, etc.).
    GeneralCraft {
        action_type: String,
        label: String,
    },
    /// Actively searching a corpse (opened via TalkScreen "Search Corpse of X").
    CorpseSearch {
        entity_id: u32,
        corpse_name: String,
    },
    /// Vendor trade window open.
    VendorBrowsing {
        npc_entity_id: u32,
        npc_name: Option<String>,
    },
    /// Storage vault UI open (saddlebag, housing, NPC account storage, etc.).
    StorageBrowsing {
        vault_owner_entity_id: u32,
        vault_name: String,
    },
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // started_at*/close_deadline_secs are read by tests and future Phase 2 attribution
pub struct ActivityContext {
    pub source: ActivitySource,
    /// Timestamp the context was opened (HH:MM:SS from player.log).
    pub started_at: String,
    /// Seconds since start-of-day for the started_at timestamp — cached so we
    /// don't re-parse on every expire check.
    pub started_at_secs: u32,
    /// Seconds after `started_at_secs` at which this context is considered
    /// stale and closed on the next line. Computed from the DoDelayLoop
    /// duration (+ slack) when available; otherwise a type-specific default.
    pub close_deadline_secs: u32,
    /// entity_id used to match an explicit `EndInteraction` close signal,
    /// when applicable. Not all sources are interaction-backed.
    pub entity_id: Option<u32>,
}

/// Slack added to a delay-loop duration before timing out a context.
/// Gives the server a moment to emit the resulting item events after the loop
/// completes, without letting the context drift indefinitely.
const DELAY_LOOP_SLACK_SECS: u32 = 2;

/// Default lifetime for contexts that don't carry an explicit duration
/// (corpse search, vendor/storage browsing). Conservative upper bound; real
/// activity typically closes via an explicit signal sooner.
const DEFAULT_CONTEXT_LIFETIME_SECS: u32 = 30;

/// Convert "HH:MM:SS" to seconds-of-day. Returns 0 on malformed input — safe
/// because mis-parsed contexts will expire immediately.
fn timestamp_to_secs(ts: &str) -> u32 {
    let mut parts = ts.split(':');
    let h: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let m: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let s: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    h * 3600 + m * 60 + s
}

/// Pending interaction metadata captured from `ProcessStartInteraction` so it
/// can enrich a subsequent screen-specific event (VendorScreen, ShowStorageVault,
/// TalkScreen corpse) before being consumed.
#[derive(Clone, Debug)]
struct PendingInteractionMeta {
    entity_id: u32,
    /// NPC name from StartInteraction, if non-empty. Often empty for
    /// non-NPC interactions (mining nodes, portals, containers).
    name: Option<String>,
    captured_at_secs: u32,
}

/// A recent chat `[Status] X added to inventory` gain, held briefly so the
/// parser can seed `ProcessAddItem`'s stack from the chat quantity instead of
/// the fallback 1.
///
/// Matching is by resolved internal item name (display-name-to-internal
/// resolution happens in the coordinator before the event is pushed into the
/// parser, since the parser has no CDN access). Entries are consumed on a
/// first-match basis to handle the multi-stack case (a single pickup split
/// across inventory stacks fires two separate chat lines and two separate
/// AddItems).
#[derive(Clone, Debug)]
struct PendingChatGain {
    item_internal_name: String,
    quantity: u32,
    captured_at_secs: u32,
}

/// Window (seconds) either side of a `ProcessAddItem` timestamp within which a
/// chat gain entry is considered a match. Chat and player.log flush
/// independently, so the chat message can arrive slightly before or after the
/// corresponding player.log line.
const CHAT_GAIN_MATCH_WINDOW_SECS: u32 = 2;

/// Maximum lifetime of an unmatched chat gain before it's discarded.
/// Longer than the match window so a chat gain that arrives slightly early
/// still has its full window available to find a match.
const CHAT_GAIN_BUFFER_LIFETIME_SECS: u32 = 10;

// ============================================================
// Parser
// ============================================================

pub struct PlayerEventParser {
    instance_registry: HashMap<u64, InstanceInfo>,
    stack_sizes: HashMap<u64, u32>,
    current_interaction: Option<InteractionContext>,
    pending_deletes: Vec<PendingDelete>,
    /// Stack of currently-active activity contexts. Most-recently-pushed is
    /// last; attribution prefers the most-specific/most-recent match.
    activity_contexts: Vec<ActivityContext>,
    /// Last-seen `ProcessStartInteraction` metadata, used to enrich the next
    /// screen-specific event. Cleared when consumed or after a short window.
    pending_interaction: Option<PendingInteractionMeta>,
    /// Buffer of chat-log `[Status] ItemGained` events awaiting correlation
    /// with an upcoming `ProcessAddItem`. Entries are consumed on first match
    /// and age out after `CHAT_GAIN_BUFFER_LIFETIME_SECS`.
    pending_chat_gains: Vec<PendingChatGain>,
}

impl PlayerEventParser {
    pub fn new() -> Self {
        Self {
            instance_registry: HashMap::new(),
            stack_sizes: HashMap::new(),
            current_interaction: None,
            pending_deletes: Vec::new(),
            activity_contexts: Vec::new(),
            pending_interaction: None,
            pending_chat_gains: Vec::new(),
        }
    }

    /// Feed a chat `[Status] X added to inventory` event into the parser's
    /// correlation buffer.
    ///
    /// The coordinator resolves the chat's display name to the CDN internal
    /// name before calling this so matching can be exact (chat uses "Rubywall
    /// Crystal" but player.log uses "RedCrystal" — different strings for the
    /// same item). The `timestamp` must be a Player.log-style "HH:MM:SS" in
    /// UTC so it aligns with player.log timestamps for correlation.
    ///
    /// The buffer is consulted during `ProcessAddItem` handling to seed new
    /// stacks from the chat's authoritative quantity instead of the fallback 1.
    pub fn feed_chat_gain(
        &mut self,
        item_internal_name: String,
        quantity: u32,
        timestamp_hms: &str,
    ) {
        let captured_at_secs = timestamp_to_secs(timestamp_hms);
        // Age out any chat gains older than the lifetime before pushing new ones.
        // Also age out any whose capture is way in the future (clock skew safety).
        self.pending_chat_gains.retain(|g| {
            let age = captured_at_secs.saturating_sub(g.captured_at_secs);
            age <= CHAT_GAIN_BUFFER_LIFETIME_SECS
        });
        self.pending_chat_gains.push(PendingChatGain {
            item_internal_name,
            quantity,
            captured_at_secs,
        });
    }

    /// Look up and consume a matching chat gain for a ProcessAddItem. Returns
    /// the quantity to seed the stack with, or `None` if no match was found.
    /// Matching is by exact internal name and within `CHAT_GAIN_MATCH_WINDOW_SECS`
    /// of the AddItem's timestamp (both directions).
    fn consume_chat_gain_for_add_item(
        &mut self,
        item_internal_name: &str,
        add_item_at_secs: u32,
    ) -> Option<u32> {
        // Find the closest-in-time matching entry. Prefer closest to avoid
        // stealing a chat gain that belongs to a different, later AddItem.
        let mut best_idx: Option<usize> = None;
        let mut best_dist: u32 = u32::MAX;
        for (i, g) in self.pending_chat_gains.iter().enumerate() {
            if g.item_internal_name != item_internal_name {
                continue;
            }
            let dist = if g.captured_at_secs >= add_item_at_secs {
                g.captured_at_secs - add_item_at_secs
            } else {
                add_item_at_secs - g.captured_at_secs
            };
            if dist <= CHAT_GAIN_MATCH_WINDOW_SECS && dist < best_dist {
                best_idx = Some(i);
                best_dist = dist;
            }
        }
        best_idx.map(|i| self.pending_chat_gains.swap_remove(i).quantity)
    }

    // ============================================================
    // Activity Context helpers
    // ============================================================

    /// Snapshot of currently-active contexts, most-recent last. Primarily for
    /// tests and future attribution logic.
    #[allow(dead_code)] // Phase 2 consumers will call this
    pub fn active_activities(&self) -> &[ActivityContext] {
        &self.activity_contexts
    }

    /// Push a new activity context. Contexts are treated as a stack; multiple
    /// can be active simultaneously (e.g., a corpse search started while a
    /// mining context hasn't yet timed out).
    fn push_activity(&mut self, ctx: ActivityContext) {
        self.activity_contexts.push(ctx);
    }

    /// Close all contexts matching the given entity_id. Used for explicit
    /// `ProcessEndInteraction` signals.
    fn close_activities_for_entity(&mut self, entity_id: u32) {
        self.activity_contexts
            .retain(|c| c.entity_id != Some(entity_id));
    }

    /// Expire any contexts whose close_deadline is at or before `now_secs`.
    /// Called at the start of each line so attribution reflects only active
    /// contexts at the line's timestamp.
    fn expire_stale_activities(&mut self, now_secs: u32) {
        self.activity_contexts
            .retain(|c| now_secs < c.close_deadline_secs);
        // Pending interaction metadata also expires after ~5 seconds so it
        // doesn't cross-pollinate unrelated later events.
        if let Some(pi) = &self.pending_interaction {
            if now_secs.saturating_sub(pi.captured_at_secs) > 5 {
                self.pending_interaction = None;
            }
        }
        // Aged-out chat gains are discarded to prevent them drifting into
        // attribution of unrelated later AddItems.
        self.pending_chat_gains.retain(|g| {
            let age = now_secs.saturating_sub(g.captured_at_secs);
            age <= CHAT_GAIN_BUFFER_LIFETIME_SECS
        });
    }

    /// Compute provenance for a gain event based on the current activity
    /// stack. Returns `UnknownSource` when nothing is active, `Attributed`
    /// when exactly one context matches, and `Uncertain` when multiple are
    /// plausible and no confident tie-breaker applies.
    ///
    /// Phase 2 keeps the rules deliberately simple: single-context
    /// attribution is `Confident`, multi-context is `Uncertain`. Richer
    /// heuristics (item-type-to-loot-table matching, recency bias) live in
    /// Phase 2.5+ once we have real-world attribution data to tune against.
    fn compute_provenance(&self) -> ItemProvenance {
        match self.activity_contexts.len() {
            0 => ItemProvenance::UnknownSource,
            1 => ItemProvenance::Attributed {
                source: self.activity_contexts[0].source.clone(),
                confidence: AttributionConfidence::Confident,
                survey_use_id: None,
            },
            _ => {
                // Tie-breaker for multi-context cases: when Mining is active
                // alongside a more passive context (CorpseSearch, VendorBrowsing,
                // StorageBrowsing), the gain almost always belongs to Mining.
                // Justification — mining is an explicit active task (delay loop
                // swinging a pick); the others are screens that happen to be open.
                // Real-world data (50x-povus dataset): players constantly fight
                // mobs between mining swings, so CorpseSearch + Mining overlap is
                // the dominant ambiguity case, and ~99% of the time the loot is
                // mining loot.
                //
                // Confidence is Probable rather than Confident to mark this as a
                // heuristic outcome — downstream code can choose to filter on
                // confidence level if it cares.
                let mining_idx = self.activity_contexts.iter().position(|c| {
                    matches!(c.source, ActivitySource::Mining { .. })
                });
                if let Some(idx) = mining_idx {
                    let others_are_passive = self
                        .activity_contexts
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != idx)
                        .all(|(_, c)| {
                            matches!(
                                c.source,
                                ActivitySource::CorpseSearch { .. }
                                    | ActivitySource::VendorBrowsing { .. }
                                    | ActivitySource::StorageBrowsing { .. }
                            )
                        });
                    if others_are_passive {
                        return ItemProvenance::Attributed {
                            source: self.activity_contexts[idx].source.clone(),
                            confidence: AttributionConfidence::Probable,
                            survey_use_id: None,
                        };
                    }
                }
                ItemProvenance::Uncertain {
                    candidates: self
                        .activity_contexts
                        .iter()
                        .map(|c| c.source.clone())
                        .collect(),
                }
            }
        }
    }

    /// Feed one log line; returns zero or more events.
    pub fn process_line(&mut self, line: &str) -> Vec<PlayerEvent> {
        let mut events = Vec::new();

        // Expire any activity contexts whose close_deadline has passed,
        // using this line's timestamp as "now". This makes provenance
        // attribution reflect only contexts that were still active when
        // the event occurred.
        if let Some(ts) = parse_timestamp(line) {
            let now_secs = timestamp_to_secs(&ts);
            self.expire_stale_activities(now_secs);
        }

        // Try to resolve pending deletes against this line
        let resolved = self.resolve_pending_deletes(line, &mut events);

        // If we resolved pending deletes via storage/vendor, the line was consumed
        if resolved {
            return events;
        }

        // Fast path: skip lines that aren't LocalPlayer Process events
        // (but still flush pending deletes above)
        if !line.contains("LocalPlayer: Process") {
            // Flush any remaining pending deletes as Unknown
            self.flush_pending_deletes(&mut events);
            return events;
        }

        // Dispatch by event type
        if line.contains("ProcessAddItem(") {
            // Flush pending deletes before processing new events
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_add_item(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessUpdateItemCode(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_update_item_code(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessDeleteItem(") {
            // Don't flush yet — buffer this delete
            self.parse_delete_item(line);
        } else if line.contains("ProcessLoadSkills(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_load_skills(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessStartInteraction(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_start_interaction(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessDeltaFavor(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_delta_favor(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessVendorAddItem(") {
            // This should have been handled in resolve_pending_deletes,
            // but handle standalone case too
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_vendor_add_item(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessVendorUpdateItem(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_vendor_update_item(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessAddToStorageVault(") {
            // This should have been handled in resolve_pending_deletes,
            // but handle standalone case too
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_add_to_storage(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessRemoveFromStorageVault(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_remove_from_storage(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessDoDelayLoop(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_delay_loop(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessScreenText(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_screen_text(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessBook(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_book(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessEndInteraction(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_end_interaction(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessTalkScreen(") {
            // No event emitted — this is context-only. Opens a CorpseSearch
            // when the TalkScreen is a corpse dialog; ignored otherwise.
            self.flush_pending_deletes(&mut events);
            self.handle_talk_screen(line);
        } else if line.contains("ProcessVendorScreen(") {
            // Context-only: vendor window open.
            self.flush_pending_deletes(&mut events);
            self.handle_vendor_screen(line);
        } else if line.contains("ProcessShowStorageVault(") {
            // Context-only: storage UI open.
            self.flush_pending_deletes(&mut events);
            self.handle_show_storage_vault(line);
        } else if line.contains("ProcessSetActiveSkills(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_set_active_skills(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessPlayerMount(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_player_mount(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessSetWeather(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_set_weather(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessUpdateRecipe(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_update_recipe(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessCombatModeStatus(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_combat_mode_status(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessVendorUpdateAvailableGold(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_vendor_update_gold(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessSetAttributes(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_set_attributes(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessLoadAbilities(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_load_abilities(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessLoadRecipes(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_load_recipes(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessSetEquippedItems(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_set_equipped_items(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessAddEffects(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_add_effects(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessRemoveEffects(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_remove_effects(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessUpdateEffectName(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_update_effect_name(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessUpdateDescription(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_update_description(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessSetCelestialInfo(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_celestial_info(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessGuildGeneralInfo(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_guild_general_info(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessCompleteDirectedGoals(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_complete_directed_goals(line) {
                events.push(ev);
            }
        } else if line.contains("ProcessSetString(") {
            self.flush_pending_deletes(&mut events);
            if let Some(ev) = self.parse_set_string(line) {
                events.push(ev);
            }
        } else {
            // Unrecognized Process line — flush pending
            self.flush_pending_deletes(&mut events);
        }

        events
    }

    /// Flush any buffered pending deletes as events.
    /// Call this at end-of-poll to ensure nothing is left hanging.
    pub fn flush_all_pending(&mut self) -> Vec<PlayerEvent> {
        let mut events = Vec::new();
        self.flush_pending_deletes(&mut events);
        events
    }

    // ============================================================
    // Pending Delete Resolution
    // ============================================================

    /// Try to match pending deletes against a storage/vendor line.
    /// Returns true if the line was consumed (was a storage/vendor event).
    fn resolve_pending_deletes(&mut self, line: &str, events: &mut Vec<PlayerEvent>) -> bool {
        if self.pending_deletes.is_empty() {
            return false;
        }

        // Check for ProcessAddToStorageVault with matching instance ID
        if line.contains("ProcessAddToStorageVault(") {
            if let Some(storage_ev) = self.parse_add_to_storage(line) {
                let storage_instance_id = match &storage_ev {
                    PlayerEvent::StorageDeposit { instance_id, .. } => *instance_id,
                    _ => 0,
                };

                // Emit matched deletes as StorageTransfer, unmatched as Unknown
                let pending = std::mem::take(&mut self.pending_deletes);
                for pd in pending {
                    let context = if pd.instance_id == storage_instance_id {
                        DeleteContext::StorageTransfer
                    } else {
                        DeleteContext::Unknown
                    };
                    events.push(PlayerEvent::ItemDeleted {
                        timestamp: pd.timestamp,
                        instance_id: pd.instance_id,
                        item_name: pd.item_name,
                        context,
                    });
                }
                events.push(storage_ev);
                return true;
            }
        }

        // Check for ProcessVendorAddItem with matching instance ID
        if line.contains("ProcessVendorAddItem(") {
            if let Some(vendor_ev) = self.parse_vendor_add_item(line) {
                let vendor_instance_id = match &vendor_ev {
                    PlayerEvent::VendorSold { instance_id, .. } => *instance_id,
                    _ => 0,
                };

                let pending = std::mem::take(&mut self.pending_deletes);
                for pd in pending {
                    let context = if pd.instance_id == vendor_instance_id {
                        DeleteContext::VendorSale
                    } else {
                        DeleteContext::Unknown
                    };
                    events.push(PlayerEvent::ItemDeleted {
                        timestamp: pd.timestamp,
                        instance_id: pd.instance_id,
                        item_name: pd.item_name,
                        context,
                    });
                }
                events.push(vendor_ev);
                return true;
            }
        }

        // Check for ProcessVendorUpdateItem (selling stackable to vendor that already has it)
        if line.contains("ProcessVendorUpdateItem(") {
            if let Some(vendor_ev) = self.parse_vendor_update_item(line) {
                let pending = std::mem::take(&mut self.pending_deletes);
                for pd in pending {
                    events.push(PlayerEvent::ItemDeleted {
                        timestamp: pd.timestamp,
                        instance_id: pd.instance_id,
                        item_name: pd.item_name,
                        context: DeleteContext::VendorSale,
                    });
                }
                events.push(vendor_ev);
                return true;
            }
        }

        false
    }

    /// Flush all pending deletes as ItemDeleted with Unknown context.
    fn flush_pending_deletes(&mut self, events: &mut Vec<PlayerEvent>) {
        let pending = std::mem::take(&mut self.pending_deletes);
        for pd in pending {
            events.push(PlayerEvent::ItemDeleted {
                timestamp: pd.timestamp,
                instance_id: pd.instance_id,
                item_name: pd.item_name,
                context: DeleteContext::Unknown,
            });
        }
    }

    // ============================================================
    // Individual Parse Functions
    // ============================================================

    /// ProcessAddItem(InternalName(instanceId), slotIndex, isNew)
    fn parse_add_item(&mut self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessAddItem(")? + "ProcessAddItem(".len();
        let args_section = &line[args_start..];

        // Extract InternalName(instanceId) — name is before '(', id is inside parens
        let inner_paren = args_section.find('(')?;
        let item_name = args_section[..inner_paren].to_string();

        let id_start = inner_paren + 1;
        let id_end = args_section[id_start..].find(')')? + id_start;
        let instance_id: u64 = args_section[id_start..id_end].parse().ok()?;

        // After the closing paren of InternalName(id), we have ", slotIndex, isNew)"
        let after_name = &args_section[id_end + 1..];
        let parts: Vec<&str> = after_name.split(',').collect();
        // parts[0] = ")", parts[1] = " slotIndex", parts[2] = " isNew)"  (or similar)
        let slot_index: i32 = parts.get(1)?.trim().parse().ok()?;
        let is_new_str = parts.get(2)?.trim().trim_end_matches(')');
        let is_new = is_new_str == "True";

        // Register in instance registry
        self.instance_registry.insert(
            instance_id,
            InstanceInfo {
                item_name: item_name.clone(),
                item_type_id: None,
            },
        );

        // Seed stack tracking for genuinely new items.
        // - slot_index >= 0 with is_new=true means a storage vault withdrawal;
        //   do NOT seed here — ProcessRemoveFromStorageVault will seed with the
        //   correct quantity so we don't report a false delta.
        // - slot_index == -1 with is_new=true means a real new acquisition (loot,
        //   craft, vendor purchase). If a matching chat [Status] gain event is
        //   buffered within a short window, seed from the chat's authoritative
        //   quantity; otherwise fall back to 1 (which was the prior behavior and
        //   remains correct for the common single-item-pickup case).
        // - is_new=false is a session-start inventory load; do NOT seed — the first
        //   ProcessUpdateItemCode will establish the baseline.
        let initial_quantity = if is_new && slot_index < 0 {
            let add_item_secs = timestamp_to_secs(&ts);
            let seed_qty = self
                .consume_chat_gain_for_add_item(&item_name, add_item_secs)
                .unwrap_or(1);
            self.stack_sizes.insert(instance_id, seed_qty);
            seed_qty
        } else {
            1
        };

        // Provenance reflects the currently-active activity stack. Session-
        // load AddItems (is_new=false) aren't gains — mark them NotApplicable
        // so downstream aggregates can filter them out without caring about
        // the surrounding context (which could be anything at login time).
        let provenance = if !is_new {
            ItemProvenance::NotApplicable
        } else {
            self.compute_provenance()
        };

        Some(PlayerEvent::ItemAdded {
            timestamp: ts,
            item_name,
            instance_id,
            slot_index,
            is_new,
            initial_quantity,
            provenance,
        })
    }

    /// ProcessUpdateItemCode(instanceId, encodedValue, fromServer)
    fn parse_update_item_code(&mut self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessUpdateItemCode(")? + "ProcessUpdateItemCode(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let instance_id: u64 = parts.get(0)?.trim().parse().ok()?;
        let encoded_value: u32 = parts.get(1)?.trim().parse().ok()?;
        let from_server = parts.get(2)?.trim() == "True";

        // Encoding is 0-based: actual stack size = (encoded >> 16) + 1.
        // Verified against JSON inventory exports where StackSize fields are 1-based.
        let new_stack_size = (encoded_value >> 16) + 1;
        let item_type_id = (encoded_value & 0xFFFF) as u16;

        let had_prior = self.stack_sizes.contains_key(&instance_id);
        let old_stack_size = self.stack_sizes.get(&instance_id).copied().unwrap_or(0);
        let delta = new_stack_size as i32 - old_stack_size as i32;

        // Update tracking state — always record the new stack size
        self.stack_sizes.insert(instance_id, new_stack_size);

        // Update type ID in registry if we have an entry
        if let Some(info) = self.instance_registry.get_mut(&instance_id) {
            info.item_type_id = Some(item_type_id);
        }

        // If we had no prior stack observation, this is normally establishing a
        // baseline (e.g., session-start inventory load). We can't compute the
        // real delta because we don't know the pre-existing stack size.
        //
        // HOWEVER: if a matching chat [Status] gain exists in the buffer at
        // this moment, the "baseline" is actually a real gain happening on top
        // of a login-loaded stack. The chat's authoritative quantity tells us
        // exactly how many items were gained, even though we don't know the
        // previous absolute stack. Emit the event with the chat quantity as
        // the delta so the gain isn't silently lost.
        if !had_prior {
            let item_name = self
                .instance_registry
                .get(&instance_id)
                .map(|info| info.item_name.clone());
            let update_secs = timestamp_to_secs(&ts);
            if let Some(ref name) = item_name {
                if let Some(chat_qty) = self.consume_chat_gain_for_add_item(name, update_secs) {
                    let provenance = self.compute_provenance();
                    return Some(PlayerEvent::ItemStackChanged {
                        timestamp: ts,
                        instance_id,
                        item_name,
                        item_type_id,
                        old_stack_size: new_stack_size.saturating_sub(chat_qty),
                        new_stack_size,
                        delta: chat_qty as i32,
                        from_server,
                        provenance,
                    });
                }
            }
            return None;
        }

        let item_name = self
            .instance_registry
            .get(&instance_id)
            .map(|info| info.item_name.clone());

        // Provenance only meaningfully applies to positive deltas (gains).
        // For zero or negative deltas (stack unchanged, consumed, split),
        // mark NotApplicable so consumers aggregating gains by source aren't
        // polluted by loss/consumption events.
        let provenance = if delta > 0 {
            self.compute_provenance()
        } else {
            ItemProvenance::NotApplicable
        };

        Some(PlayerEvent::ItemStackChanged {
            timestamp: ts,
            instance_id,
            item_name,
            item_type_id,
            old_stack_size,
            new_stack_size,
            delta,
            from_server,
            provenance,
        })
    }

    /// ProcessDeleteItem(instanceId) — buffers for context resolution
    fn parse_delete_item(&mut self, line: &str) {
        let ts = parse_timestamp(line).unwrap_or_default();
        let args_start = match line.find("ProcessDeleteItem(") {
            Some(i) => i + "ProcessDeleteItem(".len(),
            None => return,
        };
        let args_end = match line[args_start..].find(')') {
            Some(i) => args_start + i,
            None => return,
        };
        let instance_id: u64 = match line[args_start..args_end].trim().parse() {
            Ok(id) => id,
            Err(_) => return,
        };

        let item_name = self
            .instance_registry
            .get(&instance_id)
            .map(|info| info.item_name.clone());

        // Clean up tracking state
        self.instance_registry.remove(&instance_id);
        self.stack_sizes.remove(&instance_id);

        self.pending_deletes.push(PendingDelete {
            timestamp: ts,
            instance_id,
            item_name,
        });
    }

    /// ProcessLoadSkills({type=X,raw=R,bonus=B,xp=X,tnl=T,max=M}, ...)
    fn parse_load_skills(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessLoadSkills(")? + "ProcessLoadSkills(".len();

        let mut skills = Vec::new();
        let rest = &line[args_start..];

        // Iterate through {..} blocks
        let mut pos = 0;
        while pos < rest.len() {
            let block_start = match rest[pos..].find('{') {
                Some(i) => pos + i,
                None => break,
            };
            let block_end = match rest[block_start..].find('}') {
                Some(i) => block_start + i + 1,
                None => break,
            };
            let block = &rest[block_start..block_end];

            if let Some(skill) = parse_skill_block(block) {
                skills.push(skill);
            }

            pos = block_end;
        }

        if skills.is_empty() {
            return None;
        }

        Some(PlayerEvent::SkillsLoaded {
            timestamp: ts,
            skills,
        })
    }

    /// ProcessStartInteraction(entityId, interactionType, distance, canInteract, "NPC_Name")
    fn parse_start_interaction(&mut self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessStartInteraction(")? + "ProcessStartInteraction(".len();
        let args_end = line.rfind(')')?;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.splitn(5, ',').collect();
        let entity_id: u32 = parts.get(0)?.trim().parse().ok()?;
        let interaction_type: u32 = parts.get(1)?.trim().parse().ok()?;
        // parts[2] = distance, parts[3] = canInteract, parts[4] = "NPC_Name"
        let npc_name_raw = parts.get(4)?.trim();
        let npc_name = npc_name_raw.trim_matches('"').to_string();

        self.current_interaction = Some(InteractionContext {
            entity_id,
            npc_name: npc_name.clone(),
            interaction_type,
        });

        // Capture enrichment metadata for the next screen-specific event.
        // Name may be empty (common for mining nodes, containers); we keep it
        // as None in that case rather than storing an empty string.
        self.pending_interaction = Some(PendingInteractionMeta {
            entity_id,
            name: if npc_name.is_empty() {
                None
            } else {
                Some(npc_name.clone())
            },
            captured_at_secs: timestamp_to_secs(&ts),
        });

        Some(PlayerEvent::InteractionStarted {
            timestamp: ts,
            entity_id,
            interaction_type,
            npc_name,
        })
    }

    /// ProcessDeltaFavor(npcId, "NPC_Name", delta, isGift)
    fn parse_delta_favor(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessDeltaFavor(")? + "ProcessDeltaFavor(".len();
        let args_end = line.rfind(')')?;
        let args = &line[args_start..args_end];

        // Split carefully — NPC_Name is quoted
        let first_comma = args.find(',')?;
        let npc_id: u32 = args[..first_comma].trim().parse().ok()?;

        let rest = &args[first_comma + 1..];
        // Find quoted NPC name
        let q_start = rest.find('"')? + 1;
        let q_end = rest[q_start..].find('"')? + q_start;
        let npc_name = rest[q_start..q_end].to_string();

        let after_name = &rest[q_end + 1..];
        let parts: Vec<&str> = after_name.split(',').collect();
        let delta: f32 = parts.get(1)?.trim().parse().ok()?;
        let is_gift = parts.get(2)?.trim().trim_end_matches(')') == "True";

        Some(PlayerEvent::FavorChanged {
            timestamp: ts,
            npc_id,
            npc_name,
            delta,
            is_gift,
        })
    }

    /// ProcessVendorAddItem(price, InternalName(instanceId), isFromBuyback)
    fn parse_vendor_add_item(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessVendorAddItem(")? + "ProcessVendorAddItem(".len();
        let args = &line[args_start..];

        // First arg: price
        let first_comma = args.find(',')?;
        let price: u32 = args[..first_comma].trim().parse().ok()?;

        // Second arg: InternalName(instanceId)
        let rest = &args[first_comma + 1..];
        let inner_paren = rest.find('(')?;
        let item_name = rest[..inner_paren].trim().to_string();

        let id_start = inner_paren + 1;
        let id_end = rest[id_start..].find(')')? + id_start;
        let instance_id: u64 = rest[id_start..id_end].parse().ok()?;

        // Third arg: isFromBuyback
        let after_id = &rest[id_end + 1..];
        let last_comma = after_id.find(',')?;
        let buyback_str = after_id[last_comma + 1..].trim().trim_end_matches(')');
        let is_buyback = buyback_str == "True";

        Some(PlayerEvent::VendorSold {
            timestamp: ts,
            price,
            item_name,
            instance_id,
            is_buyback,
        })
    }

    /// ProcessVendorUpdateItem(instanceId, encodedValue, price)
    fn parse_vendor_update_item(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessVendorUpdateItem(")? + "ProcessVendorUpdateItem(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let instance_id: u64 = parts.get(0)?.trim().parse().ok()?;
        let encoded_value: u32 = parts.get(1)?.trim().parse().ok()?;
        let price: u32 = parts.get(2)?.trim().parse().ok()?;

        let new_stack_size = (encoded_value >> 16) + 1;
        let item_type_id = (encoded_value & 0xFFFF) as u16;

        Some(PlayerEvent::VendorStackUpdated {
            timestamp: ts,
            instance_id,
            item_type_id,
            new_stack_size,
            price,
        })
    }

    /// ProcessAddToStorageVault(npcId, -1, slot, InternalName(instanceId))
    fn parse_add_to_storage(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start =
            line.find("ProcessAddToStorageVault(")? + "ProcessAddToStorageVault(".len();
        let args = &line[args_start..];

        let parts: Vec<&str> = args.splitn(4, ',').collect();
        let npc_id: u32 = parts.get(0)?.trim().parse().ok()?;
        // parts[1] = -1 (skip)
        let slot: i32 = parts.get(2)?.trim().parse().ok()?;

        // parts[3] = " InternalName(instanceId))"
        let name_part = parts.get(3)?.trim();
        let inner_paren = name_part.find('(')?;
        let item_name = name_part[..inner_paren].to_string();

        let id_start = inner_paren + 1;
        let id_end = name_part[id_start..].find(')')? + id_start;
        let instance_id: u64 = name_part[id_start..id_end].parse().ok()?;

        let vault_key = self
            .current_interaction
            .as_ref()
            .map(|ctx| ctx.npc_name.clone());

        Some(PlayerEvent::StorageDeposit {
            timestamp: ts,
            npc_id,
            vault_key,
            slot,
            item_name,
            instance_id,
        })
    }

    /// ProcessRemoveFromStorageVault(npcId, -1, instanceId, quantity)
    fn parse_remove_from_storage(&mut self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start =
            line.find("ProcessRemoveFromStorageVault(")? + "ProcessRemoveFromStorageVault(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let npc_id: u32 = parts.get(0)?.trim().parse().ok()?;
        // parts[1] = -1 (skip)
        let instance_id: u64 = parts.get(2)?.trim().parse().ok()?;
        let quantity: u32 = parts.get(3)?.trim().parse().ok()?;

        // Seed stack size from the vault's known quantity.
        // This covers storage withdrawals where AddItem(slot>=0, True)
        // intentionally skipped seeding so we could use this authoritative value.
        // If the vault item merges into an existing inventory stack (different
        // instance_id), this seeds the vault instance which will never get an
        // UpdateItemCode — harmless.
        self.stack_sizes.entry(instance_id).or_insert(quantity);

        let vault_key = self
            .current_interaction
            .as_ref()
            .map(|ctx| ctx.npc_name.clone());

        // Storage withdrawals are definitionally attributable to the
        // currently-active StorageBrowsing context. Prefer the one whose
        // vault_owner matches this npc_id; fall back to any StorageBrowsing
        // if the id doesn't match; only then fall back to generic
        // provenance logic.
        let provenance = self
            .activity_contexts
            .iter()
            .find(|c| matches!(&c.source, ActivitySource::StorageBrowsing { vault_owner_entity_id, .. } if *vault_owner_entity_id == npc_id))
            .or_else(|| {
                self.activity_contexts
                    .iter()
                    .find(|c| matches!(&c.source, ActivitySource::StorageBrowsing { .. }))
            })
            .map(|c| ItemProvenance::Attributed {
                source: c.source.clone(),
                confidence: AttributionConfidence::Confident,
                survey_use_id: None,
            })
            .unwrap_or_else(|| self.compute_provenance());

        Some(PlayerEvent::StorageWithdrawal {
            timestamp: ts,
            npc_id,
            vault_key,
            instance_id,
            quantity,
            provenance,
        })
    }

    /// ProcessDoDelayLoop(duration, actionType, "label", entityId, abortCondition)
    fn parse_delay_loop(&mut self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessDoDelayLoop(")? + "ProcessDoDelayLoop(".len();
        let args = &line[args_start..];

        // Split into parts: duration, actionType, "label", entityId, abortCondition
        let first_comma = args.find(',')?;
        let duration: f32 = args[..first_comma].trim().parse().ok()?;

        let rest = &args[first_comma + 1..];
        let second_comma = rest.find(',')?;
        let action_type = rest[..second_comma].trim().to_string();

        // Extract quoted label
        let q_start = rest.find('"')? + 1;
        let q_end = rest[q_start..].find('"')? + q_start;
        let label = rest[q_start..q_end].to_string();

        // After closing quote: ", entityId, abortCondition)"
        let after_label = &rest[q_end + 1..];
        let parts: Vec<&str> = after_label.split(',').collect();
        let entity_id: u32 = parts.get(1)?.trim().parse().ok()?;
        let abort_condition = parts.get(2)?.trim().trim_end_matches(')').to_string();

        // Derive activity source from action_type + label. This is the
        // primary (reliable) signal — label shape is stable across game
        // versions where the decorative StartInteraction name is not.
        let started_at_secs = timestamp_to_secs(&ts);
        let duration_secs = duration.ceil().max(0.0) as u32;
        let close_deadline_secs = started_at_secs + duration_secs + DELAY_LOOP_SLACK_SECS;

        if let Some(source) = classify_delay_loop(&action_type, &label, &self.pending_interaction) {
            // If the delay loop references an entity_id that matches a recent
            // StartInteraction, adopt that for explicit-end matching. The
            // DoDelayLoop's own entity_id field is the loop target (usually
            // the player), not the interaction entity, so we prefer the
            // pending interaction's entity when available.
            let ctx_entity_id = match &source {
                ActivitySource::Mining { node_entity_id, .. } => *node_entity_id,
                _ => self.pending_interaction.as_ref().map(|p| p.entity_id),
            };

            // Prevent duplicate stacking when a delay loop repeats mid-cycle
            // (some actions re-emit DoDelayLoop on each tick). If the top of
            // the stack is already the same source kind and entity, just
            // extend its deadline instead of stacking.
            if let Some(top) = self.activity_contexts.last_mut() {
                if activity_source_matches(&top.source, &source)
                    && top.entity_id == ctx_entity_id
                {
                    top.close_deadline_secs = close_deadline_secs;
                    // leave started_at alone — first-seen is canonical
                } else {
                    self.push_activity(ActivityContext {
                        source,
                        started_at: ts.clone(),
                        started_at_secs,
                        close_deadline_secs,
                        entity_id: ctx_entity_id,
                    });
                }
            } else {
                self.push_activity(ActivityContext {
                    source,
                    started_at: ts.clone(),
                    started_at_secs,
                    close_deadline_secs,
                    entity_id: ctx_entity_id,
                });
            }
        }

        Some(PlayerEvent::DelayLoopStarted {
            timestamp: ts,
            duration,
            action_type,
            label,
            entity_id,
            abort_condition,
        })
    }

    /// ProcessScreenText(category, "message")
    fn parse_screen_text(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessScreenText(")? + "ProcessScreenText(".len();
        let args = &line[args_start..];

        // Category is before the first comma
        let first_comma = args.find(',')?;
        let category = args[..first_comma].trim().to_string();

        // Message is the quoted string
        let rest = &args[first_comma + 1..];
        let q_start = rest.find('"')? + 1;
        let q_end = rest.rfind('"')?;
        if q_start >= q_end {
            return None;
        }
        let message = rest[q_start..q_end].to_string();

        Some(PlayerEvent::ScreenText {
            timestamp: ts,
            category,
            message,
        })
    }

    /// ProcessBook("title", "content", "bookType", ...)
    fn parse_book(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessBook(")? + "ProcessBook(".len();
        let args = &line[args_start..];

        // Extract first three quoted strings
        let title = extract_quoted_string(args, 0)?;
        let after_title = &args[args.find(&format!("\"{}\"", title))? + title.len() + 2..];
        let content = extract_quoted_string(after_title, 0)?;
        let after_content =
            &after_title[after_title.find(&format!("\"{}\"", content))? + content.len() + 2..];
        let book_type = extract_quoted_string(after_content, 0)?;

        Some(PlayerEvent::BookOpened {
            timestamp: ts,
            title,
            content,
            book_type,
        })
    }

    /// ProcessEndInteraction(entityId)
    /// ProcessTalkScreen(entityId, "title", "body", "footer", ints[], strs[], N, Category)
    ///
    /// Opens a CorpseSearch context when title starts with "Search Corpse of ".
    /// Other TalkScreen categories (Generic, etc.) are ignored here. No event
    /// is emitted — this is pure context state.
    fn handle_talk_screen(&mut self, line: &str) {
        let ts = match parse_timestamp(line) {
            Some(t) => t,
            None => return,
        };
        let args_start = match line.find("ProcessTalkScreen(") {
            Some(i) => i + "ProcessTalkScreen(".len(),
            None => return,
        };
        let args = &line[args_start..];

        // entity_id is first before the comma
        let first_comma = match args.find(',') {
            Some(i) => i,
            None => return,
        };
        let entity_id: u32 = match args[..first_comma].trim().parse() {
            Ok(id) => id,
            Err(_) => return,
        };

        // Title is the first quoted string after entity_id
        let rest = &args[first_comma + 1..];
        let q_start = match rest.find('"') {
            Some(i) => i + 1,
            None => return,
        };
        let q_end = match rest[q_start..].find('"') {
            Some(i) => i + q_start,
            None => return,
        };
        let title = &rest[q_start..q_end];

        if let Some(corpse_name) = title.strip_prefix("Search Corpse of ") {
            let started_at_secs = timestamp_to_secs(&ts);
            let source = ActivitySource::CorpseSearch {
                entity_id,
                corpse_name: corpse_name.trim().to_string(),
            };
            // Deduplicate: if we already have a CorpseSearch for this entity,
            // just refresh its deadline.
            let already = self
                .activity_contexts
                .iter_mut()
                .find(|c| c.entity_id == Some(entity_id)
                    && matches!(&c.source, ActivitySource::CorpseSearch { .. }));
            if let Some(existing) = already {
                existing.close_deadline_secs = started_at_secs + DEFAULT_CONTEXT_LIFETIME_SECS;
            } else {
                self.push_activity(ActivityContext {
                    source,
                    started_at: ts,
                    started_at_secs,
                    close_deadline_secs: started_at_secs + DEFAULT_CONTEXT_LIFETIME_SECS,
                    entity_id: Some(entity_id),
                });
            }
        }
    }

    /// ProcessVendorScreen(npcId, ...) — opens VendorBrowsing context.
    /// The NPC name is enriched from a recent ProcessStartInteraction when
    /// available; it is not derivable from the VendorScreen args themselves.
    fn handle_vendor_screen(&mut self, line: &str) {
        let ts = match parse_timestamp(line) {
            Some(t) => t,
            None => return,
        };
        let args_start = match line.find("ProcessVendorScreen(") {
            Some(i) => i + "ProcessVendorScreen(".len(),
            None => return,
        };
        let first_comma = match line[args_start..].find(',') {
            Some(i) => args_start + i,
            None => return,
        };
        let npc_id: u32 = match line[args_start..first_comma].trim().parse() {
            Ok(id) => id,
            Err(_) => return,
        };

        let npc_name = self
            .pending_interaction
            .as_ref()
            .filter(|pi| pi.entity_id == npc_id)
            .and_then(|pi| pi.name.clone());

        let started_at_secs = timestamp_to_secs(&ts);
        self.push_activity(ActivityContext {
            source: ActivitySource::VendorBrowsing {
                npc_entity_id: npc_id,
                npc_name,
            },
            started_at: ts,
            started_at_secs,
            close_deadline_secs: started_at_secs + DEFAULT_CONTEXT_LIFETIME_SECS,
            entity_id: Some(npc_id),
        });
    }

    /// ProcessShowStorageVault(ownerId, _, "vault_name", ...) — opens
    /// StorageBrowsing context.
    fn handle_show_storage_vault(&mut self, line: &str) {
        let ts = match parse_timestamp(line) {
            Some(t) => t,
            None => return,
        };
        let args_start = match line.find("ProcessShowStorageVault(") {
            Some(i) => i + "ProcessShowStorageVault(".len(),
            None => return,
        };
        let args = &line[args_start..];

        let first_comma = match args.find(',') {
            Some(i) => i,
            None => return,
        };
        let owner_id: u32 = match args[..first_comma].trim().parse() {
            Ok(id) => id,
            Err(_) => return,
        };

        // Vault display name is the first quoted string after owner_id.
        let rest = &args[first_comma + 1..];
        let vault_name = match rest.find('"') {
            Some(q_start) => {
                let q_start = q_start + 1;
                match rest[q_start..].find('"') {
                    Some(q_end) => rest[q_start..(q_start + q_end)].to_string(),
                    None => String::new(),
                }
            }
            None => String::new(),
        };

        let started_at_secs = timestamp_to_secs(&ts);
        self.push_activity(ActivityContext {
            source: ActivitySource::StorageBrowsing {
                vault_owner_entity_id: owner_id,
                vault_name,
            },
            started_at: ts,
            started_at_secs,
            close_deadline_secs: started_at_secs + DEFAULT_CONTEXT_LIFETIME_SECS,
            entity_id: Some(owner_id),
        });
    }

    fn parse_end_interaction(&mut self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessEndInteraction(")? + "ProcessEndInteraction(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let entity_id: i32 = line[args_start..args_end].trim().parse().ok()?;

        // Clear interaction context
        self.current_interaction = None;

        // Close any activity contexts keyed to this entity_id. `entity_id` is
        // signed (occasional negative values for special targets like doors);
        // cast to u32 only when non-negative since activity contexts store u32.
        if entity_id >= 0 {
            self.close_activities_for_entity(entity_id as u32);
        }

        Some(PlayerEvent::InteractionEnded {
            timestamp: ts,
            entity_id,
        })
    }

    /// ProcessSetActiveSkills(Skill1, Skill2)
    fn parse_set_active_skills(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessSetActiveSkills(")? + "ProcessSetActiveSkills(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let skill1 = parts.get(0)?.trim().to_string();
        let skill2 = parts.get(1)?.trim().to_string();

        Some(PlayerEvent::ActiveSkillsChanged {
            timestamp: ts,
            skill1,
            skill2,
        })
    }

    /// ProcessPlayerMount(entityId, isMounting)
    fn parse_player_mount(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessPlayerMount(")? + "ProcessPlayerMount(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let entity_id: u32 = parts.get(0)?.trim().parse().ok()?;
        let is_mounting = parts.get(1)?.trim() == "True";

        Some(PlayerEvent::MountStateChanged {
            timestamp: ts,
            entity_id,
            is_mounting,
        })
    }

    /// ProcessSetWeather("WeatherName", boolFlag)
    fn parse_set_weather(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessSetWeather(")? + "ProcessSetWeather(".len();
        let args = &line[args_start..];

        // Extract quoted weather name
        let weather_name = extract_quoted_string(args, 0)?;

        // Find bool after the closing quote
        let after_quote = args.rfind('"')? + 1;
        let rest = &args[after_quote..];
        let last_comma = rest.find(',')?;
        let bool_str = rest[last_comma + 1..].trim().trim_end_matches(')');
        let is_active = bool_str == "True";

        Some(PlayerEvent::WeatherChanged {
            timestamp: ts,
            weather_name,
            is_active,
        })
    }

    /// ProcessUpdateRecipe(recipeId, completionCount)
    fn parse_update_recipe(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessUpdateRecipe(")? + "ProcessUpdateRecipe(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let recipe_id: u32 = parts.get(0)?.trim().parse().ok()?;
        let completion_count: u32 = parts.get(1)?.trim().parse().ok()?;

        Some(PlayerEvent::RecipeUpdated {
            timestamp: ts,
            recipe_id,
            completion_count,
        })
    }

    /// ProcessCombatModeStatus(status, System.Int32[])
    fn parse_combat_mode_status(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessCombatModeStatus(")? + "ProcessCombatModeStatus(".len();
        let args = &line[args_start..];

        // First arg before the comma is the status enum
        let first_comma = args.find(',')?;
        let status = args[..first_comma].trim();

        let in_combat = match status {
            "InCombat" => true,
            "NotInCombat" => false,
            _ => return None,
        };

        Some(PlayerEvent::CombatStateChanged {
            timestamp: ts,
            in_combat,
        })
    }

    /// ProcessVendorUpdateAvailableGold(currentGold, serverId, maxGold)
    fn parse_vendor_update_gold(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessVendorUpdateAvailableGold(")?
            + "ProcessVendorUpdateAvailableGold(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let current_gold: u32 = parts.get(0)?.trim().parse().ok()?;
        let server_id: u64 = parts.get(1)?.trim().parse().ok()?;
        let max_gold: u32 = parts.get(2)?.trim().parse().ok()?;

        Some(PlayerEvent::VendorGoldChanged {
            timestamp: ts,
            current_gold,
            server_id,
            max_gold,
        })
    }

    /// ProcessSetAttributes(entityId, "[KEY1, KEY2, ...], [val1, val2, ...]")
    fn parse_set_attributes(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessSetAttributes(")? + "ProcessSetAttributes(".len();
        let args = &line[args_start..];

        // Extract entityId (everything before the first comma that precedes the first '[')
        let first_bracket = args.find('[')?;
        let entity_part = &args[..first_bracket];
        // entityId is the first number, separated by ", " from the rest
        let first_comma = entity_part.find(',')?;
        let entity_id: u32 = args[..first_comma].trim().parse().ok()?;

        // Find the keys array: first '[' to first ']'
        let keys_start = first_bracket + 1;
        let keys_end = args[keys_start..].find(']')? + keys_start;
        let keys_str = &args[keys_start..keys_end];

        let keys: Vec<String> = keys_str
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();

        // Find the values array: second '[' to second ']'
        let after_first_array = &args[keys_end + 1..];
        let vals_bracket = after_first_array.find('[')?;
        let vals_start = vals_bracket + 1;
        let vals_end = after_first_array[vals_start..].find(']')? + vals_start;
        let vals_str = &after_first_array[vals_start..vals_end];

        let values: Vec<f32> = vals_str
            .split(',')
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<_>, _>>()
            .ok()?;

        // Arrays must match in length
        if keys.len() != values.len() {
            return None;
        }

        let attributes: Vec<AttributeValue> = keys
            .into_iter()
            .zip(values.into_iter())
            .map(|(name, value)| AttributeValue { name, value })
            .collect();

        Some(PlayerEvent::AttributesChanged {
            timestamp: ts,
            entity_id,
            attributes,
        })
    }
    /// ProcessLoadAbilities(System.Int32[], Skill1, Skill2, AbilityBarContents[])
    /// The Int32[] and AbilityBarContents[] are opaque C# serialized types.
    /// We extract only the two skill names.
    fn parse_load_abilities(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessLoadAbilities(")? + "ProcessLoadAbilities(".len();
        let args_end = line.rfind(')')?;
        let args = &line[args_start..args_end];

        // Format: "System.Int32[], Skill1, Skill2, AbilityBarContents[]"
        let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
        // parts[0] = "System.Int32[]", parts[1] = Skill1, parts[2] = Skill2, parts[3] = "AbilityBarContents[]"
        if parts.len() < 4 {
            return None;
        }

        let skill1 = parts[1].to_string();
        let skill2 = parts[2].to_string();

        Some(PlayerEvent::AbilitiesLoaded {
            timestamp: ts,
            skill1,
            skill2,
        })
    }

    /// ProcessLoadRecipes(System.Int32[], System.Int32[])
    /// Both arrays are opaque C# serialized types — no data extractable.
    /// We emit a signal event with just the timestamp.
    fn parse_load_recipes(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        // Verify the line actually contains the event
        line.find("ProcessLoadRecipes(")?;

        Some(PlayerEvent::RecipesLoaded { timestamp: ts })
    }

    /// ProcessSetEquippedItems(System.Int32[], System.Int32[], System.Int32[], "appearanceString", entityId)
    /// The Int32[] arrays are opaque. We extract the appearance string and entity ID,
    /// then parse equipment slot assignments from the appearance string.
    fn parse_set_equipped_items(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessSetEquippedItems(")? + "ProcessSetEquippedItems(".len();

        // The appearance string is quoted — find it
        let appearance = extract_quoted_string(&line[args_start..], 0)?;

        // Entity ID is after the closing quote + ", "
        let quote_end = line[args_start..].rfind('"')? + args_start;
        let after_quote = &line[quote_end + 1..];
        // after_quote looks like: ", 11921435)"
        let entity_str = after_quote
            .trim()
            .trim_start_matches(',')
            .trim()
            .trim_end_matches(')');
        let entity_id: u32 = entity_str.parse().ok()?;

        // Parse equipment slots from appearance string
        let equipment = parse_equipment_slots(&appearance);

        Some(PlayerEvent::EquipmentChanged {
            timestamp: ts,
            entity_id,
            appearance,
            equipment,
        })
    }

    /// ProcessAddEffects(entityId, sourceEntityId, "[effectId1, effectId2, ...]", boolFlag)
    fn parse_add_effects(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessAddEffects(")? + "ProcessAddEffects(".len();
        let args = &line[args_start..];

        let first_comma = args.find(',')?;
        let entity_id: u32 = args[..first_comma].trim().parse().ok()?;

        let rest = &args[first_comma + 1..];
        let second_comma = rest.find(',')?;
        let source_entity_id: u32 = rest[..second_comma].trim().parse().ok()?;

        // Effect IDs are in "[...]" — may contain quoted brackets like "[302, 303, ...]"
        let bracket_start = rest.find('[')?;
        let bracket_end = rest.find(']')?;
        let ids_str = &rest[bracket_start + 1..bracket_end];

        let effect_ids: Vec<u32> = ids_str
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .collect();

        // boolFlag is after the "]" bracket — e.g. `", False)` or `], False)`
        let rest_after = &rest[bracket_end + 1..];
        // Strip quotes, commas, parens to isolate the bool token
        let bool_str = rest_after
            .trim()
            .trim_start_matches('"')
            .trim_start_matches(',')
            .trim()
            .trim_end_matches(')')
            .trim_end_matches('"')
            .trim();
        let is_login_batch = bool_str == "False";

        Some(PlayerEvent::EffectsAdded {
            timestamp: ts,
            entity_id,
            source_entity_id,
            effect_ids,
            is_login_batch,
        })
    }

    /// ProcessRemoveEffects(entityId, System.Int32[])
    /// The int array is C#'s opaque ToString() — we can't extract individual IDs.
    fn parse_remove_effects(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessRemoveEffects(")? + "ProcessRemoveEffects(".len();
        let args = &line[args_start..];

        let first_comma = args.find(',')?;
        let entity_id: u32 = args[..first_comma].trim().parse().ok()?;

        Some(PlayerEvent::EffectsRemoved {
            timestamp: ts,
            entity_id,
        })
    }

    /// ProcessUpdateEffectName(entityId, effectInstanceId, "Effect Name, Level N")
    fn parse_update_effect_name(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessUpdateEffectName(")? + "ProcessUpdateEffectName(".len();
        let args = &line[args_start..];

        let first_comma = args.find(',')?;
        let entity_id: u32 = args[..first_comma].trim().parse().ok()?;

        let rest = &args[first_comma + 1..];
        let second_comma = rest.find(',')?;
        let effect_instance_id: u32 = rest[..second_comma].trim().parse().ok()?;

        // Display name is in quotes
        let display_name = extract_quoted_string(rest, 0)?;

        Some(PlayerEvent::EffectNameUpdated {
            timestamp: ts,
            entity_id,
            effect_instance_id,
            display_name,
        })
    }

    /// ProcessUpdateDescription(entityId, "name", "description", "action", actionType, "appearance", flags)
    fn parse_update_description(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start =
            line.find("ProcessUpdateDescription(")? + "ProcessUpdateDescription(".len();
        let args = &line[args_start..];

        // entityId is the first token before the first comma
        let first_comma = args.find(',')?;
        let entity_id: u32 = args[..first_comma].trim().parse().ok()?;

        // Quoted strings: name(0), description(1), action(2), appearance(3)
        let name = extract_quoted_string(args, 0)?;
        let description = extract_quoted_string(args, 1)?;
        let action = extract_quoted_string(args, 2)?;
        let appearance = extract_quoted_string(args, 3)?;

        // actionType is the unquoted token between the 3rd closing quote and 4th opening quote
        // Find the end of the 3rd quoted string (action), then parse the token before the 4th quote
        let action_end = {
            let mut pos = 0;
            for _ in 0..6 {
                // skip 6 quote characters (3 pairs of open+close)
                pos = args[pos..].find('"')? + pos + 1;
            }
            pos
        };
        let before_appearance = &args[action_end..];
        let action_type = before_appearance
            .split('"')
            .next()?
            .trim()
            .trim_matches(',')
            .trim()
            .to_string();

        // flags is the numeric value after the last closing quote
        let last_quote = args.rfind('"')?;
        let after_last = &args[last_quote + 1..];
        let flags_str = after_last
            .trim_start_matches(',')
            .trim()
            .trim_end_matches(')')
            .trim();
        let flags: u32 = flags_str.parse().unwrap_or(0);

        Some(PlayerEvent::EntityDescriptionUpdated {
            timestamp: ts,
            entity_id,
            name,
            description,
            action,
            action_type,
            appearance,
            flags,
        })
    }

    // ── New game state parsers ──────────────────────────────────────

    /// ProcessSetCelestialInfo(WaxingCrescentMoon)
    fn parse_celestial_info(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let start = line.find("ProcessSetCelestialInfo(")? + "ProcessSetCelestialInfo(".len();
        let end = line[start..].find(')')? + start;
        let phase = line[start..end].trim().to_string();
        if phase.is_empty() {
            return None;
        }
        Some(PlayerEvent::MoonPhaseChanged {
            timestamp: ts,
            phase,
        })
    }

    /// ProcessGuildGeneralInfo(guildId, "GuildName", "motd")
    fn parse_guild_general_info(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start =
            line.find("ProcessGuildGeneralInfo(")? + "ProcessGuildGeneralInfo(".len();
        let args = &line[args_start..];

        let first_comma = args.find(',')?;
        let guild_id: u32 = args[..first_comma].trim().parse().ok()?;

        let guild_name = extract_quoted_string(args, 0)?;
        let motd = extract_quoted_string(args, 1).unwrap_or_default();

        Some(PlayerEvent::GuildInfoLoaded {
            timestamp: ts,
            guild_id,
            guild_name,
            motd,
        })
    }

    /// ProcessCompleteDirectedGoals([3200,8000,1,70,...])
    fn parse_complete_directed_goals(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start =
            line.find("ProcessCompleteDirectedGoals(")? + "ProcessCompleteDirectedGoals(".len();
        let args = &line[args_start..];

        let bracket_start = args.find('[')? + 1;
        let bracket_end = args.find(']')?;
        let ids_str = &args[bracket_start..bracket_end];

        let goal_ids: Vec<u32> = ids_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        Some(PlayerEvent::DirectedGoalsLoaded {
            timestamp: ts,
            goal_ids,
        })
    }

    /// ProcessSetString(KEY, "value")
    /// Only emits events for known useful keys.
    fn parse_set_string(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessSetString(")? + "ProcessSetString(".len();
        let args = &line[args_start..];

        let first_comma = args.find(',')?;
        let key = args[..first_comma].trim().to_string();

        // Only persist known useful keys
        match key.as_str() {
            "NOTEPAD" | "NOTEPAD_TAB_1" | "NOTEPAD_TAB_2" | "NOTEPAD_TAB_3"
            | "NOTEPAD_TAB_4" | "NOTEPAD_TAB_NAMES" | "FRIEND_STATUS" | "PUBLIC_STATUS"
            | "HUNTING_GROUP_TITLE" => {}
            _ => return None,
        }

        let value = extract_quoted_string(args, 0)?;

        Some(PlayerEvent::PlayerStringUpdated {
            timestamp: ts,
            key,
            value,
        })
    }
}

/// Classify a DoDelayLoop into an ActivitySource, or None if the loop isn't
/// a known item-source activity.
///
/// The action_type + label combination is the reliable signal here. Labels
/// have appeared in real logs with minor formatting variations (`"Mining..."`
/// vs `"Mining ..."`) so comparisons must be lenient.
fn classify_delay_loop(
    action_type: &str,
    label: &str,
    pending_interaction: &Option<PendingInteractionMeta>,
) -> Option<ActivitySource> {
    let label_trimmed = label.trim();
    let label_trimmed_lower = label_trimmed.to_lowercase();

    // Mining: "Mining..." (possibly with a trailing space before the dots in
    // older logs). ChopLumber action_type is a reasonable additional filter
    // but we don't require it strictly since label is already diagnostic.
    // Strip spaces and trailing dots to unify variants.
    let label_core: String = label_trimmed_lower
        .trim_end_matches(|c: char| c == '.' || c.is_whitespace())
        .to_string();
    if label_core == "mining" {
        // Adopt node_entity_id / node_name from a recent StartInteraction if
        // any (interaction fires just before the delay loop when present).
        let (node_entity_id, node_name) = match pending_interaction {
            Some(pi) => (Some(pi.entity_id), pi.name.clone()),
            None => (None, None),
        };
        return Some(ActivitySource::Mining {
            node_entity_id,
            node_name,
        });
    }

    // Survey map crafting: label "Surveying" (with the Surveying skill action).
    if label_core == "surveying" {
        return Some(ActivitySource::SurveyMapCraft);
    }

    // Survey map use: label "Using <X> Survey" (or "Using <X> Motherlode Map").
    // Extract the map internal name best-effort from the label.
    if let Some(rest) = label_trimmed.strip_prefix("Using ") {
        let map_name = rest.trim().to_string();
        // Survey or motherlode map-specific suffixes
        if map_name.ends_with(" Survey") || map_name.ends_with(" Motherlode Map") {
            return Some(ActivitySource::SurveyMapUse {
                survey_map_internal_name: Some(map_name),
            });
        }
        // Other "Using X" labels (food, consumables) are generic craft/use
        // events — still worth tracking so gains during consumption are
        // attributable, even if rarely produce items.
        return Some(ActivitySource::GeneralCraft {
            action_type: action_type.to_string(),
            label: label_trimmed.to_string(),
        });
    }

    // Explicit generic-craft action types. These labels are recipe names.
    let craft_actions = [
        "ChopLumber",
        "Cook",
        "Brew",
        "Distill",
        "Refine",
        "Smelt",
        "Weave",
        "Assemble",
        "Forge",
    ];
    if craft_actions.iter().any(|a| action_type == *a) && label_core != "mining" {
        return Some(ActivitySource::GeneralCraft {
            action_type: action_type.to_string(),
            label: label_trimmed.to_string(),
        });
    }

    None
}

/// Compare two ActivitySource values for "same kind and same target" — used to
/// collapse repeated DoDelayLoop emissions on the same interaction rather than
/// stacking duplicates.
fn activity_source_matches(a: &ActivitySource, b: &ActivitySource) -> bool {
    match (a, b) {
        (
            ActivitySource::Mining {
                node_entity_id: ea, ..
            },
            ActivitySource::Mining {
                node_entity_id: eb, ..
            },
        ) => ea == eb,
        (ActivitySource::SurveyMapCraft, ActivitySource::SurveyMapCraft) => true,
        (
            ActivitySource::SurveyMapUse {
                survey_map_internal_name: a,
            },
            ActivitySource::SurveyMapUse {
                survey_map_internal_name: b,
            },
        ) => a == b,
        (
            ActivitySource::GeneralCraft {
                action_type: at_a,
                label: l_a,
            },
            ActivitySource::GeneralCraft {
                action_type: at_b,
                label: l_b,
            },
        ) => at_a == at_b && l_a == l_b,
        (
            ActivitySource::CorpseSearch { entity_id: a, .. },
            ActivitySource::CorpseSearch { entity_id: b, .. },
        ) => a == b,
        (
            ActivitySource::VendorBrowsing {
                npc_entity_id: a, ..
            },
            ActivitySource::VendorBrowsing {
                npc_entity_id: b, ..
            },
        ) => a == b,
        (
            ActivitySource::StorageBrowsing {
                vault_owner_entity_id: a,
                ..
            },
            ActivitySource::StorageBrowsing {
                vault_owner_entity_id: b,
                ..
            },
        ) => a == b,
        _ => false,
    }
}

/// Parse equipment slot assignments from an appearance string.
/// Looks for patterns like `@Chest=...;`, `@MainHand=...;`, `MainHandEquip=...;`
fn parse_equipment_slots(appearance: &str) -> Vec<EquipmentSlot> {
    let slot_prefixes = [
        "@Chest=",
        "@Head=",
        "@Legs=",
        "@Feet=",
        "@Hands=",
        "@MainHand=",
        "@OffHandShield=",
        "@Racial=",
    ];
    let equip_type_prefixes = ["MainHandEquip=", "OffHandEquip="];

    let mut slots = Vec::new();

    for prefix in &slot_prefixes {
        if let Some(start) = appearance.find(prefix) {
            let value_start = start + prefix.len();
            // Value runs until the next ';' or ')' at the same nesting depth
            let value = extract_slot_value(&appearance[value_start..]);
            if !value.is_empty() {
                slots.push(EquipmentSlot {
                    slot: prefix
                        .trim_start_matches('@')
                        .trim_end_matches('=')
                        .to_string(),
                    appearance_key: value,
                });
            }
        }
    }

    for prefix in &equip_type_prefixes {
        if let Some(start) = appearance.find(prefix) {
            let value_start = start + prefix.len();
            let value = extract_slot_value(&appearance[value_start..]);
            if !value.is_empty() {
                slots.push(EquipmentSlot {
                    slot: prefix.trim_end_matches('=').to_string(),
                    appearance_key: value,
                });
            }
        }
    }

    slots
}

/// Extract a value from an appearance string starting at a position.
/// Handles nested parentheses: `@eq-f2-chest-steel-02(^Armor=...;Color1=...)`
/// Returns everything up to the `;` or end that closes the value at depth 0.
fn extract_slot_value(s: &str) -> String {
    let mut depth = 0i32;
    let mut end = s.len();

    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    end = i;
                    break;
                }
                depth -= 1;
            }
            ';' if depth == 0 => {
                end = i;
                break;
            }
            _ => {}
        }
    }

    s[..end].to_string()
}

// ============================================================
// Helper Functions
// ============================================================

/// Parse a single {type=X,raw=R,bonus=B,xp=X,tnl=T,max=M} block
fn parse_skill_block(block: &str) -> Option<SkillSnapshot> {
    let skill_type = extract_block_field(block, "type=")?;
    let raw: u32 = extract_block_field(block, "raw=")?.parse().ok()?;
    let bonus: u32 = extract_block_field(block, "bonus=")?.parse().ok()?;
    let xp: u32 = extract_block_field(block, "xp=")?.parse().ok()?;
    let tnl: i32 = extract_block_field(block, "tnl=")?.parse().ok()?;
    let max: u32 = extract_block_field(block, "max=")?.parse().ok()?;

    Some(SkillSnapshot {
        skill_type,
        raw,
        bonus,
        xp,
        tnl,
        max,
    })
}

/// Extract a field value from within a {key=value,...} block
fn extract_block_field(block: &str, key: &str) -> Option<String> {
    let start = block.find(key)? + key.len();
    let rest = &block[start..];
    let end = rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

/// Extract the nth quoted string from text (0-indexed)
fn extract_quoted_string(text: &str, n: usize) -> Option<String> {
    let mut count = 0;
    let mut pos = 0;
    while pos < text.len() {
        if let Some(q_start) = text[pos..].find('"') {
            let abs_start = pos + q_start + 1;
            if let Some(q_end) = text[abs_start..].find('"') {
                if count == n {
                    return Some(text[abs_start..abs_start + q_end].to_string());
                }
                count += 1;
                pos = abs_start + q_end + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    None
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_add_item() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[16:17:48] LocalPlayer: ProcessAddItem(Malachite(115244857), -1, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemAdded {
                item_name,
                instance_id,
                slot_index,
                is_new,
                ..
            } => {
                assert_eq!(item_name, "Malachite");
                assert_eq!(*instance_id, 115244857);
                assert_eq!(*slot_index, -1);
                assert!(*is_new);
            }
            _ => panic!("Expected ItemAdded"),
        }
    }

    #[test]
    fn test_parse_add_item_login_load() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[16:00:00] LocalPlayer: ProcessAddItem(MetalSlab2(136937342), 5, False)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemAdded {
                item_name,
                is_new,
                slot_index,
                ..
            } => {
                assert_eq!(item_name, "MetalSlab2");
                assert!(!is_new);
                assert_eq!(*slot_index, 5);
            }
            _ => panic!("Expected ItemAdded"),
        }
    }

    #[test]
    fn test_parse_update_item_code_with_delta() {
        let mut parser = PlayerEventParser::new();

        // First register the item
        parser.process_line(
            r#"[16:00:00] LocalPlayer: ProcessAddItem(MetalSlab3(136937342), 5, False)"#,
        );
        // Set initial stack size
        parser.stack_sizes.insert(136937342, 20);

        let events = parser.process_line(
            r#"[16:17:48] LocalPlayer: ProcessUpdateItemCode(136937342, 1642723, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemStackChanged {
                instance_id,
                item_name,
                item_type_id,
                old_stack_size,
                new_stack_size,
                delta,
                from_server,
                ..
            } => {
                assert_eq!(*instance_id, 136937342);
                assert_eq!(item_name.as_deref(), Some("MetalSlab3"));
                assert_eq!(*item_type_id, 4323); // 1642723 & 0xFFFF
                assert_eq!(*old_stack_size, 20);
                assert_eq!(*new_stack_size, 26); // (1642723 >> 16) + 1; encoding is 0-based
                assert_eq!(*delta, 6);
                assert!(*from_server);
            }
            _ => panic!("Expected ItemStackChanged"),
        }
    }

    #[test]
    fn test_encoded_value_decoding() {
        // Encoding is 0-based: actual_stack = (encoded >> 16) + 1
        let encoded: u32 = 1642723;
        assert_eq!((encoded >> 16) + 1, 26); // actual stack size
        assert_eq!(encoded & 0xFFFF, 4323); // item type ID
    }

    #[test]
    fn test_delete_then_storage_transfer() {
        let mut parser = PlayerEventParser::new();

        // Register item first
        parser.process_line(
            r#"[13:27:07] LocalPlayer: ProcessAddItem(MapleWood(136093889), 10, False)"#,
        );

        // Delete item — should be buffered
        let events = parser.process_line(r#"[13:27:07] LocalPlayer: ProcessDeleteItem(136093889)"#);
        assert!(events.is_empty(), "Delete should be buffered");

        // Storage deposit — should resolve the pending delete
        let events = parser.process_line(
            r#"[13:27:07] LocalPlayer: ProcessAddToStorageVault(14804, -1, 40, MapleWood(136093889))"#
        );

        // Should have: ItemDeleted(StorageTransfer) + StorageDeposit
        assert_eq!(events.len(), 2);
        match &events[0] {
            PlayerEvent::ItemDeleted {
                instance_id,
                context,
                item_name,
                ..
            } => {
                assert_eq!(*instance_id, 136093889);
                assert_eq!(*context, DeleteContext::StorageTransfer);
                assert_eq!(item_name.as_deref(), Some("MapleWood"));
            }
            _ => panic!("Expected ItemDeleted with StorageTransfer"),
        }
        match &events[1] {
            PlayerEvent::StorageDeposit {
                item_name,
                instance_id,
                ..
            } => {
                assert_eq!(item_name, "MapleWood");
                assert_eq!(*instance_id, 136093889);
            }
            _ => panic!("Expected StorageDeposit"),
        }
    }

    #[test]
    fn test_delete_then_vendor_sale() {
        let mut parser = PlayerEventParser::new();

        parser.process_line(
            r#"[16:32:25] LocalPlayer: ProcessAddItem(AmuletOfCrushingMitigation5(115259296), 3, False)"#
        );

        let events = parser.process_line(r#"[16:32:25] LocalPlayer: ProcessDeleteItem(115259296)"#);
        assert!(events.is_empty());

        let events = parser.process_line(
            r#"[16:32:25] LocalPlayer: ProcessVendorAddItem(120, AmuletOfCrushingMitigation5(115259296), False)"#
        );
        assert_eq!(events.len(), 2);
        match &events[0] {
            PlayerEvent::ItemDeleted { context, .. } => {
                assert_eq!(*context, DeleteContext::VendorSale);
            }
            _ => panic!("Expected ItemDeleted with VendorSale"),
        }
        match &events[1] {
            PlayerEvent::VendorSold {
                price,
                item_name,
                is_buyback,
                ..
            } => {
                assert_eq!(*price, 120);
                assert_eq!(item_name, "AmuletOfCrushingMitigation5");
                assert!(!is_buyback);
            }
            _ => panic!("Expected VendorSold"),
        }
    }

    #[test]
    fn test_delete_standalone_flushed_as_unknown() {
        let mut parser = PlayerEventParser::new();

        parser.process_line(
            r#"[16:33:03] LocalPlayer: ProcessAddItem(SomeItem(114961794), 1, False)"#,
        );

        // Delete
        let events = parser.process_line(r#"[16:33:03] LocalPlayer: ProcessDeleteItem(114961794)"#);
        assert!(events.is_empty());

        // Unrelated line flushes pending
        let events = parser
            .process_line(r#"[16:33:04] entity_159956: OnAttackHitMe(Fiery Bite). Evaded = False"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemDeleted {
                instance_id,
                context,
                item_name,
                ..
            } => {
                assert_eq!(*instance_id, 114961794);
                assert_eq!(*context, DeleteContext::Unknown);
                assert_eq!(item_name.as_deref(), Some("SomeItem"));
            }
            _ => panic!("Expected ItemDeleted with Unknown"),
        }
    }

    #[test]
    fn test_parse_load_skills() {
        let mut parser = PlayerEventParser::new();
        let line = r#"[16:00:53] LocalPlayer: ProcessLoadSkills({type=Hammer,raw=70,bonus=5,xp=0,tnl=1153715,max=70},{type=Mentalism,raw=76,bonus=0,xp=2353127,tnl=2502977,max=80},{type=Gourmand,raw=49,bonus=0,xp=835,tnl=2500,max=100})"#;
        let events = parser.process_line(line);

        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::SkillsLoaded { skills, .. } => {
                assert_eq!(skills.len(), 3);
                assert_eq!(skills[0].skill_type, "Hammer");
                assert_eq!(skills[0].raw, 70);
                assert_eq!(skills[0].bonus, 5);
                assert_eq!(skills[0].tnl, 1153715);
                assert_eq!(skills[1].skill_type, "Mentalism");
                assert_eq!(skills[1].xp, 2353127);
                assert_eq!(skills[2].skill_type, "Gourmand");
                assert_eq!(skills[2].max, 100);
            }
            _ => panic!("Expected SkillsLoaded"),
        }
    }

    #[test]
    fn test_parse_load_skills_negative_tnl() {
        let mut parser = PlayerEventParser::new();
        let line = r#"[16:00:53] LocalPlayer: ProcessLoadSkills({type=Compassion,raw=50,bonus=0,xp=0,tnl=-1,max=50})"#;
        let events = parser.process_line(line);

        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::SkillsLoaded { skills, .. } => {
                assert_eq!(skills.len(), 1);
                assert_eq!(skills[0].tnl, -1);
            }
            _ => panic!("Expected SkillsLoaded"),
        }
    }

    #[test]
    fn test_parse_start_interaction() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[13:26:55] LocalPlayer: ProcessStartInteraction(14804, 7, 1200, True, "NPC_Qatik")"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::InteractionStarted {
                entity_id,
                interaction_type,
                npc_name,
                ..
            } => {
                assert_eq!(*entity_id, 14804);
                assert_eq!(*interaction_type, 7);
                assert_eq!(npc_name, "NPC_Qatik");
            }
            _ => panic!("Expected InteractionStarted"),
        }
    }

    #[test]
    fn test_parse_delta_favor() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[16:33:03] LocalPlayer: ProcessDeltaFavor(9618, "NPC_Kalaba", 2.8476, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::FavorChanged {
                npc_id,
                npc_name,
                delta,
                is_gift,
                ..
            } => {
                assert_eq!(*npc_id, 9618);
                assert_eq!(npc_name, "NPC_Kalaba");
                assert!((delta - 2.8476).abs() < 0.001);
                assert!(*is_gift);
            }
            _ => panic!("Expected FavorChanged"),
        }
    }

    #[test]
    fn test_parse_screen_text() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[17:37:01] LocalPlayer: ProcessScreenText(ImportantInfo, "The treasure is 342 meters from here.")"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ScreenText {
                category, message, ..
            } => {
                assert_eq!(category, "ImportantInfo");
                assert_eq!(message, "The treasure is 342 meters from here.");
            }
            _ => panic!("Expected ScreenText"),
        }
    }

    #[test]
    fn test_parse_remove_from_storage() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[13:28:48] LocalPlayer: ProcessRemoveFromStorageVault(14804, -1, 132702881, 11)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::StorageWithdrawal {
                npc_id,
                instance_id,
                quantity,
                ..
            } => {
                assert_eq!(*npc_id, 14804);
                assert_eq!(*instance_id, 132702881);
                assert_eq!(*quantity, 11);
            }
            _ => panic!("Expected StorageWithdrawal"),
        }
    }

    #[test]
    fn test_parse_vendor_update_item() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[16:32:27] LocalPlayer: ProcessVendorUpdateItem(115249145, 200909, 7)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::VendorStackUpdated {
                instance_id,
                item_type_id,
                new_stack_size,
                price,
                ..
            } => {
                assert_eq!(*instance_id, 115249145);
                // (200909 >> 16) + 1 = 4, 200909 & 0xFFFF = 4301
                assert_eq!(*new_stack_size, 4);
                assert_eq!(*item_type_id, 4301);
                assert_eq!(*price, 7);
            }
            _ => panic!("Expected VendorStackUpdated"),
        }
    }

    #[test]
    fn test_parse_book() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[13:26:04] LocalPlayer: ProcessBook("Yesterday's Shop Logs", "Toncom bought Guava x5", "PlayerShopLog", "", "", False, False, False, False, False, "")"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::BookOpened {
                title,
                content,
                book_type,
                ..
            } => {
                assert_eq!(title, "Yesterday's Shop Logs");
                assert_eq!(content, "Toncom bought Guava x5");
                assert_eq!(book_type, "PlayerShopLog");
            }
            _ => panic!("Expected BookOpened"),
        }
    }

    #[test]
    fn test_non_player_line_ignored() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[00:15:46] entity_192620: OnAttackHitMe(Spider Bite). Evaded = False"#,
        );
        assert!(events.is_empty());
    }

    #[test]
    fn test_flush_all_pending() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(r#"[16:00:00] LocalPlayer: ProcessAddItem(TestItem(12345), 1, False)"#);
        parser.process_line(r#"[16:33:03] LocalPlayer: ProcessDeleteItem(12345)"#);
        // No more lines — flush manually
        let events = parser.flush_all_pending();
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemDeleted {
                instance_id,
                context,
                ..
            } => {
                assert_eq!(*instance_id, 12345);
                assert_eq!(*context, DeleteContext::Unknown);
            }
            _ => panic!("Expected ItemDeleted"),
        }
    }

    #[test]
    fn test_delete_then_vendor_update_item() {
        let mut parser = PlayerEventParser::new();

        parser.process_line(
            r#"[16:32:27] LocalPlayer: ProcessAddItem(SomeStackable(115271948), 1, False)"#,
        );

        // Delete
        let events = parser.process_line(r#"[16:32:27] LocalPlayer: ProcessDeleteItem(115271948)"#);
        assert!(events.is_empty());

        // VendorUpdateItem (selling stackable vendor already has)
        let events = parser.process_line(
            r#"[16:32:27] LocalPlayer: ProcessVendorUpdateItem(115249145, 200909, 7)"#,
        );
        assert_eq!(events.len(), 2);
        match &events[0] {
            PlayerEvent::ItemDeleted { context, .. } => {
                assert_eq!(*context, DeleteContext::VendorSale);
            }
            _ => panic!("Expected ItemDeleted with VendorSale"),
        }
    }

    #[test]
    fn test_parse_delay_loop_surveying() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[00:08:36] LocalPlayer: ProcessDoDelayLoop(5, UseTeleportationCircle, "Surveying", 5305, AbortIfAttacked)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::DelayLoopStarted {
                duration,
                action_type,
                label,
                entity_id,
                abort_condition,
                ..
            } => {
                assert!((duration - 5.0).abs() < 0.01);
                assert_eq!(action_type, "UseTeleportationCircle");
                assert_eq!(label, "Surveying");
                assert_eq!(*entity_id, 5305);
                assert_eq!(abort_condition, "AbortIfAttacked");
            }
            _ => panic!("Expected DelayLoopStarted"),
        }
    }

    #[test]
    fn test_parse_delay_loop_using_survey() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[16:17:47] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Eltibule Green Mineral Survey", 5305, AbortIfAttacked)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::DelayLoopStarted {
                duration,
                action_type,
                label,
                ..
            } => {
                assert!((duration - 0.5).abs() < 0.01);
                assert_eq!(action_type, "Unset");
                assert_eq!(label, "Using Eltibule Green Mineral Survey");
            }
            _ => panic!("Expected DelayLoopStarted"),
        }
    }

    #[test]
    fn test_parse_delay_loop_eating() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[00:08:00] LocalPlayer: ProcessDoDelayLoop(1.5, Eat, "Using Gobbledygook", 6223, AbortIfAttacked)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::DelayLoopStarted {
                action_type,
                label,
                entity_id,
                ..
            } => {
                assert_eq!(action_type, "Eat");
                assert_eq!(label, "Using Gobbledygook");
                assert_eq!(*entity_id, 6223);
            }
            _ => panic!("Expected DelayLoopStarted"),
        }
    }

    #[test]
    fn test_instance_registry_baseline_no_event() {
        let mut parser = PlayerEventParser::new();
        // Existing item loaded at session start (is_new=False)
        parser.process_line(
            r#"[16:00:00] LocalPlayer: ProcessAddItem(MetalSlab2(136937342), 5, False)"#,
        );

        // First UpdateItemCode establishes baseline — no event emitted
        let events = parser.process_line(
            r#"[16:00:01] LocalPlayer: ProcessUpdateItemCode(136937342, 65536, True)"#,
        );
        assert!(events.is_empty(), "First UpdateItemCode for existing item should not emit an event");

        // Subsequent UpdateItemCode DOES emit a change event
        let events = parser.process_line(
            r#"[16:00:02] LocalPlayer: ProcessUpdateItemCode(136937342, 196608, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemStackChanged { item_name, old_stack_size, new_stack_size, delta, .. } => {
                assert_eq!(item_name.as_deref(), Some("MetalSlab2"));
                assert_eq!(*old_stack_size, 2); // (65536 >> 16) + 1
                assert_eq!(*new_stack_size, 4); // (196608 >> 16) + 1
                assert_eq!(*delta, 2);
            }
            _ => panic!("Expected ItemStackChanged"),
        }
    }

    #[test]
    fn test_new_item_seeds_stack_and_emits_change() {
        let mut parser = PlayerEventParser::new();
        // Genuinely new item (is_new=True, slot=-1) — seeds stack_size=1
        parser.process_line(
            r#"[16:00:00] LocalPlayer: ProcessAddItem(RoyalJelly(12345678), -1, True)"#,
        );

        // UpdateItemCode should emit change with delta = new - 1
        let events = parser.process_line(
            r#"[16:00:01] LocalPlayer: ProcessUpdateItemCode(12345678, 327697, True)"#,
        );
        // (327697 >> 16) + 1 = 6, 327697 & 0xFFFF = 17
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemStackChanged { item_name, old_stack_size, new_stack_size, delta, .. } => {
                assert_eq!(item_name.as_deref(), Some("RoyalJelly"));
                assert_eq!(*old_stack_size, 1); // seeded from ProcessAddItem(is_new=True)
                assert_eq!(*new_stack_size, 6); // (327697 >> 16) + 1
                assert_eq!(*delta, 5);
            }
            _ => panic!("Expected ItemStackChanged"),
        }
    }

    #[test]
    fn test_storage_withdrawal_seeds_stack_from_vault_quantity() {
        let mut parser = PlayerEventParser::new();

        // Storage withdrawal: AddItem with slot>=0 should NOT seed stack to 1
        let events = parser.process_line(
            r#"[15:46:07] LocalPlayer: ProcessAddItem(Phlogiston7(172708606), 60, True)"#,
        );
        assert_eq!(events.len(), 1);
        // Stack should NOT be seeded yet (slot>=0 = storage withdrawal)
        assert!(!parser.stack_sizes.contains_key(&172708606));

        // RemoveFromStorageVault seeds the stack from vault quantity
        let events = parser.process_line(
            r#"[15:46:07] LocalPlayer: ProcessRemoveFromStorageVault(302055, -1, 172708606, 1000)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::StorageWithdrawal { quantity, .. } => {
                assert_eq!(*quantity, 1000);
            }
            _ => panic!("Expected StorageWithdrawal"),
        }
        // Stack should now be seeded from vault quantity
        assert_eq!(parser.stack_sizes.get(&172708606), Some(&1000));

        // A subsequent UpdateItemCode should compute correct delta from 1000
        let events = parser.process_line(
            r#"[15:48:12] LocalPlayer: ProcessUpdateItemCode(172708606, 16346571, False)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemStackChanged { old_stack_size, new_stack_size, delta, .. } => {
                assert_eq!(*old_stack_size, 1000);
                // (16346571 >> 16) + 1 = 250
                assert_eq!(*new_stack_size, 250);
                assert_eq!(*delta, -750);
            }
            _ => panic!("Expected ItemStackChanged"),
        }
    }

    #[test]
    fn test_storage_withdrawal_no_false_delta_on_update_item_code() {
        let mut parser = PlayerEventParser::new();

        // Withdraw IceCore stack of 10 from storage
        parser.process_line(
            r#"[21:17:44] LocalPlayer: ProcessAddItem(IceCore(151917070), 68, True)"#,
        );
        parser.process_line(
            r#"[21:17:44] LocalPlayer: ProcessRemoveFromStorageVault(9964570, -1, 151917070, 10)"#,
        );

        // UpdateItemCode confirming stack=10 should have delta=0, not +9
        // encoded: (9 << 16) | 1153 = 590977, actual stack = 9+1 = 10
        let events = parser.process_line(
            r#"[21:17:44] LocalPlayer: ProcessUpdateItemCode(151917070, 590977, True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemStackChanged { old_stack_size, new_stack_size, delta, .. } => {
                assert_eq!(*old_stack_size, 10);
                assert_eq!(*new_stack_size, 10); // (590977 >> 16) + 1 = 10
                assert_eq!(*delta, 0); // No false gain!
            }
            _ => panic!("Expected ItemStackChanged"),
        }
    }

    // ============================================================
    // New Event Type Tests
    // ============================================================

    #[test]
    fn test_parse_end_interaction() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(r#"[13:27:00] LocalPlayer: ProcessEndInteraction(14804)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::InteractionEnded { entity_id, .. } => {
                assert_eq!(*entity_id, 14804);
            }
            _ => panic!("Expected InteractionEnded"),
        }
    }

    #[test]
    fn test_parse_end_interaction_negative_entity_id() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(r#"[16:06:27] LocalPlayer: ProcessEndInteraction(-153)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::InteractionEnded { entity_id, .. } => {
                assert_eq!(*entity_id, -153);
            }
            _ => panic!("Expected InteractionEnded"),
        }
    }

    #[test]
    fn test_end_interaction_clears_context() {
        let mut parser = PlayerEventParser::new();
        // Start an interaction
        parser.process_line(
            r#"[13:26:55] LocalPlayer: ProcessStartInteraction(14804, 7, 1200, True, "NPC_Qatik")"#,
        );
        assert!(parser.current_interaction.is_some());

        // End the interaction
        parser.process_line(r#"[13:27:00] LocalPlayer: ProcessEndInteraction(14804)"#);
        assert!(parser.current_interaction.is_none());
    }

    #[test]
    fn test_parse_set_active_skills() {
        let mut parser = PlayerEventParser::new();
        let events = parser
            .process_line(r#"[23:33:22] LocalPlayer: ProcessSetActiveSkills(Riding, Mentalism)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ActiveSkillsChanged { skill1, skill2, .. } => {
                assert_eq!(skill1, "Riding");
                assert_eq!(skill2, "Mentalism");
            }
            _ => panic!("Expected ActiveSkillsChanged"),
        }
    }

    #[test]
    fn test_parse_set_active_skills_combat() {
        let mut parser = PlayerEventParser::new();
        let events = parser
            .process_line(r#"[23:33:31] LocalPlayer: ProcessSetActiveSkills(Hammer, Mentalism)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ActiveSkillsChanged { skill1, skill2, .. } => {
                assert_eq!(skill1, "Hammer");
                assert_eq!(skill2, "Mentalism");
            }
            _ => panic!("Expected ActiveSkillsChanged"),
        }
    }

    #[test]
    fn test_parse_player_mount() {
        let mut parser = PlayerEventParser::new();
        let events =
            parser.process_line(r#"[23:33:25] LocalPlayer: ProcessPlayerMount(11921978, True)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::MountStateChanged {
                entity_id,
                is_mounting,
                ..
            } => {
                assert_eq!(*entity_id, 11921978);
                assert!(*is_mounting);
            }
            _ => panic!("Expected MountStateChanged"),
        }
    }

    #[test]
    fn test_parse_player_dismount() {
        let mut parser = PlayerEventParser::new();
        let events =
            parser.process_line(r#"[23:33:31] LocalPlayer: ProcessPlayerMount(11921978, False)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::MountStateChanged {
                entity_id,
                is_mounting,
                ..
            } => {
                assert_eq!(*entity_id, 11921978);
                assert!(!*is_mounting);
            }
            _ => panic!("Expected MountStateChanged"),
        }
    }

    #[test]
    fn test_parse_set_weather() {
        let mut parser = PlayerEventParser::new();
        let events =
            parser.process_line(r#"[23:32:47] LocalPlayer: ProcessSetWeather("Clear Sky", True)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::WeatherChanged {
                weather_name,
                is_active,
                ..
            } => {
                assert_eq!(weather_name, "Clear Sky");
                assert!(*is_active);
            }
            _ => panic!("Expected WeatherChanged"),
        }
    }

    #[test]
    fn test_parse_set_weather_cloudy() {
        let mut parser = PlayerEventParser::new();
        let events =
            parser.process_line(r#"[16:06:32] LocalPlayer: ProcessSetWeather("Cloudy 3", True)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::WeatherChanged { weather_name, .. } => {
                assert_eq!(weather_name, "Cloudy 3");
            }
            _ => panic!("Expected WeatherChanged"),
        }
    }

    #[test]
    fn test_parse_update_recipe() {
        let mut parser = PlayerEventParser::new();
        let events =
            parser.process_line(r#"[16:10:13] LocalPlayer: ProcessUpdateRecipe(21052, 151)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::RecipeUpdated {
                recipe_id,
                completion_count,
                ..
            } => {
                assert_eq!(*recipe_id, 21052);
                assert_eq!(*completion_count, 151);
            }
            _ => panic!("Expected RecipeUpdated"),
        }
    }

    #[test]
    fn test_parse_update_recipe_new() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(r#"[16:10:13] LocalPlayer: ProcessUpdateRecipe(5001, 0)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::RecipeUpdated {
                recipe_id,
                completion_count,
                ..
            } => {
                assert_eq!(*recipe_id, 5001);
                assert_eq!(*completion_count, 0);
            }
            _ => panic!("Expected RecipeUpdated"),
        }
    }

    #[test]
    fn test_parse_combat_mode_not_in_combat() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:33:33] LocalPlayer: ProcessCombatModeStatus(NotInCombat, System.Int32[])"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::CombatStateChanged { in_combat, .. } => {
                assert!(!*in_combat);
            }
            _ => panic!("Expected CombatStateChanged"),
        }
    }

    #[test]
    fn test_parse_combat_mode_in_combat() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:33:33] LocalPlayer: ProcessCombatModeStatus(InCombat, System.Int32[])"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::CombatStateChanged { in_combat, .. } => {
                assert!(*in_combat);
            }
            _ => panic!("Expected CombatStateChanged"),
        }
    }

    #[test]
    fn test_parse_vendor_update_gold() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[16:32:25] LocalPlayer: ProcessVendorUpdateAvailableGold(14880, 123456789, 15000)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::VendorGoldChanged {
                current_gold,
                server_id,
                max_gold,
                ..
            } => {
                assert_eq!(*current_gold, 14880);
                assert_eq!(*server_id, 123456789);
                assert_eq!(*max_gold, 15000);
            }
            _ => panic!("Expected VendorGoldChanged"),
        }
    }

    #[test]
    fn test_parse_vendor_update_gold_after_sale() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[16:32:27] LocalPlayer: ProcessVendorUpdateAvailableGold(14776, 123456789, 15000)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::VendorGoldChanged { current_gold, .. } => {
                assert_eq!(*current_gold, 14776);
            }
            _ => panic!("Expected VendorGoldChanged"),
        }
    }

    #[test]
    fn test_parse_set_attributes_single() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:33:25] LocalPlayer: ProcessSetAttributes(11921978, "[IS_MOUNTED], [1]")"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::AttributesChanged {
                entity_id,
                attributes,
                ..
            } => {
                assert_eq!(*entity_id, 11921978);
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].name, "IS_MOUNTED");
                assert!((attributes[0].value - 1.0).abs() < 0.001);
            }
            _ => panic!("Expected AttributesChanged"),
        }
    }

    #[test]
    fn test_parse_set_attributes_multiple() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:33:33] LocalPlayer: ProcessSetAttributes(11921978, "[CUR_HEALTH, MAX_HEALTH, CUR_POWER, MAX_POWER], [667, 667, 442, 442]")"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::AttributesChanged {
                entity_id,
                attributes,
                ..
            } => {
                assert_eq!(*entity_id, 11921978);
                assert_eq!(attributes.len(), 4);
                assert_eq!(attributes[0].name, "CUR_HEALTH");
                assert!((attributes[0].value - 667.0).abs() < 0.001);
                assert_eq!(attributes[1].name, "MAX_HEALTH");
                assert_eq!(attributes[2].name, "CUR_POWER");
                assert!((attributes[2].value - 442.0).abs() < 0.001);
                assert_eq!(attributes[3].name, "MAX_POWER");
            }
            _ => panic!("Expected AttributesChanged"),
        }
    }

    #[test]
    fn test_parse_set_attributes_float_value() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:32:47] LocalPlayer: ProcessSetAttributes(11921435, "[WORKORDER_COIN_REWARD_MOD], [1.36]")"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::AttributesChanged { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].name, "WORKORDER_COIN_REWARD_MOD");
                assert!((attributes[0].value - 1.36).abs() < 0.001);
            }
            _ => panic!("Expected AttributesChanged"),
        }
    }

    // ============================================================
    // Login Snapshot Event Tests
    // ============================================================

    #[test]
    fn test_parse_load_abilities() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:32:47] LocalPlayer: ProcessLoadAbilities(System.Int32[], Hammer, Mentalism, AbilityBarContents[])"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::AbilitiesLoaded { skill1, skill2, .. } => {
                assert_eq!(skill1, "Hammer");
                assert_eq!(skill2, "Mentalism");
            }
            _ => panic!("Expected AbilitiesLoaded"),
        }
    }

    #[test]
    fn test_parse_load_abilities_riding() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:33:25] LocalPlayer: ProcessLoadAbilities(System.Int32[], Riding, Mentalism, AbilityBarContents[])"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::AbilitiesLoaded { skill1, skill2, .. } => {
                assert_eq!(skill1, "Riding");
                assert_eq!(skill2, "Mentalism");
            }
            _ => panic!("Expected AbilitiesLoaded"),
        }
    }

    #[test]
    fn test_parse_load_recipes() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:46:08] LocalPlayer: ProcessLoadRecipes(System.Int32[], System.Int32[])"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::RecipesLoaded { timestamp, .. } => {
                assert_eq!(timestamp, "23:46:08");
            }
            _ => panic!("Expected RecipesLoaded"),
        }
    }

    #[test]
    fn test_parse_set_equipped_items() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:32:47] LocalPlayer: ProcessSetEquippedItems(System.Int32[], System.Int32[], System.Int32[], "@Base2-f(sex=f;race=h;@Chest=@eq-{sex}2-chest-steel-02(^Armor={sex}2-body-steel-02-thorian2;Color1=500050);@MainHand=eq-x-hammer1;MainHandEquip=Hammer;@Feet=@eq-{sex}2-feet-greaves-steel-02(^Armor={sex}2-feet-greaves-steel-02-thorian2;Color1=800080))", 11921435)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EquipmentChanged {
                entity_id,
                equipment,
                ..
            } => {
                assert_eq!(*entity_id, 11921435);
                // Should find Chest, MainHand, Feet, MainHandEquip
                let slot_names: Vec<&str> = equipment.iter().map(|s| s.slot.as_str()).collect();
                assert!(
                    slot_names.contains(&"Chest"),
                    "Missing Chest slot: {:?}",
                    slot_names
                );
                assert!(
                    slot_names.contains(&"MainHand"),
                    "Missing MainHand slot: {:?}",
                    slot_names
                );
                assert!(
                    slot_names.contains(&"Feet"),
                    "Missing Feet slot: {:?}",
                    slot_names
                );
                assert!(
                    slot_names.contains(&"MainHandEquip"),
                    "Missing MainHandEquip: {:?}",
                    slot_names
                );

                // Verify MainHand value
                let main_hand = equipment.iter().find(|s| s.slot == "MainHand").unwrap();
                assert_eq!(main_hand.appearance_key, "eq-x-hammer1");

                // Verify MainHandEquip value
                let equip_type = equipment
                    .iter()
                    .find(|s| s.slot == "MainHandEquip")
                    .unwrap();
                assert_eq!(equip_type.appearance_key, "Hammer");
            }
            _ => panic!("Expected EquipmentChanged"),
        }
    }

    #[test]
    fn test_parse_set_equipped_items_mount_entity() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:33:25] LocalPlayer: ProcessSetEquippedItems(System.Int32[], System.Int32[], System.Int32[], "@Base2-f(sex=f;race=h;@Head=FloatingGem2;@OffHandShield=eq-x-shield5;OffHandEquip=Shield)", 11921978)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EquipmentChanged {
                entity_id,
                equipment,
                ..
            } => {
                assert_eq!(*entity_id, 11921978);
                let slot_names: Vec<&str> = equipment.iter().map(|s| s.slot.as_str()).collect();
                assert!(
                    slot_names.contains(&"Head"),
                    "Missing Head: {:?}",
                    slot_names
                );
                assert!(
                    slot_names.contains(&"OffHandShield"),
                    "Missing OffHandShield: {:?}",
                    slot_names
                );
                assert!(
                    slot_names.contains(&"OffHandEquip"),
                    "Missing OffHandEquip: {:?}",
                    slot_names
                );
            }
            _ => panic!("Expected EquipmentChanged"),
        }
    }

    #[test]
    fn test_equipment_slot_parsing_nested_parens() {
        // Verify that nested parens in appearance values are handled correctly
        let slots = parse_equipment_slots(
            "@Chest=@eq-f2-chest-steel-02(^Armor=f2-body-steel-02;Color1=500050;Color2=C0C0C0);@Legs=@eq-f2-legs-steel-02(^Armor=f2-body-steel-02;Color1=500050)"
        );
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].slot, "Chest");
        assert_eq!(
            slots[0].appearance_key,
            "@eq-f2-chest-steel-02(^Armor=f2-body-steel-02;Color1=500050;Color2=C0C0C0)"
        );
        assert_eq!(slots[1].slot, "Legs");
        assert_eq!(
            slots[1].appearance_key,
            "@eq-f2-legs-steel-02(^Armor=f2-body-steel-02;Color1=500050)"
        );
    }

    // ── Effect Event Tests ───────────────────────────────────────────────

    #[test]
    fn test_parse_add_effects_login_batch() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:32:46] LocalPlayer: ProcessAddEffects(11921435, 0, "[302, 303, 13330, 26297]", False)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EffectsAdded {
                entity_id,
                source_entity_id,
                effect_ids,
                is_login_batch,
                ..
            } => {
                assert_eq!(*entity_id, 11921435);
                assert_eq!(*source_entity_id, 0);
                assert_eq!(*effect_ids, vec![302, 303, 13330, 26297]);
                assert!(*is_login_batch);
            }
            _ => panic!("Expected EffectsAdded"),
        }
    }

    #[test]
    fn test_parse_add_effects_gameplay() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:32:47] LocalPlayer: ProcessAddEffects(11921435, 11921435, "[13304, ]", True)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EffectsAdded {
                source_entity_id,
                effect_ids,
                is_login_batch,
                ..
            } => {
                assert_eq!(*source_entity_id, 11921435);
                assert_eq!(*effect_ids, vec![13304]);
                assert!(!*is_login_batch);
            }
            _ => panic!("Expected EffectsAdded"),
        }
    }

    #[test]
    fn test_parse_remove_effects() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:35:12] LocalPlayer: ProcessRemoveEffects(11921435, System.Int32[])"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EffectsRemoved { entity_id, .. } => {
                assert_eq!(*entity_id, 11921435);
            }
            _ => panic!("Expected EffectsRemoved"),
        }
    }

    #[test]
    fn test_parse_update_effect_name() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[23:32:48] LocalPlayer: ProcessUpdateEffectName(11921435, 123456, "Performance Appreciation, Level 0")"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EffectNameUpdated {
                entity_id,
                effect_instance_id,
                display_name,
                ..
            } => {
                assert_eq!(*entity_id, 11921435);
                assert_eq!(*effect_instance_id, 123456);
                assert_eq!(display_name, "Performance Appreciation, Level 0");
            }
            _ => panic!("Expected EffectNameUpdated"),
        }
    }

    #[test]
    fn test_parse_update_description_gardening() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[21:12:31] LocalPlayer: ProcessUpdateDescription(9960026, "Ripe Onion", "This onion is fully grown and in peak condition.", "Harvest Onion", UseItem, "Onion(Scale=1)", 0)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EntityDescriptionUpdated {
                entity_id,
                name,
                description,
                action,
                action_type,
                appearance,
                flags,
                ..
            } => {
                assert_eq!(*entity_id, 9960026);
                assert_eq!(name, "Ripe Onion");
                assert_eq!(
                    description,
                    "This onion is fully grown and in peak condition."
                );
                assert_eq!(action, "Harvest Onion");
                assert_eq!(action_type, "UseItem");
                assert_eq!(appearance, "Onion(Scale=1)");
                assert_eq!(*flags, 0);
            }
            _ => panic!("Expected EntityDescriptionUpdated"),
        }
    }

    #[test]
    fn test_parse_update_description_crafting() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[14:05:22] LocalPlayer: ProcessUpdateDescription(1601585, "Rising Simple Sourdough", "Proofing for 00:00:05", "Bake Simple Sourdough", UseItem, "Dough(Scale=0.36547)", 0)"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::EntityDescriptionUpdated {
                entity_id,
                name,
                description,
                action,
                ..
            } => {
                assert_eq!(*entity_id, 1601585);
                assert_eq!(name, "Rising Simple Sourdough");
                assert_eq!(description, "Proofing for 00:00:05");
                assert_eq!(action, "Bake Simple Sourdough");
            }
            _ => panic!("Expected EntityDescriptionUpdated"),
        }
    }

    // ============================================================
    // Activity Context tests (Phase 1)
    // ============================================================

    #[test]
    fn test_activity_mining_named_node() {
        // Real-world case: StartInteraction carries "MiningNodeFromSurvey9",
        // DoDelayLoop(6, ChopLumber, "Mining...") follows within the same second.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );

        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 1);
        match &ctxs[0].source {
            ActivitySource::Mining {
                node_entity_id,
                node_name,
            } => {
                assert_eq!(*node_entity_id, Some(435138));
                assert_eq!(node_name.as_deref(), Some("MiningNodeFromSurvey9"));
            }
            other => panic!("Expected Mining source, got {:?}", other),
        }
        assert_eq!(ctxs[0].entity_id, Some(435138));
    }

    #[test]
    fn test_activity_mining_unnamed_node() {
        // Motherlode / plain world nodes often have empty interaction names.
        // We must still detect Mining from the DoDelayLoop label alone.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[22:14:23] LocalPlayer: ProcessStartInteraction(1176954, 7, 0, False, "")"#,
        );
        parser.process_line(
            r#"[22:14:23] LocalPlayer: ProcessDoDelayLoop(7, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );

        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 1, "Mining context should be detected despite empty name");
        match &ctxs[0].source {
            ActivitySource::Mining {
                node_entity_id,
                node_name,
            } => {
                assert_eq!(*node_entity_id, Some(1176954));
                assert_eq!(node_name.as_deref(), None, "name was empty — stored as None");
            }
            other => panic!("Expected Mining source, got {:?}", other),
        }
    }

    #[test]
    fn test_activity_mining_label_space_variant() {
        // Both "Mining..." and "Mining ..." have appeared across game versions.
        // The classifier must treat them identically.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining ...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert!(matches!(
            parser.active_activities()[0].source,
            ActivitySource::Mining { .. }
        ));
    }

    #[test]
    fn test_activity_survey_map_use() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[15:43:36] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Serbule Blue Mineral Survey", 5305, AbortIfAttacked)"#,
        );

        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 1);
        match &ctxs[0].source {
            ActivitySource::SurveyMapUse {
                survey_map_internal_name,
            } => {
                assert_eq!(
                    survey_map_internal_name.as_deref(),
                    Some("Serbule Blue Mineral Survey")
                );
            }
            other => panic!("Expected SurveyMapUse, got {:?}", other),
        }
    }

    #[test]
    fn test_activity_survey_map_craft() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[15:24:59] LocalPlayer: ProcessDoDelayLoop(5, UseTeleportationCircle, "Surveying", 5305, AbortIfAttacked)"#,
        );
        assert_eq!(
            parser.active_activities()[0].source,
            ActivitySource::SurveyMapCraft
        );
    }

    #[test]
    fn test_activity_explicit_end_closes_mining() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert_eq!(parser.active_activities().len(), 1);

        parser.process_line(r#"[18:04:29] LocalPlayer: ProcessEndInteraction(435138)"#);
        assert_eq!(
            parser.active_activities().len(),
            0,
            "EndInteraction should close matching mining context"
        );
    }

    #[test]
    fn test_activity_times_out_after_deadline() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert_eq!(parser.active_activities().len(), 1);

        // 6s duration + 2s slack = should still be active at 18:04:30
        parser
            .process_line(r#"[18:04:30] LocalPlayer: ProcessCombatModeStatus(NotInCombat, [])"#);
        assert_eq!(parser.active_activities().len(), 1);

        // By 18:04:32 (9s after start) the context should have expired
        parser
            .process_line(r#"[18:04:32] LocalPlayer: ProcessCombatModeStatus(NotInCombat, [])"#);
        assert_eq!(parser.active_activities().len(), 0);
    }

    #[test]
    fn test_activity_repeated_delay_loop_does_not_stack() {
        // Some delay loops re-emit on every tick for the same activity. We
        // must refresh the deadline in place, not push duplicate contexts.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        parser.process_line(
            r#"[18:04:24] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        parser.process_line(
            r#"[18:04:25] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        assert_eq!(parser.active_activities().len(), 1, "same-entity repeated loops collapse");
    }

    #[test]
    fn test_activity_corpse_search_from_talk_screen() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:38] LocalPlayer: ProcessStartInteraction(435269, 10, 0, False, "")"#,
        );
        parser.process_line(
            r#"[18:04:38] LocalPlayer: ProcessTalkScreen(435269, "Search Corpse of Ratkin Miner", "", "", System.Int32[], System.String[], 0, Corpse)"#,
        );

        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 1);
        match &ctxs[0].source {
            ActivitySource::CorpseSearch {
                entity_id,
                corpse_name,
            } => {
                assert_eq!(*entity_id, 435269);
                assert_eq!(corpse_name, "Ratkin Miner");
            }
            other => panic!("Expected CorpseSearch, got {:?}", other),
        }
    }

    #[test]
    fn test_activity_vendor_browsing_enriched_with_npc_name() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[15:54:17] LocalPlayer: ProcessStartInteraction(32857, 7, 522.75, True, "NPC_FarsightFlemmings")"#,
        );
        parser.process_line(
            r#"[15:54:18] LocalPlayer: ProcessVendorScreen(32857, Friends, 20000, 0, 20000, "", VendorInfo[], VendorInfo[], VendorInfo[], VendorPurchaseCap[], [-1201,-1601,-1101,-1301,-2001,-1801,], System.String[], -1601)"#,
        );

        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 1);
        match &ctxs[0].source {
            ActivitySource::VendorBrowsing {
                npc_entity_id,
                npc_name,
            } => {
                assert_eq!(*npc_entity_id, 32857);
                assert_eq!(npc_name.as_deref(), Some("NPC_FarsightFlemmings"));
            }
            other => panic!("Expected VendorBrowsing, got {:?}", other),
        }
    }

    #[test]
    fn test_activity_storage_browsing_captures_vault_name() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[15:45:38] LocalPlayer: ProcessShowStorageVault(302055, 114, "Saddlebag", "Access saddlebag contents here", 80, System.Collections.Generic.List`1[Item], System.String[], "", [], System.String[], 0)"#,
        );

        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 1);
        match &ctxs[0].source {
            ActivitySource::StorageBrowsing {
                vault_owner_entity_id,
                vault_name,
            } => {
                assert_eq!(*vault_owner_entity_id, 302055);
                assert_eq!(vault_name, "Saddlebag");
            }
            other => panic!("Expected StorageBrowsing, got {:?}", other),
        }
    }

    #[test]
    fn test_activity_overlapping_contexts_coexist() {
        // Mining gets interrupted by combat, player kills attacker, searches
        // corpse — all before the mining context times out. Both contexts
        // should be present on the stack during the overlap.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        parser.process_line(
            r#"[18:04:29] LocalPlayer: ProcessStartInteraction(435214, 10, 0, False, "")"#,
        );
        parser.process_line(
            r#"[18:04:29] LocalPlayer: ProcessTalkScreen(435214, "Search Corpse of Ratkin Miner", "", "", System.Int32[], System.String[], 0, Corpse)"#,
        );

        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 2, "both mining and corpse-search should be active");
        // Mining was pushed first, corpse search second — stack preserves order
        assert!(matches!(ctxs[0].source, ActivitySource::Mining { .. }));
        assert!(matches!(ctxs[1].source, ActivitySource::CorpseSearch { .. }));
    }

    #[test]
    fn test_activity_closing_one_context_leaves_others() {
        // When the corpse search ends but mining hasn't expired, mining remains.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        parser.process_line(
            r#"[18:04:25] LocalPlayer: ProcessStartInteraction(435214, 10, 0, False, "")"#,
        );
        parser.process_line(
            r#"[18:04:25] LocalPlayer: ProcessTalkScreen(435214, "Search Corpse of Ratkin Miner", "", "", System.Int32[], System.String[], 0, Corpse)"#,
        );
        assert_eq!(parser.active_activities().len(), 2);

        parser.process_line(r#"[18:04:26] LocalPlayer: ProcessEndInteraction(435214)"#);
        let ctxs = parser.active_activities();
        assert_eq!(ctxs.len(), 1);
        assert!(matches!(ctxs[0].source, ActivitySource::Mining { .. }));
    }

    #[test]
    fn test_activity_non_matching_end_interaction_no_effect() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );

        // EndInteraction for a different entity — mining context should remain
        parser.process_line(r#"[18:04:24] LocalPlayer: ProcessEndInteraction(999999)"#);
        assert_eq!(parser.active_activities().len(), 1);
    }

    // ============================================================
    // Provenance attribution tests (Phase 2)
    // ============================================================

    /// Helper: extract the provenance from the first gain event in a batch.
    fn first_provenance(events: &[PlayerEvent]) -> ItemProvenance {
        for ev in events {
            match ev {
                PlayerEvent::ItemAdded { provenance, .. } => return provenance.clone(),
                PlayerEvent::ItemStackChanged { provenance, .. } => return provenance.clone(),
                PlayerEvent::StorageWithdrawal { provenance, .. } => return provenance.clone(),
                _ => {}
            }
        }
        panic!("no gain event with provenance in batch");
    }

    #[test]
    fn test_provenance_unknown_when_no_context() {
        // Genuine new item drop with nothing else going on: should be
        // UnknownSource. This bucket is expected to be sizeable.
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[12:00:00] LocalPlayer: ProcessAddItem(RandomLoot(12345), -1, True)"#,
        );
        assert_eq!(first_provenance(&events), ItemProvenance::UnknownSource);
    }

    #[test]
    fn test_provenance_session_load_is_not_applicable() {
        // AddItem with is_new=false is a session-load reload — not a gain,
        // even if contexts happen to be active.
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[12:00:00] LocalPlayer: ProcessAddItem(ExistingItem(999), -1, False)"#,
        );
        assert_eq!(first_provenance(&events), ItemProvenance::NotApplicable);
    }

    #[test]
    fn test_provenance_mining_confident_attribution() {
        // Mining context active → item drop attributes to Mining confidently.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:41] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:41] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        let events = parser.process_line(
            r#"[18:04:47] LocalPlayer: ProcessAddItem(Orichalcum(185976137), -1, True)"#,
        );

        match first_provenance(&events) {
            ItemProvenance::Attributed {
                source, confidence, ..
            } => {
                assert!(matches!(source, ActivitySource::Mining { .. }));
                assert_eq!(confidence, AttributionConfidence::Confident);
            }
            other => panic!("Expected Attributed Mining, got {:?}", other),
        }
    }

    #[test]
    fn test_provenance_survey_map_use() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[15:43:36] LocalPlayer: ProcessDoDelayLoop(0.5, Unset, "Using Serbule Blue Mineral Survey", 5305, AbortIfAttacked)"#,
        );
        let events = parser.process_line(
            r#"[15:43:36] LocalPlayer: ProcessAddItem(Fluorite(181871021), -1, True)"#,
        );
        match first_provenance(&events) {
            ItemProvenance::Attributed { source, .. } => {
                assert!(matches!(source, ActivitySource::SurveyMapUse { .. }));
            }
            other => panic!("Expected Attributed SurveyMapUse, got {:?}", other),
        }
    }

    #[test]
    fn test_provenance_mining_wins_over_passive_corpse_search() {
        // Real overlap scenario from the 50x-povus dataset: player mines a
        // multihit node, gets attacked between swings, kills the attacker,
        // and a corpse-search screen opens. Then the next mining swing
        // produces loot. With both Mining and CorpseSearch contexts active,
        // the loot belongs to Mining (the explicit active task). The
        // parser's tie-breaker should pick Mining with Probable confidence.
        //
        // This matters at scale: in the povus dataset, this overlap drops
        // ~40% of mining loot into Uncertain without the tie-breaker. With
        // it, mining-loot attribution rises from ~60% to ~99%.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessStartInteraction(435138, 7, 0, False, "MiningNodeFromSurvey9")"#,
        );
        parser.process_line(
            r#"[18:04:23] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        parser.process_line(
            r#"[18:04:25] LocalPlayer: ProcessStartInteraction(435214, 10, 0, False, "")"#,
        );
        parser.process_line(
            r#"[18:04:25] LocalPlayer: ProcessTalkScreen(435214, "Search Corpse of Ratkin Miner", "", "", System.Int32[], System.String[], 0, Corpse)"#,
        );
        let events = parser.process_line(
            r#"[18:04:26] LocalPlayer: ProcessAddItem(Orichalcum(99999), -1, True)"#,
        );
        match first_provenance(&events) {
            ItemProvenance::Attributed {
                source,
                confidence,
                ..
            } => {
                assert!(matches!(source, ActivitySource::Mining { .. }),
                    "expected Mining attribution, got {:?}", source);
                assert_eq!(confidence, AttributionConfidence::Probable,
                    "tie-breaker outcome should be Probable, not Confident");
            }
            other => panic!("Expected Attributed Mining, got {:?}", other),
        }
    }

    #[test]
    fn test_provenance_uncertain_when_two_non_mining_contexts() {
        // The Mining-vs-passive tie-breaker only applies when Mining is one
        // of the candidates. Other multi-context overlaps (e.g., corpse
        // search while a vendor screen is open) still produce Uncertain so
        // downstream code doesn't get a confident wrong answer.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[12:00:00] LocalPlayer: ProcessStartInteraction(100, 7, 0, False, "NPC_Vendor")"#,
        );
        parser.process_line(
            r#"[12:00:00] LocalPlayer: ProcessVendorScreen(100, Friends, 20000, 0, 20000, "", VendorInfo[], VendorInfo[], VendorInfo[], VendorPurchaseCap[], [], System.String[], -1601)"#,
        );
        parser.process_line(
            r#"[12:00:01] LocalPlayer: ProcessStartInteraction(200, 10, 0, False, "")"#,
        );
        parser.process_line(
            r#"[12:00:01] LocalPlayer: ProcessTalkScreen(200, "Search Corpse of Some Mob", "", "", System.Int32[], System.String[], 0, Corpse)"#,
        );
        let events = parser.process_line(
            r#"[12:00:02] LocalPlayer: ProcessAddItem(MysteryItem(77777), -1, True)"#,
        );
        match first_provenance(&events) {
            ItemProvenance::Uncertain { candidates } => {
                assert_eq!(candidates.len(), 2,
                    "expected two candidates for non-mining overlap");
            }
            other => panic!("Expected Uncertain, got {:?}", other),
        }
    }

    #[test]
    fn test_provenance_on_stack_change_positive_delta() {
        // Existing stack gaining items during mining → ItemStackChanged gets
        // Mining provenance. Simulates survey-loot-into-existing-stack.
        let mut parser = PlayerEventParser::new();
        // Establish baseline for an item
        parser.process_line(
            r#"[17:00:00] LocalPlayer: ProcessAddItem(Rubyquartz(555), -1, False)"#,
        );
        parser.process_line(
            r#"[17:00:01] LocalPlayer: ProcessUpdateItemCode(555, 131072, True)"#, // stack=3
        );

        // Open mining context
        parser.process_line(
            r#"[17:05:00] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        // Stack grows from 3 to 5 (+2) — should be attributed to Mining
        let events =
            parser.process_line(r#"[17:05:05] LocalPlayer: ProcessUpdateItemCode(555, 262144, True)"#);

        match first_provenance(&events) {
            ItemProvenance::Attributed { source, .. } => {
                assert!(matches!(source, ActivitySource::Mining { .. }));
            }
            other => panic!("Expected Attributed Mining, got {:?}", other),
        }
    }

    #[test]
    fn test_provenance_negative_delta_is_not_applicable() {
        // Stack shrinking (consumption during crafting, selling, etc.) is a
        // loss, not a gain. Provenance should be NotApplicable regardless of
        // active contexts so gain-aggregates don't include it.
        let mut parser = PlayerEventParser::new();
        parser.process_line(r#"[10:00:00] LocalPlayer: ProcessAddItem(Ink(42), -1, False)"#);
        parser.process_line(r#"[10:00:01] LocalPlayer: ProcessUpdateItemCode(42, 262144, True)"#); // stack=5

        // Open a crafting context
        parser.process_line(
            r#"[10:01:00] LocalPlayer: ProcessDoDelayLoop(5, UseTeleportationCircle, "Surveying", 5305, AbortIfAttacked)"#,
        );
        // Stack drops 5 → 4 (consumption)
        let events =
            parser.process_line(r#"[10:01:05] LocalPlayer: ProcessUpdateItemCode(42, 196608, True)"#);

        // Stack delta is -1 → should be NotApplicable
        assert_eq!(first_provenance(&events), ItemProvenance::NotApplicable);
    }

    #[test]
    fn test_provenance_storage_withdrawal_attributes_to_storage() {
        // Storage withdrawal should attribute to the StorageBrowsing context
        // whose owner matches the vault npc_id.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[15:45:38] LocalPlayer: ProcessShowStorageVault(302055, 114, "Saddlebag", "Access saddlebag contents here", 80, System.Collections.Generic.List`1[Item], System.String[], "", [], System.String[], 0)"#,
        );
        // ProcessAddItem for the withdrawal (slot>=0 branch doesn't seed stack)
        parser.process_line(
            r#"[15:46:07] LocalPlayer: ProcessAddItem(Phlogiston7(172708606), 60, True)"#,
        );
        let events = parser.process_line(
            r#"[15:46:07] LocalPlayer: ProcessRemoveFromStorageVault(302055, -1, 172708606, 1000)"#,
        );

        // There may be an ItemAdded event first; find the StorageWithdrawal.
        let prov = events
            .iter()
            .find_map(|ev| match ev {
                PlayerEvent::StorageWithdrawal { provenance, .. } => Some(provenance.clone()),
                _ => None,
            })
            .expect("StorageWithdrawal event present");

        match prov {
            ItemProvenance::Attributed {
                source, confidence, ..
            } => {
                match source {
                    ActivitySource::StorageBrowsing {
                        vault_owner_entity_id,
                        vault_name,
                    } => {
                        assert_eq!(vault_owner_entity_id, 302055);
                        assert_eq!(vault_name, "Saddlebag");
                    }
                    other => panic!("Expected StorageBrowsing, got {:?}", other),
                }
                assert_eq!(confidence, AttributionConfidence::Confident);
            }
            other => panic!("Expected Attributed StorageBrowsing, got {:?}", other),
        }
    }

    #[test]
    fn test_provenance_after_context_timeout_is_unknown() {
        // A mining context that times out before the item drops should
        // produce UnknownSource, not attribute to the stale context.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[12:00:00] LocalPlayer: ProcessDoDelayLoop(6, ChopLumber, "Mining...", 0, AbortIfAttacked, IsInteractorDelayLoop)"#,
        );
        // Item drops way past the 6s + 2s slack deadline
        let events = parser.process_line(
            r#"[12:01:00] LocalPlayer: ProcessAddItem(LateGain(777), -1, True)"#,
        );
        assert_eq!(first_provenance(&events), ItemProvenance::UnknownSource);
    }

    // ============================================================
    // Chat-gain quantity seeding tests (Phase 3)
    // ============================================================

    #[test]
    fn test_chat_gain_seeds_stack_from_chat_quantity() {
        // Classic vendor-buy case: player bought 5 brains. Chat reports qty=5
        // while player.log's AddItem has no quantity info. The chat event
        // arrives slightly before the player.log AddItem (or vice versa);
        // the parser must correlate them and seed the stack to 5.
        let mut parser = PlayerEventParser::new();
        parser.feed_chat_gain("Brain".to_string(), 5, "15:56:03");

        parser.process_line(
            r#"[15:56:03] LocalPlayer: ProcessAddItem(Brain(187042326), -1, True)"#,
        );

        // Stack should now be seeded at 5. A subsequent UpdateItemCode going
        // from 5 to 6 should report delta=+1, not +5 (as it would with the
        // old seed-to-1 behavior).
        let events = parser.process_line(
            // encoded 328790 → (328790 >> 16) + 1 = 6, typeId = 1110
            r#"[15:56:36] LocalPlayer: ProcessUpdateItemCode(187042326, 328790, True)"#,
        );
        let change = events.iter().find_map(|ev| match ev {
            PlayerEvent::ItemStackChanged { old_stack_size, new_stack_size, delta, .. } => {
                Some((*old_stack_size, *new_stack_size, *delta))
            }
            _ => None,
        }).expect("ItemStackChanged expected");

        assert_eq!(change.0, 5, "prior stack should have been seeded to 5 from chat");
        assert_eq!(change.1, 6);
        assert_eq!(change.2, 1);
    }

    #[test]
    fn test_chat_gain_no_match_falls_back_to_one() {
        // AddItem with no matching chat gain in the buffer should still
        // seed to 1 (the prior behavior) — correct for single-item pickups.
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[12:00:00] LocalPlayer: ProcessAddItem(Mushroom8(12345), -1, True)"#,
        );
        assert_eq!(*parser.stack_sizes.get(&12345).unwrap(), 1);
    }

    #[test]
    fn test_chat_gain_outside_window_is_ignored() {
        // Chat gain far in the past shouldn't match a new AddItem.
        let mut parser = PlayerEventParser::new();
        parser.feed_chat_gain("Fluorite".to_string(), 3, "12:00:00");

        // AddItem 10 minutes later — outside the match window
        parser.process_line(
            r#"[12:10:00] LocalPlayer: ProcessAddItem(Fluorite(999), -1, True)"#,
        );
        assert_eq!(
            *parser.stack_sizes.get(&999).unwrap(),
            1,
            "out-of-window chat gain must not seed"
        );
    }

    #[test]
    fn test_chat_gain_consumed_on_first_match() {
        // Two AddItems for the same item type, only one chat gain buffered.
        // The first AddItem should claim the chat quantity; the second falls
        // back to 1. This mirrors the real game behavior where a single
        // pickup can split across stacks — each chat line corresponds to one
        // destination stack.
        let mut parser = PlayerEventParser::new();
        parser.feed_chat_gain("Phlogiston7".to_string(), 750, "15:51:42");

        parser.process_line(
            r#"[15:51:42] LocalPlayer: ProcessAddItem(Phlogiston7(111), -1, True)"#,
        );
        parser.process_line(
            r#"[15:51:42] LocalPlayer: ProcessAddItem(Phlogiston7(222), -1, True)"#,
        );

        // First got the 750, second fell back
        assert_eq!(*parser.stack_sizes.get(&111).unwrap(), 750);
        assert_eq!(*parser.stack_sizes.get(&222).unwrap(), 1);
    }

    #[test]
    fn test_chat_gain_multiple_buffered_all_matchable() {
        // Split pickup case from paired log data: pickup of 1000 phlog
        // fires TWO chat lines (x750 + x250) for the two destination stacks.
        // Both should seed their respective AddItems.
        let mut parser = PlayerEventParser::new();
        parser.feed_chat_gain("Phlogiston7".to_string(), 750, "15:51:42");
        parser.feed_chat_gain("Phlogiston7".to_string(), 250, "15:51:42");

        parser.process_line(
            r#"[15:51:42] LocalPlayer: ProcessAddItem(Phlogiston7(111), -1, True)"#,
        );
        parser.process_line(
            r#"[15:51:42] LocalPlayer: ProcessAddItem(Phlogiston7(222), -1, True)"#,
        );

        // Both instances seeded from chat. Order is not guaranteed
        // (swap_remove reorders the buffer), so just check total seeded mass.
        let a = *parser.stack_sizes.get(&111).unwrap();
        let b = *parser.stack_sizes.get(&222).unwrap();
        assert_eq!(a + b, 1000);
        assert!((a == 750 && b == 250) || (a == 250 && b == 750));
    }

    #[test]
    fn test_chat_gain_different_item_no_cross_match() {
        // Chat gain for one item must not seed a different item.
        let mut parser = PlayerEventParser::new();
        parser.feed_chat_gain("Fluorite".to_string(), 3, "15:43:36");

        parser.process_line(
            r#"[15:43:36] LocalPlayer: ProcessAddItem(BlueSpinel(999), -1, True)"#,
        );
        assert_eq!(*parser.stack_sizes.get(&999).unwrap(), 1, "different item must not be seeded");
    }

    // ============================================================
    // ProvenanceColumns tests (Phase 4)
    // ============================================================

    #[test]
    fn test_provenance_columns_attributed_mining() {
        let prov = ItemProvenance::Attributed {
            source: ActivitySource::Mining {
                node_entity_id: Some(435138),
                node_name: Some("MiningNodeFromSurvey9".to_string()),
            },
            confidence: AttributionConfidence::Confident,
            survey_use_id: None,
        };
        let cols = prov.to_columns();
        assert_eq!(cols.source_kind.as_deref(), Some("mining"));
        assert_eq!(cols.confidence.as_deref(), Some("confident"));
        let details = cols.source_details.expect("mining should have details JSON");
        assert!(details.contains("435138"));
        assert!(details.contains("MiningNodeFromSurvey9"));
    }

    #[test]
    fn test_provenance_columns_attributed_unnamed_mining() {
        // Nameless motherlode nodes — node_name None must serialize cleanly
        // without breaking downstream JSON parsers.
        let prov = ItemProvenance::Attributed {
            source: ActivitySource::Mining {
                node_entity_id: Some(1176954),
                node_name: None,
            },
            confidence: AttributionConfidence::Confident,
            survey_use_id: None,
        };
        let cols = prov.to_columns();
        assert_eq!(cols.source_kind.as_deref(), Some("mining"));
        let details = cols.source_details.unwrap();
        assert!(details.contains("1176954"));
        // Should include a JSON null for node_name — not a quoted empty string
        assert!(details.contains("\"node_name\":null"));
    }

    #[test]
    fn test_provenance_columns_uncertain_preserves_candidates() {
        let prov = ItemProvenance::Uncertain {
            candidates: vec![
                ActivitySource::Mining {
                    node_entity_id: Some(1),
                    node_name: None,
                },
                ActivitySource::CorpseSearch {
                    entity_id: 2,
                    corpse_name: "Rat".to_string(),
                },
            ],
        };
        let cols = prov.to_columns();
        assert_eq!(cols.source_kind.as_deref(), Some("uncertain"));
        assert_eq!(cols.confidence, None);
        let details = cols.source_details.expect("uncertain must preserve candidates");
        // Must contain both candidate kinds so future re-classification can
        // inspect what was active at the time.
        assert!(details.contains("Mining"));
        assert!(details.contains("CorpseSearch"));
        assert!(details.contains("Rat"));
    }

    #[test]
    fn test_provenance_columns_attributed_with_survey_use_id() {
        // A3 stitching: when a feature aggregator (the survey tracker) sets
        // survey_use_id, it must appear inside source_details JSON for query
        // joins via `source_details->>'survey_use_id'`.
        let prov = ItemProvenance::Attributed {
            source: ActivitySource::Mining {
                node_entity_id: Some(435138),
                node_name: Some("MiningNodeFromSurvey9".to_string()),
            },
            confidence: AttributionConfidence::Confident,
            survey_use_id: Some(42),
        };
        let cols = prov.to_columns();
        let details = cols.source_details.expect("details JSON expected");
        // Original Mining fields preserved
        assert!(details.contains("435138"));
        assert!(details.contains("MiningNodeFromSurvey9"));
        // Stitching link present
        assert!(details.contains("\"survey_use_id\":42"));

        // Round-trip parse to confirm the JSON is valid
        let parsed: serde_json::Value = serde_json::from_str(&details).unwrap();
        assert_eq!(parsed.get("survey_use_id").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_provenance_columns_attributed_survey_use_id_no_other_details() {
        // Edge case: SurveyMapCraft has no native details JSON. With a
        // survey_use_id the function must synthesize an object containing
        // only survey_use_id rather than producing invalid JSON.
        let prov = ItemProvenance::Attributed {
            source: ActivitySource::SurveyMapCraft,
            confidence: AttributionConfidence::Confident,
            survey_use_id: Some(7),
        };
        let cols = prov.to_columns();
        let details = cols.source_details.expect("synthesized details expected");
        let parsed: serde_json::Value = serde_json::from_str(&details).unwrap();
        assert_eq!(parsed.get("survey_use_id").and_then(|v| v.as_i64()), Some(7));
    }

    #[test]
    fn test_provenance_columns_unknown_source() {
        let cols = ItemProvenance::UnknownSource.to_columns();
        assert_eq!(cols.source_kind.as_deref(), Some("unknown"));
        assert_eq!(cols.source_details, None);
        assert_eq!(cols.confidence, None);
    }

    #[test]
    fn test_provenance_columns_not_applicable() {
        let cols = ItemProvenance::NotApplicable.to_columns();
        assert_eq!(cols.source_kind.as_deref(), Some("not_applicable"));
        assert_eq!(cols.source_details, None);
        assert_eq!(cols.confidence, None);
    }

    #[test]
    fn test_provenance_columns_all_attributed_kinds() {
        // Sanity: every ActivitySource variant maps to a distinct source_kind.
        // If a new variant is added without updating activity_source_to_kind_and_details
        // the match will fail to compile — that's the intended guard.
        let cases = [
            (
                ActivitySource::SurveyMapCraft,
                "survey_map_craft",
                false, // no details
            ),
            (
                ActivitySource::SurveyMapUse {
                    survey_map_internal_name: Some("Serbule Blue Mineral Survey".to_string()),
                },
                "survey_map_use",
                true,
            ),
            (
                ActivitySource::GeneralCraft {
                    action_type: "Cook".to_string(),
                    label: "Baking Biscuits".to_string(),
                },
                "general_craft",
                true,
            ),
            (
                ActivitySource::VendorBrowsing {
                    npc_entity_id: 32857,
                    npc_name: Some("NPC_FarsightFlemmings".to_string()),
                },
                "vendor_browsing",
                true,
            ),
            (
                ActivitySource::StorageBrowsing {
                    vault_owner_entity_id: 302055,
                    vault_name: "Saddlebag".to_string(),
                },
                "storage_browsing",
                true,
            ),
        ];
        for (src, expected_kind, expects_details) in cases {
            let cols = ItemProvenance::Attributed {
                source: src,
                confidence: AttributionConfidence::Confident,
                survey_use_id: None,
            }
            .to_columns();
            assert_eq!(cols.source_kind.as_deref(), Some(expected_kind));
            assert_eq!(cols.confidence.as_deref(), Some("confident"));
            assert_eq!(
                cols.source_details.is_some(),
                expects_details,
                "{} details presence mismatch",
                expected_kind
            );
        }
    }

    #[test]
    fn test_chat_gain_storage_withdrawal_does_not_consume_chat_entry() {
        // Storage withdrawal AddItems (slot >= 0) get their qty from
        // RemoveFromStorageVault, not from chat. A chat gain meant for a
        // future real pickup must not be wrongly consumed here.
        let mut parser = PlayerEventParser::new();
        parser.feed_chat_gain("Phlogiston7".to_string(), 750, "15:47:00");

        // Storage withdrawal — slot >= 0 branch, should not consume chat gain
        parser.process_line(
            r#"[15:46:07] LocalPlayer: ProcessAddItem(Phlogiston7(111), 60, True)"#,
        );
        parser.process_line(
            r#"[15:46:07] LocalPlayer: ProcessRemoveFromStorageVault(302055, -1, 111, 1000)"#,
        );
        assert_eq!(*parser.stack_sizes.get(&111).unwrap(), 1000, "seeded from vault qty");

        // Later, a real pickup: should claim the chat gain
        parser.process_line(
            r#"[15:47:00] LocalPlayer: ProcessAddItem(Phlogiston7(222), -1, True)"#,
        );
        assert_eq!(*parser.stack_sizes.get(&222).unwrap(), 750);
    }

    // ── New game state parser tests ─────────────────────────────

    #[test]
    fn test_parse_celestial_info() {
        let mut parser = PlayerEventParser::new();
        let events = parser
            .process_line(r#"[23:32:47] LocalPlayer: ProcessSetCelestialInfo(WaxingCrescentMoon)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::MoonPhaseChanged { phase, .. } => {
                assert_eq!(phase, "WaxingCrescentMoon");
            }
            other => panic!("Expected MoonPhaseChanged, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_celestial_info_full_moon() {
        let mut parser = PlayerEventParser::new();
        let events = parser
            .process_line(r#"[12:00:00] LocalPlayer: ProcessSetCelestialInfo(FullMoon)"#);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::MoonPhaseChanged { phase, .. } => {
                assert_eq!(phase, "FullMoon");
            }
            other => panic!("Expected MoonPhaseChanged, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_guild_general_info() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[15:16:24] LocalPlayer: ProcessGuildGeneralInfo(57, "SpaceMagic", "Welcome to the guild!")"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::GuildInfoLoaded {
                guild_id,
                guild_name,
                motd,
                ..
            } => {
                assert_eq!(*guild_id, 57);
                assert_eq!(guild_name, "SpaceMagic");
                assert_eq!(motd, "Welcome to the guild!");
            }
            other => panic!("Expected GuildInfoLoaded, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_guild_general_info_empty_motd() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[15:16:24] LocalPlayer: ProcessGuildGeneralInfo(42, "TestGuild", "")"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::GuildInfoLoaded {
                guild_id,
                guild_name,
                motd,
                ..
            } => {
                assert_eq!(*guild_id, 42);
                assert_eq!(guild_name, "TestGuild");
                assert_eq!(motd, "");
            }
            other => panic!("Expected GuildInfoLoaded, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_complete_directed_goals() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[15:16:24] LocalPlayer: ProcessCompleteDirectedGoals([3200,8000,1,70,3400,5000,])"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::DirectedGoalsLoaded { goal_ids, .. } => {
                assert_eq!(goal_ids, &[3200, 8000, 1, 70, 3400, 5000]);
            }
            other => panic!("Expected DirectedGoalsLoaded, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_complete_directed_goals_empty() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[15:16:24] LocalPlayer: ProcessCompleteDirectedGoals([])"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::DirectedGoalsLoaded { goal_ids, .. } => {
                assert!(goal_ids.is_empty());
            }
            other => panic!("Expected DirectedGoalsLoaded, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_set_string_notepad() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[15:16:24] LocalPlayer: ProcessSetString(NOTEPAD, "My notes here")"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::PlayerStringUpdated { key, value, .. } => {
                assert_eq!(key, "NOTEPAD");
                assert_eq!(value, "My notes here");
            }
            other => panic!("Expected PlayerStringUpdated, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_set_string_friend_status() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[15:16:24] LocalPlayer: ProcessSetString(FRIEND_STATUS, "110 Hammer, 101 Ment")"#,
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::PlayerStringUpdated { key, value, .. } => {
                assert_eq!(key, "FRIEND_STATUS");
                assert_eq!(value, "110 Hammer, 101 Ment");
            }
            other => panic!("Expected PlayerStringUpdated, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_set_string_skips_unknown_keys() {
        let mut parser = PlayerEventParser::new();
        let events = parser.process_line(
            r#"[15:16:24] LocalPlayer: ProcessSetString(MOUNT_APPEARANCE, "@Horse1(stuff)")"#,
        );
        assert!(events.is_empty(), "MOUNT_APPEARANCE should be skipped");
    }

    #[test]
    fn test_parse_set_string_all_notepad_tabs() {
        let mut parser = PlayerEventParser::new();
        for tab in &[
            "NOTEPAD_TAB_1",
            "NOTEPAD_TAB_2",
            "NOTEPAD_TAB_3",
            "NOTEPAD_TAB_4",
            "NOTEPAD_TAB_NAMES",
        ] {
            let line = format!(
                r#"[15:16:24] LocalPlayer: ProcessSetString({}, "content")"#,
                tab
            );
            let events = parser.process_line(&line);
            assert_eq!(events.len(), 1, "Should parse {}", tab);
            match &events[0] {
                PlayerEvent::PlayerStringUpdated { key, .. } => {
                    assert_eq!(key, *tab);
                }
                other => panic!("Expected PlayerStringUpdated for {}, got {:?}", tab, other),
            }
        }
    }
}
