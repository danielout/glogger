export interface ParsedSearchQuery {
  text: string
  textWords: string[]
  sender?: string
  channel?: string
}

/**
 * Parses structured operators from a search query string.
 *
 * Supported operators:
 *   from:PlayerName    — filter by sender
 *   from:"Player Name" — quoted form for names with spaces
 *   in:Trade           — filter by channel
 *   in:"Global"        — quoted form
 *
 * Operators are case-insensitive. Remaining text after stripping
 * operators becomes the free-text search term for FTS.
 */
export function parseSearchQuery(raw: string): ParsedSearchQuery {
  let text = raw
  let sender: string | undefined
  let channel: string | undefined

  // Match operator:value or operator:"quoted value"
  // Case-insensitive operator names
  const operatorPattern = /\b(from|in):(?:"([^"]*)"|([\S]+))/gi

  let match: RegExpExecArray | null
  while ((match = operatorPattern.exec(text)) !== null) {
    const op = match[1].toLowerCase()
    const value = match[2] ?? match[3] // quoted or unquoted

    if (op === 'from') {
      sender = value
    } else if (op === 'in') {
      channel = value
    }
  }

  // Remove all matched operators from the text
  text = text.replace(operatorPattern, '').trim().replace(/\s+/g, ' ')

  const textWords = text ? text.split(/\s+/).filter(Boolean) : []

  return {
    text,
    textWords,
    ...(sender && { sender }),
    ...(channel && { channel }),
  }
}
