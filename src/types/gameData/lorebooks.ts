export interface LorebookEntry {
  id: number
  title: string | null
  internal_name: string | null
  category: string | null
  text: string | null
  location_hint: string | null
  keywords: string[]
  visibility: string | null
  is_client_local: boolean | null
}

export interface LorebookCategoryInfo {
  key: string
  title: string | null
  sub_title: string | null
  sort_title: string | null
}
