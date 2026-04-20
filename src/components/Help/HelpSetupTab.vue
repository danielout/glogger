<template>
  <div class="flex flex-col gap-8">
    <div>
      <p class="text-sm text-text-muted mt-1">
        Getting started with Glogger and Project: Gorgon.
      </p>
    </div>

    <!-- Initial Setup -->
    <section>
      <h2 class="text-sm font-bold text-text-secondary uppercase tracking-wider m-0 mb-3">Initial Setup</h2>
      <div class="flex flex-col gap-3">
        <SetupStep number="1" title="Set your Game Data path">
          Go to <button class="text-accent-gold bg-transparent border-none cursor-pointer underline p-0 font-inherit text-inherit" @click="$emit('navigate', 'settings')">Settings &rarr; General</button>
          and set the path to your Project: Gorgon installation folder.
          This is the folder containing your <code class="text-accent-gold bg-surface-dark px-1.5 py-0.5 rounded-sm text-xs">Player.log</code> file. This should have been auto-detected during the setup, but in case that failed you can manually set it here.
        </SetupStep>
        <SetupStep number="2" title="Enable Chat Logging in-game">
          In Project Gorgon, open Settings &rarr; GUI and enable
          <span class="text-text-primary font-medium">"Log All Chat"</span>. It is all the way at the bottom of this settings menu.
          This lets Glogger read chat messages for item linking, watchwords, and other chat-driven features.
        </SetupStep>
        <SetupStep number="3" title="Import your Character JSON (optional)">
          Go to Settings -> VIP in Project Gorgon to export. This populates your skills, recipes, and quest data.
        </SetupStep>
        <SetupStep number="4" title="Import your VIP Inventory JSON (optional, requires VIP)">
          Go to Settings -> VIP in Project Gorgon to export. This gives accurate item quantities and mod data.
        </SetupStep>
      </div>
    </section>

    <!-- Keyboard Shortcuts -->
    <section>
      <h2 class="text-sm font-bold text-text-secondary uppercase tracking-wider m-0 mb-3">Keyboard Shortcuts</h2>
      <div class="flex flex-col gap-1.5">
        <ShortcutRow keys="Ctrl+F" description="Quick search overlay" />
        <ShortcutRow keys="Ctrl+K" description="Quick search (alias)" />
        <ShortcutRow keys="Ctrl+D" description="Toggle Data Browser" />
        <ShortcutRow keys="Ctrl+[" description="Previous sub-tab" />
        <ShortcutRow keys="Ctrl+]" description="Next sub-tab" />
      </div>
    </section>

    <!-- In-Game Commands -->
    <section>
      <h2 class="text-sm font-bold text-text-secondary uppercase tracking-wider m-0 mb-3">Useful In-Game Commands</h2>
      <div class="flex flex-col gap-1.5">
        <ShortcutRow keys="/behaviorreport" description="Display (and let glogger save) some character stats" />
        <ShortcutRow keys="/age" description="Display (and let glogger save) some playtime data" />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import SetupStep from './SetupStep.vue'
import ShortcutRow from './ShortcutRow.vue'

defineEmits<{
  navigate: [view: string]
}>()
</script>
