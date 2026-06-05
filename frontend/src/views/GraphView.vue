<template>
  <div class="graph-view">
    <div class="graph-container" ref="graphContainerRef">
      <RelationGraph
        :characters="graphChars"
        :relationships="graphRelationships"
        :width="graphWidth"
        :height="graphHeight"
        @node-click="handleNodeClick"
      />
      <div class="graph-controls-overlay">
        <GraphControls @zoom-in="zoomIn" @zoom-out="zoomOut" @reset="resetZoom" />
      </div>
    </div>
    <div v-if="selectedNode" class="graph-info">
      <div class="graph-info-header">
        <span class="graph-info-name">{{ selectedNode.name }}</span>
        <button class="btn btn-sm" @click="selectedNode = null">&times;</button>
      </div>
      <div class="graph-info-body">
        <div class="info-row"><span class="info-label">ID</span><span class="info-value">{{ selectedNode.id }}</span></div>
        <div class="info-row"><span class="info-label">HP</span><span class="info-value">{{ selectedNode.hp }}</span></div>
        <div class="info-row"><span class="info-label">Status</span><span class="info-value">{{ selectedNode.status }}</span></div>
        <div class="info-row"><span class="info-label">Faction</span><span class="info-value">{{ selectedNode.group }}</span></div>
        <button class="btn btn-sm" @click="$router.push('/character/' + selectedNode.id)">View Character</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, inject, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import RelationGraph from '../components/graph/RelationGraph.vue'
import GraphControls from '../components/graph/GraphControls.vue'
import { getCharacters } from '../api/character.js'
import { getCharacterRelationships } from '../api/character.js'

const router = useRouter()
const characters = inject('characters', { value: [] })
const ws = inject('ws')

const graphContainerRef = ref(null)
const graphWidth = ref(800)
const graphHeight = ref(600)
const graphRelationships = ref([])
const selectedNode = ref(null)

const graphChars = computed(() => {
  const chars = typeof characters === 'object' && characters.value !== undefined
    ? characters.value
    : characters
  return Array.isArray(chars) ? chars : []
})

function updateSize() {
  if (graphContainerRef.value) {
    graphWidth.value = graphContainerRef.value.clientWidth || 800
    graphHeight.value = graphContainerRef.value.clientHeight || 600
  }
}

function handleNodeClick(node) {
  selectedNode.value = node
}

function zoomIn() {}
function zoomOut() {}
function resetZoom() {}

async function loadRelationships() {
  const chars = graphChars.value
  const allRels = []
  // Load relationships for first 20 characters max
  const targets = chars.slice(0, 20)
  for (const char of targets) {
    try {
      const data = await getCharacterRelationships(char.id)
      const rels = Array.isArray(data) ? data : (data.relationships || data.data || [])
      allRels.push(...rels)
    } catch (_) {}
  }
  graphRelationships.value = allRels
}

let unsub = null

onMounted(() => {
  updateSize()
  window.addEventListener('resize', updateSize)
  loadRelationships()

  if (ws && ws.onMessage) {
    unsub = ws.onMessage('character_update', () => {
      setTimeout(loadRelationships, 500)
    })
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', updateSize)
  if (unsub) unsub()
})
</script>

<style scoped>
.graph-view {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  gap: 12px;
}
.graph-container {
  flex: 1;
  position: relative;
}
.graph-info {
  width: 220px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-self: flex-start;
}
.graph-info-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.graph-info-name {
  font-weight: 600;
  font-size: 14px;
}
.graph-info-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.info-row {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}
.info-label {
  color: var(--text-muted);
}
.info-value {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
}
</style>
