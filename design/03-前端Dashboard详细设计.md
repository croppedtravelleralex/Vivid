# 零下家园 · 小说世界模拟引擎 — 前端 Dashboard 详细设计

> ⚠️ **本文档已由 `15-Dashboard完整交互设计.md` 全面替代并扩展。**
> 03 保留作为早期设计参考，所有实现应基于 15。
>
> 定义 Vue 3 前端的页面布局、组件树、路由设计、数据流和可视化方案。
> 设计目标：MiroFish 级别的生产力工具感，但针对小说世界模拟的场景定制。

---

## 一、页面路由与布局

### 1.1 路由设计

```javascript
const routes = [
  { path: '/',                    name: 'Dashboard',     component: Dashboard },
  { path: '/simulation',          name: 'Simulation',    component: SimulationView },
  { path: '/character/:id',       name: 'CharacterDetail', component: CharacterView },
  { path: '/timeline',            name: 'Timeline',      component: TimelineView },
  { path: '/graph',               name: 'Graph',         component: GraphView },
  { path: '/world',               name: 'World',         component: WorldView },
  { path: '/settings',            name: 'Settings',      component: SettingsView },
]
```

### 1.2 全局布局

```
┌──────────────────────────────────────────────────────────┐
│  AppHeader                                                │
│  ┌────────────────────────────────────────────────────┐   │
│  │  BRAND  │  导航 Tab                                │  │   │
│  │  NOVEL  │  世界 | 角色 | 关系图 | 时间线 | 模拟控制  │  ⏱  │
│  │  ENGINE │                                          │  │   │
│  └────────────────────────────────────────────────────┘   │
├──────────┬───────────────────────────────────────────────┤
│          │                                               │
│ AppSide  │         <router-view />                       │
│ bar      │         主内容区域                             │
│          │                                               │
│ 世界概览  │                                               │
│ 快速操作  │                                               │
│ 角色列表  │                                               │
│          │                                               │
├──────────┴───────────────────────────────────────────────┤
│  StatusBar (模拟运行状态 / 时间 / tick / LLM 调用数)      │
└──────────────────────────────────────────────────────────┘
```

**AppHeader** — 全局顶部栏
- 左侧：品牌名 + 当前模拟时间（格式：`2025-12-03 18:00 冬季 -5℃`）
- 中间：导航 Tab（世界 / 角色 / 关系图 / 时间线 / 模拟控制）
- 右侧：模拟速度指示器 + 运行/暂停按钮 + 设置入口

**AppSidebar** — 左侧窄栏
- 世界概览（温度/季节/幸存者数）
- 快速操作（保存/加载/单步推进）
- 角色名单（快速跳转）

**StatusBar** — 底部状态栏
- 模拟时间 | Tick 数 | 运行模式 (FF/DETAILED/PAUSED) | LLM 调用统计 | 最后事件

---

## 二、Dashboard 页面（首页）

### 2.1 布局

```
┌──────────────────────────────────────────────────────┐
│  Dashboard                                            │
│                                                       │
│  ┌───────────┐ ┌────────────────────────────────┐    │
│  │ 环境状态    │ │ 时间线摘要                       │    │
│  │ 卡          │ │ (近 24h 事件条形图)              │    │
│  │ -47℃       │ │                              │    │
│  │ 暴风雪     │ │ ■■■■□□□□□□□□□□□□□□□□□□□□□      │    │
│  │ 冬季       │ │                              │    │
│  └───────────┘ └────────────────────────────────┘    │
│                                                       │
│  ┌──────────────────────┐ ┌────────────────────┐     │
│  │ 角色状态总览           │ │ 资源概览            │     │
│  │ (头像 + 名字 + HP +   │ │ 木材  ■■■■■■□□     │     │
│  │  饥饿 + 精神 + 位置)  │ │ 食物  ■■■□□□□□     │     │
│  │                       │ │ 燃料  ■■■■■□□□     │     │
│  │ [张远] ████████░░ 75% │ │ 药品  ■■□□□□□□     │     │
│  │ [林霜] ██████░░░░ 60% │ │                    │     │
│  │ [陈磊] █████████░ 90% │ │                    │     │
│  | [老孙] ████░░░░░░ 40% │ │                    │     │
│  └──────────────────────┘ └────────────────────┘     │
│                                                       │
│  ┌──────────────────────────────────────────────┐    │
│  │ 最近事件流                                      │    │
│  │ [15:30] 温度降至 -47℃，陈磊检查了柴油发电机      │    │
│  │ [14:15] 张远发现院墙西北角有松动，加固了栅栏      │    │
│  │ [12:00] 林霜从超市带回一批罐头和绷带              │    │
│  │ [10:30] 老孙抱怨食物分配不公，被张远说服          │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```

### 2.2 组件树

```
Dashboard
├── EnvironmentCard           # 环境状态卡片（温度/天气/季节/风速）
├── TimelineSummary           # 时间线摘要（事件密度图）
├── CharacterStatusGrid      # 角色状态网格
│   └── CharacterMiniCard × N  # 每个角色的缩略卡片
├── ResourceOverview          # 资源概览条形图
└── RecentEventFeed           # 最近事件流列表
    └── EventItem × N          # 单条事件
```

### 2.3 数据源

```javascript
// Dashboard 页面加载时调用的 API
const worldSummary = await api.getWorldSummary()
// {
//   environment: { temperature: -47, weather: '暴风雪', season: '冬季', ... },
//   characters: [{ id, name, hp, hunger, mental, location, ... }],
//   resources: { wood: 65, food: 32, fuel: 55, medicine: 18 },
//   recentEvents: [{ time, content, ... }],
//   tickCount: 1234,
//   simTime: '2025-12-03 15:30'
// }
```

---

## 三、角色详情页 CharacterView

### 3.1 布局

```
┌──────────────────────────────────────────────────────────┐
│  ← 返回                    角色详情：张远                  │
├──────────────────────────────────────────────────────────┤
│  ┌──────────────┐ ┌────────────────────────────────┐     │
│  │ 角色卡         │ │ 状态仪表盘                       │     │
│  │              │ │                                 │     │
│  │  [头像占位]   │ │  健康 ████████░░ 78%   体温 ██░░ │     │
│  │              │ │  饥饿 ██████░░░░ 62%   精神 ████ │     │
│  │  张远         │ │  疲劳 ████░░░░░░ 40%   压力 ██   │     │
│  │  26岁 男      │ │                                 │     │
│  │  五金店       │ │  技能：修理 ████████ 驾驶 ████   │     │
│  │              │ │        战斗 ██░░ 领导 ██████     │     │
│  │  [编辑按钮]   │ │  ┌─────────────────────────┐     │     │
│  └──────────────┘ │  │ 当前计划                    │     │     │
│                    │  │ ① 检查院墙 (已完成)         │     │     │
│  ┌──────────────┐  │  │ ② 分配今日口粮 (进行中)     │     │     │
│  │ 关系网络      │  │ ③ 去超市探路 (待执行)         │     │     │
│  │ (力导向图)   │  │  └─────────────────────────┘     │     │
│  │              │ └────────────────────────────────┘     │
│  │  [林霜] ←→   │                                        │
│  │  战友/暧昧    │  ┌────────────────────────────────┐  │
│  │              │  │ 记忆流                            │  │
│  │  [陈磊] ←→   │  │ [15:30] 检查院墙发现松动，加固了  │  │
│  │  发小/信任   │  │ [12:00] 和林霜谈话，她要去超市   │  │
│  │              │  │ [10:00] 分配口粮，老孙有意见      │  │
│  │  [老孙] ←→   │  │ [昨天] 发现了丧尸活动迹象         │  │
│  │  发小/担心   │  │ ... 更多 ▽                       │  │
│  └──────────────┘  └────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### 3.2 组件树

```
CharacterView
├── CharacterCard              # 角色基底信息
│   ├── AvatarSection          # 头像/名字/基本信息
│   ├── AppearanceBlock        # 外貌特征
│   ├── PersonalityBar         # 人格维度条形图（OCEAN 5维）
│   └── BackgroundBlock        # 背景故事
├── CharacterDashboard         # 状态仪表盘
│   ├── HealthGauge            # 健康仪表
│   ├── HungerGauge
│   ├── WarmthGauge
│   ├── MentalGauge
│   ├── StressGauge
│   ├── SkillBars              # 技能进度条
│   └── CurrentPlan            # 当前计划
├── RelationshipGraph          # 关系网络（d3-force 子图）
└── MemoryStream               # 记忆流（时间倒序）
    └── MemoryItem × N
```

### 3.3 人格可视化（OCEAN 五维雷达图）

用 chart.js 或 SVG 画雷达图：

```
           开放性
             ██
             ██
   宜人性   ████████   尽责性
          ██████████
          ██████████
          ██    ██
         ██      ██   外向性
        ██
        神经质(敏感性)
```

敏感 vs 钝感的视觉区分：
- 神经质维度高的角色用暖色（橙/红），底色偏暗
- 神经质低的角色用冷色（蓝/青），底色偏亮
- 雷达图旁的文本提示："高敏感·容易注意到环境细节·睡眠浅"

---

## 四、关系图页面 GraphView

### 4.1 布局（全屏力导向图）

```
┌──────────────────────────────────────────────────────────┐
│  关系图                                     [+ 控制面板] │
├──────────────────────────────────────────────────────────┤
│                                                          │
│            ┌───── 林霜 ─────┐                             │
│            │   战友/暧昧     │                             │
│            │                ▼                             │
│   老孙 ◄──► 张远 ◄─────────► 陈磊                        │
│   发小      │   ↑           发小                           │
│   担心      │   │                                          │
│             │   │ 父子                                       │
│             │   │                                          │
│             └───张建国                                      │
│                 │                                          │
│                 │ 夫妻                                      │
│                 ▼                                          │
│              张远妈                                        │
│                                                          │
│   [筛选：全部 | 仅信任>0 | 仅敌对]  [布局：力导向 | 环形] │
│   [选中节点详情] 张远 ↔ 林霜 信任:+6 熟悉:8 好感:+4      │
└──────────────────────────────────────────────────────────┘
```

### 4.2 交互设计

| 交互 | 行为 |
|------|------|
| **拖拽节点** | 手动调整布局，释放后力模拟重新收敛 |
| **悬停节点** | 高亮该节点及其直连边，其他节点变暗 |
| **单击节点** | 右侧面板显示角色摘要 + 跳转链接 |
| **双击节点** | 进入"以该节点为中心"模式，只显示 2 度以内邻居 |
| **滚轮缩放** | d3-zoom，缩放范围 0.2x ~ 5x |
| **边悬停** | 显示关系详情（信任值 + 交互历史） |
| **右键空白** | 上下文菜单（重置视图 / 截图导出 / 全屏） |

### 4.3 图数据格式

```javascript
// API 返回的数据结构（GET /api/v1/graph/relationships）
{
  nodes: [
    { id: 'zhangyuan', name: '张远', group: 'protagonist', 
      gender: 'male', status: 'healthy', importance: 1.0 },
    { id: 'linshuang', name: '林霜', group: 'love_interest',
      gender: 'female', status: 'healthy', importance: 0.8 },
    // ...
  ],
  edges: [
    { source: 'zhangyuan', target: 'linshuang', 
      label: '战友', trust: 6, familiarity: 8, sentiment: 4 },
    { source: 'zhangyuan', target: 'chenlei',
      label: '发小', trust: 8, familiarity: 10, sentiment: 7 },
    // ...
  ]
}
```

### 4.4 d3-force 配置

```javascript
// 力模拟参数
const simulation = d3.forceSimulation(nodes)
  .force('link', d3.forceLink(edges)
    .id(d => d.id)
    .distance(d => 200 - d.trust * 10)  // 信任越高越靠近
    .strength(d => d.familiarity / 10))  // 熟悉度越高边越强
  .force('charge', d3.forceManyBody()
    .strength(d => -200 * d.importance))  // 重要角色斥力大（显眼）
  .force('center', d3.forceCenter(width / 2, height / 2))
  .force('collision', d3.forceCollide(30))
```

### 4.5 边样式编码

| 信任值 | 颜色 | 线型 | 宽度 |
|--------|------|------|------|
| +8 ~ +10 | 绿 `#2d8a4e` | 实线 | 4px |
| +4 ~ +7 | 青 `#2d7a8a` | 实线 | 3px |
| +1 ~ +3 | 蓝灰 `#5a7a8a` | 虚线 | 2px |
| 0 | 灰 `#999` | 点线 | 1px |
| -1 ~ -3 | 橙 `#c97a2d` | 虚线 | 2px |
| -4 ~ -7 | 红 `#c93a2d` | 实线 | 3px |
| -8 ~ -10 | 深红 `#8a1a1a` | 双线 | 4px |

---

## 五、时间线页面 TimelineView

### 5.1 布局

```
┌──────────────────────────────────────────────────────────┐
│  时间线               [过滤器: 全部/战斗/日常/环境/社交]   │
├──────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐    │
│  │  时间轴 (d3 水平缩放条)                           │    │
│  │  │▬▬▬▬│▬▬▬▬▬▬▬│▬▬▬│▬▬▬▬▬▬▬▬▬▬│▬▬▬▬│▬▬▬▬▬▬│    │    │
│  │  12/3  12/4    12/5  12/6        12/7  12/8        │    │
│  │  18:00        06:00              14:00             │    │
│  └──────────────────────────────────────────────────┘    │
│                                                          │
│  ┌──────────────────────────────────────────────────┐    │
│  │ 事件日志（按时间倒序）                             │    │
│  │                                                   │    │
│  │ [12/8 14:00] 事件等级图标 事件描述                │    │
│  │    └─ 参与者：张远、林霜 · 影响：信任+2           │    │
│  │    └─ 关联片段：点击展开正文段落                   │    │
│  │                                                   │    │
│  │ [12/8 10:00] ■ 陈磊发现燃料即将耗尽               │    │
│  │    └─ 地点：修车铺 · 影响：触发资源危机            │    │
│  │                                                   │    │
│  │ [12/7 22:00] ◆ 张远和林霜在炉边谈话                │    │
│  │    └─ 推演片段：「你明天别一个人去。」              │    │
│  │    └─ 「你也怕一个人？」林霜没有回头。             │    │
│  │                                                   │    │
│  │ [12/7 15:00] ▲ 温度降至 -30℃ 水管冻裂             │    │
│  │    └─ 影响：五金店水源中断 · 应对：陈磊焊接修复    │    │
│  │                                                   │    │
│  │ [12/6 08:00] 老孙第一次提出食物分配不公            │    │
│  │    └─ 张远决策：维持现行分配，承诺下次优先补给      │    │
│  │                                                   │    │
│  └──────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────┘
```

### 5.2 事件等级图标

```javascript
const EVENT_ICONS = {
  critical: '🔴',   // 生死攸关（死亡/重伤/重大危机）
  major:    '🟠',   // 重大（冲突/资源危机/重要发现）
  normal:   '🟡',   // 普通（日常对话/常规行动）
  minor:    '⚪',   // 琐事（休息/吃饭/巡逻）
}
```

### 5.3 数据格式

```javascript
// GET /api/v1/timeline?from=2025-12-03&to=2025-12-10&filter=all
{
  summary: {
    total: 234,
    critical: 3,
    major: 18,
    normal: 145,
    minor: 68,
    timeRange: { from: '2025-12-03 18:00', to: '2025-12-10 14:00' }
  },
  timeline: [
    {
      time: '2025-12-08 14:00',
      severity: 'normal',
      eventType: 'dialogue',
      title: '张远和林霜在炉边谈话',
      description: '寒冷让两个人靠得更近，但林霜仍然保持距离。',
      participants: [{ id: 'zhangyuan', name: '张远' }, { id: 'linshuang', name: '林霜' }],
      location: '五金店',
      effects: [{ type: 'relationship', charA: '张远', charB: '林霜', trust: '+2' }],
      novelSegment: '"你明天别一个人去。"张远盯着炉火说。\n"你也怕一个人？"林霜没有回头。',
      relatedEvents: ['uuid-xxx', 'uuid-yyy']
    },
    // ...
  ],
  relationships: [
    // 可选的关系变化曲线数据
    { pair: '张远-林霜', data: [{ time: '12/3', trust: 0 }, { time: '12/8', trust: 6 }] }
  ]
}
```

### 5.4 关系变化曲线

时间线页面底部可选显示"关系变化曲线"：

```
信任值
10 ┤
 9 ┤
 8 ┤                      ┌─── 张远-陈磊
 7 ┤                 ┌────┘
 6 ┤           ┌─────┘
 5 ┤      ┌────┘                 ┌─── 张远-林霜
 4 ┤     ┌┘                ┌─────┘
 3 ┤     │           ┌─────┘
 2 ┤     │     ┌─────┘
 1 ┤     │─────┘
 0 ┤─────┘
   └───────────────────────────────────────▶ 时间
     12/3  12/4  12/5  12/6  12/7  12/8
```

使用 d3 的面积图 + 路径动画。

---

## 六、模拟控制面板

### 6.1 模拟控制组件

```
┌──────────────────────────────────────┐
│  模拟控制                              │
│                                      │
│  当前状态：● 运行中 (Detailed)         │
│  模拟时间：2025-12-08 14:00           │
│  Tick：1,234     LLM调用：3,456      │
│                                      │
│  ┌──────┐ ┌──────┐ ┌──────────────┐ │
│  │ ▶ 运行 │ │ ⏸ 暂停 │ │ ⏭ 单步推进  │ │
│  └──────┘ └──────┘ └──────────────┘ │
│                                      │
│  速度控制                              │
│  ○ 暂停 (编辑模式)                     │
│  ● 精细模拟 (5min/tick，含LLM)        │
│  ○ 快速推进 (1h/tick，不含LLM)        │
│                                      │
│  ┌──────────┐ ┌──────────────────┐   │
│  │ 💾 保存   │ │ 🔄 加载检查点     │   │
│  └──────────┘ └──────────────────┘   │
└──────────────────────────────────────┘
```

### 6.2 速度控制逻辑

三个单选按钮对应三种模式，后端根据选择切换 `SimSpeed`：

```
暂停 ────→ 精细模拟 ────→ 快速推进
  │            │              │
  │            │    空闲>3次   │
  │            └──────自然降速──┘
  │                     │
  │                     事件触发
  │                     └───自然升速──┐
  │                                     │
  └────────────────────────────────────┘
```

用户手动选择速度时，后端**尊重用户选择**不自动切换（除非用户在 Dashboard 中开启了"自动速度"开关）。

---

## 七、世界总览页面 WorldView

### 7.1 布局

```
┌──────────────────────────────────────────────────────────┐
│  世界总览                                                  │
├──────────────────────────────────────────────────────────┤
│  ┌──────────────────────┐ ┌──────────────────────────┐   │
│  │ 环境仪表盘             │ │ 温度趋势图（7天）           │   │
│  │                      │ │                           │   │
│  │  温度    -30℃         │ │  -20┤ ╱╲                  │   │
│  │  体感    -38℃         │ │  -30┤╱  ╲╱╲╱╲             │   │
│  │  风速    25km/h       │ │  -40┤      ╲╱╲╱╲╱╲        │   │
│  │  湿度    82%          │ │  -50┤          ╲╱╲╱╲      │   │
│  │  能见度  200m         │ │     └─────────────────    │   │
│  │  日照    16:10-17:30  │ │     12/3 12/5 12/7 12/9   │   │
│  │  季节    冬季         │ │                           │   │
│  └──────────────────────┘ └──────────────────────────┘   │
│                                                          │
│  ┌──────────────────────────────────────────────────┐    │
│  │ 地点地图（节点拓扑图）                              │    │
│  │                                                  │    │
│  │  [五金店] ─── [修车铺]                             │    │
│  │     │            │                                │    │
│  │  [超市] ──── [卫生院]                              │    │
│  │     │                                              │    │
│  │  [学校] (探索中)                                    │    │
│  │                                                  │    │
│  │  图例：■ 安全据点 ■ 已探索 ■ 未探索 ■ 危险区域     │    │
│  └──────────────────────────────────────────────────┘    │
│                                                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │ 阵营面板   │ │ 资源面板  │ │ 事件日志  │ │ 统计面板 │   │
│  │           │ │          │ │          │ │          │   │
│  │ 张远小队   │ │ 木材 65  │ │ 今日+12  │ │ 存活 5人 │   │
│  │ 血手帮    │ │ 食物 32  │ │ 本周+89  │ │ 丧尸 ？  │   │
│  │ (未知)    │ │ 燃料 55  │ │ 总计 234 │ │ 天数 6   │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
└──────────────────────────────────────────────────────────┘
```

---

## 八、WebSocket 实时同步

### 8.1 前端连接管理 (`composables/useWebSocket.js`)

```javascript
// 组合式函数：管理 WebSocket 连接 + 自动重连
export function useWebSocket() {
  const ws = ref(null)
  const connected = ref(false)
  const lastEvent = ref(null)
  
  // 注入 worldState 响应式对象
  const worldState = inject('worldState')
  
  function connect() {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    ws.value = new WebSocket(`${protocol}//${location.host}/api/v1/ws`)
    
    ws.value.onmessage = (event) => {
      const data = JSON.parse(event.data)
      handleEngineEvent(data)
    }
    
    ws.value.onclose = () => {
      connected.value = false
      // 5 秒后自动重连
      setTimeout(connect, 5000)
    }
  }
  
  function handleEngineEvent(event) {
    lastEvent.value = event
    switch (event.type) {
      case 'Tick':
      case 'DetailedTick':
        worldState.tick = event.tick
        worldState.simTime = event.time
        // 增量更新，不替换整个对象
        break
      case 'CharacterUpdate':
        updateCharacter(event.char_id, event.state_diff)
        break
      case 'EventTriggered':
        worldState.eventLog.unshift(event)
        if (worldState.eventLog.length > 200) worldState.eventLog.pop()
        break
      case 'EnvironmentUpdate':
        Object.assign(worldState.environment, event)
        break
    }
  }
  
  return { connected, lastEvent, connect }
}
```

### 8.2 增量更新而非全量替换

关键优化：WebSocket 收到的 tick 事件不替换整个 worldState，而是**增量 patch**：

```javascript
// 只更新 diff
function applyStateDiff(diff) {
  // diff = { character_updates: [{ id, fields: { hp: 75, hunger: 62 } }], ... }
  for (const update of diff.character_updates) {
    const character = worldState.characters.find(c => c.id === update.id)
    if (character) {
      Object.assign(character, update.fields)  // 局部更新
    }
  }
}
```

这样 Vue 的响应式系统只触发受影响的组件重渲染，而不是整个页面。

---

## 九、组件数据流

```
Rust 引擎 (每 tick)
  │
  │ 序列化为 EngineEvent (JSON)
  ▼
WebSocket
  │
  │ 推送
  ▼
useWebSocket.js (composable)
  │
  ├─→ worldState (reactive object, provide/inject)
  │      │
  │      ├─→ App.vue (timer display)
  │      ├─→ Dashboard (summary)
  │      ├─→ CharacterView (detail)
  │      ├─→ TimelineView (event feed)
  │      └─→ GraphView (relationship data)
  │
  └─→ console / debug panel (可选)
```

**主动请求走 HTTP：**
```
用户操作 (点击"暂停" / "切换速度" / "保存检查点")
  │
  ├─→ api.simulation.pause()
  ├─→ api.simulation.setSpeed('fast')
  ├─→ api.checkpoint.save('tag')
  │
  ▼
Axios HTTP → Axum API → Engine
```

**原则**：
- 推（WebSocket）：引擎 → 前端（状态更新、事件通知）
- 拉（HTTP GET）：前端首次加载 + 用户主动刷新
- 命令（HTTP POST）：前端 → 引擎（控制指令）

---

## 十、与 MiroFish 前端的对比

| MiroFish 前端 | 本 Dashboard | 变化原因 |
|---|---|---|
| `GraphPanel.vue` (知识图谱) | `RelationGraph.vue` (力导向关系图) | 从实体知识图 → 角色社交关系 |
| `Step1GraphBuild` ~ `Step5` (五步向导) | 无向导，Dashboard 持续监控 | 本引擎是模拟而非向导流程 |
| 固定的 5-step workflow | 自由导航（Tab 切换） | 模拟引擎需要实时监控多维度 |
| i18n 中英切换 | 保留 | 有用 |
| OASIS 模拟状态轮询 | WebSocket 实时推送 | LLM 决策延迟高，推送减少前端请求 |
| 无关系变化曲线 | `RelationshipGraph` 有时间维度 | 用户要求角色关系随时间变化 |
| 无环境可视化 | `EnvironmentCard` + 温度趋势图 | 用户要求环境系统可视化 |
| 模拟结果 Report 页面 | `TimelineView` 事件流 | 更实时、更细粒度 |
