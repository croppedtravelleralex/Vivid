import { ref, onUnmounted, shallowRef } from 'vue'
import * as d3 from 'd3'

export function useGraph() {
  const svgRef = ref(null)
  const nodes = shallowRef([])
  const links = shallowRef([])
  const simulation = ref(null)
  const zoomTransform = ref(null)

  let sim = null
  let zoomBehavior = null
  let isDestroyed = false

  function initGraph(options = {}) {
    if (!svgRef.value) return
    const svg = d3.select(svgRef.value)
    svg.selectAll('*').remove()

    const width = options.width || 800
    const height = options.height || 600

    const g = svg.append('g')

    zoomBehavior = d3.zoom()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform)
        zoomTransform.value = event.transform
      })

    svg.call(zoomBehavior)

    sim = d3.forceSimulation()
      .force('link', d3.forceLink().id(d => d.id).distance(120))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30))

    simulation.value = sim
  }

  function updateData(newNodes, newLinks) {
    nodes.value = newNodes
    links.value = newLinks
    if (!sim) return

    sim.nodes(newNodes)
    sim.force('link').links(newLinks)
    sim.alpha(1).restart()
  }

  function zoomIn() {
    if (!svgRef.value || !zoomBehavior) return
    const svg = d3.select(svgRef.value)
    svg.transition().duration(300).call(zoomBehavior.scaleBy, 1.4)
  }

  function zoomOut() {
    if (!svgRef.value || !zoomBehavior) return
    const svg = d3.select(svgRef.value)
    svg.transition().duration(300).call(zoomBehavior.scaleBy, 0.7)
  }

  function resetZoom() {
    if (!svgRef.value || !zoomBehavior) return
    const svg = d3.select(svgRef.value)
    svg.transition().duration(400).call(zoomBehavior.transform, d3.zoomIdentity)
    zoomTransform.value = d3.zoomIdentity
  }

  function destroy() {
    if (sim) {
      sim.stop()
      sim = null
    }
    if (svgRef.value) {
      d3.select(svgRef.value).selectAll('*').remove()
    }
    simulation.value = null
    zoomBehavior = null
  }

  onUnmounted(() => {
    isDestroyed = true
    destroy()
  })

  return {
    svgRef,
    nodes,
    links,
    simulation,
    zoomTransform,
    initGraph,
    updateData,
    zoomIn,
    zoomOut,
    resetZoom,
    destroy
  }
}
