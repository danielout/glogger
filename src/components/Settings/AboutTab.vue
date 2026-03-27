<template>
  <div class="max-w-lg">
    <!-- Hero -->
    <div class="settings-section flex flex-col items-center text-center py-10">
      <img src="/glogger.png" alt="glogger" class="size-24 rounded-2xl mb-4 ring-2 ring-border-default" />
      <h2 class="m-0 text-accent-gold text-2xl tracking-wide">{{ appName }}</h2>
      <p class="m-0 mt-1 text-text-muted text-sm font-mono">v{{ appVersion }}</p>
      <p class="m-0 mt-4 text-text-secondary text-sm leading-relaxed max-w-sm">
        Built to help alievate some spreadsheet tracking and to help me learn Rust.
      </p>
      <p class="m-0 mt-1.5 text-text-muted text-xs">
        Some portions copyright 2026 Elder Game, LLC.
      </p>

      <button
        @click="openLink('https://buymeacoffee.com/danielout')"
        class="btn btn-primary mt-5">
        Buy Me a Coffee
      </button>
    </div>

    <!-- Special Thanks -->
    <div class="settings-section">
      <h3>Special Thanks</h3>
      <div class="flex flex-col gap-3">
        <div v-for="person in thanks" :key="person.name" class="flex gap-3">
          <span class="text-accent-gold text-sm shrink-0 mt-px">&#9830;</span>
          <div>
            <span class="text-text-primary text-sm font-medium">{{ person.name }}</span>
            <span v-if="person.location" class="text-text-muted text-xs ml-1.5">({{ person.location }})</span>
            <p class="m-0 mt-0.5 text-text-secondary text-xs leading-relaxed">{{ person.reason }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Built With -->
    <div class="settings-section">
      <h3>Built With</h3>
      <div class="flex flex-wrap gap-2">
        <span
          v-for="tech in techStack"
          :key="tech"
          class="px-3 py-1 rounded-full bg-surface-dark border border-border-default text-text-secondary text-xs">
          {{ tech }}
        </span>
      </div>
    </div>

  
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getName, getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

const appName = ref("glogger");
const appVersion = ref("");

const thanks = [
  {
    name: "Citan & Co",
    location: null,
    reason: "For not only making PG, but providing so much info for 3rd party app developers. Without dev support for things like this, Glogger would never be possible.",
  },
  {
    name: "Reyetta",
    location: "Dreva @ Stall CI-6",
    reason: "Endless suggestions and tips, particularly around crafting and gourmand features.",
  },
  {
    name: "Wogan",
    location: "Dreva @ Stall CP-2",
    reason: "The bovine surveyor for their help with gems and gem related activities.",
  },
  {
    name: "Cakedydidooda",
    location: "Dreva @ Stall CS-1",
    reason: "For economy related feature feedback.",
  },
];

const techStack = ["Tauri v2", "Vue 3", "Rust", "SQLite", "Tailwind CSS"];

onMounted(async () => {
  appName.value = await getName();
  appVersion.value = await getVersion();
});

function openLink(url: string) {
  openUrl(url);
}
</script>
