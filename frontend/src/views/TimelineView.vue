<template>
  <div class="timeline-view">
    <div class="tv-controls">
      <SpeedControl
        :status="sim.status.value"
        :speed="sim.speed.value"
        @start="sim.start"
        @pause="sim.pause"
        @step="sim.stepOnce"
        @stop="sim.stop"
        @set-speed="sim.setSpeed"
      />
      <div class="tv-filter">
        <input
          type="text"
          v-model="eventFilter"
          placeholder="Filter events..."
          class="tv-filter-input"
        />
      </div>
    </div>

    <div class="tv-timeline card">
      <TimelineBar
        :events="timelineEvents"
        :current-time="sim.simTime.value"
      />
    </div>

    <div class="tv-log card">
      <EventLog :events="timelineEvents" :filter="eventFilter" />
    </div>
  </div>
</template>

<script setup>
import { ref, inject, onMounted, onUnmounted } from 'vue'
import SpeedControl from '../components/timeline/SpeedControl.vue'
import TimelineBar from '../components/timeline/TimelineBar.vue'
import EventLog from '../components/timeline/EventLog.vue'
import { getTimelineEvents } from '../api/timeline.js'

const sim = inject('simulation')
const ws = inject('ws')

const timelineEvents = ref([])
const eventFilter = ref('')

let loadTimer = null

async function loadEvents() {
  try {
    const data = await getTimelineEvents({ limit: 200 })
    const list = Array.isArray(data) ? data : (data.events || data.data || [])
    timelineEvents.value = list
  } catch (_) {}
}

function onWsEvent(event) {
  const d = event.data || event
  const events = timelineEvents.value
  timelineEvents.value = [...events.slice(-499), d]
}

let unsub = null

onMounted(() => {
  loadEvents()
  // Listen for new events via websocket
  if (ws && ws.onMessage) {
    unsub = ws.onMessage('*', onWsEvent)
  }
  // Also poll periodically
  loadTimer = setInterval(loadEvents, 5000)
})

onUnmounted(() => {
  if (unsub) unsub()
  if (loadTimer) {
    clearInterval(loadTimer)
    loadTimer = null
  }
})
</script>

<style scoped>
.timeline-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}
.tv-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}
.tv-filter {
  display: flex;
  align-items: center;
}
.tv-filter-input {
  width: 200px;
  padding: 5px 10px;
  font-size: 12px;
}
.tv-timeline {
  flex-shrink: 0;
}
.tv-log {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
