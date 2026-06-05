<template>
  <div class="settings-view">
    <h2 class="sv-title">Settings</h2>

    <div class="sv-section card">
      <div class="card-header">Simulation Config</div>
      <div class="sv-form">
        <div class="form-row">
          <label class="form-label">Tick Interval (ms)</label>
          <input type="number" v-model.number="config.tickInterval" class="form-input" />
        </div>
        <div class="form-row">
          <label class="form-label">Max Characters</label>
          <input type="number" v-model.number="config.maxCharacters" class="form-input" />
        </div>
        <div class="form-row">
          <label class="form-label">LLM Model</label>
          <input type="text" v-model="config.llmModel" class="form-input" />
        </div>
        <div class="form-row">
          <label class="form-label">Config JSON</label>
          <JsonEditor ref="jsonEditorRef" v-model="configJson" :rows="10" />
        </div>
        <div class="form-actions">
          <button class="btn btn-primary" @click="saveConfig">Save Config</button>
          <button class="btn" @click="reloadConfig">Reload</button>
        </div>
      </div>
    </div>

    <div class="sv-section card">
      <div class="card-header">Checkpoints</div>
      <div class="sv-checkpoint-row">
        <input
          type="text"
          v-model="checkpointTag"
          placeholder="Checkpoint tag..."
          class="form-input"
          @keyup.enter="saveCheckpoint"
        />
        <button class="btn btn-primary" @click="saveCheckpoint" :disabled="!checkpointTag.trim()">Save</button>
      </div>
      <div class="sv-checkpoint-list">
        <div v-for="cp in checkpoints" :key="cp.tag || cp.id" class="cp-item">
          <div class="cp-info">
            <span class="cp-tag">{{ cp.tag || cp.id }}</span>
            <span class="cp-time">{{ cp.created_at || cp.createdAt || cp.timestamp || '' }}</span>
          </div>
          <div class="cp-actions">
            <button class="btn btn-sm" @click="loadCheckpoint(cp.tag || cp.id)">Load</button>
            <button class="btn btn-sm btn-danger" @click="deleteCheckpoint(cp.tag || cp.id)">Del</button>
          </div>
        </div>
        <div v-if="checkpoints.length === 0" class="sv-empty">No checkpoints saved</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, inject, onMounted } from 'vue'
import JsonEditor from '../components/common/JsonEditor.vue'
import { getCheckpoints, saveCheckpoint as apiSaveCheckpoint, loadCheckpoint as apiLoadCheckpoint } from '../api/timeline.js'

const notification = inject('notification')

const config = ref({
  tickInterval: 1000,
  maxCharacters: 20,
  llmModel: 'gpt-4'
})

const configJson = ref('{}')
const jsonEditorRef = ref(null)
const checkpointTag = ref('')
const checkpoints = ref([])

function reloadConfig() {
  configJson.value = JSON.stringify(config.value, null, 2)
}

function saveConfig() {
  try {
    const parsed = JSON.parse(configJson.value)
    Object.assign(config.value, parsed)
    notification.show('Config saved', 'success')
  } catch (err) {
    notification.show('Invalid JSON: ' + err.message, 'error')
  }
}

async function saveCheckpoint() {
  const tag = checkpointTag.value.trim()
  if (!tag) return
  try {
    await apiSaveCheckpoint(tag)
    notification.show('Checkpoint "' + tag + '" saved', 'success')
    checkpointTag.value = ''
    await loadCheckpoints()
  } catch (err) {
    notification.show('Failed to save checkpoint: ' + err.message, 'error')
  }
}

async function loadCheckpoint(tag) {
  try {
    await apiLoadCheckpoint(tag)
    notification.show('Checkpoint "' + tag + '" loaded', 'success')
  } catch (err) {
    notification.show('Failed to load checkpoint: ' + err.message, 'error')
  }
}

async function loadCheckpoints() {
  try {
    const data = await getCheckpoints()
    checkpoints.value = Array.isArray(data) ? data : (data.checkpoints || data.data || [])
  } catch (_) {
    checkpoints.value = []
  }
}

async function deleteCheckpoint(tag) {
  try {
    const { default: api } = await import('../api/index.js')
    await api.delete('/timeline/checkpoints/' + encodeURIComponent(tag))
    notification.show('Checkpoint deleted', 'success')
    await loadCheckpoints()
  } catch (err) {
    notification.show('Failed to delete: ' + err.message, 'error')
  }
}

onMounted(() => {
  reloadConfig()
  loadCheckpoints()
})
</script>

<style scoped>
.settings-view {
  max-width: 720px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.sv-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}
.sv-section {
  display: flex;
  flex-direction: column;
}
.sv-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.form-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.form-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}
.form-input {
  width: 100%;
}
.form-actions {
  display: flex;
  gap: 8px;
  padding-top: 4px;
}
.sv-checkpoint-row {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.sv-checkpoint-row .form-input {
  flex: 1;
}
.sv-checkpoint-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.cp-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  background: var(--bg-input);
  border-radius: var(--radius-sm);
}
.cp-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.cp-tag {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}
.cp-time {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}
.cp-actions {
  display: flex;
  gap: 4px;
}
.sv-empty {
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
