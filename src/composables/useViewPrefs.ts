import { ref, type Ref } from "vue";
import { useSettingsStore } from "../stores/settingsStore";

/** Shared ref cache so multiple components using the same screenKey share reactive state. */
const _sharedRefs = new Map<string, { prefs: Ref<any>; update: (partial: any) => void }>();

export function useViewPrefs<T extends Record<string, unknown>>(
  screenKey: string,
  defaults: T
): { prefs: Ref<T>; update: (partial: Partial<T>) => void } {
  // Return existing shared instance if one already exists for this key
  if (_sharedRefs.has(screenKey)) {
    return _sharedRefs.get(screenKey) as { prefs: Ref<T>; update: (partial: Partial<T>) => void };
  }

  const settingsStore = useSettingsStore();
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Read existing prefs or use defaults
  const stored = settingsStore.settings.viewPreferences?.[screenKey] as Partial<T> | undefined;
  const initial = { ...defaults, ...(stored ?? {}) } as T;
  const prefs = ref(initial) as Ref<T>;

  function update(partial: Partial<T>) {
    prefs.value = { ...prefs.value, ...partial };

    // Debounced save to avoid write storms
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      const allPrefs = { ...(settingsStore.settings.viewPreferences ?? {}) };
      allPrefs[screenKey] = { ...prefs.value };
      settingsStore.updateSettings({ viewPreferences: allPrefs });
    }, 500);
  }

  _sharedRefs.set(screenKey, { prefs, update });

  return { prefs, update };
}
