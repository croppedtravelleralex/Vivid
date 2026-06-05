<template>
  <footer class="status-bar">
    <div class="status-left">
      <span class="stat-item">
        <span class="stat-label">Tick</span>
        <span class="stat-value">{{ tick }}</span>
      </span>
      <span class="stat-divider"></span>
      <span class="stat-item">
        <span class="stat-label">Time</span>
        <span class="stat-value">{{ simTime }}</span>
      </span>
      <span class="stat-divider"></span>
      <span class="stat-item">
        <span class="stat-label">Speed</span>
        <span class="stat-value">{{ speed }}x</span>
      </span>
    </div>
    <div class="status-right">
      <span class="stat-item">
        <span class="stat-label">Characters</span>
        <span class="stat-value">{{ characterCount }}</span>
      </span>
      <span class="stat-divider"></span>
      <span class="stat-item">
        <span class="stat-label">LLM Calls</span>
        <span class="stat-value">{{ llmCalls }}</span>
      </span>
      <span class="stat-divider"></span>
      <span class="stat-item ws-status" :class="{ 'ws-connected': connected }">
        <span class="ws-dot"></span>
        {{ connected ? 'Connected' : 'Disconnected' }}
      </span>
    </div>
  </footer>
</template>

<script setup>
defineProps({
  tick: { type: Number, default: 0 },
  simTime: { type: String, default: '00:00:00' },
  speed: { type: Number, default: 1 },
  characterCount: { type: Number, default: 0 },
  llmCalls: { type: Number, default: 0 },
  connected: { type: Boolean, default: false }
})
</script>

<style scoped>
.status-bar {
  grid-area: statusbar;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-primary);
  border-top: 1px solid var(--border);
  font-size: 11px;
  z-index: var(--z-statusbar);
}
.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
}
.stat-label {
  color: var(--text-muted);
}
.stat-value {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
}
.stat-divider {
  width: 1px;
  height: 12px;
  background: var(--border);
}
.ws-status {
  color: var(--text-muted);
  transition: color var(--transition-fast);
}
.ws-status.ws-connected {
  color: var(--success);
}
.ws-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--text-muted);
}
.ws-connected .ws-dot {
  background: var(--success);
}
</style>
