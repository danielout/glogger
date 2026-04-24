<template>
  <div class="flex flex-col gap-3 h-full">
    <TabBar v-model="activeTab" :tabs="tabs" />
    <div class="flex-1 min-h-0">
      <SurveyTrackerView v-if="activeTab === 'session'" />
      <AnalyticsTab v-else-if="activeTab === 'analytics'" />
    </div>
  </div>
</template>

<script setup lang="ts">
// Two tabs: Session (unified active + historical) and Analytics.
// The Session tab merges what was previously separate "Session" and
// "Session History" tabs into one PaneLayout view — the left panel
// lists all sessions (active pinned at top), the center shows detail
// for the selected session, and the right shows economics/notes.
import { computed } from 'vue'
import TabBar from '../Shared/TabBar.vue'
import SurveyTrackerView from '../Surveying/SurveyTrackerView.vue'
import AnalyticsTab from '../Surveying/AnalyticsTab.vue'
import { useViewPrefs } from '../../composables/useViewPrefs'

const tabs = [
  { id: 'session', label: 'Session' },
  { id: 'analytics', label: 'Analytics' },
]

const { prefs, update } = useViewPrefs('economics.survey', { activeTab: 'session' })
const activeTab = computed({
  get: () => prefs.value.activeTab,
  set: (val: string) => update({ activeTab: val }),
})
</script>
