<template>
  <div class="json-editor">
    <textarea
      ref="textareaRef"
      :value="modelValue"
      :rows="rows"
      :disabled="disabled"
      class="json-textarea"
      :class="{ 'has-error': localError }"
      @input="handleInput"
      @keydown.tab.prevent="handleTab"
      spellcheck="false"
    ></textarea>
    <div class="json-footer">
      <span v-if="localError" class="json-error">{{ localError }}</span>
      <span v-else-if="modelValue" class="json-valid">Valid JSON</span>
      <span v-else class="json-empty">Empty</span>
      <button v-if="modelValue" class="btn btn-sm btn-icon" @click="format" title="Format JSON">{} fmt</button>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  modelValue: { type: String, default: '' },
  rows: { type: Number, default: 8 },
  disabled: { type: Boolean, default: false }
})

const emit = defineEmits(['update:modelValue'])

const textareaRef = ref(null)
const localError = ref(null)

watch(() => props.modelValue, () => {
  validate(props.modelValue)
})

function validate(value) {
  if (!value) {
    localError.value = null
    return
  }
  try {
    JSON.parse(value)
    localError.value = null
  } catch (err) {
    localError.value = err.message
  }
}

function handleInput(e) {
  const val = e.target.value
  emit('update:modelValue', val)
}

function handleTab(e) {
  const ta = e.target
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const val = ta.value
  ta.value = val.substring(0, start) + '  ' + val.substring(end)
  ta.selectionStart = ta.selectionEnd = start + 2
}

function format() {
  if (!props.modelValue) return
  try {
    const parsed = JSON.parse(props.modelValue)
    const formatted = JSON.stringify(parsed, null, 2)
    emit('update:modelValue', formatted)
    localError.value = null
  } catch (err) {
    localError.value = err.message
  }
}

function validateNow() {
  return validate(props.modelValue)
}

defineExpose({ validate: validateNow, textareaRef })
</script>

<style scoped>
.json-editor {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.json-textarea {
  width: 100%;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
  padding: 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  resize: vertical;
  outline: none;
  transition: border-color var(--transition-fast);
}
.json-textarea:focus {
  border-color: var(--border-focus);
}
.json-textarea.has-error {
  border-color: var(--danger);
}
.json-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.json-error {
  font-size: 11px;
  color: var(--danger);
  font-family: var(--font-mono);
}
.json-valid {
  font-size: 11px;
  color: var(--success);
}
.json-empty {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
