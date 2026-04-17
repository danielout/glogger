#!/usr/bin/env node
/**
 * Correlation analyzer: match [Status] chat "ItemGained" messages to Player.log AddItem/UpdateItemCode events.
 *
 * For each [Status] "X added to inventory" (optionally "x N"):
 *   - find candidate Player.log events within +/- window seconds that could represent that gain
 *   - candidates: ProcessAddItem(X, _, True) or ProcessUpdateItemCode on an instance named X with positive delta
 *   - report: exact match, ambiguous (multiple candidates), orphan (no candidates)
 *
 * Usage: node analyze_chat_correlation.js <chat_log> <player_log> [--tz=-7] [--window=2]
 *
 * Chat timestamps are local time with TZ listed in the login line ("Timezone Offset -07:00:00").
 * Player.log timestamps are UTC. We convert chat→UTC by adding -tz hours.
 */
'use strict';

const fs = require('fs');

function parseArgs() {
  const args = process.argv.slice(2);
  const positional = [];
  const opts = { window: 2, tz: null };
  for (const a of args) {
    if (a.startsWith('--window=')) opts.window = parseInt(a.slice(9), 10);
    else if (a.startsWith('--tz=')) opts.tz = parseFloat(a.slice(5));
    else positional.push(a);
  }
  if (positional.length < 2) {
    console.error('Usage: node analyze_chat_correlation.js <chat_log> <player_log> [--tz=-7] [--window=2]');
    process.exit(1);
  }
  return { chatPath: positional[0], logPath: positional[1], ...opts };
}

// "26-04-14 08:46:07\t[Status] Great Phlogiston x1000 added to inventory." → {ts, item, qty}
function parseChatLine(line) {
  line = line.replace(/\r$/, '');
  const m = line.match(/^(\d{2})-(\d{2})-(\d{2})\s+(\d{2}):(\d{2}):(\d{2})\s+\[Status\]\s+(.+?)\s+added to inventory\.?$/);
  if (!m) return null;
  const [, yy, mm, dd, hh, mi, ss, payload] = m;
  const hms = parseInt(hh, 10) * 3600 + parseInt(mi, 10) * 60 + parseInt(ss, 10);
  // "Great Phlogiston x1000" or "Egg" or "Gazluk Map"
  const qm = payload.match(/^(.*?)\s+x(\d+)$/);
  const item = qm ? qm[1] : payload;
  const qty = qm ? parseInt(qm[2], 10) : 1;
  return { date: `${yy}-${mm}-${dd}`, chatHms: hms, rawTs: `${hh}:${mi}:${ss}`, item, qty };
}

// Extract Timezone Offset from chat log header "Timezone Offset -07:00:00."
function detectChatTz(chatLines) {
  for (const ln of chatLines.slice(0, 20)) {
    const m = ln.match(/Timezone Offset ([+\-]\d{2}):(\d{2}):\d{2}/);
    if (m) {
      const sign = m[1].startsWith('-') ? -1 : 1;
      const h = parseInt(m[1].replace('-', '').replace('+', ''), 10);
      const mins = parseInt(m[2], 10);
      return sign * (h + mins / 60);
    }
  }
  return null;
}

// Player.log events we care about:
//   [HH:MM:SS] LocalPlayer: ProcessAddItem(Name(id), slot, isNew)
//   [HH:MM:SS] LocalPlayer: ProcessUpdateItemCode(id, encoded, fromServer)
function parsePlayerLine(line) {
  line = line.replace(/\r$/, '');
  const tsMatch = line.match(/^\[(\d{2}):(\d{2}):(\d{2})\]/);
  if (!tsMatch) return null;
  const hms = parseInt(tsMatch[1], 10) * 3600 + parseInt(tsMatch[2], 10) * 60 + parseInt(tsMatch[3], 10);

  let m = line.match(/ProcessAddItem\((\w+)\((\d+)\),\s*(-?\d+),\s*(True|False)\)/);
  if (m) {
    return {
      kind: 'AddItem',
      hms,
      name: m[1],
      id: m[2],
      slot: parseInt(m[3], 10),
      isNew: m[4] === 'True',
    };
  }
  m = line.match(/ProcessUpdateItemCode\((\d+),\s*(\d+),\s*(True|False)\)/);
  if (m) {
    const encoded = parseInt(m[2], 10);
    return {
      kind: 'UpdateItemCode',
      hms,
      id: m[1],
      encoded,
      stackActual: (encoded >>> 16) + 1,
      typeId: encoded & 0xFFFF,
      fromServer: m[3] === 'True',
    };
  }
  m = line.match(/ProcessRemoveFromStorageVault\(\d+,\s*-?\d+,\s*(\d+),\s*(\d+)\)/);
  if (m) {
    return {
      kind: 'RemoveFromStorageVault',
      hms,
      id: m[1],
      quantity: parseInt(m[2], 10),
    };
  }
  return null;
}

// Build instance_id → name registry (most recent seen name), walking the log once.
// Also build per-instance stack history to compute deltas.
function buildPlayerHistory(logPath) {
  const lines = fs.readFileSync(logPath, 'utf8').split('\n');
  const events = [];
  const nameById = new Map();
  const lastStack = new Map();
  for (const line of lines) {
    if (!line.includes('LocalPlayer: Process')) continue;
    const ev = parsePlayerLine(line);
    if (!ev) continue;
    if (ev.kind === 'AddItem') {
      nameById.set(ev.id, ev.name);
      // Genuine new acquisition (slot<0, isNew=True): seed stack=1, contributes +1 gain
      // Storage withdrawal (slot>=0, isNew=True): gain qty unknown until RemoveFromStorageVault
      // Session load (isNew=False): no gain
      if (ev.isNew && ev.slot < 0) {
        lastStack.set(ev.id, 1);
        events.push({ ...ev, gainQty: 1, resolvedName: ev.name });
      } else if (ev.isNew && ev.slot >= 0) {
        // Defer: emitted below as placeholder with gainQty=0; actual gain comes from UpdateItemCode delta later
        events.push({ ...ev, gainQty: 0, resolvedName: ev.name, isStorageWithdrawalAdd: true });
      } else {
        events.push({ ...ev, gainQty: 0, resolvedName: ev.name });
      }
    } else if (ev.kind === 'UpdateItemCode') {
      const name = nameById.get(ev.id) || null;
      const prior = lastStack.get(ev.id);
      let delta = 0;
      let isBaselineEstablish = false;
      if (prior !== undefined) {
        delta = ev.stackActual - prior;
      } else {
        // First sighting of this instance (session-load baseline). Don't count as gain.
        isBaselineEstablish = true;
      }
      lastStack.set(ev.id, ev.stackActual);
      events.push({
        ...ev,
        gainQty: delta > 0 ? delta : 0,
        deltaSigned: delta,
        resolvedName: name,
        isBaselineEstablish,
      });
    } else if (ev.kind === 'RemoveFromStorageVault') {
      const name = nameById.get(ev.id) || null;
      // Seed stack from vault qty (mimics our parser fix)
      if (!lastStack.has(ev.id)) {
        lastStack.set(ev.id, ev.quantity);
      }
      // The vault qty IS the gain to the player when withdrawing
      events.push({ ...ev, gainQty: ev.quantity, resolvedName: name });
    }
  }
  return events;
}

// Simple display-name → internal-name candidate matching.
// Chat uses display names ("Great Phlogiston", "Large Prism", "Osslar Skull").
// Player.log uses internal names ("Phlogiston7", "Prism2", "OsslarSkull").
// We normalize both by stripping spaces+numerics and comparing case-insensitively.
function normalizeName(s) {
  return s.replace(/\s+/g, '').replace(/\d+$/, '').toLowerCase();
}

function namesMightMatch(displayName, internalName) {
  if (!internalName) return false;
  const a = normalizeName(displayName);
  const b = normalizeName(internalName);
  if (a === b) return true;
  // loose containment for graded items like "ArmorPatchKit5" vs "Master Armor Patch Kit"
  if (a.includes(b) || b.includes(a)) return true;
  return false;
}

function main() {
  const { chatPath, logPath, window: windowSecs, tz: tzOverride } = parseArgs();

  const chatLines = fs.readFileSync(chatPath, 'utf8').split('\n');
  const tz = tzOverride !== null ? tzOverride : detectChatTz(chatLines);
  if (tz === null) {
    console.error('Could not detect timezone from chat log; pass --tz=-7 (or whatever)');
    process.exit(1);
  }
  const tzShiftSecs = Math.round(-tz * 3600); // chat local → UTC: add -tz hours

  const chatEvents = [];
  for (const line of chatLines) {
    const c = parseChatLine(line);
    if (c) chatEvents.push(c);
  }

  const playerEvents = buildPlayerHistory(logPath);
  const gainEvents = playerEvents.filter((e) => e.gainQty > 0);

  // Summary counters
  let chatTotal = 0;
  let chatTotalQty = 0;
  let exactMatch = 0; // exactly one candidate with matching qty
  let qtyMatchAmbiguous = 0; // multiple candidates, one with matching qty
  let singleMismatch = 0; // one candidate but qty differs
  let multipleCandidates = 0; // ambiguous, no single qty match
  let orphan = 0; // no candidates
  const orphanSamples = [];
  const mismatchSamples = [];

  for (const ce of chatEvents) {
    chatTotal++;
    chatTotalQty += ce.qty;
    const chatUtcSecs = ce.chatHms + tzShiftSecs;
    // Wrap negative / overflow: normalize into 0..86400
    const chatUtcMod = ((chatUtcSecs % 86400) + 86400) % 86400;

    const candidates = gainEvents.filter((pe) => {
      if (!namesMightMatch(ce.item, pe.resolvedName)) return false;
      // distance modulo 24h
      let d = Math.abs(pe.hms - chatUtcMod);
      if (d > 43200) d = 86400 - d;
      return d <= windowSecs;
    });

    if (candidates.length === 0) {
      orphan++;
      if (orphanSamples.length < 10) {
        orphanSamples.push({ chat: ce, chatUtc: chatUtcMod });
      }
      continue;
    }

    const qtyMatches = candidates.filter((c) => c.gainQty === ce.qty);
    const totalGainSum = candidates.reduce((a, c) => a + c.gainQty, 0);
    const sumMatches = totalGainSum === ce.qty;

    if (qtyMatches.length === 1 && candidates.length === 1) {
      exactMatch++;
    } else if (qtyMatches.length >= 1) {
      qtyMatchAmbiguous++;
    } else if (sumMatches) {
      // Multi-event coalesce: chat qty = sum of candidate gains
      qtyMatchAmbiguous++;
    } else if (candidates.length === 1) {
      singleMismatch++;
      if (mismatchSamples.length < 10) {
        mismatchSamples.push({ chat: ce, chatUtc: chatUtcMod, cand: candidates[0], totalGainSum });
      }
    } else {
      multipleCandidates++;
      if (mismatchSamples.length < 10) {
        mismatchSamples.push({ chat: ce, chatUtc: chatUtcMod, cands: candidates, totalGainSum });
      }
    }
  }

  console.log('=== Chat [Status] → Player.log correlation analysis ===');
  console.log(`Chat log:    ${chatPath}`);
  console.log(`Player log:  ${logPath}`);
  console.log(`Timezone:    ${tz}h (chat → UTC)`);
  console.log(`Window:      ±${windowSecs}s`);
  console.log('');
  console.log(`Total chat ItemGained messages: ${chatTotal}`);
  console.log(`Total chat quantity:            ${chatTotalQty}`);
  console.log(`Total player.log gain events:   ${gainEvents.length}`);
  console.log('');
  console.log(`Exact match (1 cand, qty ok):   ${exactMatch}`);
  console.log(`Ambiguous w/ qty match:         ${qtyMatchAmbiguous}`);
  console.log(`Single cand, qty MISMATCH:      ${singleMismatch}`);
  console.log(`Multiple cands, no qty match:   ${multipleCandidates}`);
  console.log(`Orphan (no cand):               ${orphan}`);
  console.log('');
  const matchable = exactMatch + qtyMatchAmbiguous;
  console.log(`Quantity match rate: ${matchable}/${chatTotal} (${((matchable / chatTotal) * 100).toFixed(1)}%)`);
  console.log('');

  if (orphanSamples.length > 0) {
    console.log('--- Orphan samples (chat msg with no player.log candidate) ---');
    for (const o of orphanSamples) {
      const h = Math.floor(o.chatUtc / 3600);
      const m = Math.floor((o.chatUtc % 3600) / 60);
      const s = o.chatUtc % 60;
      console.log(`  [${o.chat.date} ${o.chat.rawTs} → UTC ~${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}] ${o.chat.item} x${o.chat.qty}`);
    }
    console.log('');
  }

  if (mismatchSamples.length > 0) {
    console.log('--- Quantity mismatch samples ---');
    for (const m of mismatchSamples) {
      const h = Math.floor(m.chatUtc / 3600);
      const mm = Math.floor((m.chatUtc % 3600) / 60);
      const s = m.chatUtc % 60;
      const ts = `~${String(h).padStart(2, '0')}:${String(mm).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
      if (m.cand) {
        console.log(`  [${ts}] chat: ${m.chat.item} x${m.chat.qty}  ||  log: ${m.cand.kind} ${m.cand.resolvedName}(${m.cand.id}) gainQty=${m.cand.gainQty} (sum=${m.totalGainSum})`);
      } else {
        const candSummary = m.cands.map(c => `${c.kind}(${c.id}:${c.gainQty})`).join(', ');
        console.log(`  [${ts}] chat: ${m.chat.item} x${m.chat.qty}  ||  ${m.cands.length} cands: ${candSummary} (sum=${m.totalGainSum})`);
      }
    }
    console.log('');
  }
}

main();
