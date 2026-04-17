<template>
  <div class="flex flex-col gap-2">
    <input
      v-model="search"
      type="text"
      placeholder="Search by code or destination..."
      class="w-full px-2 py-1 rounded bg-surface-2 border border-border text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-blue" />

    <div class="flex flex-wrap gap-1">
      <button
        v-for="zone in zones"
        :key="zone"
        class="px-2 py-0.5 rounded text-xs transition-colors"
        :class="
          selectedZone === zone
            ? 'bg-accent-blue text-white'
            : 'bg-surface-2 text-text-secondary hover:bg-surface-3'
        "
        @click="selectedZone = selectedZone === zone ? null : zone">
        {{ zone }}
      </button>
    </div>

    <div class="flex flex-col gap-1.5 overflow-y-auto max-h-80 pr-1">
      <div
        v-for="group in groupedResults"
        :key="group.zone + group.destination"
        class="py-1 px-1 rounded hover:bg-surface-2">
        <div class="text-sm">
          <span class="text-text-dim">{{ group.zone }}</span>
          <span class="text-text-dim mx-1">&mdash;</span>
          <span class="text-text-secondary">{{ group.destination }}</span>
        </div>
        <div class="flex flex-wrap gap-1.5 mt-0.5">
          <span
            v-for="code in group.codes"
            :key="code"
            class="font-mono text-xs text-accent-gold bg-surface-2 px-1.5 py-0.5 rounded">
            {{ code }}
          </span>
        </div>
      </div>

      <div v-if="groupedResults.length === 0" class="text-xs text-text-dim italic py-2">
        No codes match your search.
      </div>
    </div>

    <div class="text-xs text-text-dim">
      {{ totalCodes }} codes across {{ groupedResults.length }} destinations. Codes last verified March 2025.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const search = ref('')
const selectedZone = ref<string | null>(null)

interface TeleportCode {
  code: string
  destination: string
  zone: string
}

interface CodeGroup {
  zone: string
  destination: string
  codes: string[]
}

const TELEPORT_CODES: TeleportCode[] = [
  // Anagoge Island
  { code: '0055', destination: 'Docks', zone: 'Anagoge Island' },
  { code: '3049', destination: 'Docks', zone: 'Anagoge Island' },
  { code: '0014', destination: 'Outside Records Facility', zone: 'Anagoge Island' },
  { code: '0049', destination: 'Outside Records Facility', zone: 'Anagoge Island' },
  { code: '0116', destination: 'Outside Records Facility', zone: 'Anagoge Island' },
  { code: '0738', destination: 'Outside Records Facility', zone: 'Anagoge Island' },
  { code: '0020', destination: 'Inside Records Facility Entrance', zone: 'Anagoge Island' },
  { code: '2020', destination: 'Inside Records Facility Entrance', zone: 'Anagoge Island' },
  { code: '2348', destination: 'Inside Records Facility Entrance', zone: 'Anagoge Island' },
  { code: '4329', destination: 'Inside Records Facility Entrance', zone: 'Anagoge Island' },
  { code: '5750', destination: 'Inside Records Facility Entrance', zone: 'Anagoge Island' },

  // Serbule Hills
  { code: '0047', destination: 'Portal to Serbule', zone: 'Serbule Hills' },
  { code: '0082', destination: 'Portal to Serbule', zone: 'Serbule Hills' },
  { code: '3667', destination: 'Portal to Serbule', zone: 'Serbule Hills' },
  { code: '0001', destination: 'Outside Spider Cave', zone: 'Serbule Hills' },
  { code: '0128', destination: 'North Center Farm', zone: 'Serbule Hills' },
  { code: '0103', destination: 'Julius Patton Farm', zone: 'Serbule Hills' },
  { code: '4932', destination: 'Julius Patton Farm', zone: 'Serbule Hills' },
  { code: '0067', destination: 'Brianna Willer Farm', zone: 'Serbule Hills' },
  { code: '0042', destination: 'Near Western Teleportation Pad', zone: 'Serbule Hills' },
  { code: '0109', destination: 'Near Ranalon Den', zone: 'Serbule Hills' },
  { code: '4200', destination: 'Near Ranalon Den', zone: 'Serbule Hills' },
  { code: '0134', destination: 'Outside Ranalon Den', zone: 'Serbule Hills' },
  { code: '3687', destination: 'Outside Ranalon Den', zone: 'Serbule Hills' },
  { code: '0007', destination: 'Southern Teleportation Pad', zone: 'Serbule Hills' },
  { code: '9696', destination: 'Southern Teleportation Pad', zone: 'Serbule Hills' },

  // Serbule
  { code: '0096', destination: 'Docks', zone: 'Serbule' },
  { code: '0834', destination: 'Docks', zone: 'Serbule' },
  { code: '9247', destination: 'Docks', zone: 'Serbule' },
  { code: '9985', destination: 'Docks', zone: 'Serbule' },
  { code: '0035', destination: 'Keep Market Roof', zone: 'Serbule' },
  { code: '5765', destination: 'Keep Market Roof', zone: 'Serbule' },
  { code: '0004', destination: 'Teleport Pad Outside Borghild', zone: 'Serbule' },
  { code: '0012', destination: 'Near Mushroom Circle', zone: 'Serbule' },
  { code: '7692', destination: 'Near Mushroom Circle', zone: 'Serbule' },
  { code: '0093', destination: "Ivyn's Farm", zone: 'Serbule' },
  { code: '0320', destination: "Ivyn's Farm", zone: 'Serbule' },
  { code: '0166', destination: 'Outside Carpal Tunnels', zone: 'Serbule' },
  { code: '0039', destination: 'Outside Crystal Cavern', zone: 'Serbule' },
  { code: '0033', destination: 'Northwest Hill', zone: 'Serbule' },
  { code: '5763', destination: 'Northwest Hill', zone: 'Serbule' },
  { code: '8367', destination: 'Northwest Hill', zone: 'Serbule' },
  { code: '0064', destination: 'Southeast Hill', zone: 'Serbule' },
  { code: '7998', destination: 'Southeast Hill', zone: 'Serbule' },
  { code: '0131', destination: 'Southwest Hill', zone: 'Serbule' },
  { code: '0137', destination: 'Inside PvP Arena Entrance', zone: 'Serbule' },
  { code: '3690', destination: 'Inside PvP Arena Entrance', zone: 'Serbule' },
  { code: '0032', destination: 'Inside Sewers Mid-Point', zone: 'Serbule' },
  { code: '5762', destination: 'Inside Sewers Mid-Point', zone: 'Serbule' },
  { code: '0078', destination: 'Inside Crypt Entrance', zone: 'Serbule' },
  { code: '0026', destination: 'Inside Brain Bug Cave Mid-Point', zone: 'Serbule' },
  { code: '0048', destination: 'Inside Brain Bug Cave Mid-Point', zone: 'Serbule' },
  { code: '1238', destination: 'Inside Brain Bug Cave Mid-Point', zone: 'Serbule' },
  { code: '5756', destination: 'Inside Brain Bug Cave Mid-Point', zone: 'Serbule' },
  { code: '0051', destination: 'Inside Borghild Entrance', zone: 'Serbule' },
  { code: '0010', destination: 'Inside Myconian Cave Entrance', zone: 'Serbule' },
  { code: '9966', destination: 'Inside Myconian Cave Entrance', zone: 'Serbule' },
  { code: '0165', destination: 'Inside Carpal Tunnels Entrance', zone: 'Serbule' },

  // Sun Vale
  { code: '0063', destination: 'Bell to Serbule', zone: 'Sun Vale' },
  { code: '0117', destination: 'Top of Giant Tree', zone: 'Sun Vale' },
  { code: '2015', destination: 'Top of Giant Tree', zone: 'Sun Vale' },
  { code: '0088', destination: 'North Shore', zone: 'Sun Vale' },
  { code: '0108', destination: 'Northeast Island Center', zone: 'Sun Vale' },
  { code: '0015', destination: 'Northeast Island Resort', zone: 'Sun Vale' },
  { code: '0050', destination: 'Northeast Island Resort', zone: 'Sun Vale' },
  { code: '2422', destination: 'Northeast Island Resort', zone: 'Sun Vale' },
  { code: '0028', destination: 'Animal Town', zone: 'Sun Vale' },
  { code: '5758', destination: 'Animal Town', zone: 'Sun Vale' },
  { code: '0155', destination: 'Outside Winter Nexus', zone: 'Sun Vale' },
  { code: '3592', destination: 'Outside Winter Nexus', zone: 'Sun Vale' },
  { code: '7667', destination: 'Outside Winter Nexus', zone: 'Sun Vale' },
  { code: '0143', destination: 'Southeast Shore', zone: 'Sun Vale' },
  { code: '0057', destination: 'Inside Sacrificial Sea Cave Entrance', zone: 'Sun Vale' },
  { code: '9124', destination: 'Inside Sacrificial Sea Cave Entrance', zone: 'Sun Vale' },
  { code: '0153', destination: 'Inside Molybdenum Mine Entrance', zone: 'Sun Vale' },
  { code: '0118', destination: 'Inside Fish Bowl Cavern Entrance', zone: 'Sun Vale' },
  { code: '2016', destination: 'Inside Fish Bowl Cavern Entrance', zone: 'Sun Vale' },
  { code: '9269', destination: 'Inside Fish Bowl Cavern Entrance', zone: 'Sun Vale' },

  // Eltibule
  { code: '0000', destination: 'Portal to Serbule', zone: 'Eltibule' },
  { code: '0212', destination: 'Portal to Serbule', zone: 'Eltibule' },
  { code: '0790', destination: 'Portal to Serbule', zone: 'Eltibule' },
  { code: '9363', destination: 'Portal to Serbule', zone: 'Eltibule' },
  { code: '0034', destination: 'Graveyard', zone: 'Eltibule' },
  { code: '5764', destination: 'Graveyard', zone: 'Eltibule' },
  { code: '0111', destination: 'Northeast Hill (Boss Event)', zone: 'Eltibule' },
  { code: '0227', destination: 'Northeast Hill (Boss Event)', zone: 'Eltibule' },
  { code: '0136', destination: 'Back Door to Goblin Dungeon', zone: 'Eltibule' },
  { code: '0167', destination: 'Inside Goblin Dungeon Entrance', zone: 'Eltibule' },
  { code: '0327', destination: 'Inside Goblin Dungeon Entrance', zone: 'Eltibule' },
  { code: '0059', destination: 'Inside Dark Chapel Entrance', zone: 'Eltibule' },
  { code: '0065', destination: 'Inside Boarded Up Basement Entrance', zone: 'Eltibule' },

  // Red Wing Casino
  { code: '9357', destination: 'Rahu Exit', zone: 'Red Wing Casino' },
  { code: '9552', destination: 'Rahu Exit', zone: 'Red Wing Casino' },
  { code: '0023', destination: 'Employee Exit to Eltibule', zone: 'Red Wing Casino' },
  { code: '5753', destination: 'Employee Exit to Eltibule', zone: 'Red Wing Casino' },
  { code: '0156', destination: 'Outside Guild Hall', zone: 'Red Wing Casino' },

  // Kur Mountains
  { code: '0011', destination: 'Portal to Eltibule', zone: 'Kur Mountains' },
  { code: '8967', destination: 'Portal to Eltibule', zone: 'Kur Mountains' },
  { code: '0077', destination: 'Portal to Ilmari', zone: 'Kur Mountains' },
  { code: '0138', destination: 'Portal to Gazluk', zone: 'Kur Mountains' },
  { code: '0022', destination: 'Jace Soral Campfire', zone: 'Kur Mountains' },
  { code: '5752', destination: 'Jace Soral Campfire', zone: 'Kur Mountains' },
  { code: '0043', destination: 'North of Town', zone: 'Kur Mountains' },
  { code: '0036', destination: 'Outside Yeti Cave', zone: 'Kur Mountains' },
  { code: '1283', destination: 'Outside Yeti Cave', zone: 'Kur Mountains' },
  { code: '5766', destination: 'Outside Yeti Cave', zone: 'Kur Mountains' },
  { code: '0008', destination: 'Western Dock', zone: 'Kur Mountains' },
  { code: '0842', destination: 'Western Dock', zone: 'Kur Mountains' },
  { code: '0085', destination: 'Inside Yeti Cave Entrance', zone: 'Kur Mountains' },
  { code: '0421', destination: 'The Village', zone: 'Kur Mountains' },

  // Ilmari
  { code: '0068', destination: 'Portal to Kur Mountains', zone: 'Ilmari' },
  { code: '0097', destination: "Arlan's Oasis", zone: 'Ilmari' },
  { code: '6173', destination: "Arlan's Oasis", zone: 'Ilmari' },
  { code: '9364', destination: "Arlan's Oasis", zone: 'Ilmari' },
  { code: '0090', destination: 'Center of Map', zone: 'Ilmari' },
  { code: '0192', destination: 'Northwest Desert', zone: 'Ilmari' },
  { code: '0030', destination: 'Southwest Desert', zone: 'Ilmari' },
  { code: '5760', destination: 'Southwest Desert', zone: 'Ilmari' },

  // Rahu
  { code: '0017', destination: 'Docks', zone: 'Rahu' },
  { code: '0113', destination: 'Docks', zone: 'Rahu' },
  { code: '0700', destination: 'Docks', zone: 'Rahu' },
  { code: '3587', destination: 'Docks', zone: 'Rahu' },
  { code: '5747', destination: 'Docks', zone: 'Rahu' },
  { code: '0058', destination: 'Portal to Ilmari', zone: 'Rahu' },
  { code: '0106', destination: 'Portal to Mysterious Locale', zone: 'Rahu' },
  { code: '0045', destination: 'Road Into Desert', zone: 'Rahu' },
  { code: '0080', destination: 'Road Into Desert', zone: 'Rahu' },
  { code: '0574', destination: 'Road Into Desert', zone: 'Rahu' },
  { code: '5348', destination: 'Road Into Desert', zone: 'Rahu' },
  { code: '0002', destination: 'Inside Rahu Sewers Entrance', zone: 'Rahu' },

  // Gazluk
  { code: '0087', destination: 'Portal to Kur Mountains', zone: 'Gazluk' },
  { code: '0465', destination: 'Portal to Kur Mountains', zone: 'Gazluk' },
  { code: '0003', destination: 'Prestonbule Teleportation Pad', zone: 'Gazluk' },
  { code: '0024', destination: 'Tower Outside Tower View Cave', zone: 'Gazluk' },
  { code: '2018', destination: 'Tower Outside Tower View Cave', zone: 'Gazluk' },
  { code: '5754', destination: 'Tower Outside Tower View Cave', zone: 'Gazluk' },
  { code: '0009', destination: '3-Crystal Patch North of New Prestonbule Cave', zone: 'Gazluk' },
  { code: '0056', destination: 'Outside Amaluk Valley Cave', zone: 'Gazluk' },
  { code: '0091', destination: 'Outside Amaluk Valley Cave (Inside)', zone: 'Gazluk' },
  { code: '0158', destination: 'Outside Windy View Cave', zone: 'Gazluk' },
  { code: '0274', destination: 'Outside Windy View Cave', zone: 'Gazluk' },
  { code: '0053', destination: 'Outside Gazluk Keep', zone: 'Gazluk' },
  { code: '0164', destination: 'Outside Gazluk Keep', zone: 'Gazluk' },
  { code: '0324', destination: 'Outside Gazluk Keep', zone: 'Gazluk' },
  { code: '0670', destination: 'Outside Gazluk Keep', zone: 'Gazluk' },
  { code: '0084', destination: 'Inside Amaluk Valley Cave Western Entrance', zone: 'Gazluk' },
  { code: '2012', destination: 'Inside Amaluk Valley Cave Eastern Entrance', zone: 'Gazluk' },
  { code: '0031', destination: 'Inside No-Name Cave Entrance', zone: 'Gazluk' },
  { code: '5761', destination: 'Inside No-Name Cave Entrance', zone: 'Gazluk' },
  { code: '0139', destination: 'Inside New Prestonbule Cave Entrance', zone: 'Gazluk' },
  { code: '0098', destination: 'Inside Gazluk Shadow Cave Entrance', zone: 'Gazluk' },
  { code: '0081', destination: 'Inside Gazluk Keep Entrance', zone: 'Gazluk' },
  { code: '0264', destination: 'Inside Gazluk Keep Entrance', zone: 'Gazluk' },
  { code: '6660', destination: 'Inside Gazluk Keep Entrance', zone: 'Gazluk' },
  { code: '0018', destination: 'Inside Gazluk Keep Lower Level After Beakhorse', zone: 'Gazluk' },
  { code: '0381', destination: 'Inside Gazluk Keep Lower Level After Beakhorse', zone: 'Gazluk' },
  { code: '5748', destination: 'Inside Gazluk Keep Lower Level After Beakhorse', zone: 'Gazluk' },

  // Povus
  { code: '0046', destination: 'Portal to Gazluk', zone: 'Povus' },
  { code: '0079', destination: 'Portal to Vidaria', zone: 'Povus' },
  { code: '6969', destination: 'Portal to Vidaria', zone: 'Povus' },
  { code: '7853', destination: 'Portal to Vidaria', zone: 'Povus' },
  { code: '0005', destination: 'Shore North of Town', zone: 'Povus' },
  { code: '9368', destination: 'Shore North of Town', zone: 'Povus' },
  { code: '0021', destination: 'Near Orc Forge', zone: 'Povus' },
  { code: '5751', destination: 'Near Orc Forge', zone: 'Povus' },
  { code: '6129', destination: 'Near Orc Forge', zone: 'Povus' },
  { code: '0044', destination: 'South-West Mountain', zone: 'Povus' },
  { code: '0040', destination: "North Swamp Near Errruka's Cave", zone: 'Povus' },
  { code: '2776', destination: "North Swamp Near Errruka's Cave", zone: 'Povus' },
  { code: '0073', destination: "Outside Errruka's Cave", zone: 'Povus' },
  { code: '0006', destination: "Inside Errruka's Cave Entrance", zone: 'Povus' },
  { code: '0027', destination: "Inside Errruka's Cave Entrance", zone: 'Povus' },
  { code: '0041', destination: "Inside Errruka's Cave Entrance", zone: 'Povus' },
  { code: '0369', destination: "Inside Errruka's Cave Entrance", zone: 'Povus' },
  { code: '0666', destination: "Inside Errruka's Cave Entrance", zone: 'Povus' },
  { code: '5757', destination: "Inside Errruka's Cave Entrance", zone: 'Povus' },
  { code: '5865', destination: "Inside Errruka's Cave Entrance", zone: 'Povus' },
  { code: '0019', destination: "Inside Errruka's Cave (Egg Run Location)", zone: 'Povus' },
  { code: '2019', destination: "Inside Errruka's Cave (Egg Run Location)", zone: 'Povus' },
  { code: '5749', destination: "Inside Errruka's Cave (Egg Run Location)", zone: 'Povus' },
  { code: '9664', destination: "Inside Errruka's Cave (Egg Run Location)", zone: 'Povus' },
  { code: '9824', destination: "Inside Errruka's Cave (Egg Run Location)", zone: 'Povus' },
  { code: '0092', destination: 'Inside Nightmare Caves (Skill Trainers)', zone: 'Povus' },
  { code: '0037', destination: 'Inside Nightmare Caves (Ship in a Bottle)', zone: 'Povus' },
  { code: '6767', destination: 'Inside Nightmare Caves (Ship in a Bottle)', zone: 'Povus' },
  { code: '6927', destination: 'Inside Nightmare Caves (Ship in a Bottle)', zone: 'Povus' },
  { code: '0025', destination: 'Inside Nightmare Caves (Yellow Room)', zone: 'Povus' },
  { code: '0060', destination: 'Inside Nightmare Caves (Yellow Room)', zone: 'Povus' },
  { code: '5755', destination: 'Inside Nightmare Caves (Yellow Room)', zone: 'Povus' },
  { code: '0066', destination: "Inside Nightmare Caves (Lady Eleanor's Chamber)", zone: 'Povus' },
  { code: '0127', destination: "Inside Nightmare Caves (Lady Eleanor's Chamber)", zone: 'Povus' },
  { code: '2025', destination: "Inside Nightmare Caves (Lady Eleanor's Chamber)", zone: 'Povus' },
  { code: '9639', destination: "Inside Nightmare Caves (Lady Eleanor's Chamber)", zone: 'Povus' },

  // Vidaria
  { code: '0083', destination: 'Portal to Gazluk', zone: 'Vidaria' },
  { code: '4552', destination: 'Portal to Gazluk', zone: 'Vidaria' },
  { code: '0029', destination: 'Portal to Povus', zone: 'Vidaria' },
  { code: '5759', destination: 'Portal to Povus', zone: 'Vidaria' },
  { code: '0013', destination: 'Road/Swamp Close to Povus Portal', zone: 'Vidaria' },
  { code: '0112', destination: 'Northeast Mushroom Circle', zone: 'Vidaria' },
  { code: '0054', destination: 'Southwest Mushroom Circle', zone: 'Vidaria' },
  { code: '8351', destination: 'Southwest Mushroom Circle', zone: 'Vidaria' },
  { code: '9973', destination: 'Southwest Mushroom Circle', zone: 'Vidaria' },
  { code: '0076', destination: 'Southwest Farms', zone: 'Vidaria' },
  { code: '8300', destination: 'Southwest Farms', zone: 'Vidaria' },
  { code: '0038', destination: 'Southwest Control Sphere', zone: 'Vidaria' },
  { code: '0016', destination: 'Western Windmill', zone: 'Vidaria' },

  // Statehelm
  { code: '3487', destination: 'Docks', zone: 'Statehelm' },
  { code: '6792', destination: 'Docks', zone: 'Statehelm' },
  { code: '9369', destination: 'Gardening Area at Tree Top', zone: 'Statehelm' },
  { code: '0052', destination: 'The Graveyard', zone: 'Statehelm' },
  { code: '2017', destination: 'The Graveyard', zone: 'Statehelm' },
  { code: '5782', destination: 'The Graveyard', zone: 'Statehelm' },
]

const zones = computed(() => {
  const zoneSet = new Set(TELEPORT_CODES.map(c => c.zone))
  return [...zoneSet]
})

const groupedResults = computed(() => {
  const q = search.value.toLowerCase().trim()

  const filtered = TELEPORT_CODES.filter(entry => {
    if (selectedZone.value && entry.zone !== selectedZone.value) return false
    if (q) {
      return (
        entry.code.includes(q) ||
        entry.destination.toLowerCase().includes(q) ||
        entry.zone.toLowerCase().includes(q)
      )
    }
    return true
  })

  const groups = new Map<string, CodeGroup>()
  for (const entry of filtered) {
    const key = `${entry.zone}|${entry.destination}`
    const existing = groups.get(key)
    if (existing) {
      existing.codes.push(entry.code)
    } else {
      groups.set(key, { zone: entry.zone, destination: entry.destination, codes: [entry.code] })
    }
  }

  return [...groups.values()]
})

const totalCodes = computed(() =>
  groupedResults.value.reduce((sum, g) => sum + g.codes.length, 0)
)
</script>
