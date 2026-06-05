<template>
  <div class="location-map-container">
    <svg ref="svgRef" class="graph-svg"></svg>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, onUnmounted } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  locations: { type: Array, default: () => [] },
  width: { type: Number, default: 600 },
  height: { type: Number, default: 400 }
})

const svgRef = ref(null)
let sim = null

function render() {
  if (!svgRef.value) return

  const svg = d3.select(svgRef.value)
  svg.selectAll('*').remove()
  svg.attr('viewBox', `0 0 ${props.width} ${props.height}`)

  const locs = props.locations.map((l, i) => ({
    id: l.id || l.name || `loc-${i}`,
    name: l.name || l.id || `Location ${i}`,
    occupants: l.occupants || l.occupant_count || l.occupantCount || 0,
    type: l.type || 'unknown',
    connections: l.connections || []
  }))

  if (locs.length === 0) {
    svg.append('text')
      .attr('x', props.width / 2)
      .attr('y', props.height / 2)
      .attr('text-anchor', 'middle')
      .attr('fill', 'var(--text-muted)')
      .attr('font-size', '14px')
      .text('No locations available')
    return
  }

  const g = svg.append('g')

  const zoom = d3.zoom()
    .scaleExtent([0.2, 3])
    .on('zoom', (event) => g.attr('transform', event.transform))
  svg.call(zoom)

  const links = []
  locs.forEach(loc => {
    if (Array.isArray(loc.connections)) {
      loc.connections.forEach(conn => {
        links.push({ source: loc.id, target: conn })
      })
    }
  })

  const linkGroup = g.append('g').attr('class', 'links')
  const nodeGroup = g.append('g').attr('class', 'nodes')

  const linkElements = linkGroup.selectAll('line')
    .data(links)
    .join('line')
    .attr('class', 'link link-neutral')

  const nodeElements = nodeGroup.selectAll('g')
    .data(locs)
    .join('g')
    .attr('class', 'node')
    .call(d3.drag()
      .on('start', (event, d) => {
        if (!event.active) sim.alphaTarget(0.3).restart()
        d.fx = d.x
        d.fy = d.y
      })
      .on('drag', (event, d) => {
        d.fx = event.x
        d.fy = event.y
      })
      .on('end', (event, d) => {
        if (!event.active) sim.alphaTarget(0)
        d.fx = null
        d.fy = null
      })
    )

  nodeElements.append('rect')
    .attr('rx', 6)
    .attr('ry', 6)
    .attr('width', d => Math.max(80, d.name.length * 9 + 20))
    .attr('height', 36)
    .attr('x', d => -(Math.max(80, d.name.length * 9 + 20)) / 2)
    .attr('y', -18)
    .attr('class', d => `location-node ${d.type}`)
    .attr('fill', 'var(--warning-bg)')
    .attr('stroke', 'var(--warning)')
    .attr('stroke-width', 1.5)

  nodeElements.append('text')
    .attr('class', 'location-label')
    .attr('dy', -3)
    .text(d => d.name)

  nodeElements.append('text')
    .attr('class', 'location-label')
    .attr('dy', 14)
    .attr('font-size', '9px')
    .attr('fill', 'var(--text-muted)')
    .text(d => `${d.occupants} occupants`)

  sim = d3.forceSimulation(locs)
    .force('link', d3.forceLink(links).id(d => d.id).distance(150))
    .force('charge', d3.forceManyBody().strength(-400))
    .force('center', d3.forceCenter(props.width / 2, props.height / 2))
    .force('collision', d3.forceCollide().radius(60))
    .on('tick', () => {
      linkElements
        .attr('x1', d => d.source.x)
        .attr('y1', d => d.source.y)
        .attr('x2', d => d.target.x)
        .attr('y2', d => d.target.y)
      nodeElements
        .attr('transform', d => `translate(${d.x},${d.y})`)
    })
}

onMounted(() => render())

watch(() => props.locations, () => {
  if (sim) { sim.stop(); sim = null }
  render()
}, { deep: true })

onUnmounted(() => {
  if (sim) { sim.stop(); sim = null }
})
</script>

<style scoped>
.location-map-container {
  width: 100%;
  height: 100%;
  min-height: 300px;
  border-radius: var(--radius-md);
  overflow: hidden;
}
</style>
