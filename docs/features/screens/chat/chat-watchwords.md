# Chat — Watchwords

## Overview

Rule-based chat monitoring with configurable alerts. Create rules that match messages by text content, item links, or sender name, and get notified via toast popups, sounds, or message highlighting.

## How It Works

### Rule Management
- **Create/Edit/Delete** watch rules with descriptive names
- **Enable/Disable** individual rules without deleting them
- **Channel scope** — restrict a rule to specific channels or apply it to all

### Match Conditions

Each rule has one or more conditions:
- **ContainsText** — case-insensitive search in message body and item link names
- **ContainsItemLink** — match against item link names only
- **FromSender** — exact sender name match (case-insensitive)

### Match Logic
- **AND mode** — all conditions must match
- **OR mode** — any condition triggers the rule

### Notification Options
- **Toast popup** — notification banner when a match is found
- **Sound alert** — audio notification
- **Message highlighting** — visual highlight on matching messages in chat views

### Rule Preview
Select a rule to see all historical messages that match it, loaded via `get_watch_rule_messages()`.

## Storage

Watch rules persist in `settingsStore` (app settings file), not the database.
