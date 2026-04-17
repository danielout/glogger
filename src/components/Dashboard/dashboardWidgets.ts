import type { Component } from "vue";
import ContextBar from "./ContextBar.vue";
import ContextBarConfig from "./ContextBarConfig.vue";
import ZoneNpcsWidgetConfig from "./widgets/ZoneNpcsWidgetConfig.vue";
import PlayerNotes from "./PlayerNotes.vue";
import SkillTrackingWidget from "./widgets/SkillTrackingWidget.vue";
import ItemsIncomingWidget from "./widgets/ItemsIncomingWidget.vue";
import ItemsOutgoingWidget from "./widgets/ItemsOutgoingWidget.vue";
import CouncilsWidget from "./widgets/CouncilsWidget.vue";
import FavorChangesWidget from "./widgets/FavorChangesWidget.vue";
import CriticalResourcesWidget from "./widgets/CriticalResourcesWidget.vue";
import StatehelmSummaryWidget from "./widgets/StatehelmSummaryWidget.vue";
import WatchwordDetectionsWidget from "./widgets/WatchwordDetectionsWidget.vue";
import DeathTrackerWidget from "./widgets/DeathTrackerWidget.vue";
import RecipeItemsWidget from "./widgets/RecipeItemsWidget.vue";
import ZoneNpcsWidget from "./widgets/ZoneNpcsWidget.vue";
import GiftWatcherWidget from "./widgets/GiftWatcherWidget.vue";
import TeleportCodesWidget from "./widgets/TeleportCodesWidget.vue";

export type WidgetSize = "small" | "medium" | "large";

export interface DashboardWidget {
  id: string;
  name: string;
  component: Component;
  configComponent?: Component;
  defaultSize: WidgetSize;
}

/**
 * Column spans for each size class in the responsive grid.
 * Grid uses repeat(auto-fill, minmax(280px, 1fr)) so column count adapts to screen width.
 * Small = 1 column, Medium = 2 columns, Large = full row.
 */
export const SIZE_CLASSES: Record<WidgetSize, string> = {
  small: "",
  medium: "col-span-2",
  large: "col-span-4",
};

/**
 * Master widget registry. Order here defines the default card order
 * for new users (before any preferences are saved).
 */
export const DASHBOARD_WIDGETS: DashboardWidget[] = [
  {
    id: "context-bar",
    name: "Status",
    component: ContextBar,
    configComponent: ContextBarConfig,
    defaultSize: "small",
  },
  {
    id: "skill-tracking",
    name: "Live Skill Tracking",
    component: SkillTrackingWidget,
    defaultSize: "large",
  },
  {
    id: "zone-npcs",
    name: "Zone NPCs",
    component: ZoneNpcsWidget,
    configComponent: ZoneNpcsWidgetConfig,
    defaultSize: "medium",
  },
  {
    id: "items-incoming",
    name: "Items Incoming",
    component: ItemsIncomingWidget,
    defaultSize: "medium",
  },
  {
    id: "items-outgoing",
    name: "Items Outgoing",
    component: ItemsOutgoingWidget,
    defaultSize: "medium",
  },
  {
    id: "councils",
    name: "Councils",
    component: CouncilsWidget,
    defaultSize: "medium",
  },

  {
    id: "favor-changes",
    name: "Favor Changes",
    component: FavorChangesWidget,
    defaultSize: "medium",
  },
  {
    id: "player-notes",
    name: "Notes",
    component: PlayerNotes,
    defaultSize: "medium",
  },

  {
    id: "critical-resources",
    name: "Critical Resources",
    component: CriticalResourcesWidget,
    defaultSize: "small",
  },
  {
    id: "statehelm-summary",
    name: "Statehelm Gifting",
    component: StatehelmSummaryWidget,
    defaultSize: "medium",
  },
  {
    id: "watchword-detections",
    name: "Watchword Alerts",
    component: WatchwordDetectionsWidget,
    defaultSize: "medium",
  },
  {
    id: "death-tracker",
    name: "Death Tracker",
    component: DeathTrackerWidget,
    defaultSize: "medium",
  },
  {
    id: "recipe-items",
    name: "Recipe Items",
    component: RecipeItemsWidget,
    defaultSize: "medium",
  },
  {
    id: "gift-watcher",
    name: "Gift Watcher",
    component: GiftWatcherWidget,
    defaultSize: "medium",
  },
  {
    id: "teleport-codes",
    name: "Teleport Machine Codes",
    component: TeleportCodesWidget,
    defaultSize: "medium",
  },
];

/** Default card order — used when no user preferences exist */
export const DEFAULT_CARD_ORDER = DASHBOARD_WIDGETS.map((w) => w.id);
