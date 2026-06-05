<template>
  <div class="character-card" :class="{ 'card-dead': character.status === 'dead' }" @click="$emit('click', character)">
    <div class="card-name-row">
      <span class="card-name truncate">{{ character.name || character.id }}</span>
      <span class="card-status" :class="'status-' + (character.status || 'alive')">
        {{ character.status || 'alive' }}
      </span>
    </div>

    <div class="card-bars">
      <div class="bar-row">
        <span class="bar-label">HP</span>
        <div class="bar-track">
          <div class="bar-fill bar-hp" :style="{ width: hpPercent + '%' }"></div>
        </div>
        <span class="bar-value">{{ character.hp ?? '?' }}</span>
      </div>

      <div class="bar-row">
        <span class="bar-label">Hunger</span>
        <div class="bar-track">
          <div class="bar-fill bar-hunger" :style="{ width: hungerPercent + '%' }"></div>
        </div>
        <span class="bar-value">{{ character.hunger ?? character.fullness ?? '?' }}</span>
      </div>

      <div class="bar-row">
        <span class="bar-label">Warmth</span>
        <div class="bar-track">
          <div class="bar-fill bar-warmth" :style="{ width: warmthPercent + '%' }"></div>
        </div>
        <span class="bar-value">{{ character.warmth ?? '?' }}</span>
      </div>
    </div>

    <div class="card-meta">
      <span v-if="character.location" class="card-location">
        &#9901; {{ character.location }}
      </span>
      <span v-if="character.faction" class="card-faction">{{ character.faction }}</span>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  character: { type: Object, required: true }
})

defineEmits(['click'])

const hpPercent = computed(() => {
  const hp = props.character.hp ?? 100
  return Math.max(0, Math.min(100, hp))
})

const hungerPercent = computed(() => {
  const h = props.character.hunger ?? props.character.fullness ?? 100
  return Math.max(0, Math.min(100, h))
})

const warmthPercent = computed(() => {
  const w = props.character.warmth ?? 100
  return Math.max(0, Math.min(100, w))
})
</script>

<style scoped>
.character-card {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 14px;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.character-card:hover {
  border-color: var(--accent);
  box-shadow: var(--shadow-sm);
}
.character-card.card-dead {
  opacity: 0.5;
}
.card-name-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}
.card-status {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 8px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.status-alive {
  background: var(--success-bg);
  color: var(--success);
}
.status-dead {
  background: var(--danger-bg);
  color: var(--danger);
}
.status-asleep,
.status-unconscious {
  background: var(--info-bg);
  color: var(--info);
}
.card-bars {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.bar-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.bar-label {
  font-size: 10px;
  color: var(--text-muted);
  min-width: 42px;
  text-align: right;
}
.bar-track {
  flex: 1;
  height: 6px;
  background: var(--bg-input);
  border-radius: 3px;
  overflow: hidden;
}
.bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width var(--transition-normal);
}
.bar-hp {
  background: var(--hp-bar);
}
.bar-hunger {
  background: var(--hunger-bar);
}
.bar-warmth {
  background: var(--warmth-bar);
}
.bar-value {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  min-width: 24px;
  text-align: right;
}
.card-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 10px;
  font-size: 11px;
  color: var(--text-muted);
}
.card-faction {
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--accent-muted);
  color: var(--accent);
  font-size: 10px;
}
</style>
