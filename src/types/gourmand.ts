export type FoodCategory = 'Meal' | 'Snack' | 'Instant-Snack'

export interface FoodItem {
  item_id: number
  name: string
  icon_id: number | null
  food_category: FoodCategory
  food_level: number
  gourmand_req: number | null
  effect_descs: string[]
  keywords: string[]
  value: number | null
}

export interface GourmandFoodEntry {
  name: string
  count: number
}

export interface GourmandImportResult {
  foods_imported: number
}
