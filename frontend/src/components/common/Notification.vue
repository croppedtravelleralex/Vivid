<template>
  <teleport to="body">
    <transition name="notif-slide">
      <div v-if="visible" class="notification" :class="typeClass">
        <span class="notif-icon">{{ iconMap[type] }}</span>
        <span class="notif-text">{{ message }}</span>
        <button class="notif-close" @click="close">&times;</button>
      </div>
    </transition>
  </teleport>
</template>

<script setup>
import { ref, computed, watch } from 'vue'

const props = defineProps({
  message: { type: String, default: '' },
  type: { type: String, default: 'info' },
  duration: { type: Number, default: 4000 },
  show: { type: Boolean, default: false }
})

const emit = defineEmits(['update:show'])

const visible = ref(false)
let timer = null

const iconMap = {
  info: 'i',
  success: '✓',
  warning: '!',
  error: '✗'
}

const typeClass = computed(() => `notif-${props.type}`)

watch(() => props.show, (val) => {
  if (val) {
    show()
  } else {
    hide()
  }
})

function show() {
  if (timer) clearTimeout(timer)
  visible.value = true
  if (props.duration > 0) {
    timer = setTimeout(() => {
      close()
    }, props.duration)
  }
}

function hide() {
  visible.value = false
}

function close() {
  hide()
  emit('update:show', false)
}

function trigger(msg, type = 'info', duration = 4000) {
  if (timer) clearTimeout(timer)
  emit('update:show', true)
  visible.value = true
  if (duration > 0) {
    timer = setTimeout(() => close(), duration)
  }
}

</script>

<style scoped>
.notification {
  position: fixed;
  top: 56px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-radius: var(--radius-md);
  border: 1px solid;
  box-shadow: var(--shadow-md);
  z-index: var(--z-notification);
  max-width: 380px;
  font-size: 13px;
}
.notif-icon {
  font-weight: 700;
  font-size: 14px;
}
.notif-text {
  flex: 1;
}
.notif-close {
  background: none;
  border: none;
  color: inherit;
  font-size: 18px;
  cursor: pointer;
  opacity: 0.6;
  padding: 0;
  line-height: 1;
}
.notif-close:hover {
  opacity: 1;
}
.notif-info {
  background: var(--info-bg);
  border-color: var(--info);
  color: var(--info);
}
.notif-success {
  background: var(--success-bg);
  border-color: var(--success);
  color: var(--success);
}
.notif-warning {
  background: var(--warning-bg);
  border-color: var(--warning);
  color: var(--warning);
}
.notif-error {
  background: var(--danger-bg);
  border-color: var(--danger);
  color: var(--danger);
}
.notif-slide-enter-active,
.notif-slide-leave-active {
  transition: all 0.3s ease;
}
.notif-slide-enter-from {
  transform: translateX(100%);
  opacity: 0;
}
.notif-slide-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
