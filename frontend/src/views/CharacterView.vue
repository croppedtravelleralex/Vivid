<template>
  <div class="character-view">
    <div class="cv-list-panel">
      <CharacterList :characters="characterList" @select="selectCharacter" />
    </div>
    <div v-if="selectedChar" class="cv-detail-panel">
      <div class="cv-detail-scroll">
        <div class="cv-detail-top">
          <button class="btn btn-sm" @click="selectedChar = null">&larr; Back</button>
          <h2 class="cv-char-name">{{ selectedChar.name || selectedChar.id }}</h2>
          <span class="cv-char-status" :class="'status-' + (selectedChar.status || 'alive')">
            {{ selectedChar.status || 'alive' }}
          </span>
        </div>
        <CharacterState :character="selectedChar" />
        <CharacterMemory :memories="memories" />
      </div>
    </div>
    <div v-else class="cv-empty">
      <p>Select a character to view details</p>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, inject, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import CharacterList from '../components/character/CharacterList.vue'
import CharacterState from '../components/character/CharacterState.vue'
import CharacterMemory from '../components/character/CharacterMemory.vue'
import { getCharacter, getCharacterMemory } from '../api/character.js'

const route = useRoute()
const router = useRouter()
const characters = inject('characters', { value: [] })

const selectedChar = ref(null)
const memories = ref([])
const loading = ref(false)

const characterList = computed(() => {
  const chars = typeof characters === 'object' && characters.value !== undefined
    ? characters.value
    : characters
  return Array.isArray(chars) ? chars : []
})

async function selectCharacter(char) {
  loading.value = true
  selectedChar.value = char
  memories.value = []
  router.replace('/character/' + char.id).catch(() => {})
  try {
    const [detail, memData] = await Promise.all([
      getCharacter(char.id).catch(() => null),
      getCharacterMemory(char.id).catch(() => [])
    ])
    if (detail) selectedChar.value = { ...char, ...detail }
    if (Array.isArray(memData)) memories.value = memData
    else if (memData && Array.isArray(memData.memories || memData.data)) {
      memories.value = memData.memories || memData.data
    }
  } catch (_) {} finally {
    loading.value = false
  }
}

// Handle direct route /character/:id
watch(() => route.params.id, async (id) => {
  if (!id) return
  const chars = characterList.value
  let found = chars.find(c => String(c.id) === String(id))
  if (found) {
    await selectCharacter(found)
  } else {
    // Try loading from API
    loading.value = true
    try {
      const detail = await getCharacter(id)
      if (detail) {
        selectedChar.value = detail
        const memData = await getCharacterMemory(id).catch(() => [])
        if (Array.isArray(memData)) memories.value = memData
        else if (memData?.memories) memories.value = memData.memories
      }
    } catch (_) {} finally {
      loading.value = false
    }
  }
}, { immediate: true })

onMounted(() => {
  if (route.params.id) {
    // wait for watch to trigger
  }
})
</script>

<style scoped>
.character-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  height: 100%;
}
.cv-list-panel {
  overflow-y: auto;
}
.cv-detail-panel {
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.cv-detail-scroll {
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 4px;
}
.cv-detail-top {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 4px;
}
.cv-char-name {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}
.cv-char-status {
  font-size: 10px;
  padding: 2px 10px;
  border-radius: 8px;
  font-weight: 500;
  text-transform: uppercase;
}
.cv-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 14px;
}
</style>
