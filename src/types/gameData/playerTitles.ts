export interface PlayerTitleInfo {
  id: number
  title: string | null
  tooltip: string | null
  keywords: string[]
  account_wide: boolean | null
  soul_wide: boolean | null
  raw_json: Record<string, unknown>
}
