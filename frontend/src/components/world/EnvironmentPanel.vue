<template>
  <div class="environment-panel card">
    <div class="card-header">Environment</div>
    <div class="env-grid">
      <div class="env-item">
        <span class="env-label">Temperature</span>
        <span class="env-value">{{ environment.temperature ?? '--' }}°C</span>
      </div>
      <div class="env-item">
        <span class="env-label">Weather</span>
        <span class="env-value capitalize">{{ environment.weather || '--' }}</span>
      </div>
      <div class="env-item">
        <span class="env-label">Season</span>
        <span class="env-value capitalize">{{ environment.season || '--' }}</span>
      </div>
      <div class="env-item">
        <span class="env-label">Time of Day</span>
        <span class="env-value capitalize">{{ environment.time_of_day || environment.timeOfDay || '--' }}</span>
      </div>
      <div v-if="environment.humidity != null" class="env-item">
        <span class="env-label">Humidity</span>
        <span class="env-value">{{ environment.humidity }}%</span>
      </div>
      <div v-if="environment.wind != null" class="env-item">
        <span class="env-label">Wind</span>
        <span class="env-value">{{ environment.wind }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  environment: { type: Object, default: () => ({}) }
})
</script>

<style scoped>
.env-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.env-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px;
  background: var(--bg-input);
  border-radius: var(--radius-sm);
}
.env-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.env-value {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  font-family: var(--font-mono);
}
.capitalize {
  text-transform: capitalize;
}
</style>
