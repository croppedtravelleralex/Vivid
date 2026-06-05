import { ref, onUnmounted } from 'vue'
import { getStatus, start as apiStart, pause as apiPause, step as apiStep, setSpeed as apiSetSpeed, stop as apiStop, getStats } from '../api/simulation.js'

export function useSimulation() {
  const status = ref('idle')
  const speed = ref(1)
  const tick = ref(0)
  const simTime = ref('00:00:00')
  const llmCalls = ref(0)
  const error = ref(null)

  let pollTimer = null
  let isDestroyed = false

  async function pollStatus() {
    if (isDestroyed) return
    try {
      const data = await getStatus()
      status.value = data.status || data.state || 'idle'
      speed.value = data.speed ?? speed.value
      tick.value = data.tick ?? tick.value
      simTime.value = data.sim_time || data.simTime || simTime.value
    } catch (err) {
      error.value = err.message
    }
  }

  async function pollStats() {
    if (isDestroyed) return
    try {
      const data = await getStats()
      llmCalls.value = data.llm_calls ?? data.llmCalls ?? llmCalls.value
    } catch (_) {}
  }

  function startPolling() {
    stopPolling()
    pollStatus()
    pollStats()
    pollTimer = setInterval(() => {
      pollStatus()
      pollStats()
    }, 2000)
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  async function start() {
    try {
      error.value = null
      await apiStart()
      status.value = 'running'
      startPolling()
    } catch (err) {
      error.value = err.message
    }
  }

  async function pause() {
    try {
      error.value = null
      await apiPause()
      status.value = 'paused'
    } catch (err) {
      error.value = err.message
    }
  }

  async function stepOnce() {
    try {
      error.value = null
      await apiStep()
      await pollStatus()
    } catch (err) {
      error.value = err.message
    }
  }

  async function setSpeed(val) {
    try {
      error.value = null
      await apiSetSpeed(val)
      speed.value = val
    } catch (err) {
      error.value = err.message
    }
  }

  async function stop() {
    try {
      error.value = null
      await apiStop()
      status.value = 'idle'
      stopPolling()
    } catch (err) {
      error.value = err.message
    }
  }

  onUnmounted(() => {
    isDestroyed = true
    stopPolling()
  })

  return {
    status,
    speed,
    tick,
    simTime,
    llmCalls,
    error,
    start,
    pause,
    stepOnce,
    setSpeed,
    stop,
    refresh: pollStatus
  }
}
