import api from './index.js'

export function getCharacters() {
  return api.get('/characters').then(r => r.data)
}

export function getCharacter(id) {
  return api.get(`/characters/${id}`).then(r => r.data)
}

export function getCharacterMemory(id) {
  return api.get(`/characters/${id}/memory`).then(r => r.data)
}

export function getCharacterRelationships(id) {
  return api.get(`/characters/${id}/relationships`).then(r => r.data)
}
