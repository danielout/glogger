# Interactive Maps

Zone maps with NPC locations, landmarks, and player context overlays.

## The Problem

Players frequently need to find NPCs, landmarks, storage vaults, and other points of interest across zones. Currently this means alt-tabbing to the wiki. An in-app map with glogger's existing game state data overlaid would be significantly more useful than a static wiki map.

## What Makes This Different from a Wiki Map

Glogger already tracks:
- **Current zone** (from `game_state_area`)
- **NPC favor levels** per character (color-code NPCs by favor)
- **Vendor gold/restock status** (highlight vendors with gold available)
- **Storage vault locations** (from CDN data)
- **Bind locations** (from trip routing — `ProcessSetBinding` events)
- **Teleport destinations** (from trip routing zone graph)

An interactive map that overlays this live data is far more valuable than a static reference.

## Data Sources

### CDN Data (already loaded)
- Area definitions with zone metadata
- NPC data with location references
- Need to verify: do CDN area or NPC records include x,y,z coordinates? If not, coordinates may need to come from wiki data or community-sourced files.

### Map Images
- PG wiki has zone map images
- Need to source, cache, and serve these (licensing/attribution to verify)
- Coordinate transform needed: PG 3D world coords → 2D pixel positions on map images

### Live Game State (already tracked)
- Current area from `ProcessSetArea`
- NPC interactions from `InteractionStarted` events
- Vendor state from `VendorGoldChanged`
- Favor levels from game state

## Feature Areas

### 1. Static Zone Maps
Base map images with fixed POI markers (NPCs, storage, landmarks). Zoom, pan, search. This is the minimum viable version.

### 2. Player Context Overlay
Layer glogger's live data onto the map: current zone highlight, NPC favor coloring, vendor gold status, storage capacity indicators.

### 3. Route Visualization
Show trip planner routes on the map. The trip routing system already calculates multi-zone paths with teleport awareness — visualizing these on connected zone maps would be a natural extension.

### 4. Custom Markers
Let users place their own markers (farming spots, gathering routes, personal notes). Persisted per character.

## Technical Approach

- **Frontend-heavy:** Most work is Leaflet.js (or similar) integration in Vue
- **Backend:** Minimal — serve coordinate data from CDN, possibly a Tauri command for map image paths
- **Coordinate mapping** is the main technical risk — PG uses 3D coordinates and map images may not have consistent projections

## Open Questions

- Do CDN records include NPC/landmark position coordinates? Need to check the actual JSON fields.
- What's the licensing situation for PG wiki map images?
- Is there a community-maintained coordinate dataset we could use?
- Should this be a full screen or an overlay/panel (like the data browser)?
