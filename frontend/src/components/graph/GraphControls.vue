<template>
  <div class="graph-controls">
    <button class="btn btn-icon btn-sm" @click="$emit('zoomIn')" title="Zoom In">+</button>
    <button class="btn btn-icon btn-sm" @click="$emit('zoomOut')" title="Zoom Out">-</button>
    <button class="btn btn-icon btn-sm" @click="$emit('reset')" title="Reset View">R</button>
  </div>
</template>

<script setup>
defineEmits(['zoomIn', 'zoomOut', 'reset'])
</script>

<style scoped>
.graph-controls {
  display: flex;
  gap: 4px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 4px;
  box-shadow: var(--shadow-md);
}
.graph-controls .btn-icon {
  font-size: 14px;
  min-width: 28px;
  height: 28px;
}
</style>
