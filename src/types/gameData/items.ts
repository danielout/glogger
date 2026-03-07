export interface ItemInfo {
  id: number
  name: string
  description: string | null
  icon_id: number | null
  value: number | null
  max_stack_size: number | null
  keywords: string[]
  effect_descs: string[]
}
