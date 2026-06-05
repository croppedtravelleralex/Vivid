<template>
  <div class="dashboard">
    <div class="dashboard-top">
      <div class="dash-col dash-col-wide">
        <SpeedControl
          :status="sim.status.value"
          :speed="sim.speed.value"
          @start="sim.start"
          @pause="sim.pause"
          @step="sim.stepOnce"
          @stop="sim.stop"
          @set-speed="sim.setSpeed"
        />
        <div class="dash-stats">
          <div class="stat-box">
            <span class="stat-box-value">{{ sim.tick.value }}</span>
            <span class="stat-box-label">Tick</span>
          </div>
          <div class="stat-box">
            <span class="stat-box-value">{{ sim.simTime.value }}</span>
            <span class="stat-box-label">Time</span>
          </div>
          <div class="stat-box">
            <span class="stat-box-value">{{ sim.speed.value }}x</span>
            <span class="stat-box-label">Speed</span>
          </div>
          <div class="stat-box">
            <span class="stat-box-value">{{ sim.llmCalls.value }}</span>
            <span class="stat-box-label">LLM Calls</span>
          </div>
          <div class="stat-box">
            <span class="stat-box-value">{{ characterCount }}</span>
            <span class="stat-box-label">Characters</span>
          </div>
        </div>
      </div>
      <div class="dash-col">
        <EnvironmentPanel :environment="worldState.environment" />
      </div>
    </div>

    <div class="dashboard-mid">
      <div class="dash-col dash-col-wide">
        <div class="card">
          <div class="card-header">Characters</div>
          <div class="dash-char-grid">
            <CharacterCard
              v-for="char in dashboardChars"
              :key="char.id"
              :character="char"
              @click="$router.push('/character/' + char.id)"
            />
          </div>
          <div v-if="dashboardChars.length === 0" class="dash-empty">No characters</div>
        </div>
      </div>
      <div class="dash-col">
        <div class="card">
          <div class="card-header">Recent Events</div>
          <EventLog :events="recentEvents" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, inject } from 'vue'
import { useRouter } from 'vue-router'
import SpeedControl from '../components/timeline/SpeedControl.vue'
import EventLog from '../components/timeline/EventLog.vue'
import EnvironmentPanel from '../components/world/EnvironmentPanel.vue'
import CharacterCard from '../components/character/CharacterCard.vue'

const router = useRouter()
const sim = inject('simulation')
const ws = inject('ws')
const worldState = inject('worldState')
const characters = inject('characters', { value: [] })
const characterCount = inject('characterCount', { value: 0 })

// For characterCount, it could be a ref or a plain number
const countVal = typeof characterCount === 'object' && characterCount.value !== undefined
  ? characterCount.value
  : characterCount

const dashboardChars = computed(() => {
  const chars = typeof characters === 'object' && characters.value !== undefined
    ? characters.value
    : characters
  return Array.isArray(chars) ? chars.slice(0, 6) : []
})

const recentEvents = computed(() => {
  const evts = typeof ws.events === 'object' && ws.events.value !== undefined
    ? ws.events.value
    : []
  return Array.isArray(evts) ? evts.slice(-20).reverse() : []
})
</script>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}
.dashboard-top,
.dashboard-mid {
  display: flex;
  gap: 16px;
}
.dash-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}
.dash-col-wide {
  flex: 2;
}
.dash-stats {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.stat-box {
  flex: 1;
  min-width: 80px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.stat-box-value {
  font-size: 20px;
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--text-primary);
}
.stat-box-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.dash-char-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 8px;
}
.dash-empty {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
