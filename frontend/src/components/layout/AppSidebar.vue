<template>
  <aside class="app-sidebar">
    <nav class="sidebar-nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: isActive(item.path) }"
      >
        <span class="nav-icon">{{ item.icon }}</span>
        <span class="nav-label">{{ item.label }}</span>
      </router-link>
    </nav>
    <div class="sidebar-footer">
      <div class="version-text">v0.1.0</div>
    </div>
  </aside>
</template>

<script setup>
import { useRoute } from 'vue-router'

const route = useRoute()

const navItems = [
  { path: '/', icon: '◈', label: 'Dashboard' },
  { path: '/characters', icon: '◆', label: 'Characters' },
  { path: '/timeline', icon: '◈', label: 'Timeline' },
  { path: '/graph', icon: '⬡', label: 'Graph' },
  { path: '/world', icon: '◐', label: 'World' },
  { path: '/settings', icon: '⚙', label: 'Settings' }
]

function isActive(path) {
  if (path === '/') return route.path === '/'
  return route.path.startsWith(path)
}
</script>

<style scoped>
.app-sidebar {
  grid-area: sidebar;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  border-right: 1px solid var(--border);
  overflow-y: auto;
  z-index: var(--z-sidebar);
}
.sidebar-nav {
  display: flex;
  flex-direction: column;
  padding: 12px 8px;
  gap: 2px;
  flex: 1;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 14px;
  transition: all var(--transition-fast);
  user-select: none;
}
.nav-item:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}
.nav-item.active {
  background: var(--accent-muted);
  color: var(--accent);
  font-weight: 500;
}
.nav-icon {
  width: 20px;
  text-align: center;
  font-size: 15px;
  opacity: 0.8;
}
.nav-label {
  font-size: 13px;
}
.sidebar-footer {
  padding: 10px 16px;
  border-top: 1px solid var(--border);
}
.version-text {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
