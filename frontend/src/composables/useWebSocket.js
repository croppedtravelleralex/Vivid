import { ref, onMounted, onUnmounted, shallowRef, readonly } from 'vue'

const RECONNECT_DELAY = 3000
const WS_URL = `${location.protocol === 'https:' ? 'wss:' : 'ws:'}//${location.host}/api/v1/ws`

export function useWebSocket() {
  const connected = ref(false)
  const lastEvent = ref(null)
  const events = shallowRef([])
  const listeners = new Map()
  const MAX_EVENTS = 500

  let ws = null
  let reconnectTimer = null
  let isDestroyed = false

  function connect() {
    if (isDestroyed) return
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return

    try {
      ws = new WebSocket(WS_URL)
    } catch (err) {
      console.error('[WS] Connection error:', err)
      scheduleReconnect()
      return
    }

    ws.onopen = () => {
      connected.value = true
      if (reconnectTimer) {
        clearTimeout(reconnectTimer)
        reconnectTimer = null
      }
    }

    ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data)
        lastEvent.value = data
        const arr = events.value
        if (arr.length >= MAX_EVENTS) {
          arr.splice(0, arr.length - MAX_EVENTS + 1)
        }
        events.value = [...arr, data]

        const type = data.type || data.event || 'unknown'
        if (listeners.has(type)) {
          listeners.get(type).forEach(fn => fn(data))
        }
        if (listeners.has('*')) {
          listeners.get('*').forEach(fn => fn(data))
        }
      } catch (err) {
        console.warn('[WS] Parse error:', err)
      }
    }

    ws.onclose = () => {
      connected.value = false
      ws = null
      if (!isDestroyed) scheduleReconnect()
    }

    ws.onerror = () => {
      // onclose will fire after this
    }
  }

  function scheduleReconnect() {
    if (isDestroyed || reconnectTimer) return
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null
      connect()
    }, RECONNECT_DELAY)
  }

  function onMessage(type, fn) {
    if (!listeners.has(type)) listeners.set(type, new Set())
    listeners.get(type).add(fn)
    const unsubscribe = () => {
      const set = listeners.get(type)
      if (set) {
        set.delete(fn)
        if (set.size === 0) listeners.delete(type)
      }
    }
    return unsubscribe
  }

  function send(data) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(typeof data === 'string' ? data : JSON.stringify(data))
    }
  }

  function disconnect() {
    isDestroyed = true
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    if (ws) {
      ws.onclose = null
      ws.onerror = null
      ws.onmessage = null
      ws.close()
      ws = null
    }
    connected.value = false
    listeners.clear()
  }

  onMounted(() => {
    isDestroyed = false
    connect()
  })

  onUnmounted(() => {
    disconnect()
  })

  return {
    connected: readonly(connected),
    lastEvent: readonly(lastEvent),
    events: readonly(events),
    onMessage,
    send,
    disconnect
  }
}
