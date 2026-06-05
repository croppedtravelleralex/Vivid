<template>
  <teleport to="body">
    <div v-if="visible" class="confirm-overlay" @click.self="handleCancel">
      <div class="confirm-dialog">
        <div class="confirm-header">
          <h3 class="confirm-title">{{ title }}</h3>
        </div>
        <div class="confirm-body">
          <p>{{ message }}</p>
        </div>
        <div class="confirm-footer">
          <button class="btn" @click="handleCancel">{{ cancelText }}</button>
          <button class="btn" :class="confirmClass" @click="handleConfirm">{{ confirmText }}</button>
        </div>
      </div>
    </div>
  </teleport>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  visible: { type: Boolean, default: false },
  title: { type: String, default: 'Confirm' },
  message: { type: String, default: 'Are you sure?' },
  confirmText: { type: String, default: 'Confirm' },
  cancelText: { type: String, default: 'Cancel' },
  danger: { type: Boolean, default: false }
})

const emit = defineEmits(['confirm', 'cancel', 'update:visible'])

const confirmClass = computed(() => props.danger ? 'btn-danger' : 'btn-primary')

function handleConfirm() {
  emit('confirm')
  emit('update:visible', false)
}

function handleCancel() {
  emit('cancel')
  emit('update:visible', false)
}
</script>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}
.confirm-dialog {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 24px;
  min-width: 340px;
  max-width: 460px;
  box-shadow: var(--shadow-lg);
}
.confirm-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}
.confirm-body {
  margin: 16px 0;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.5;
}
.confirm-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
