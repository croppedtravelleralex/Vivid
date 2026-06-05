<template>
  <div class="relation-graph-container">
    <svg ref="svgRef" class="graph-svg"></svg>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, onUnmounted } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  characters: { type: Array, default: () => [] },
  relationships: { type: Array, default: () => [] },
  width: { type: Number, default: 800 },
  height: { type: Number, default: 600 }
})

const emit = defineEmits(['nodeClick'])

const svgRef = ref(null)
let sim = null

function buildGraphData() {
  const nodes = props.characters.map(c => ({
    id: c.id,
    name: c.name || c.id,
    hp: c.hp ?? 100,
    status: c.status || 'alive',
    group: c.faction || 'default'
  }))

  const linkMap = new Map()
  props.relationships.forEach(r => {
    const source = r.source || r.character_a || r.from
    const target = r.target || r.character_b || r.to
    if (!source || !target) return
    const key = [source, target].sort().join('::')
    if (linkMap.has(key)) {
      const existing = linkMap.get(key)
      existing.strength = (existing.strength + (r.strength || r.weight || 1)) / 2
      if (r.type || r.sentiment) existing.type = r.type || r.sentiment
    } else {
      linkMap.set(key, {
        source,
        target,
        strength: r.strength || r.weight || 1,
        type: r.type || r.sentiment || 'neutral',
        label: r.label || ''
      })
    }
  })

  const links = Array.from(linkMap.values())
  return { nodes, links }
}

function render() {
  if (!svgRef.value) return

  const svg = d3.select(svgRef.value)
  svg.selectAll('*').remove()
  svg.attr('viewBox', `0 0 ${props.width} ${props.height}`)

  const { nodes, links } = buildGraphData()
  if (nodes.length === 0) return

  const g = svg.append('g')

  const zoom = d3.zoom()
    .scaleExtent([0.1, 4])
    .on('zoom', (event) => g.attr('transform', event.transform))
  svg.call(zoom)

  const linkGroup = g.append('g').attr('class', 'links')
  const nodeGroup = g.append('g').attr('class', 'nodes')

  const linkElements = linkGroup.selectAll('line')
    .data(links)
    .join('line')
    .attr('class', d => {
      let cls = 'link '
      if (d.strength > 0.7) cls += 'link-strong '
      else if (d.strength > 0.3) cls += 'link-medium '
      else cls += 'link-weak '
      if (d.type === 'positive' || d.type === 'friend') cls += 'link-positive'
      else if (d.type === 'negative' || d.type === 'enemy') cls += 'link-negative'
      else cls += 'link-neutral'
      return cls
    })

  const nodeElements = nodeGroup.selectAll('g')
    .data(nodes)
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
    .on('click', (event, d) => {
      emit('nodeClick', d)
    })

  nodeElements.append('circle')
    .attr('class', 'node-circle')
    .attr('r', d => Math.max(8, (d.hp / 100) * 18))
    .attr('fill', d => {
      if (d.status === 'dead') return 'var(--text-muted)'
      if (d.hp > 60) return 'var(--accent)'
      if (d.hp > 30) return 'var(--warning)'
      return 'var(--danger)'
    })

  nodeElements.append('text')
    .attr('class', 'node-label')
    .attr('dy', d => Math.max(8, (d.hp / 100) * 18) + 14)
    .text(d => d.name)

  sim = d3.forceSimulation(nodes)
    .force('link', d3.forceLink(links).id(d => d.id).distance(130))
    .force('charge', d3.forceManyBody().strength(-250))
    .force('center', d3.forceCenter(props.width / 2, props.height / 2))
    .force('collision', d3.forceCollide().radius(30))
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

onMounted(() => {
  render()
})

watch(() => [props.characters, props.relationships], () => {
  if (sim) {
    sim.stop()
    sim = null
  }
  render()
}, { deep: true })

onUnmounted(() => {
  if (sim) {
    sim.stop()
    sim = null
  }
})
</script>

<style scoped>
.relation-graph-container {
  width: 100%;
  height: 100%;
  min-height: 400px;
  border-radius: var(--radius-md);
  overflow: hidden;
}
</style>
