import api from './index.js'

export function start() {
  return api.post('/simulation/start').then(r => r.data)
}

export function pause() {
  return api.post('/simulation/pause').then(r => r.data)
}

export function setSpeed(speed) {
  return api.post('/simulation/speed', { speed }).then(r => r.data)
}

export function step() {
  return api.post('/simulation/step').then(r => r.data)
}

export function stop() {
  return api.post('/simulation/stop').then(r => r.data)
}

export function getStatus() {
  return api.get('/simulation/status').then(r => r.data)
}

export function getStats() {
  return api.get('/simulation/stats').then(r => r.data)
}
