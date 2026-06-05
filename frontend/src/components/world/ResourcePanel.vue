<template>
  <div class="resource-panel card">
    <div class="card-header">Resources</div>
    <div class="chart-wrapper">
      <canvas ref="canvasRef"></canvas>
    </div>
    <div v-if="!hasData" class="resource-empty">No resource data</div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, onUnmounted } from 'vue'
import { Chart, registerables } from 'chart.js'

Chart.register(...registerables)

const props = defineProps({
  resources: { type: Object, default: () => ({}) }
})

const canvasRef = ref(null)
let chartInstance = null
const hasData = ref(false)

function renderChart() {
  if (!canvasRef.value) return

  const entries = Object.entries(props.resources)
  if (entries.length === 0) {
    hasData.value = false
    if (chartInstance) {
      chartInstance.destroy()
      chartInstance = null
    }
    return
  }
  hasData.value = true

  const labels = entries.map(([k]) => k)
  const values = entries.map(([, v]) => typeof v === 'number' ? v : 0)
  const colors = labels.map((_, i) => {
    const hue = (i * 47) % 360
    return `hsl(${hue}, 60%, 55%)`
  })

  const config = {
    type: 'bar',
    data: {
      labels,
      datasets: [{
        data: values,
        backgroundColor: colors,
        borderRadius: 4,
        borderSkipped: false
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: { duration: 300 },
      plugins: {
        legend: { display: false }
      },
      scales: {
        x: {
          grid: { display: false },
          ticks: { color: '#5a6080', font: { size: 10 } }
        },
        y: {
          beginAtZero: true,
          grid: { color: '#2a2e3a' },
          ticks: { color: '#5a6080', font: { size: 10 } }
        }
      }
    }
  }

  if (chartInstance) chartInstance.destroy()
  chartInstance = new Chart(canvasRef.value, config)
}

onMounted(() => renderChart())

watch(() => props.resources, () => renderChart(), { deep: true })

onUnmounted(() => {
  if (chartInstance) chartInstance.destroy()
})
</script>

<style scoped>
.resource-panel {
  display: flex;
  flex-direction: column;
}
.chart-wrapper {
  height: 200px;
  position: relative;
}
.resource-empty {
  padding: 32px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
