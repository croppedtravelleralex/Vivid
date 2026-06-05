import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('../views/Dashboard.vue')
  },
  {
    path: '/characters',
    name: 'Characters',
    component: () => import('../views/CharacterView.vue')
  },
  {
    path: '/character/:id',
    name: 'CharacterDetail',
    component: () => import('../views/CharacterView.vue')
  },
  {
    path: '/timeline',
    name: 'Timeline',
    component: () => import('../views/TimelineView.vue')
  },
  {
    path: '/graph',
    name: 'Graph',
    component: () => import('../views/GraphView.vue')
  },
  {
    path: '/world',
    name: 'World',
    component: () => import('../views/WorldView.vue')
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../views/SettingsView.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
