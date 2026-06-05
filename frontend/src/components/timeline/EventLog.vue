<template>
  <div class="event-log">
    <div class="event-log-header">
      <span class="event-log-title">Event Log</span>
      <span class="event-log-count">{{ filteredEvents.length }} events</span>
    </div>
    <div class="event-log-list" ref="listRef">
      <div
        v-for="(event, i) in filteredEvents"
        :key="event.id || event.tick || i"
        class="event-item"
        :class="severityClass(event)"
      >
        <span class="event-tick" :title="'Tick: ' + (event.tick ?? '?')">
          #{{ event.tick ?? '?' }}
        </span>
        <span class="event-type">{{ event.type || event.event || 'unknown' }}</span>
        <span class="event-desc truncate">{{ event.description || event.message || '' }}</span>
        <span class="event-time">{{ event.sim_time || event.simTime || '' }}</span>
      </div>
      <div v-if="filteredEvents.length === 0" class="event-empty">
        No events yet
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  events: { type: Array, default: () => [] },
  filter: { type: String, default: '' }
})

const listRef = ref(null)

const filteredEvents = computed(() => {
  if (!props.filter) return props.events
  const f = props.filter.toLowerCase()
  return props.events.filter(e =>
    (e.type || '').toLowerCase().includes(f) ||
    (e.event || '').toLowerCase().includes(f) ||
    (e.description || '').toLowerCase().includes(f) ||
    (e.message || '').toLowerCase().includes(f)
  )
})

function severityClass(event) {
  const sev = (event.severity || event.level || 'info').toLowerCase()
  return `severity-${sev}`
}
</script>

<style scoped>
.event-log {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.event-log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.event-log-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-secondary);
}
.event-log-count {
  font-size: 11px;
  color: var(--text-muted);
}
.event-log-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.event-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 5px 10px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  border-left: 3px solid transparent;
  transition: background var(--transition-fast);
}
.event-item:hover {
  background: var(--bg-surface-hover);
}
.event-tick {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  min-width: 50px;
}
.event-type {
  font-weight: 500;
  min-width: 90px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.event-desc {
  flex: 1;
  color: var(--text-secondary);
}
.event-time {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
}
.event-empty {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
.severity-critical {
  border-left-color: var(--danger);
}
.severity-critical .event-type {
  color: var(--danger);
}
.severity-high {
  border-left-color: var(--warning);
}
.severity-high .event-type {
  color: var(--warning);
}
.severity-medium {
  border-left-color: var(--info);
}
.severity-medium .event-type {
  color: var(--info);
}
.severity-low,
.severity-info {
  border-left-color: var(--text-muted);
}
.severity-low .event-type,
.severity-info .event-type {
  color: var(--text-muted);
}
</style>
