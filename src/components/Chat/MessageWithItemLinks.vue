<template>
  <span>
    <template v-for="(part, index) in messageParts" :key="index">
      <template v-if="part.type === 'text'">{{ part.content }}</template>
      <ItemInline
        v-else-if="part.type === 'item' && part.link"
        :reference="part.link.item_name"
      />
    </template>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ChatItemLink } from '../../types/database'
import ItemInline from '../Shared/Item/ItemInline.vue'

const props = defineProps<{
  message: string
  itemLinks: ChatItemLink[]
}>()

interface MessagePart {
  type: 'text' | 'item'
  content?: string
  link?: ChatItemLink
}

const messageParts = computed((): MessagePart[] => {
  if (!props.itemLinks || props.itemLinks.length === 0) {
    return [{ type: 'text', content: props.message }]
  }

  const parts: MessagePart[] = []
  let lastIndex = 0

  // Find all item link positions in the message
  const linkPositions = props.itemLinks.map(link => {
    const pos = props.message.indexOf(link.raw_text, lastIndex)
    return { link, pos, endPos: pos + link.raw_text.length }
  }).filter(item => item.pos !== -1)

  // Sort by position
  linkPositions.sort((a, b) => a.pos - b.pos)

  // Build parts array
  for (const { link, pos, endPos } of linkPositions) {
    // Add text before the link
    if (pos > lastIndex) {
      parts.push({
        type: 'text',
        content: props.message.substring(lastIndex, pos)
      })
    }

    // Add the item link
    parts.push({
      type: 'item',
      link
    })

    lastIndex = endPos
  }

  // Add remaining text after last link
  if (lastIndex < props.message.length) {
    parts.push({
      type: 'text',
      content: props.message.substring(lastIndex)
    })
  }

  return parts
})
</script>
