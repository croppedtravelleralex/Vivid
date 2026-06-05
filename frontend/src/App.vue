<template>
  <div id="app-root">
    <AppHeader
      :status="simulation.status.value"
      :speed="simulation.speed.value"
      @update:speed="simulation.setSpeed"
    />
    <AppSidebar />
    <main class="app-main">
      <router-view />
    </main>
    <StatusBar
      :tick="simulation.tick.value"
      :sim-time="simulation.simTime.value"
      :speed="simulation.speed.value"
      :character-count="characterCount"
      :llm-calls="simulation.llmCalls.value"
      :connected="ws.connected.value"
    />
    <Notification ref="notificationRef" v-model:show="notifShow" :message="notifMsg" :type="notifType" />
    <ConfirmDialog v-model:visible="confirmVisible" :title="confirmTitle" :message="confirmMsg" @confirm="onConfirm" />
  </div>
</template>

<script setup>
import { ref, provide, onMounted } from 'vue'
import AppHeader from './components/layout/AppHeader.vue'
import AppSidebar from './components/layout/AppSidebar.vue'
import StatusBar from './components/layout/StatusBar.vue'
import Notification from './components/common/Notification.vue'
import ConfirmDialog from './components/common/ConfirmDialog.vue'
import { useSimulation } from './composables/useSimulation.js'
import { useWebSocket } from './composables/useWebSocket.js'
import { useWorldState } from './composables/useWorldState.js'
import { getCharacters } from './api/character.js'

// Initialize composables
const simulation = useSimulation()
const ws = useWebSocket()
const worldState = useWorldState()

// Character tracking for status bar
const characterCount = ref(0)
const characters = ref([])

// Notification state
const notificationRef = ref(null)
const notifShow = ref(false)
const notifMsg = ref('')
const notifType = ref('info')

// Confirm dialog state
const confirmVisible = ref(false)
const confirmTitle = ref('')
const confirmMsg = ref('')
let confirmCallback = null

function showNotification(msg, type = 'info') {
  notifMsg.value = msg
  notifType.value = type
  notifShow.value = false
  setTimeout(() => {
    notifShow.value = true
  }, 10)
}

function showConfirm(title, msg, cb) {
  confirmTitle.value = title
  confirmMsg.value = msg
  confirmCallback = cb
  confirmVisible.value = true
}

function onConfirm() {
  if (confirmCallback) confirmCallback()
  confirmCallback = null
}

// Provide shared state to views
provide('simulation', simulation)
provide('ws', ws)
provide('worldState', worldState)
provide('characters', characters)
provide('characterCount', characterCount)
provide('notification', { show: showNotification })
provide('confirm', showConfirm)

// Load initial data
onMounted(async () => {
  try {
    const chars = await getCharacters()
    const list = Array.isArray(chars) ? chars : (chars.characters || chars.data || [])
    characters.value = list
    characterCount.value = list.length
  } catch (_) {}

  worldState.fetchAll().catch(() => {})

  // Wire socket events
  ws.onMessage('character_update', (data) => {
    const c = data.data || data
    const idx = characters.value.findIndex(ch => ch.id === c.id)
    if (idx >= 0) {
      const updated = [...characters.value]
      updated[idx] = { ...updated[idx], ...c }
      characters.value = updated
    }
  })

  ws.onMessage('character_added', (data) => {
    const c = data.data || data
    characters.value = [...characters.value, c]
    characterCount.value = characters.value.length
  })

  ws.onMessage('character_removed', (data) => {
    const id = (data.data || data).id
    characters.value = characters.value.filter(ch => ch.id !== id)
    characterCount.value = characters.value.length
  })

  ws.onMessage('world_update', (data) => {
    worldState.applyEvent(data)
  })

  ws.onMessage('simulation_tick', (data) => {
    const d = data.data || data
    if (d.tick != null) simulation.tick.value = d.tick
    if (d.sim_time || d.simTime) simulation.simTime.value = d.sim_time || d.simTime
  })

  ws.onMessage('event_triggered', (data) => {
    const d = data.data || data
    const msg = d.description || d.message || `${d.type || d.event || 'Event'} triggered`
    showNotification(msg, d.severity === 'critical' ? 'error' : d.severity === 'high' ? 'warning' : 'info')
  })
})
</script>

<style scoped>
.app-main {
  grid-area: main;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px;
}
</style>
