<template>
  <div class="character-list">
    <div class="character-list-header">
      <span class="character-list-title">Characters</span>
      <span class="character-list-count">{{ characters.length }} total</span>
    </div>
    <div class="character-grid">
      <CharacterCard
        v-for="char in characters"
        :key="char.id"
        :character="char"
        @click="$emit('select', char)"
      />
    </div>
    <div v-if="characters.length === 0" class="character-empty">
      No characters loaded
    </div>
  </div>
</template>

<script setup>
import CharacterCard from './CharacterCard.vue'

defineProps({
  characters: { type: Array, default: () => [] }
})

defineEmits(['select'])
</script>

<style scoped>
.character-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.character-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.character-list-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-secondary);
}
.character-list-count {
  font-size: 11px;
  color: var(--text-muted);
}
.character-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}
.character-empty {
  padding: 40px;
  text-align: center;
  color: var(--text-muted);
  font-size: 14px;
}
</style>
