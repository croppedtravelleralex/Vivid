<template>
  <div class="world-view">
    <div class="wv-top">
      <div class="wv-col">
        <EnvironmentPanel :environment="worldState.environment" />
      </div>
      <div class="wv-col">
        <ResourcePanel :resources="resources" />
      </div>
    </div>
    <div class="wv-bottom">
      <div class="wv-col wv-col-wide">
        <LocationPanel :locations="worldState.locations" />
      </div>
      <div class="wv-col">
        <div class="card wv-map-card">
          <div class="card-header">Location Map</div>
          <LocationMap :locations="worldState.locations" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, inject, onMounted } from 'vue'
import EnvironmentPanel from '../components/world/EnvironmentPanel.vue'
import ResourcePanel from '../components/world/ResourcePanel.vue'
import LocationPanel from '../components/world/LocationPanel.vue'
import LocationMap from '../components/graph/LocationMap.vue'

const worldState = inject('worldState')

const resources = ref({
  food: 0,
  water: 0,
  wood: 0,
  stone: 0,
  metal: 0
})

onMounted(() => {
  if (worldState.world && worldState.world.value) {
    const w = worldState.world.value
    if (w.resources) resources.value = { ...resources.value, ...w.resources }
  }
})
</script>

<style scoped>
.world-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}
.wv-top,
.wv-bottom {
  display: flex;
  gap: 16px;
}
.wv-col {
  flex: 1;
  min-width: 0;
}
.wv-col-wide {
  flex: 1.5;
}
.wv-map-card {
  display: flex;
  flex-direction: column;
}
</style>
