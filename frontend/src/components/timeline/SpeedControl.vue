<template>
  <div class="speed-control">
    <button
      class="btn btn-sm"
      :class="{ 'btn-primary': status === 'running' }"
      @click="$emit('start')"
      :disabled="status === 'running'"
    >
      &#9654; Start
    </button>
    <button
      class="btn btn-sm"
      :class="{ 'btn-primary': status === 'paused' }"
      @click="$emit('pause')"
      :disabled="status !== 'running'"
    >
      &#10074;&#10074; Pause
    </button>
    <button
      class="btn btn-sm"
      @click="$emit('step')"
      :disabled="status !== 'paused' && status !== 'running'"
    >
      &#9654;| Step
    </button>
    <button
      class="btn btn-sm btn-danger"
      @click="$emit('stop')"
      :disabled="status === 'idle'"
    >
      &#9632; Stop
    </button>
    <div class="speed-group">
      <label class="speed-group-label">Speed</label>
      <div class="speed-buttons">
        <button
          v-for="opt in speedOptions"
          :key="opt"
          class="btn btn-sm"
          :class="{ 'btn-primary': speed === opt }"
          @click="$emit('setSpeed', opt)"
        >
          {{ opt }}x
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  status: { type: String, default: 'idle' },
  speed: { type: Number, default: 1 }
})

defineEmits(['start', 'pause', 'step', 'stop', 'setSpeed'])

const speedOptions = [0.5, 1, 2, 5, 10, 50]
</script>

<style scoped>
.speed-control {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
}
.speed-group {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: 12px;
}
.speed-group-label {
  font-size: 11px;
  color: var(--text-muted);
}
.speed-buttons {
  display: flex;
  gap: 3px;
}
.speed-buttons .btn {
  min-width: 38px;
  justify-content: center;
}
</style>
