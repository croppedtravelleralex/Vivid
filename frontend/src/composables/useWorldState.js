import { ref, readonly, onUnmounted } from 'vue'
import { getWorld, getEnvironment, getLocations } from '../api/world.js'

export function useWorldState() {
  const world = ref(null)
  const environment = ref({
    temperature: 20,
    weather: 'clear',
    season: 'spring',
    time_of_day: 'day'
  })
  const locations = ref([])
  const loading = ref(false)
  const error = ref(null)

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      const [worldData, envData, locsData] = await Promise.all([
        getWorld().catch(() => null),
        getEnvironment().catch(() => null),
        getLocations().catch(() => [])
      ])
      if (worldData) world.value = worldData
      if (envData) environment.value = { ...environment.value, ...envData }
      if (Array.isArray(locsData)) locations.value = locsData
    } catch (err) {
      error.value = err.message
    } finally {
      loading.value = false
    }
  }

  function applyEvent(event) {
    if (!event) return
    const data = event.data || event
    if (data.environment) {
      environment.value = { ...environment.value, ...data.environment }
    }
    if (data.locations && Array.isArray(data.locations)) {
      locations.value = data.locations
    }
    if (data.world) {
      world.value = { ...(world.value || {}), ...data.world }
    }
  }

  onUnmounted(() => {})

  return {
    world: readonly(world),
    environment,
    locations,
    loading: readonly(loading),
    error: readonly(error),
    fetchAll,
    applyEvent
  }
}
