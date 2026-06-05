<template>
  <div class="timeline-bar" ref="containerRef">
    <div class="timeline-header">
      <span class="timeline-title">Timeline</span>
      <span class="timeline-range">{{ formatTime(range[0]) }} - {{ formatTime(range[1]) }}</span>
    </div>
    <div class="timeline-svg-wrapper" ref="wrapperRef">
      <svg ref="svgRef" class="timeline-svg"></svg>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, onUnmounted } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  events: { type: Array, default: () => [] },
  currentTime: { type: String, default: '00:00:00' }
})

const emit = defineEmits(['seek'])

const containerRef = ref(null)
const wrapperRef = ref(null)
const svgRef = ref(null)

let resizeObserver = null

function getEventTime(e) {
  return e.sim_time || e.simTime || e.timestamp || ''
}

function formatTime(t) {
  return t || '00:00:00'
}

function render() {
  if (!svgRef.value || !wrapperRef.value) return

  const width = wrapperRef.value.clientWidth || 600
  const height = 80
  const margin = { top: 20, right: 16, bottom: 20, left: 16 }
  const innerWidth = width - margin.left - margin.right

  const svg = d3.select(svgRef.value)
  svg.selectAll('*').remove()
  svg.attr('viewBox', `0 0 ${width} ${height}`)

  const g = svg.append('g').attr('transform', `translate(${margin.left},${margin.top})`)

  // Scale
  const times = props.events.map(getEventTime).filter(Boolean)
  const allTimes = [props.currentTime, ...times]
  const xScale = d3.scalePoint()
    .domain([...new Set(allTimes)])
    .range([0, innerWidth])

  // Baseline
  g.append('line')
    .attr('x1', 0)
    .attr('y1', 20)
    .attr('x2', innerWidth)
    .attr('y2', 20)
    .attr('stroke', 'var(--border)')
    .attr('stroke-width', 2)

  // Current time marker
  if (xScale(props.currentTime) != null) {
    g.append('line')
      .attr('x1', xScale(props.currentTime))
      .attr('y1', 8)
      .attr('x2', xScale(props.currentTime))
      .attr('y2', 50)
      .attr('stroke', 'var(--accent)')
      .attr('stroke-width', 2)
      .attr('stroke-dasharray', '4,3')

    g.append('circle')
      .attr('cx', xScale(props.currentTime))
      .attr('cy', 20)
      .attr('r', 5)
      .attr('fill', 'var(--accent)')
      .attr('stroke', 'var(--bg-primary)')
      .attr('stroke-width', 2)
  }

  // Event markers
  const severityColors = {
    critical: 'var(--danger)',
    high: 'var(--warning)',
    medium: 'var(--info)',
    low: 'var(--text-muted)',
    info: 'var(--text-muted)'
  }

  g.selectAll('.event-marker')
    .data(props.events)
    .join('g')
    .attr('class', 'event-marker')
    .attr('transform', d => {
      const t = getEventTime(d)
      const x = xScale(t)
      return x != null ? `translate(${x}, 20)` : `translate(-100, 20)`
    })
    .append('circle')
    .attr('r', d => {
      const sev = (d.severity || d.level || 'info').toLowerCase()
      return sev === 'critical' ? 6 : sev === 'high' ? 5 : sev === 'medium' ? 4 : 3
    })
    .attr('fill', d => {
      const sev = (d.severity || d.level || 'info').toLowerCase()
      return severityColors[sev] || 'var(--text-muted)'
    })
    .attr('stroke', 'var(--bg-primary)')
    .attr('stroke-width', 1.5)
    .style('cursor', 'pointer')
    .append('title')
    .text(d => `${d.type || d.event || 'event'}: ${d.description || ''}`)
}

onMounted(() => {
  render()
  if (wrapperRef.value) {
    resizeObserver = new ResizeObserver(() => render())
    resizeObserver.observe(wrapperRef.value)
  }
})

watch(() => [props.events, props.currentTime], () => render(), { deep: true })

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
})
</script>

<style scoped>
.timeline-bar {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.timeline-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-secondary);
}
.timeline-range {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}
.timeline-svg-wrapper {
  width: 100%;
  overflow-x: auto;
}
.timeline-svg {
  display: block;
  width: 100%;
  height: 80px;
}
</style>
