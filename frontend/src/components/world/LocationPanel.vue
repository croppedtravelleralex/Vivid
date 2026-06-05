<template>
  <div class="location-panel card">
    <div class="card-header">Locations</div>
    <div class="location-list">
      <div v-for="loc in locations" :key="loc.id || loc.name" class="location-item">
        <div class="loc-name">{{ loc.name || loc.id || 'Unknown' }}</div>
        <div class="loc-details">
          <span class="loc-occupants">{{ loc.occupants ?? loc.occupant_count ?? loc.occupantCount ?? 0 }} occupants</span>
          <span v-if="loc.type" class="loc-type">{{ loc.type }}</span>
        </div>
        <div v-if="loc.description" class="loc-desc">{{ loc.description }}</div>
      </div>
      <div v-if="locations.length === 0" class="loc-empty">No locations</div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  locations: { type: Array, default: () => [] }
})
</script>

<style scoped>
.location-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 400px;
  overflow-y: auto;
}
.location-item {
  padding: 10px;
  background: var(--bg-input);
  border-radius: var(--radius-sm);
  border-left: 3px solid var(--warning);
}
.loc-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}
.loc-details {
  display: flex;
  gap: 10px;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}
.loc-type {
  padding: 0 6px;
  border-radius: 4px;
  background: var(--accent-muted);
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: 9px;
}
.loc-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
  line-height: 1.4;
}
.loc-empty {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
