export const FAVOR_TIERS = [
  'SoulMates',
  'LikeFamily',
  'BestFriends',
  'CloseFriends',
  'Friends',
  'Comfortable',
  'Neutral',
  'Despised',
] as const

export type FavorTier = (typeof FAVOR_TIERS)[number]

const TIER_INDEX: Record<string, number> = {}
FAVOR_TIERS.forEach((t, i) => { TIER_INDEX[t] = i })

/** Points required to advance FROM this tier to the next one up */
const POINTS_TO_NEXT: Record<string, number> = {
  Despised: 1800,
  Neutral: 100,
  Comfortable: 200,
  Friends: 300,
  CloseFriends: 600,
  BestFriends: 800,
  LikeFamily: 1000,
}

const FAVOR_COLORS: Record<string, string> = {
  SoulMates: 'text-purple-400',
  LikeFamily: 'text-pink-400',
  BestFriends: 'text-blue-400',
  CloseFriends: 'text-cyan-400',
  Friends: 'text-green-400',
  Comfortable: 'text-yellow-400',
  Neutral: 'text-text-muted',
  Despised: 'text-red-400',
}

const FAVOR_BG_COLORS: Record<string, string> = {
  SoulMates: 'bg-purple-400/20 border-purple-400/40 text-purple-300',
  LikeFamily: 'bg-pink-400/20 border-pink-400/40 text-pink-300',
  BestFriends: 'bg-blue-400/20 border-blue-400/40 text-blue-300',
  CloseFriends: 'bg-cyan-400/20 border-cyan-400/40 text-cyan-300',
  Friends: 'bg-green-400/20 border-green-400/40 text-green-300',
  Comfortable: 'bg-yellow-400/20 border-yellow-400/40 text-yellow-300',
  Neutral: 'bg-surface-elevated border-border-default text-text-muted',
  Despised: 'bg-red-400/20 border-red-400/40 text-red-300',
}

export function tierIndex(tier: string): number {
  return TIER_INDEX[tier] ?? FAVOR_TIERS.length
}

/** Returns true if playerTier is at or above requiredTier (lower index = higher rank) */
export function isTierAtOrAbove(playerTier: string | null, requiredTier: string): boolean {
  if (!playerTier) return false
  return tierIndex(playerTier) <= tierIndex(requiredTier)
}

export function favorColor(tier: string): string {
  return FAVOR_COLORS[tier] ?? 'text-text-secondary'
}

export function favorBadgeClasses(tier: string): string {
  return FAVOR_BG_COLORS[tier] ?? 'bg-surface-elevated border-border-default text-text-muted'
}

/** Points needed to go from currentTier to the next tier up. null if already max or unknown. */
export function pointsToNextTier(currentTier: string): number | null {
  return POINTS_TO_NEXT[currentTier] ?? null
}

/** Display-friendly tier name with spaces */
export function tierDisplayName(tier: string): string {
  switch (tier) {
    case 'SoulMates': return 'Soul Mates'
    case 'LikeFamily': return 'Like Family'
    case 'BestFriends': return 'Best Friends'
    case 'CloseFriends': return 'Close Friends'
    default: return tier
  }
}
