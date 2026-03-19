/// Player log event parser — foundational module for parsing all ProcessXxx events
/// from Player.log into structured, identity-resolved events.
///
/// This is a core system that features subscribe to. It maintains:
/// - Instance registry: maps instance IDs to item names/type IDs
/// - Stack tracking: current stack size per instance
/// - Interaction context: current NPC interaction for contextualizing events
/// - Pending delete buffer: 1-line lookahead to disambiguate storage/vendor/consumed

use std::collections::HashMap;
use crate::parsers::parse_timestamp;

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
        slot: i32,
        item_name: String,
        instance_id: u64,
    },
    StorageWithdrawal {
        timestamp: String,
        npc_id: u32,
        instance_id: u64,
        quantity: u32,
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
}

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum DeleteContext {
    StorageTransfer,
    VendorSale,
    Consumed,
    Unknown,
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
struct InteractionContext {
    entity_id: u32,
    npc_name: String,
    interaction_type: u32,
}

// ============================================================
// Parser
// ============================================================

pub struct PlayerEventParser {
    instance_registry: HashMap<u64, InstanceInfo>,
    stack_sizes: HashMap<u64, u32>,
    current_interaction: Option<InteractionContext>,
    pending_deletes: Vec<PendingDelete>,
}

impl PlayerEventParser {
    pub fn new() -> Self {
        Self {
            instance_registry: HashMap::new(),
            stack_sizes: HashMap::new(),
            current_interaction: None,
            pending_deletes: Vec::new(),
        }
    }

    /// Feed one log line; returns zero or more events.
    pub fn process_line(&mut self, line: &str) -> Vec<PlayerEvent> {
        let mut events = Vec::new();

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
        self.instance_registry.insert(instance_id, InstanceInfo {
            item_name: item_name.clone(),
            item_type_id: None,
        });

        Some(PlayerEvent::ItemAdded {
            timestamp: ts,
            item_name,
            instance_id,
            slot_index,
            is_new,
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

        let new_stack_size = encoded_value >> 16;
        let item_type_id = (encoded_value & 0xFFFF) as u16;

        let old_stack_size = self.stack_sizes.get(&instance_id).copied().unwrap_or(0);
        let delta = new_stack_size as i32 - old_stack_size as i32;

        // Update tracking state
        self.stack_sizes.insert(instance_id, new_stack_size);

        // Update type ID in registry if we have an entry
        if let Some(info) = self.instance_registry.get_mut(&instance_id) {
            info.item_type_id = Some(item_type_id);
        }

        let item_name = self.instance_registry.get(&instance_id)
            .map(|info| info.item_name.clone());

        Some(PlayerEvent::ItemStackChanged {
            timestamp: ts,
            instance_id,
            item_name,
            item_type_id,
            old_stack_size,
            new_stack_size,
            delta,
            from_server,
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

        let item_name = self.instance_registry.get(&instance_id)
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

        let new_stack_size = encoded_value >> 16;
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
        let args_start = line.find("ProcessAddToStorageVault(")? + "ProcessAddToStorageVault(".len();
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

        Some(PlayerEvent::StorageDeposit {
            timestamp: ts,
            npc_id,
            slot,
            item_name,
            instance_id,
        })
    }

    /// ProcessRemoveFromStorageVault(npcId, -1, instanceId, quantity)
    fn parse_remove_from_storage(&self, line: &str) -> Option<PlayerEvent> {
        let ts = parse_timestamp(line)?;
        let args_start = line.find("ProcessRemoveFromStorageVault(")? + "ProcessRemoveFromStorageVault(".len();
        let args_end = line[args_start..].find(')')? + args_start;
        let args = &line[args_start..args_end];

        let parts: Vec<&str> = args.split(',').collect();
        let npc_id: u32 = parts.get(0)?.trim().parse().ok()?;
        // parts[1] = -1 (skip)
        let instance_id: u64 = parts.get(2)?.trim().parse().ok()?;
        let quantity: u32 = parts.get(3)?.trim().parse().ok()?;

        Some(PlayerEvent::StorageWithdrawal {
            timestamp: ts,
            npc_id,
            instance_id,
            quantity,
        })
    }

    /// ProcessDoDelayLoop(duration, actionType, "label", entityId, abortCondition)
    fn parse_delay_loop(&self, line: &str) -> Option<PlayerEvent> {
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
        let after_content = &after_title[after_title.find(&format!("\"{}\"", content))? + content.len() + 2..];
        let book_type = extract_quoted_string(after_content, 0)?;

        Some(PlayerEvent::BookOpened {
            timestamp: ts,
            title,
            content,
            book_type,
        })
    }
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

    Some(SkillSnapshot { skill_type, raw, bonus, xp, tnl, max })
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
            r#"[16:17:48] LocalPlayer: ProcessAddItem(Malachite(115244857), -1, True)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemAdded { item_name, instance_id, slot_index, is_new, .. } => {
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
            r#"[16:00:00] LocalPlayer: ProcessAddItem(MetalSlab2(136937342), 5, False)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemAdded { item_name, is_new, slot_index, .. } => {
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
            r#"[16:00:00] LocalPlayer: ProcessAddItem(MetalSlab3(136937342), 5, False)"#
        );
        // Set initial stack size
        parser.stack_sizes.insert(136937342, 20);

        let events = parser.process_line(
            r#"[16:17:48] LocalPlayer: ProcessUpdateItemCode(136937342, 1642723, True)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemStackChanged {
                instance_id, item_name, item_type_id,
                old_stack_size, new_stack_size, delta, from_server, ..
            } => {
                assert_eq!(*instance_id, 136937342);
                assert_eq!(item_name.as_deref(), Some("MetalSlab3"));
                assert_eq!(*item_type_id, 4323);  // 1642723 & 0xFFFF
                assert_eq!(*old_stack_size, 20);
                assert_eq!(*new_stack_size, 25);   // 1642723 >> 16
                assert_eq!(*delta, 5);
                assert!(*from_server);
            }
            _ => panic!("Expected ItemStackChanged"),
        }
    }

    #[test]
    fn test_encoded_value_decoding() {
        // From the docs: 1642723 >> 16 = 25, 1642723 & 0xFFFF = 4323
        let encoded: u32 = 1642723;
        assert_eq!(encoded >> 16, 25);
        assert_eq!(encoded & 0xFFFF, 4323);
    }

    #[test]
    fn test_delete_then_storage_transfer() {
        let mut parser = PlayerEventParser::new();

        // Register item first
        parser.process_line(
            r#"[13:27:07] LocalPlayer: ProcessAddItem(MapleWood(136093889), 10, False)"#
        );

        // Delete item — should be buffered
        let events = parser.process_line(
            r#"[13:27:07] LocalPlayer: ProcessDeleteItem(136093889)"#
        );
        assert!(events.is_empty(), "Delete should be buffered");

        // Storage deposit — should resolve the pending delete
        let events = parser.process_line(
            r#"[13:27:07] LocalPlayer: ProcessAddToStorageVault(14804, -1, 40, MapleWood(136093889))"#
        );

        // Should have: ItemDeleted(StorageTransfer) + StorageDeposit
        assert_eq!(events.len(), 2);
        match &events[0] {
            PlayerEvent::ItemDeleted { instance_id, context, item_name, .. } => {
                assert_eq!(*instance_id, 136093889);
                assert_eq!(*context, DeleteContext::StorageTransfer);
                assert_eq!(item_name.as_deref(), Some("MapleWood"));
            }
            _ => panic!("Expected ItemDeleted with StorageTransfer"),
        }
        match &events[1] {
            PlayerEvent::StorageDeposit { item_name, instance_id, .. } => {
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

        let events = parser.process_line(
            r#"[16:32:25] LocalPlayer: ProcessDeleteItem(115259296)"#
        );
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
            PlayerEvent::VendorSold { price, item_name, is_buyback, .. } => {
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
            r#"[16:33:03] LocalPlayer: ProcessAddItem(SomeItem(114961794), 1, False)"#
        );

        // Delete
        let events = parser.process_line(
            r#"[16:33:03] LocalPlayer: ProcessDeleteItem(114961794)"#
        );
        assert!(events.is_empty());

        // Unrelated line flushes pending
        let events = parser.process_line(
            r#"[16:33:04] entity_159956: OnAttackHitMe(Fiery Bite). Evaded = False"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemDeleted { instance_id, context, item_name, .. } => {
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
            r#"[13:26:55] LocalPlayer: ProcessStartInteraction(14804, 7, 1200, True, "NPC_Qatik")"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::InteractionStarted { entity_id, interaction_type, npc_name, .. } => {
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
            r#"[16:33:03] LocalPlayer: ProcessDeltaFavor(9618, "NPC_Kalaba", 2.8476, True)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::FavorChanged { npc_id, npc_name, delta, is_gift, .. } => {
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
            PlayerEvent::ScreenText { category, message, .. } => {
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
            r#"[13:28:48] LocalPlayer: ProcessRemoveFromStorageVault(14804, -1, 132702881, 11)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::StorageWithdrawal { npc_id, instance_id, quantity, .. } => {
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
            r#"[16:32:27] LocalPlayer: ProcessVendorUpdateItem(115249145, 200909, 7)"#
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::VendorStackUpdated { instance_id, item_type_id, new_stack_size, price, .. } => {
                assert_eq!(*instance_id, 115249145);
                // 200909 >> 16 = 3, 200909 & 0xFFFF = 4301
                assert_eq!(*new_stack_size, 3);
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
            PlayerEvent::BookOpened { title, content, book_type, .. } => {
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
            r#"[00:15:46] entity_192620: OnAttackHitMe(Spider Bite). Evaded = False"#
        );
        assert!(events.is_empty());
    }

    #[test]
    fn test_flush_all_pending() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[16:00:00] LocalPlayer: ProcessAddItem(TestItem(12345), 1, False)"#
        );
        parser.process_line(
            r#"[16:33:03] LocalPlayer: ProcessDeleteItem(12345)"#
        );
        // No more lines — flush manually
        let events = parser.flush_all_pending();
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlayerEvent::ItemDeleted { instance_id, context, .. } => {
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
            r#"[16:32:27] LocalPlayer: ProcessAddItem(SomeStackable(115271948), 1, False)"#
        );

        // Delete
        let events = parser.process_line(
            r#"[16:32:27] LocalPlayer: ProcessDeleteItem(115271948)"#
        );
        assert!(events.is_empty());

        // VendorUpdateItem (selling stackable vendor already has)
        let events = parser.process_line(
            r#"[16:32:27] LocalPlayer: ProcessVendorUpdateItem(115249145, 200909, 7)"#
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
            PlayerEvent::DelayLoopStarted { duration, action_type, label, entity_id, abort_condition, .. } => {
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
            PlayerEvent::DelayLoopStarted { duration, action_type, label, .. } => {
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
            PlayerEvent::DelayLoopStarted { action_type, label, entity_id, .. } => {
                assert_eq!(action_type, "Eat");
                assert_eq!(label, "Using Gobbledygook");
                assert_eq!(*entity_id, 6223);
            }
            _ => panic!("Expected DelayLoopStarted"),
        }
    }

    #[test]
    fn test_instance_registry_populated_from_add_item() {
        let mut parser = PlayerEventParser::new();
        parser.process_line(
            r#"[16:00:00] LocalPlayer: ProcessAddItem(MetalSlab2(136937342), 5, False)"#
        );

        // UpdateItemCode should resolve item name from registry
        let events = parser.process_line(
            r#"[16:00:01] LocalPlayer: ProcessUpdateItemCode(136937342, 65536, True)"#
        );
        match &events[0] {
            PlayerEvent::ItemStackChanged { item_name, .. } => {
                assert_eq!(item_name.as_deref(), Some("MetalSlab2"));
            }
            _ => panic!("Expected ItemStackChanged"),
        }
    }
}
