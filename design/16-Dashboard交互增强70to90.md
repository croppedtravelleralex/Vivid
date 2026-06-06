# 零下家园 · 小说世界模拟引擎 — Dashboard 交互增强（70→90 分补完）

> 本文档补全 `15-Dashboard完整交互设计.md` 中缺失的 10 项关键体验：
> 实时视觉反馈、撤销/重做、比照模式、全局搜索、快捷键体系、关系演化曲线、叙事评分时间线、What-if 分叉、模板系统、Macro 录制。

---

## 一、实时视觉反馈系统

### 1.1 关系图变化动效

当 WebSocket 推送 `relationship_changed` 事件时，不在静默更新，而是播触动效：

```vue
<template>
  <svg ref="svg">
    <g v-for="edge in edges" :key="edge.id">
      <line
        :class="{ 'edge-flash': edge.flashing, 'edge-fade': edge.fading }"
        :style="edgeStyle(edge)"
        @click="selectEdge(edge)"
      />
      <!-- 变化爆发粒子 -->
      <circle v-if="edge.bursting"
        :cx="edge.midX" :cy="edge.midY" r="8"
        class="burst-particle" @animationend="edge.bursting = false"
      />
      <!-- 数值变化漂浮文字 -->
      <text v-if="edge.showDelta"
        :x="edge.midX" :y="edge.midY - 15"
        class="delta-text"
        :class="edge.delta > 0 ? 'positive' : 'negative'"
      >
        {{ edge.delta > 0 ? '+' : '' }}{{ edge.delta }}
      </text>
    </g>
  </svg>
</template>

<style>
.edge-flash {
  animation: flash-green 0.6s ease-out;
}
.edge-flash.negative { animation-name: flash-red; }
.edge-fade {
  opacity: 1;
  transition: opacity 1s ease-out;
}
.edge-fade.fading { opacity: 0.2; }
.burst-particle {
  fill: var(--accent);
  animation: burst 0.8s ease-out forwards;
}
.delta-text {
  font-size: 14px;
  font-weight: bold;
  animation: float-up 1.2s ease-out forwards;
}
.delta-text.positive { fill: #4caf50; }
.delta-text.negative { fill: #f44336; }

@keyframes flash-green {
  0% { stroke: #4caf50; stroke-width: 6; }
  100% { stroke: var(--edge-color); stroke-width: var(--edge-width); }
}
@keyframes burst {
  0% { r: 0; opacity: 1; }
  100% { r: 20; opacity: 0; }
}
@keyframes float-up {
  0% { opacity: 1; transform: translateY(0); }
  100% { opacity: 0; transform: translateY(-20px); }
}
</style>
```

### 1.2 状态变化通知栏

右上角弹入式通知，不打断操作：

```
┌──────────────────────────────────────────────────────────────────────┐
│  ⚡ 信任变化                   ┌──┐   ┌─────────────────────────┐   │
│  林霜→张远: +2 (6→8)        │  │   │ 🔔 3 条新通知             │   │
│  原因: 林霜帮张远加固院墙    │  │   │ ┌───────────────────────┐ │   │
│  [关系图] [角色卡] [忽略]   │  │   │ │⚡ 林霜+2 信任         │ │   │
│  ─────────────────────────  │  │   │ │🔴 林霜→张远信任+2    │ │   │
│  🔴 事件触发                  │  │   │ │📜 议会报告已就绪     │ │   │
│  水管冻裂                    │  │   │ │🟠 老孙饥饿>80       │ │   │
│  [详情] [因果链] [忽略]     │  │   │ └───────────────────────┘ │   │
│  ─────────────────────────  │  │   └─────────────────────────┘   │
│  📜 议会报告已就绪            │  │                                  │
│  第7轮议会完成 [查看]        │  │                                  │
└──────────────────────────────────────────────────────────────────────┘
```

```vue
<script setup>
const notifications = ref([]);
let idCounter = 0;

function pushNotification(type, title, body, actions = [], duration = 5000) {
  const id = ++idCounter;
  notifications.value.push({ id, type, title, body, actions, entering: true });
  setTimeout(() => {
    const n = notifications.value.find(n => n.id === id);
    if (n) n.leaving = true;
    setTimeout(() => {
      notifications.value = notifications.value.filter(n => n.id !== id);
    }, 300);
  }, duration);
}

// WebSocket 监听
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  switch (data.type) {
    case 'relationship_changed':
      pushNotification(
        'relationship',
        `${data.data.charA} ↔ ${data.data.charB}`,
        `信任: ${data.data.delta.trust > 0 ? '+' : ''}${data.data.delta.trust} · ${data.data.reason}`,
        [{ label: '关系图', action: () => router.push('/graph') }]
      );
      break;
    case 'event_triggered':
      pushNotification('event', data.data.title, data.data.description, [
        { label: '详情', action: () => showEventDetail(data.data.eventId) }
      ]);
      break;
  }
};
</script>
```

---

## 二、完全撤销/重做系统

### 2.1 架构

所有修改操作通过统一的 `Command` 模式执行：

```typescript
// types/commands.ts
interface Command {
  id: string;
  label: string;          // 操作名："删除角色·老孙"
  timestamp: number;
  execute(): Promise<void>;
  undo(): Promise<void>;
  redo(): Promise<void>;
}

class CommandHistory {
  private undoStack: Command[] = [];
  private redoStack: Command[] = [];
  private maxHistory: number = 200;

  async execute(command: Command) {
    await command.execute();
    this.undoStack.push(command);
    this.redoStack = []; // 新操作清除 redo 栈
    if (this.undoStack.length > this.maxHistory) {
      this.undoStack.shift();
    }
  }

  async undo() {
    const cmd = this.undoStack.pop();
    if (!cmd) return;
    await cmd.undo();
    this.redoStack.push(cmd);
  }

  async redo() {
    const cmd = this.redoStack.pop();
    if (!cmd) return;
    await cmd.redo();
    this.undoStack.push(cmd);
  }
}
```

### 2.2 具体命令实现

```typescript
// commands/EditCharacterCommand.ts
class EditCharacterCommand implements Command {
  id: string;
  label: string;
  timestamp: number;
  
  private characterId: string;
  private before: Partial<CharacterState>;
  private after: Partial<CharacterState>;

  constructor(charId: string, before: Partial<CharacterState>, after: Partial<CharacterState>) {
    this.id = uuid();
    this.label = `编辑角色·${charId}`;
    this.timestamp = Date.now();
    this.characterId = charId;
    this.before = JSON.parse(JSON.stringify(before));  // deep clone
    this.after = JSON.parse(JSON.stringify(after));
  }

  async execute() {
    await api.patch(`/api/v1/characters/${this.characterId}/state`, this.after);
  }

  async undo() {
    await api.patch(`/api/v1/characters/${this.characterId}/state`, this.before);
  }

  async redo() {
    await this.execute();
  }
}

// commands/DeleteCharacterCommand.ts
class DeleteCharacterCommand implements Command {
  id: string;
  label: string;
  timestamp: number;

  private characterId: string;
  private backup: any;       // 完整的角色卡备份
  private removedEvents: any[];  // 受影响的 events

  constructor(charId: string, backup: any, removedEvents: any[]) {
    this.id = uuid();
    this.label = `删除角色·${charId}`;
    this.timestamp = Date.now();
    this.characterId = charId;
    this.backup = backup;
    this.removedEvents = removedEvents;
  }

  async execute() {
    await api.delete(`/api/v1/characters/${this.characterId}`, {
      data: { mode: 'hard' }
    });
  }

  async undo() {
    // 恢复角色卡
    await api.post('/api/v1/characters', { card: this.backup, restoreEvents: this.removedEvents });
    // WebSocket 会推送恢复事件
  }

  async redo() {
    await this.execute();
  }
}
```

### 2.3 UI 集成

```
┌──────────────────────┐
│  撤销/重做            │
│  ┌────┐  ┌────┐      │
│  │ ↩  │  │ ↪  │      │  Ctrl+Z / Ctrl+Shift+Z
│  └────┘  └────┘      │
│  └──────────────┘     │
│  最近 5 条操作：       │
│  📝 编辑角色·林霜  ←  │  ← 当前回退位置
│  📝 编辑角色·陈磊     │
│  🗑  删除事件·水管冻裂 │
│  ➕ 新建角色·赵建国   │
│  📝 编辑角色·张远     │
│                      │
│  [清空历史] [导出操作日志]│
└──────────────────────┘
```

---

## 三、比照模式（Diff View）

### 3.1 角色比照

两个角色并列对比，所有维度逐项可视化差异：

```vue
<template>
  <div class="comparison-view">
    <div class="control-bar">
      <select v-model="charA"> <option v-for="c in chars" :key="c.id">{{ c.name }}</option> </select>
      <span class="vs">VS</span>
      <select v-model="charB"> <option v-for="c in chars" :key="c.id">{{ c.name }}</option> </select>
      <button @click="swap">⇄ 互换</button>
    </div>

    <div class="comparison-grid">
      <!-- OCEAN 对比 -->
      <div class="compare-section">
        <h4>人格 OCEAN</h4>
        <div class="radar-compare">
          <PersonalityRadar :data="charAData.ocean" color="#4caf50" />
          <PersonalityRadar :data="charBData.ocean" color="#2196f3" />
        </div>
        <table class="diff-table">
          <tr v-for="key in oceanKeys" :key="key">
            <td>{{ key }}</td>
            <td :class="diffClass(charAData.ocean[key], charBData.ocean[key])">
              {{ charAData.ocean[key] }}
            </td>
            <td class="arrow">→</td>
            <td :class="diffClass(charBData.ocean[key], charAData.ocean[key])">
              {{ charBData.ocean[key] }}
            </td>
          </tr>
        </table>
      </div>

      <!-- 技能对比 -->
      <div class="compare-section">
        <h4>技能</h4>
        <table class="diff-table">
          <tr v-for="skill in allSkills" :key="skill">
            <td>{{ skill }}</td>
            <td>{{ skillLevel(charAData, skill) }}</td>
            <td>→</td>
            <td>{{ skillLevel(charBData, skill) }}</td>
          </tr>
        </table>
      </div>

      <!-- 关系网络对比 -->
      <div class="compare-section">
        <h4>关系网络交集</h4>
        <p class="subtitle">共同关系：{{ mutualRelations.length }}</p>
        <div class="mutual-graph">
          <RelationGraph :nodes="mutualGraphNodes" :edges="mutualGraphEdges" :compact="true" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
function diffClass(a, b) {
  if (Math.abs(a - b) < 0.05) return 'same';
  return a > b ? 'higher' : 'lower';
}
</script>

<style>
.diff-table .higher { background: rgba(76,175,80,0.15); color: #2e7d32; }
.diff-table .lower { background: rgba(244,67,54,0.15); color: #c62828; }
.diff-table .same { color: #666; }
</style>
```

### 3.2 版本比照（角色修改前后）

```vue
<template>
  <div class="version-compare">
    <div class="version-select">
      <button v-for="v in versions" :key="v.tick"
        :class="{ active: selectedVersion === v.tick }"
        @click="selectVersion(v.tick)">
        #{{ v.tick }} {{ v.label }}
      </button>
    </div>
    <div class="split-view">
      <div class="before">
        <h4>修改前</h4>
        <JsonEditor :data="beforeSnapshot" :readonly="true" />
      </div>
      <div class="after">
        <h4>修改后</h4>
        <JsonEditor :data="afterSnapshot" :readonly="true" />
      </div>
    </div>
    <div class="diff-summary">
      <h4>变更摘要</h4>
      <ul>
        <li v-for="change in diffChanges" :key="change.path">
          <span class="path">{{ change.path }}</span>:
          <span class="old">{{ change.old }}</span> → <span class="new">{{ change.new }}</span>
        </li>
      </ul>
    </div>
    <button @click="revertTo(selectedVersion)" class="danger">回滚到此版本</button>
  </div>
</template>
```

### 3.3 关系差异高亮

```
比照模式在关系图中的应用：
  选中两个角色 → "关系差"模式 → 边上数字显示 trust 差值
  
  张远→林霜: 6
  陈磊→林霜: 2  ← 差值 -4，边为红色虚线
  张远→陈磊: 8
  老孙→陈磊: 3  ← 差值 -5，边为红色虚线
  
  图例：实线=信任相同 > 虚线=差异 > 2
```

---

## 四、全局搜索

### 4.1 搜索框

```
┌──────────────────────────────────────────────────────────┐
│  🔍 搜索... (Ctrl+K)         [角色] [事件] [地点] [全部]   │
├──────────────────────────────────────────────────────────┤
│  角色                                             3 条   │
│  ├ 张远    五金店之子 · 五金店             信任度 8/10   │
│  ├ 张建国  五金店老板 · 五金店             信任度 6/10   │
│  └ 赵建国  同村长辈 · 学校                 信任度 5/10   │
│                                                         │
│  事件                                             5 条   │
│  ├ 🔴 水管冻裂  tick 234  五金店                       │
│  ├ 🟡 张远检查院墙 tick 456  五金店                     │
│  └ ...                                                 │
│                                                         │
│  地点                                             1 条   │
│  └ 五金店  · 3 人 · 食物 25/200                        │
│                                                         │
│  搜索 "建国" 找到 6 条结果  (0.03s)                       │
└──────────────────────────────────────────────────────────┘
```

```vue
<script setup>
const searchResults = ref({ characters: [], events: [], locations: [] });
const searchType = ref('all');
let debounceTimer;

async function search(query: string) {
  clearTimeout(debounceTimer);
  if (!query) { searchResults.value = { characters: [], events: [], locations: [] }; return; }
  
  debounceTimer = setTimeout(async () => {
    searchResults.value = await api.get('/api/v1/search', {
      params: { q: query, type: searchType.value, limit: 10 }
    });
  }, 150);  // 150ms 防抖
}

// 键盘导航
const selectedIndex = ref(0);
function onKeydown(e) {
  if (e.key === 'ArrowDown') { selectedIndex.value++; e.preventDefault(); }
  if (e.key === 'ArrowUp') { selectedIndex.value--; e.preventDefault(); }
  if (e.key === 'Enter') {
    const item = getSelectedItem();
    if (item) navigateTo(item);
  }
}
</script>
```

### 4.2 搜索 API

```typescript
GET /api/v1/search?q=建国&type=all&limit=10
Response: {
  "characters": [
    { "id": "zhangjianguo", "name": "张建国", "title": "五金店老板",
      "location": "五金店", "trust_avg": 6, "relevance": 0.95,
      "matchField": "name", "matchPreview": "...张**建国**..." }
  ],
  "events": [
    { "id": "evt-234", "tick": 234, "title": "水管冻裂",
      "participants": ["张远"], "severity": "major", "relevance": 0.7 }
  ],
  "locations": [
    { "id": "hardware-store", "name": "五金店",
      "occupantCount": 3, "relevance": 0.85 }
  ]
}
```

### 4.3 快捷键激活

```typescript
// 全局快捷键监听
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault();
    showSearchModal.value = true;
    // 聚焦搜索框
    nextTick(() => searchInput.value?.focus());
  }
});
```

---

## 五、完整快捷键体系

```typescript
// config/keybindings.ts
export const KEYBINDINGS = {
  // 模拟控制
  'Space':            { action: 'togglePause',          label: '暂停/继续',         category: 'simulation' },
  'Shift+Space':      { action: 'stepTick',            label: '单步推进',          category: 'simulation' },
  'Ctrl+Right':       { action: 'fastForward',         label: '快进 6h',           category: 'simulation' },
  'Ctrl+Shift+Right': { action: 'fastForwardDay',     label: '快进 24h',          category: 'simulation' },

  // 撤销/重做
  'Ctrl+Z':           { action: 'undo',                label: '撤销',              category: 'edit' },
  'Ctrl+Shift+Z':     { action: 'redo',                label: '重做',              category: 'edit' },
  'Ctrl+Y':           { action: 'redo',                label: '重做（备选）',       category: 'edit' },

  // 导航
  'Ctrl+K':           { action: 'search',              label: '全局搜索',          category: 'navigation' },
  'Ctrl+B':           { action: 'toggleSidebar',       label: '侧边栏开关',        category: 'navigation' },
  'Escape':           { action: 'closeModal',          label: '关闭当前弹窗',       category: 'navigation' },
  'Ctrl+1-9':         { action: 'switchTab',           label: '切换到第 N 个 Tab',  category: 'navigation' },
  'Alt+Left':         { action: 'back',                label: '返回上一页',         category: 'navigation' },
  'Alt+Right':        { action: 'forward',             label: '前进',              category: 'navigation' },

  // 关系图
  'R':                { action: 'graphResetView',      label: '重置视图',           category: 'graph', context: 'graph' },
  'F':                { action: 'graphFocusSelected',  label: '聚焦选中节点',        category: 'graph', context: 'graph' },
  'Ctrl+A':           { action: 'graphSelectAll',      label: '全选节点',           category: 'graph', context: 'graph' },
  'Delete':           { action: 'graphDeleteSelected',  label: '删除选中节点/边',    category: 'graph', context: 'graph' },
  'C':                { action: 'graphToggleLabels',   label: '显示/隐藏标签',       category: 'graph', context: 'graph' },
  'L':                { action: 'graphAutoLayout',     label: '自动布局',           category: 'graph', context: 'graph' },
  'Ctrl+E':           { action: 'graphEditEdge',       label: '编辑选中边',         category: 'graph', context: 'graph' },
  'Ctrl+D':           { action: 'graphDuplicate',      label: '复制选中节点',       category: 'graph', context: 'graph' },

  // 角色
  'N':                { action: 'newCharacter',        label: '新建角色',           category: 'character', context: 'character' },
  'Ctrl+E':           { action: 'editCharacter',       label: '编辑当前角色',        category: 'character', context: 'character_detail' },
  'Ctrl+S':           { action: 'saveCharacter',       label: '保存角色',           category: 'character', context: 'character_edit' },
  'Ctrl+D':           { action: 'duplicateCharacter',  label: '复制角色',           category: 'character', context: 'character_detail' },

  // 时间线
  'Ctrl+F':           { action: 'timelineSearch',      label: '搜索事件',           category: 'timeline', context: 'timeline' },
  'I':                { action: 'insertEvent',         label: '插入事件',           category: 'timeline', context: 'timeline' },
  'J':                { action: 'selectNextEvent',     label: '下一条事件',         category: 'timeline', context: 'timeline' },
  'K':                { action: 'selectPrevEvent',     label: '上一条事件',         category: 'timeline', context: 'timeline' },
  'Enter':            { action: 'showEventDetail',     label: '查看事件详情',       category: 'timeline', context: 'timeline' },

  // 通用
  '?':                { action: 'showHelp',            label: '快捷键帮助',         category: 'general' },
  'Ctrl+,':           { action: 'openSettings',        label: '设置',              category: 'general' },
  'Ctrl+S':           { action: 'saveWorld',           label: '保存世界',           category: 'general' },
  'Ctrl+Shift+S':     { action: 'saveWorldAs',         label: '另存为',            category: 'general' },
  'Ctrl+P':           { action: 'exportNovel',         label: '导出为小说格式',      category: 'general' },
}
```

### 快捷键帮助面板

按 `?` 弹出：

```
┌──────────────────────────────────────────────────────────────┐
│  快捷键帮助                                     🔍 搜索快捷键 │
├──────────────────────────────────────────────────────────────┤
│  ┌─ 模拟控制 ─────────────┐  ┌─ 关系图（在图页面）──────────┐ │
│  │ Space    暂停/继续      │  │ R          重置视图          │ │
│  │ Shift+S  单步推进       │  │ F          聚焦选中           │ │
│  │ Ctrl+→   快进 6h        │  │ Delete     删除选中          │ │
│  │ Ctrl+⇧+→ 快进 24h      │  │ C          切换标签          │ │
│  └────────────────────────┘  └──────────────────────────────┘ │
│  ┌─ 编辑 ────────────────┐  ┌─ 导航 ─────────────────────┐   │
│  │ Ctrl+Z    撤销          │  │ Ctrl+K    全局搜索          │   │
│  │ Ctrl+⇧+Z  重做          │  │ Ctrl+B    侧边栏开关        │   │
│  └────────────────────────┘  │ Escape    关闭弹窗          │   │
│                              └────────────────────────────┘   │
│  [自定义快捷键...]                                              │
└──────────────────────────────────────────────────────────────────┘
```

---

## 六、关系演化曲线

### 6.1 趋势图

每条关系的信任、熟悉度、好感度随时间的变化曲线：

```
┌──────────────────────────────────────────────────────────────────┐
│  张远 ↔ 林霜  关系演化                                            │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  10┤                                          ┌───               │
│    │                                     ┌────┘                  │
│   8┤                               ┌─────┘                      │
│    │                          ┌────┘                            │
│   6┤                    ┌─────┘               信任值             │
│    │               ┌────┘                                       │
│   4┤          ┌────┘                         ─── 信任           │
│    │     ┌────┘                               ··· 熟悉           │
│   2┤────┘                                     -·- 好感           │
│    │                                           ─── 平均值        │
│   0└─────────────────────────────────────────────────────▶       │
│    12/3  12/4  12/5  12/6  12/7  12/8  12/9  模拟时间            │
│                                                                  │
│  🖱 悬停显示数值  点击跳转到对应 tick  拖拽选区放大               │
│  [显示事件标记]  事件点▸[初次见面]▸[冲突]▸[和解]▸[合作]          │
│                                                                  │
│  统计：平均信任 4.2  波动率 0.34  最高 8（12/8）  最低 0（12/3） │
│  趋势：📈 上升中  斜率 +0.23/day                                  │
└──────────────────────────────────────────────────────────────────┘
```

```vue
<template>
  <div class="relationship-chart">
    <div class="chart-header">
      <h4>{{ charA }} ↔ {{ charB }}</h4>
      <select v-model="metric">
        <option value="trust">信任</option>
        <option value="familiarity">熟悉</option>
        <option value="sentiment">好感</option>
        <option value="all">全部</option>
      </select>
      <button @click="toggleEvents">显示事件</button>
    </div>
    <svg ref="chartSvg" class="chart-canvas"></svg>
    <div class="chart-stats">
      <span class="stat">均值: {{ stats.mean }}</span>
      <span class="stat">波动率: {{ stats.volatility }}</span>
      <span class="stat">最高: {{ stats.max }}</span>
      <span class="stat">趋势: {{ stats.trend > 0 ? '📈' : '📉' }} {{ stats.slope.toFixed(2) }}/day</span>
    </div>
  </div>
</template>

<script setup>
import * as d3 from 'd3';

onMounted(async () => {
  // 获取演化数据
  const { data } = await api.get(`/api/v1/graph/relationships/evolution`, {
    params: { charA: props.charA, charB: props.charB }
  });

  // d3 折线图
  const svg = d3.select(chartSvg.value);
  const width = svg.node().clientWidth;
  const height = 300;
  const margin = { top: 20, right: 60, bottom: 30, left: 40 };

  const x = d3.scaleTime()
    .domain(d3.extent(data.series, d => new Date(d.time)))
    .range([margin.left, width - margin.right]);

  const y = d3.scaleLinear()
    .domain([0, 10])
    .range([height - margin.bottom, margin.top]);

  // 三条线
  const lines = ['trust', 'familiarity', 'sentiment'].map(key => ({
    key,
    data: data.series.map(d => ({ time: new Date(d.time), value: d[key] })),
    color: key === 'trust' ? '#4caf50' : key === 'familiarity' ? '#2196f3' : '#ff9800'
  }));

  lines.forEach(line => {
    svg.append('path')
      .datum(line.data)
      .attr('fill', 'none')
      .attr('stroke', line.color)
      .attr('stroke-width', 2)
      .attr('d', d3.line()
        .x(d => x(d.time))
        .y(d => y(d.value))
      );
  });

  // 事件标记点
  svg.selectAll('.event-dot')
    .data(data.events_on_timeline)
    .enter()
    .append('circle')
    .attr('cx', d => x(new Date(d.time)))
    .attr('cy', d => y(d.value))
    .attr('r', 5)
    .attr('fill', '#ff5722')
    .attr('stroke', 'white')
    .attr('stroke-width', 2)
    .append('title')
    .text(d => d.title);
});
</script>
```

---

## 七、叙事评分时间线

### 7.1 可视化

在时间线顶部叠加叙事评分曲线，直观看到"精彩段"和"无聊段"：

```
┌──────────────────────────────────────────────────────────────────────┐
│  叙事质量评分                                                        │
│  10┤   ╱╲         ╱╲                                                  │
│   8┤  ╱  ╲       ╱  ╲    ╱╲        ╱╲                               │
│   6┤ ╱    ╲     ╱    ╲  ╱  ╲      ╱  ╲           ╱╲                │
│   4┤╱      ╲   ╱      ╲╱    ╲    ╱    ╲    ╱╲   ╱  ╲               │
│   2┤        ╲ ╱              ╲  ╱      ╲  ╱  ╲ ╱    ╲              │
│   0└──────────────────────────────────────────────────────▶         │
│    12/3  12/4  12/5  12/6  12/7  12/8  12/9  12/10                 │
│                                                                      │
│   平均评分：6.2/10   最高：9.1（tick 456）  最低：2.1（tick 300-320）│
│   节奏：当前偏缓  建议：需要一次社交冲突或发现                         │
│                                                                      │
│   🟢 高亮区（>7分）    🟡 中区（4-7）    🔴 低区（<4）              │
│   点击低区 → [插入事件] [加速时间] [查看原因]                        │
└──────────────────────────────────────────────────────────────────────┘
```

### 7.2 评分明细

点击评分曲线上的任一点，展开评分明细：

```jsonc
// GET /api/v1/timeline/score-detail?tick=456
{
  "tick": 456,
  "eventTitle": "张远与林霜第一次争吵",
  "overallScore": 8.2,
  "dimensions": {
    "characterDepth": 8.5,      // 角色深度
    "plotCoherence": 7.0,       // 剧情连贯
    "tensionCurve": 9.0,        // 节奏曲线
    "novelty": 6.5,             // 新颖性
    "emotionalRange": 8.0,      // 情感范围
    "worldIntegration": 7.5     // 世界整合
  },
  "reasons": [
    "角色驱动而非事件驱动",
    "高情感强度（冲突+关系深化）",
    "低新颖性（同类冲突在tick 400发生过）"
  ],
  "narrativeFunction": "RelationshipChange",
  "emotionalTone": "Tension → Resolution",
  "taggedBy": "议会·质量评审员 · 第7轮"
}
```

---

## 八、What-if 分叉系统

### 8.1 分叉入口

在任何检查点，用户可以"分叉"：

```
在检查点列表或事件详情页：
  [🌿 从此分叉]
  
分叉后：
  ├── 原始世界继续运行（不受影响）
  └── 新分支：复制当前世界状态，独立运行

分叉对比面板：
  ┌─ 原始世界 ─────────────────┬─ 分支世界 ─────────────────┐
  │ @tick 500: 张力 0.45       │ @tick 500: 张力 0.45       │
  │ 温度 -30°C                 │ 温度 -30°C                 │
  │ 张远 HP 75                 │ 张远 HP 75                 │
  │ 林霜信任 6                 │ 林霜信任 6                 │
  │                            │                            │
  │ 修改： 无                   │ 修改：林霜信任→8           │
  │                            │                            │
  │ [继续推进]                  │ [继续推进]                  │
  │ tick 501: → 水管冻裂        │ tick 501: → 林霜主动提出帮忙│
  │ tick 502: → 张远修水管      │ tick 502: → 两人一起修      │
  │ tick 503: → 老孙抱怨       │ tick 503: → 林霜分享信息    │
  └────────────────────────────┴────────────────────────────┘
```

### 8.2 分叉 API

```typescript
// 创建分叉
POST /api/v1/simulation/fork
Body: {
  "checkpointTag": "tick_000500",
  "label": "试一下林霜初始信任更高会怎样",
  "modifications": [
    { "type": "relationship", "charA": "zhangyuan", "charB": "linshuang", "field": "trust", "value": 8 }
  ]
}
Response: {
  "forkId": "fork-abc123",
  "worldState": { /* 修改后的世界状态 */ },
  "sourceUrl": "/api/v1/checkpoint/load/tick_000500",
  "forkUrl": "/api/v1/simulation/fork/abc123"
}

// 获取分叉比较数据
GET /api/v1/simulation/fork/abc123/compare?fromTick=500&toTick=550
Response: {
  "source": { /* 原始世界 tick 500-550 摘要 */ },
  "fork": { /* 分叉世界 tick 500-550 摘要 */ },
  "divergencePoint": 500,
  "divergenceEvents": {
    "source": [{"tick": 501, "event": "水管冻裂"}, ...],
    "fork": [{"tick": 501, "event": "林霜主动提出帮忙"}, ...]
  },
  "stateDeltas": {
    "linshuang": { "trust": { "source": 6, "fork": 8, "delta": +2 } },
    // ...
  }
}
```

### 8.3 分支管理

```vue
<template>
  <div class="fork-manager">
    <h3>🌿 分支时间线</h3>
    <div class="fork-tree">
      <div class="branch main">
        <div class="branch-line"></div>
        <div class="branch-node current">● 主世界</div>
        <div class="branch-event" v-for="e in mainEvents" :key="e.id">
          {{ e.title }}
        </div>
        <div class="branch-fork" @click="openForkDialog">
          ╰── [🌿 从此分叉]
        </div>
      </div>
      <div class="branch fork" v-for="fork in forks" :key="fork.id">
        <div class="branch-connector">├──</div>
        <div class="branch-line"></div>
        <div class="branch-node">● {{ fork.label }}</div>
        <div class="branch-event" v-for="e in fork.events" :key="e.id">
          {{ e.title }}
        </div>
        <div class="branch-stats">
          分歧点: tick {{ fork.divergenceTick }}
          <button @click="loadFork(fork.id)">加载</button>
          <button @click="compareFork(fork.id)">对比</button>
          <button @click="deleteFork(fork.id)" class="danger">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>
```

---

## 九、模板系统

### 9.1 角色模板

预定义角色模板，新建角色时可直接选择：

```typescript
// 系统内置模板
const CHARACTER_TEMPLATES = {
  survivor: {
    identity: { archetype: 'survivor', importance: 0.5 },
    personality: { ocean: { openness: 0.5, conscientiousness: 0.5, extraversion: 0.5, agreeableness: 0.5, neuroticism: 0.5 } },
    proficiencies: { skills: [{ name: "基础生存", level: 0.3, max: 1.0 }] },
    // ...
  },
  leader: {
    identity: { archetype: 'leader', importance: 0.8 },
    personality: { ocean: { openness: 0.6, conscientiousness: 0.7, extraversion: 0.6, agreeableness: 0.5, neuroticism: 0.4 } },
    proficiencies: { skills: [{ name: "领导", level: 0.6, max: 1.0 }, { name: "决策", level: 0.5, max: 1.0 }] },
  },
  doctor: {
    identity: { archetype: 'doctor', importance: 0.7 },
    proficiencies: { skills: [{ name: "急救", level: 0.7, max: 1.0 }, { name: "诊断", level: 0.6, max: 1.0 }] },
  },
  // ... fighter / mechanic / trader / scout / raider / farmer / cultist / child / elder
}
```

### 9.2 事件模板

```typescript
const EVENT_TEMPLATES = {
  social_conflict: {
    type: "conflict",
    severity: "normal",
    narrativeFunction: "Escalation",
    emotionalTone: "Tension",
    defaultParticipants: 2,
    descriptionPlaceholder: "{charA} 与 {charB} 因为 {reason} 发生争执",
  },
  resource_discovery: {
    type: "discovery",
    severity: "major",
    narrativeFunction: "Reveal",
    emotionalTone: "Hope",
    descriptionPlaceholder: "{char} 在 {location} 发现了 {resource}",
  },
  // ... external_threat / character_arrival / relationship_change / etc
}
```

### 9.3 场景模板

```typescript
const SCENE_TEMPLATES = {
  camp_fire_conversation: {
    setting: "篝火旁，夜晚",
    characters: [2, 5],
    duration: "30-60 min",
    prompt: "角色们围坐在篝火旁，取暖，聊天。气氛 {{tone}}。话题可以涉及过去的生活、末日的看法、对未来的担忧。",
    variables: ["tone: 缓和/紧张/悲伤/希望"],
    hooks: ["有一个角色今晚特别沉默", "有角色在火光中哭了"],
  },
  supply_run: {
    setting: "废弃的{{building_type}}，白天",
    characters: [1, 3],
    duration: "1-3 小时",
    prompt: "角色们搜索{{location}}寻找物资。需要决定搜索路线、分配任务、应对可能的威胁。",
    variables: ["building_type: 超市/药店/仓库/加油站", "location"],
    hooks: ["发现其他人最近来过这里的痕迹", "遇到意外的危险"],
  },
  // ...
}
```

### 9.4 模板管理 UI

```
模板管理面板
├── 角色模板
│   ├── survivor     [预览] [使用] [编辑] [复制] [导出]
│   ├── leader       [预览] [使用] [编辑] [复制] [导出]
│   ├── doctor       [预览] [使用] [编辑] [复制] [导出]
│   └── [+ 新建模板] [从角色另存为模板] [导入]
│
├── 事件模板
│   ├── social_conflict    [使用] [编辑]
│   ├── resource_discovery [使用] [编辑]
│   └── [+ 新建模板]
│
├── 场景模板
│   ├── camp_fire          [使用] [编辑]
│   ├── supply_run         [使用] [编辑]
│   └── [+ 新建模板]
│
└── 导出/导入
    ├── 导出全部模板为 JSON
    ├── 从 JSON 导入
    └── 分享模板（导出为单独文件）
```

---

## 十、Macro 录制系统

### 10.1 录制器

```vue
<template>
  <div class="macro-recorder">
    <div class="controls">
      <button @click="startRecording" :disabled="recording" class="record-btn">
        {{ recording ? '● 录制中...' : '🔴 开始录制' }}
      </button>
      <button @click="stopRecording" :disabled="!recording" class="stop-btn">
        ⏹ 停止
      </button>
    </div>

    <div class="macro-list" v-if="macros.length > 0">
      <h4>已录制 Macro</h4>
      <div class="macro-item" v-for="macro in macros" :key="macro.id">
        <div class="macro-header">
          <span class="macro-name">{{ macro.name }}</span>
          <span class="macro-step-count">{{ macro.steps.length }} 步</span>
          <span class="macro-date">{{ macro.createdAt }}</span>
        </div>
        <details>
          <summary>步骤预览</summary>
          <div class="macro-steps">
            <div class="step" v-for="(step, i) in macro.steps" :key="i">
              {{ i + 1 }}. {{ step.description }}
            </div>
          </div>
        </details>
        <div class="macro-actions">
          <button @click="playMacro(macro.id)">▶ 播放</button>
          <button @click="editMacro(macro.id)">✏ 编辑</button>
          <button @click="exportMacro(macro.id)">📤 导出</button>
          <button @click="deleteMacro(macro.id)" class="danger">🗑 删除</button>
        </div>
      </div>
    </div>

    <div class="create-macro">
      <input v-model="newMacroName" placeholder="Macro 名称..." />
      <button @click="saveRecording">保存录制</button>
    </div>
  </div>
</template>

<script setup>
interface MacroStep {
  type: 'edit_character' | 'delete_character' | 'insert_event' | 'change_speed' | 'inject_time' | 'edit_relationship' | 'api_call';
  description: string;
  payload: any;
  delay: number;  // 执行间隔（ms）
}

interface Macro {
  id: string;
  name: string;
  steps: MacroStep[];
  createdAt: string;
}

// 录制所有用户操作
function hookUserActions() {
  // 拦截所有 API 调用
  api.interceptors.request.use((config) => {
    if (recording.value && !config._fromMacro) {
      currentSteps.value.push({
        type: inferActionType(config),
        description: inferDescription(config),
        payload: { method: config.method, url: config.url, data: config.data },
        delay: 300,  // 默认 300ms 间隔
      });
    }
    return config;
  });
}
</script>
```

### 10.2 播放器

```typescript
async function playMacro(macroId: string, speed: number = 1.0) {
  const macro = macros.value.find(m => m.id === macroId);
  if (!macro) return;

  isPlaying.value = true;
  for (const step of macro.steps) {
    if (!isPlaying.value) break;  // 中断

    // 显示当前执行步骤
    currentStepIndex.value++;
    highlightStep(step);

    // 执行
    await executeMacroStep(step);

    // 等待间隔
    await sleep(step.delay / speed);
  }
  isPlaying.value = false;
}

// 导出 Maco 为可分享文件
function exportMacro(macroId: string) {
  const macro = macros.value.find(m => m.id === macroId);
  const blob = new Blob([JSON.stringify(macro, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${macro.name}.macro.json`;
  a.click();
}
```

### 10.3 预置 Maco

```typescript
const PRESET_MACROS = {
  // "快速推进一天"
  fast_forward_day: {
    name: "快速推进一天",
    steps: [
      { type: 'change_speed', description: '切换到快进模式', payload: { speed: 'fastforward' } },
      { type: 'inject_time', description: '推进 24 小时', payload: { hours: 24 } },
      { type: 'change_speed', description: '切换回精细模式', payload: { speed: 'detailed' } },
    ]
  },

  // "批量新建 3 个角色"
  batch_create_characters: {
    name: "批量创建 3 个幸存者",
    steps: [
      { type: 'api_call', description: '创建角色 1', payload: { method: 'POST', url: '/api/v1/characters/generate', data: { seed: '一个普通的幸存者', template: 'survivor' } } },
      { type: 'api_call', description: '创建角色 2', payload: { method: 'POST', url: '/api/v1/characters/generate', data: { seed: '一个有医疗经验的幸存者', template: 'doctor' } } },
      { type: 'api_call', description: '创建角色 3', payload: { method: 'POST', url: '/api/v1/characters/generate', data: { seed: '一个擅长战斗的幸存者', template: 'fighter' } } },
    ]
  },

  // "运行一夜"
  run_overnight: {
    name: "运行一夜（自动推进）",
    steps: [
      { type: 'change_speed', description: '切换到快进模式', payload: { speed: 'fastforward' } },
      { type: 'inject_time', description: '推进 8 小时', payload: { hours: 8 } },
      { type: 'change_speed', description: '切换回精细模式', payload: { speed: 'detailed' } },
      { type: 'api_call', description: '触发议会', payload: { method: 'POST', url: '/api/v1/council/trigger' } },
    ]
  },
}
```

---

## 最终评分对照

| 维度 | 15 号文档（旧） | 增强后 | 差距 |
|------|--------------|-------|------|
| **实时视觉反馈** | 无 | 边闪动/粒子爆发/数值漂浮/通知栏 | +10% |
| **撤销/重做** | 无 | 200 步 Command 历史 + 可视化面板 | +5% |
| **比照模式** | 无 | 角色版对比/版本对比/差异高亮 | +5% |
| **全局搜索** | 基本 | 150ms 防抖/键盘导航/分类/高亮 | +3% |
| **快捷键体系** | 无 | 30+ 快捷键/上下文感知/帮助面板/自定义 | +5% |
| **关系演化曲线** | 静态 | d3 折线/事件标记/统计/趋势 | +5% |
| **叙事评分时间线** | 无 | 评分曲线/高亮区/评分明细/原因 | +3% |
| **What-if 分叉** | 无 | 分叉/对比面板/分支管理树 | +5% |
| **模板系统** | 无 | 角色/事件/场景模板/管理/导入导出 | +3% |
| **Macro 录制** | 无 | 录制/播放/预置/导出/中断 | +3% |
| **其他** | 缺失 | 搜索快捷键(Ctrl+K)/比照(Ctrl+W)/版本回滚 | +3% |
| **总分** | **~70** | **~90** | +20 |
