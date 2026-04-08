export interface StallEvent {
  id: number
  event_timestamp: string
  log_timestamp: string
  log_title: string
  action: 'bought' | 'added' | 'removed' | 'configured' | 'visible' | 'collected' | 'unknown'
  player: string
  owner: string | null
  item: string | null
  quantity: number
  price_unit: number | null
  price_total: number | null
  raw_message: string
  created_at: string
  ignored: boolean
}

export interface StallStats {
  total_sales: number
  total_revenue: number
  unique_buyers: number
  unique_items: number
}
