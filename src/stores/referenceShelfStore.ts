import { defineStore } from "pinia";
import { ref } from "vue";
import { useSettingsStore } from "./settingsStore";
import type { EntityType } from "../composables/useEntityNavigation";

export interface PinnedEntity {
  type: EntityType;
  reference: string;
  label: string;
}

const PREFS_KEY = "referenceShelf";

export const useReferenceShelfStore = defineStore("referenceShelf", () => {
  const pins = ref<PinnedEntity[]>([]);
  const collapsed = ref(false);
  let initialized = false;

  function load() {
    if (initialized) return;
    initialized = true;
    const settingsStore = useSettingsStore();
    const saved = settingsStore.settings.viewPreferences[PREFS_KEY] as
      | { pins?: PinnedEntity[]; collapsed?: boolean }
      | undefined;
    if (saved?.pins) pins.value = saved.pins;
    if (saved?.collapsed !== undefined) collapsed.value = saved.collapsed;
  }

  function persist() {
    const settingsStore = useSettingsStore();
    settingsStore.updateSettings({
      viewPreferences: {
        ...settingsStore.settings.viewPreferences,
        [PREFS_KEY]: {
          pins: pins.value,
          collapsed: collapsed.value,
        },
      },
    });
  }

  function isPinned(type: EntityType, reference: string): boolean {
    return pins.value.some((p) => p.type === type && p.reference === reference);
  }

  function pin(entity: PinnedEntity) {
    if (isPinned(entity.type, entity.reference)) return;
    pins.value.push({ ...entity });
    persist();
  }

  function unpin(type: EntityType, reference: string) {
    const idx = pins.value.findIndex(
      (p) => p.type === type && p.reference === reference,
    );
    if (idx !== -1) {
      pins.value.splice(idx, 1);
      persist();
    }
  }

  function togglePin(entity: PinnedEntity) {
    if (isPinned(entity.type, entity.reference)) {
      unpin(entity.type, entity.reference);
    } else {
      pin(entity);
    }
  }

  function toggleCollapsed() {
    collapsed.value = !collapsed.value;
    persist();
  }

  return { pins, collapsed, load, isPinned, pin, unpin, togglePin, toggleCollapsed };
});
