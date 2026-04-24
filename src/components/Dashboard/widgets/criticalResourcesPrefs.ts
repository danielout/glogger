/** Critical Resources widget preferences — persisted via useViewPrefs. */

export interface CriticalResourcesPrefs {
  [key: string]: unknown
  /** Item names to track in the widget. */
  trackedItems: string[]
}

export const CRITICAL_RESOURCES_DEFAULTS: CriticalResourcesPrefs = {
  trackedItems: [
    'Diamond',
    'Amethyst',
    'Aquamarine',
    'Eternal Greens',
    'Salt',
    'Fire Dust',
  ],
}
