<template>
  <div class="character-state">
    <div class="card">
      <div class="card-header">Vital Signs</div>
      <div class="gauge-grid">
        <div v-for="g in gauges" :key="g.key" class="gauge-item">
          <div class="gauge-label">{{ g.label }}</div>
          <div class="gauge-track">
            <div
              class="gauge-fill"
              :style="{
                width: g.percent + '%',
                background: g.color
              }"
            ></div>
          </div>
          <div class="gauge-value">{{ g.value }}/100</div>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="card-header">Traits & Status</div>
      <div class="traits-list">
        <div v-for="(val, key) in traits" :key="key" class="trait-row">
          <span class="trait-key">{{ key }}</span>
          <span class="trait-val">{{ val }}</span>
        </div>
        <div v-if="Object.keys(traits).length === 0" class="trait-empty">No traits data</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  character: { type: Object, default: () => ({}) }
})

function clamp(v, max = 100) {
  return Math.max(0, Math.min(max, v ?? 0))
}

const gauges = computed(() => [
  { key: 'hp', label: 'HP', value: clamp(props.character.hp ?? 100), percent: clamp(props.character.hp ?? 100), color: 'var(--hp-bar)' },
  { key: 'hunger', label: 'Hunger', value: clamp(props.character.hunger ?? props.character.fullness ?? 100), percent: clamp(props.character.hunger ?? props.character.fullness ?? 100), color: 'var(--hunger-bar)' },
  { key: 'warmth', label: 'Warmth', value: clamp(props.character.warmth ?? 100), percent: clamp(props.character.warmth ?? 100), color: 'var(--warmth-bar)' },
  { key: 'energy', label: 'Energy', value: clamp(props.character.energy ?? 100), percent: clamp(props.character.energy ?? 100), color: 'var(--info)' },
  { key: 'mental', label: 'Mental', value: clamp(props.character.mental ?? props.character.sanity ?? 100), percent: clamp(props.character.mental ?? props.character.sanity ?? 100), color: '#a78bfa' }
])

const traits = computed(() => {
  const t = props.character.traits || props.character.attributes || {}
  const exclude = ['id', 'name', 'hp', 'hunger', 'warmth', 'fullness', 'energy', 'mental', 'sanity', 'status', 'location', 'faction', 'inventory', 'memory', 'relationships']
  const result = {}
  Object.entries(props.character).forEach(([k, v]) => {
    if (!exclude.includes(k) && typeof v !== 'object' && v !== null && v !== undefined) {
      result[k] = String(v)
    }
  })
  Object.entries(t).forEach(([k, v]) => {
    result[k] = String(v)
  })
  return result
})
</script>

<style scoped>
.character-state {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.gauge-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.gauge-item {
  display: flex;
  align-items: center;
  gap: 8px;
}
.gauge-label {
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 56px;
}
.gauge-track {
  flex: 1;
  height: 8px;
  background: var(--bg-input);
  border-radius: 4px;
  overflow: hidden;
}
.gauge-fill {
  height: 100%;
  border-radius: 4px;
  transition: width var(--transition-normal);
}
.gauge-value {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  min-width: 36px;
  text-align: right;
}
.traits-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.trait-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}
.trait-row:last-child {
  border-bottom: none;
}
.trait-key {
  color: var(--text-muted);
  text-transform: capitalize;
}
.trait-val {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
}
.trait-empty {
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
  padding: 12px;
}
</style>
