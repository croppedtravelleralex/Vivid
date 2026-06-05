<template>
  <div class="character-memory">
    <div class="card">
      <div class="card-header">
        <span>Memory Stream</span>
        <span class="memory-count">{{ memories.length }} memories</span>
      </div>
      <div class="memory-list">
        <div v-for="(mem, i) in memories" :key="mem.id || i" class="memory-item">
          <div class="memory-time">
            <span class="memory-tick">Tick #{{ mem.tick ?? '?' }}</span>
            <span class="memory-simtime">{{ mem.sim_time || mem.simTime || '' }}</span>
          </div>
          <div class="memory-content">{{ mem.content || mem.text || mem.description || '(empty)' }}</div>
          <div v-if="mem.type || mem.category" class="memory-type">
            {{ mem.type || mem.category }}
          </div>
        </div>
        <div v-if="memories.length === 0" class="memory-empty">
          No memories recorded
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  memories: { type: Array, default: () => [] }
})
</script>

<style scoped>
.character-memory {
  display: flex;
  flex-direction: column;
}
.memory-count {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 400;
  text-transform: none;
  letter-spacing: 0;
}
.memory-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
}
.memory-item {
  padding: 10px;
  background: var(--bg-input);
  border-radius: var(--radius-sm);
  border-left: 3px solid var(--accent);
}
.memory-time {
  display: flex;
  gap: 10px;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  margin-bottom: 4px;
}
.memory-content {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.5;
}
.memory-type {
  margin-top: 4px;
  font-size: 10px;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.memory-empty {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
