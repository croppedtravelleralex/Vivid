<template>
  <header class="app-header">
    <div class="header-left">
      <span class="header-logo">Vivid</span>
      <span class="header-subtitle">Novel Simulation Engine</span>
    </div>
    <div class="header-center">
      <span class="sim-badge" :class="statusClass">
        <span class="sim-dot"></span>
        {{ statusLabel }}
      </span>
    </div>
    <div class="header-right">
      <label class="speed-label" for="speed-select">Speed</label>
      <select
        id="speed-select"
        class="speed-select"
        :value="speed"
        @change="$emit('update:speed', Number($event.target.value))"
      >
        <option :value="0.5">0.5x</option>
        <option :value="1">1x</option>
        <option :value="2">2x</option>
        <option :value="5">5x</option>
        <option :value="10">10x</option>
        <option :value="50">50x</option>
      </select>
    </div>
  </header>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  status: { type: String, default: 'idle' },
  speed: { type: Number, default: 1 }
})

defineEmits(['update:speed'])

const statusLabel = computed(() => {
  const map = { idle: 'Idle', running: 'Running', paused: 'Paused', stopped: 'Stopped' }
  return map[props.status] || props.status
})

const statusClass = computed(() => {
  const map = { idle: 'badge-idle', running: 'badge-running', paused: 'badge-paused', stopped: 'badge-stopped' }
  return map[props.status] || 'badge-idle'
})
</script>

<style scoped>
.app-header {
  grid-area: header;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  z-index: var(--z-header);
}
.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}
.header-logo {
  font-size: 18px;
  font-weight: 700;
  color: var(--accent);
  letter-spacing: 0.03em;
}
.header-subtitle {
  font-size: 12px;
  color: var(--text-muted);
}
.header-center {
  display: flex;
  align-items: center;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.speed-label {
  font-size: 12px;
  color: var(--text-secondary);
}
.speed-select {
  min-width: 75px;
  font-size: 12px;
  padding: 3px 8px;
}
.sim-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}
.sim-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  display: inline-block;
}
.badge-idle {
  background: rgba(90, 96, 128, 0.15);
  color: var(--text-muted);
}
.badge-idle .sim-dot {
  background: var(--text-muted);
}
.badge-running {
  background: var(--success-bg);
  color: var(--success);
}
.badge-running .sim-dot {
  background: var(--success);
  animation: pulse-dot 1.5s ease-in-out infinite;
}
.badge-paused {
  background: var(--warning-bg);
  color: var(--warning);
}
.badge-paused .sim-dot {
  background: var(--warning);
}
.badge-stopped {
  background: var(--danger-bg);
  color: var(--danger);
}
.badge-stopped .sim-dot {
  background: var(--danger);
}
@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
