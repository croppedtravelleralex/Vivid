import api from './index.js'

export function getWorld() {
  return api.get('/world').then(r => r.data)
}

export function getEnvironment() {
  return api.get('/world/environment').then(r => r.data)
}

export function getLocations() {
  return api.get('/world/locations').then(r => r.data)
}
