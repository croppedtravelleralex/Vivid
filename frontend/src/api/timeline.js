import api from './index.js'

export function getTimeline() {
  return api.get('/timeline').then(r => r.data)
}

export function getTimelineEvents(params = {}) {
  return api.get('/timeline/events', { params }).then(r => r.data)
}

export function getCheckpoints() {
  return api.get('/timeline/checkpoints').then(r => r.data)
}

export function saveCheckpoint(tag) {
  return api.post('/timeline/checkpoints', { tag }).then(r => r.data)
}

export function loadCheckpoint(tag) {
  return api.post('/timeline/checkpoints/load', { tag }).then(r => r.data)
}
