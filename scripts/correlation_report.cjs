#!/usr/bin/env node
/**
 * Bi-directional correlation report between chat [Status] "added to inventory" messages
 * and Player.log inventory gain events (ProcessAddItem, ProcessUpdateItemCode, ProcessRemoveFromStorageVault).
 *
 * For each pair of logs, produces:
 *   1. Counts: chat ItemGained entries vs player.log gain events
 *   2. Correlation summary: exact matches, coalesced-sum matches, ambiguous, orphans
 *   3. List of unmatched CHAT entries (chat says gained but no player.log candidate)
 *   4. List of unmatched PLAYER.LOG entries (player.log says gained but no chat candidate)
 *
 * Usage:
 *   node correlation_report.cjs <chat_log> <player_log> [--tz=-7] [--window=2] [--no-lists]
 *   node correlation_report.cjs --all          # runs all paired datasets under test_data & docs/samples
 */
'use strict';

const fs = require('fs');
const path = require('path');

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = { window: 2, tz: null, noLists: false, all: false };
  const positional = [];
  for (const a of args) {
    if (a === '--all') opts.all = true;
    else if (a === '--no-lists') opts.noLists = true;
    else if (a.startsWith('--window=')) opts.window = parseInt(a.slice(9), 10);
    else if (a.startsWith('--tz=')) opts.tz = parseFloat(a.slice(5));
    else positional.push(a);
  }
  return { ...opts, positional };
}

// ========== Parsing ==========

function parseChatLine(line) {
  line = line.replace(/\r$/, '');
  const m = line.match(/^(\d{2})-(\d{2})-(\d{2})\s+(\d{2}):(\d{2}):(\d{2})\s+\[Status\]\s+(.+?)\s+added to inventory\.?$/);
  if (!m) return null;
  const [, yy, mm, dd, hh, mi, ss, payload] = m;
  const hms = parseInt(hh, 10) * 3600 + parseInt(mi, 10) * 60 + parseInt(ss, 10);
  const qm = payload.match(/^(.*?)\s+x(\d+)$/);
  const item = qm ? qm[1] : payload;
  const qty = qm ? parseInt(qm[2], 10) : 1;
  return { date: `${yy}-${mm}-${dd}`, hms, rawTs: `${hh}:${mi}:${ss}`, item, qty, raw: line };
}

function detectChatTz(chatLines) {
  for (const ln of chatLines.slice(0, 20)) {
    const m = ln.match(/Timezone Offset ([+\-]?\d{2}):(\d{2}):\d{2}/);
    if (m) {
      const sign = m[1].startsWith('-') ? -1 : 1;
      const h = parseInt(m[1].replace('-', '').replace('+', ''), 10);
      const mins = parseInt(m[2], 10);
      return sign * (h + mins / 60);
    }
  }
  return null;
}

function parsePlayerLine(line) {
  line = line.replace(/\r$/, '');
  const tsMatch = line.match(/^\[(\d{2}):(\d{2}):(\d{2})\]/);
  if (!tsMatch) return null;
  const hms = parseInt(tsMatch[1], 10) * 3600 + parseInt(tsMatch[2], 10) * 60 + parseInt(tsMatch[3], 10);
  const rawTs = `${tsMatch[1]}:${tsMatch[2]}:${tsMatch[3]}`;

  let m = line.match(/ProcessAddItem\((\w+)\((\d+)\),\s*(-?\d+),\s*(True|False)\)/);
  if (m) {
    return {
      kind: 'AddItem',
      hms, rawTs,
      name: m[1],
      id: m[2],
      slot: parseInt(m[3], 10),
      isNew: m[4] === 'True',
      raw: line,
    };
  }
  m = line.match(/ProcessUpdateItemCode\((\d+),\s*(\d+),\s*(True|False)\)/);
  if (m) {
    const encoded = parseInt(m[2], 10);
    return {
      kind: 'UpdateItemCode',
      hms, rawTs,
      id: m[1],
      encoded,
      stackActual: (encoded >>> 16) + 1,
      typeId: encoded & 0xFFFF,
      fromServer: m[3] === 'True',
      raw: line,
    };
  }
  m = line.match(/ProcessRemoveFromStorageVault\(\d+,\s*-?\d+,\s*(\d+),\s*(\d+)\)/);
  if (m) {
    return {
      kind: 'RemoveFromStorageVault',
      hms, rawTs,
      id: m[1],
      quantity: parseInt(m[2], 10),
      raw: line,
    };
  }
  return null;
}

// Walk the player log, computing per-instance stack history and producing
// "gain events" — occasions where items entered the player's inventory.
function buildPlayerGainEvents(logPath) {
  const content = fs.readFileSync(logPath, 'utf8');
  const lines = content.split('\n');
  const events = [];
  const nameById = new Map();
  const lastStack = new Map();

  for (const line of lines) {
    if (!line.includes('LocalPlayer: Process')) continue;
    const ev = parsePlayerLine(line);
    if (!ev) continue;

    if (ev.kind === 'AddItem') {
      nameById.set(ev.id, ev.name);
      if (ev.isNew && ev.slot < 0) {
        // Genuine new stack. Seed=1, count as +1 gain (we don't yet know the real qty).
        lastStack.set(ev.id, 1);
        events.push({ ...ev, gainQty: 1, resolvedName: ev.name, gainKind: 'AddItem-new' });
      } else if (ev.isNew && ev.slot >= 0) {
        // Storage withdrawal — defer; pair with RemoveFromStorageVault below.
        events.push({ ...ev, gainQty: 0, resolvedName: ev.name, gainKind: 'AddItem-withdrawal' });
      } else {
        // Session load; no gain.
        // Don't emit a gain event at all.
      }
    } else if (ev.kind === 'UpdateItemCode') {
      const name = nameById.get(ev.id) || null;
      const prior = lastStack.get(ev.id);
      let delta = 0;
      let gainKind;
      if (prior !== undefined) {
        delta = ev.stackActual - prior;
        gainKind = delta > 0 ? 'UpdateItemCode-gain' : (delta < 0 ? 'UpdateItemCode-loss' : 'UpdateItemCode-same');
      } else {
        gainKind = 'UpdateItemCode-baseline';
      }
      lastStack.set(ev.id, ev.stackActual);
      // Only emit events when there's a positive gain or a baseline establishment
      if (delta > 0 || gainKind === 'UpdateItemCode-baseline') {
        events.push({
          ...ev,
          gainQty: delta > 0 ? delta : 0,
          deltaSigned: delta,
          resolvedName: name,
          gainKind,
          priorStack: prior,
        });
      }
    } else if (ev.kind === 'RemoveFromStorageVault') {
      const name = nameById.get(ev.id) || null;
      // Vault qty = the gain. Seed stack if not already.
      if (!lastStack.has(ev.id)) {
        lastStack.set(ev.id, ev.quantity);
      }
      events.push({ ...ev, gainQty: ev.quantity, resolvedName: name, gainKind: 'RemoveFromStorageVault' });
    }
  }

  return { events, nameById };
}

// ========== Name matching ==========
// Chat uses display names (e.g., "Great Phlogiston", "Master Armor Patch Kit").
// Player.log uses internal names (e.g., "Phlogiston7", "ArmorPatchKit5").
// Build an authoritative map from items.json.

function loadNameMap(itemsJsonPath) {
  try {
    const items = JSON.parse(fs.readFileSync(itemsJsonPath, 'utf8'));
    const m = new Map(); // displayNameLower → Set<internalName>
    for (const v of Object.values(items)) {
      if (!v || typeof v !== 'object') continue;
      const internal = v.InternalName;
      const display = v.Name;
      if (!internal || !display) continue;
      const key = display.toLowerCase();
      if (!m.has(key)) m.set(key, new Set());
      m.get(key).add(internal);
    }
    return m;
  } catch (e) {
    console.warn(`[warn] could not load items.json from ${itemsJsonPath}: ${e.message}`);
    return new Map();
  }
}

const ITEMS_JSON = 'e:/glogger/docs/samples/CDN-full-examples/items.json';
const displayToInternal = loadNameMap(ITEMS_JSON);

function namesMightMatch(displayName, internalName) {
  if (!internalName) return false;
  const candidates = displayToInternal.get(displayName.toLowerCase());
  if (candidates && candidates.has(internalName)) return true;
  // Fallback heuristic for items not in the static CDN snapshot
  const norm = (s) => s.replace(/['\s\-]/g, '').replace(/\d+$/, '').toLowerCase();
  const a = norm(displayName);
  const b = norm(internalName);
  if (a === b) return true;
  if (a.includes(b) && b.length >= 4) return true;
  if (b.includes(a) && a.length >= 4) return true;
  return false;
}

// ========== Correlation ==========

function correlateChatWithPlayer(chatEvents, playerGainEvents, tzShiftSecs, windowSecs) {
  // Each chat entry is assigned 0..N player.log events; each player.log event
  // can be assigned to at most one chat entry (best-match greedy).
  const chatMatches = chatEvents.map((ce) => {
    const chatUtcSecs = ce.hms + tzShiftSecs;
    const chatUtcMod = ((chatUtcSecs % 86400) + 86400) % 86400;
    return { chat: ce, chatUtcMod, candidates: [], assigned: [] };
  });

  // Build candidate list per chat entry
  for (const cm of chatMatches) {
    for (let i = 0; i < playerGainEvents.length; i++) {
      const pe = playerGainEvents[i];
      if (!namesMightMatch(cm.chat.item, pe.resolvedName)) continue;
      let d = Math.abs(pe.hms - cm.chatUtcMod);
      if (d > 43200) d = 86400 - d;
      if (d <= windowSecs) {
        cm.candidates.push({ idx: i, pe, dist: d });
      }
    }
  }

  // Greedy exclusive assignment, preferring: (1) exact qty match, (2) closest timestamp
  const claimed = new Set();
  // Pass 1: exact qty + time match
  for (const cm of chatMatches) {
    const exact = cm.candidates
      .filter((c) => !claimed.has(c.idx) && c.pe.gainQty === cm.chat.qty)
      .sort((a, b) => a.dist - b.dist);
    if (exact.length > 0) {
      cm.assigned.push(exact[0]);
      claimed.add(exact[0].idx);
    }
  }
  // Pass 2: try to find a subset whose gainQty sums to chat qty
  for (const cm of chatMatches) {
    if (cm.assigned.length > 0) continue;
    const avail = cm.candidates.filter((c) => !claimed.has(c.idx));
    if (avail.length === 0) continue;
    const subset = findSubsetSummingTo(avail, cm.chat.qty);
    if (subset) {
      for (const c of subset) {
        cm.assigned.push(c);
        claimed.add(c.idx);
      }
    }
  }
  // Pass 3: assign single nearest candidate even if qty mismatch (marks as mismatch)
  for (const cm of chatMatches) {
    if (cm.assigned.length > 0) continue;
    const avail = cm.candidates.filter((c) => !claimed.has(c.idx));
    if (avail.length > 0) {
      avail.sort((a, b) => a.dist - b.dist);
      cm.assigned.push(avail[0]);
      claimed.add(avail[0].idx);
    }
  }

  return { chatMatches, claimed };
}

// Small subset-sum helper (up to ~6 events in a time window — fine for exhaustive search)
function findSubsetSummingTo(items, target) {
  if (items.length > 12) items = items.slice(0, 12); // safety cap
  const n = items.length;
  for (let mask = 1; mask < (1 << n); mask++) {
    let sum = 0;
    const subset = [];
    for (let i = 0; i < n; i++) {
      if (mask & (1 << i)) {
        sum += items[i].pe.gainQty;
        subset.push(items[i]);
      }
    }
    if (sum === target) return subset;
  }
  return null;
}

// ========== Report ==========

function runReport(chatPath, logPath, opts) {
  const chatLines = fs.readFileSync(chatPath, 'utf8').split('\n');
  let tz = opts.tz;
  if (tz === null) tz = detectChatTz(chatLines);
  if (tz === null) {
    console.error(`[${path.basename(chatPath)}] Could not detect timezone; pass --tz=N`);
    return null;
  }
  const tzShiftSecs = Math.round(-tz * 3600);

  const chatEvents = [];
  for (const line of chatLines) {
    const c = parseChatLine(line);
    if (c) chatEvents.push(c);
  }

  const { events: playerGainEvents } = buildPlayerGainEvents(logPath);
  const windowSecs = opts.window;

  const { chatMatches, claimed } = correlateChatWithPlayer(chatEvents, playerGainEvents, tzShiftSecs, windowSecs);

  // Classify chat matches
  let exactMatches = 0;
  let coalesced = 0;
  let mismatches = 0;
  let unmatched = 0;
  const unmatchedChat = [];
  const mismatchDetails = [];

  for (const cm of chatMatches) {
    if (cm.assigned.length === 0) {
      unmatched++;
      unmatchedChat.push(cm.chat);
      continue;
    }
    const sum = cm.assigned.reduce((a, x) => a + x.pe.gainQty, 0);
    if (cm.assigned.length === 1 && cm.assigned[0].pe.gainQty === cm.chat.qty) {
      exactMatches++;
    } else if (sum === cm.chat.qty) {
      coalesced++;
    } else {
      mismatches++;
      mismatchDetails.push({ chat: cm.chat, sum, assignments: cm.assigned });
    }
  }

  // Unmatched player events
  const unmatchedPlayer = [];
  for (let i = 0; i < playerGainEvents.length; i++) {
    if (!claimed.has(i)) unmatchedPlayer.push(playerGainEvents[i]);
  }

  const header = `\n╔══ Report: ${path.basename(path.dirname(chatPath))}/${path.basename(chatPath)} × ${path.basename(logPath)}`;
  console.log(header);
  console.log(`║  chat path:   ${chatPath}`);
  console.log(`║  player path: ${logPath}`);
  console.log(`║  timezone:    ${tz}h, window: ±${windowSecs}s`);
  console.log('╠══ Counts');
  console.log(`║  chat [Status] "added to inventory" entries:  ${chatEvents.length}`);
  console.log(`║  chat total quantity:                          ${chatEvents.reduce((a, c) => a + c.qty, 0)}`);
  console.log(`║  player.log inventory gain events:            ${playerGainEvents.length}`);
  console.log(`║    breakdown:`);
  const breakdown = {};
  for (const pe of playerGainEvents) {
    breakdown[pe.gainKind] = (breakdown[pe.gainKind] || 0) + 1;
  }
  for (const [k, v] of Object.entries(breakdown)) console.log(`║      ${k}: ${v}`);
  console.log('╠══ Chat → Player correlation');
  console.log(`║  Exact match (1 event, qty ok):     ${exactMatches}`);
  console.log(`║  Coalesced match (N events sum ok): ${coalesced}`);
  console.log(`║  Single cand, qty MISMATCH:         ${mismatches}`);
  console.log(`║  No candidate found:                ${unmatched}`);
  const matchableTotal = exactMatches + coalesced;
  console.log(`║  Match rate (exact+coalesced):      ${matchableTotal}/${chatEvents.length} (${((matchableTotal / Math.max(1, chatEvents.length)) * 100).toFixed(1)}%)`);
  console.log('╠══ Player → Chat correlation');
  console.log(`║  Player gain events unmatched:      ${unmatchedPlayer.length}/${playerGainEvents.length} (${((unmatchedPlayer.length / Math.max(1, playerGainEvents.length)) * 100).toFixed(1)}%)`);
  console.log('╚══════════════════════════════════════════════════════════');

  if (!opts.noLists) {
    if (unmatchedChat.length > 0) {
      console.log('\n--- Unmatched CHAT entries (no player.log candidate) ---');
      const bucketed = bucketByItem(unmatchedChat.map((c) => ({ item: c.item, qty: c.qty, ts: c.rawTs })));
      for (const [item, rows] of Object.entries(bucketed)) {
        const totalQty = rows.reduce((a, r) => a + r.qty, 0);
        console.log(`  [${rows.length}× total qty ${totalQty}] ${item}`);
        for (const r of rows.slice(0, 5)) console.log(`    · ${r.ts} x${r.qty}`);
        if (rows.length > 5) console.log(`    · ...and ${rows.length - 5} more`);
      }
    }
    if (mismatchDetails.length > 0) {
      console.log('\n--- Quantity MISMATCH (assigned best candidate with wrong qty) ---');
      for (const m of mismatchDetails.slice(0, 20)) {
        const a = m.assignments.map((x) => `${x.pe.gainKind}(${x.pe.resolvedName || '?'}:${x.pe.id || ''}, qty=${x.pe.gainQty})`).join(' + ');
        console.log(`  [${m.chat.rawTs}] chat: ${m.chat.item} x${m.chat.qty}   ||   log: ${a} (sum=${m.sum})`);
      }
      if (mismatchDetails.length > 20) console.log(`  ...and ${mismatchDetails.length - 20} more`);
    }
    if (unmatchedPlayer.length > 0) {
      console.log('\n--- Unmatched PLAYER.LOG gain events (no chat candidate) ---');
      const bucketed = {};
      for (const pe of unmatchedPlayer) {
        const key = `${pe.gainKind} :: ${pe.resolvedName || '?'}`;
        if (!bucketed[key]) bucketed[key] = [];
        bucketed[key].push(pe);
      }
      for (const [key, rows] of Object.entries(bucketed)) {
        const totalQty = rows.reduce((a, r) => a + r.gainQty, 0);
        console.log(`  [${rows.length}× total gain ${totalQty}] ${key}`);
        for (const r of rows.slice(0, 5)) console.log(`    · ${r.rawTs} id=${r.id || '-'} gainQty=${r.gainQty}`);
        if (rows.length > 5) console.log(`    · ...and ${rows.length - 5} more`);
      }
    }
  }

  return {
    dataset: path.basename(path.dirname(chatPath)),
    chatTotal: chatEvents.length,
    playerTotal: playerGainEvents.length,
    exactMatches,
    coalesced,
    mismatches,
    unmatched,
    unmatchedPlayer: unmatchedPlayer.length,
  };
}

function bucketByItem(items) {
  const buckets = {};
  for (const it of items) {
    if (!buckets[it.item]) buckets[it.item] = [];
    buckets[it.item].push(it);
  }
  return buckets;
}

// ========== Dataset discovery ==========

function discoverPairedDatasets() {
  const pairs = [
    {
      chat: 'e:/glogger/test_data/inventory/InventoryChatLog.txt',
      log: 'e:/glogger/test_data/inventory/InventoryActions.log',
    },
    {
      chat: 'e:/glogger/test_data/surveyLogs/100x-serbcrystal-withring/ChatLog.txt',
      log: 'e:/glogger/test_data/surveyLogs/100x-serbcrystal-withring/Player.log',
    },
    {
      chat: 'e:/glogger/test_data/surveyLogs/100x-eltmetal-ringandpick/ChatLog.txt',
      log: 'e:/glogger/test_data/surveyLogs/100x-eltmetal-ringandpick/Player-prev.log',
    },
    {
      chat: 'e:/glogger/test_data/surveyLogs/50x-povusmarvelous-ringandpick/ChatLog.txt',
      log: 'e:/glogger/test_data/surveyLogs/50x-povusmarvelous-ringandpick/Player.log',
    },
    {
      chat: 'e:/glogger/docs/samples/chat+logCombos/Gazluk-Motherlodes-and-Nodes-Chat.log',
      log: 'e:/glogger/docs/samples/chat+logCombos/Gazluk-Motherlodes-and-Nodes.log',
    },
  ];
  return pairs.filter((p) => fs.existsSync(p.chat) && fs.existsSync(p.log));
}

// ========== Entrypoint ==========

function main() {
  const opts = parseArgs();

  if (opts.all) {
    const pairs = discoverPairedDatasets();
    const results = [];
    for (const p of pairs) {
      const r = runReport(p.chat, p.log, opts);
      if (r) results.push(r);
    }
    console.log('\n==== Aggregate summary ====');
    console.log(`${'Dataset'.padEnd(42)} ${'chat'.padStart(5)} ${'log'.padStart(5)} ${'exact'.padStart(6)} ${'coalsc'.padStart(6)} ${'misma'.padStart(5)} ${'unmch'.padStart(5)} ${'plrUnmch'.padStart(9)}`);
    for (const r of results) {
      console.log(
        `${r.dataset.padEnd(42)} ${String(r.chatTotal).padStart(5)} ${String(r.playerTotal).padStart(5)} ` +
        `${String(r.exactMatches).padStart(6)} ${String(r.coalesced).padStart(6)} ` +
        `${String(r.mismatches).padStart(5)} ${String(r.unmatched).padStart(5)} ${String(r.unmatchedPlayer).padStart(9)}`
      );
    }
  } else if (opts.positional.length >= 2) {
    runReport(opts.positional[0], opts.positional[1], opts);
  } else {
    console.error('Usage:');
    console.error('  node correlation_report.cjs <chat_log> <player_log> [--tz=-7] [--window=2] [--no-lists]');
    console.error('  node correlation_report.cjs --all                        # run all paired datasets');
    process.exit(1);
  }
}

main();
