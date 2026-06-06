# 零下家园 · 小说世界模拟引擎 — 数据模型与 Schema 定义

> **注意**：角色卡格式 v1 已由 v3（详见 14-角色卡扩展与世界数据热插拔.md）替代。引擎加载时自动通过 upgrade_v1_to_v3() 迁移。

> 定义所有核心数据结构的完整 JSON Schema。这些 Schema 同时是 Rust `serde` 结构和前端 TypeScript 接口的基础。

---

## 一、角色卡 Schema (`characters/*.json`)

```jsonc
{
  "$schema": "vivid/character.schema.json",
  "title": "Character Card",
  "description": "小说角色卡。静态基底 + 可演进状态 + 快照历史。"
}
```

### 1.1 角色卡文件格式

```json
{
  // ==================== 元数据 ====================
  "schemaVersion": "1.0",
  
  // ==================== 静态基底（永不改变） ====================
  "base": {
    "id": "zhangyuan",
    "name": "张远",
    "alias": ["阿远", "小张"],
    "gender": "男",
    "birthYear": 1999,
    "appearance": "中等身材，手指粗糙有老茧（常年修东西），耳朵尖冬天冻得红亮，脸上有很深的法令纹",
    "corePersonality": "务实、沉默、观察力强。不是话多的人，但心里一直在盘算。危机中保持冷静，但也会犹豫。骨子里有情义，但表达克制。",
    "background": "小镇五金店老板的儿子，跟着父亲干了一辈子五金修理。动手能力强，熟悉各种工具和机械。学历不高但逻辑清楚。开局时是个普通青年，因触碰阁楼的短波电台激活了生存系统。",
    "speechStyle": "话少，简短，有用才说。不擅抒情，表达关心靠行动而非语言。偶尔冷幽默。",
    "personalityTraits": {
      "openness": 0.60,
      "conscientiousness": 0.80,
      "extraversion": 0.30,
      "agreeableness": 0.70,
      "neuroticism": 0.50,
      "courage": 0.65,
      "ruthlessness": 0.40,
      "optimism": 0.55,
      "loyalty": 0.85,
      "curiosity": 0.60
    },
    "initialSkills": [
      { "name": "五金修理", "level": 0.85, "max": 1.0, "category": "technical" },
      { "name": "基础电工", "level": 0.60, "max": 1.0, "category": "technical" },
      { "name": "工具使用", "level": 0.80, "max": 1.0, "category": "technical" },
      { "name": "电动车维修", "level": 0.75, "max": 1.0, "category": "technical" }
    ],
    "arc": "普通人 → 下得去手 → 合格的领袖 → 冷酷的决策者 → 与自己和解",
    "initialRelationships": {
      "chenlei": { "label": "发小", "trust": 8, "familiarity": 10, "sentiment": 7, "description": "从小一起长大，最信任的人" },
      "zhangjianguo": { "label": "父子", "trust": 6, "familiarity": 9, "sentiment": 5, "description": "关系疏离但互相在乎" }
    }
  },
  
  // ==================== 可演进状态（引擎运行时维护） ====================
  "state": {
    "hp": 100,
    "maxHp": 100,
    "hunger": 0,
    "warmth": 100,
    "fatigue": 0,
    "mental": 100,
    "stress": 0,
    "injuries": [],
    "skills": [],
    "inventory": [],
    "currentGoal": "",
    "innerConflict": "想做个好人，但末日逼他做越来越残酷的选择"
  },
  
  // ==================== 初始计划（可选，留空由 LLM 生成） ====================
  "dailyPlan": []
}
```

### 1.2 角色卡文件清单

| 文件 | 角色 | 类型 | 重要性 |
|------|------|------|--------|
| `zhangyuan.json` | 张远 | 主角 | 1.0 |
| `linshuang.json` | 林霜 | 女一 | 0.8 |
| `chenlei.json` | 陈磊 | 核心配角 | 0.7 |
| `zhangjianguo.json` | 张建国 | 配角 | 0.5 |
| `laosun.json` | 老孙 | 配角 | 0.5 |
| `zhaojianguo.json` | 赵建国 | 配角 | 0.3 |
| `xiaotong.json` | 小彤 | 配角 | 0.3 |
| `...` | 后续扩展 | — | — |

---

## 二、地点 Schema (`data/world/locations.json`)

```json
{
  "schemaVersion": "1.0",
  "locations": [
    {
      "id": "hardware-store",
      "name": "五金店",
      "category": "building",
      "subcategory": "shop",
      "description": "张远家的五金店，前店后宅。店面不大但进深深，货架堆到最里面。卷帘门拉上去一半挡风，门口地上永远是黑的——机油和鞋印和雨水混在一起。",
      "condition": 0.85,
      "maxOccupancy": 10,
      "tags": ["安全区", "据点", "有水源"],
      "resources": {
        "food": { "current": 25, "max": 200, "unit": "kg", "dailyConsumption": 2.5 },
        "fuel": { "current": 15, "max": 50, "unit": "L", "dailyConsumption": 3.0 },
        "wood": { "current": 40, "max": 200, "unit": "kg", "dailyConsumption": 8.0 },
        "medicine": { "current": 5, "max": 30, "unit": "unit", "dailyConsumption": 0.2 }
      },
      "position": { "x": 15, "y": 20 },
      "connections": [
        { "targetId": "repair-shop", "distanceKm": 0.5, "difficulty": "easy", "description": "一条水泥路，两侧是民房" },
        { "targetId": "supermarket", "distanceKm": 1.0, "difficulty": "easy", "description": "穿过镇中心广场" }
      ]
    },
    {
      "id": "repair-shop",
      "name": "陈磊修车铺",
      "category": "building",
      "subcategory": "workshop",
      "description": "陈磊开的修车铺。院子门口堆着废旧轮胎和汽车零件。一栋两层的自建房，有院子。",
      "condition": 0.70,
      "maxOccupancy": 6,
      "tags": ["有工具", "有车辆"],
      "resources": {
        "fuel": { "current": 80, "max": 200, "unit": "L", "dailyConsumption": 1.0 },
        "parts": { "current": 30, "max": 50, "unit": "set", "dailyConsumption": 0 }
      },
      "position": { "x": 20, "y": 25 },
      "connections": [
        { "targetId": "hardware-store", "distanceKm": 0.5, "difficulty": "easy" }
      ]
    },
    {
      "id": "supermarket",
      "name": "小镇超市",
      "category": "building",
      "subcategory": "store",
      "description": "镇中心最大的超市，两层。玻璃门已经被砸碎，货架倒了大半。不确定还有多少剩余物资。",
      "condition": 0.30,
      "maxOccupancy": 20,
      "tags": ["已部分搜索", "可能有丧尸"],
      "resources": {
        "food": { "current": 80, "max": 500, "unit": "kg", "dailyConsumption": 0 },
        "medicine": { "current": 15, "max": 50, "unit": "unit", "dailyConsumption": 0 }
      },
      "position": { "x": 18, "y": 18 },
      "connections": [
        { "targetId": "hardware-store", "distanceKm": 1.0, "difficulty": "easy" },
        { "targetId": "clinic", "distanceKm": 0.8, "difficulty": "easy" }
      ]
    },
    {
      "id": "clinic",
      "name": "镇卫生院",
      "category": "building",
      "subcategory": "medical",
      "description": "小镇唯一的卫生院，一栋三层小楼。药房可能还有剩余药品。",
      "condition": 0.50,
      "maxOccupancy": 15,
      "tags": ["可能有药品", "有丧尸出没"],
      "resources": {
        "medicine": { "current": 40, "max": 100, "unit": "unit", "dailyConsumption": 0 }
      },
      "position": { "x": 22, "y": 16 },
      "connections": [
        { "targetId": "supermarket", "distanceKm": 0.8, "difficulty": "easy" }
      ]
    },
    {
      "id": "school",
      "name": "镇中学",
      "category": "building",
      "subcategory": "school",
      "description": "镇上的中学，有围墙。主教学楼四层，后面有一个操场。如果条件允许可以改造成据点。",
      "condition": 0.60,
      "maxOccupancy": 50,
      "tags": ["未探索", "有围墙", "可做据点"],
      "resources": {},
      "position": { "x": 25, "y": 22 },
      "connections": [
        { "targetId": "supermarket", "distanceKm": 2.0, "difficulty": "moderate" }
      ]
    }
  ]
}
```

---

## 三、阵营 Schema (`data/world/factions.json`)

```json
{
  "schemaVersion": "1.0",
  "factions": [
    {
      "id": "zhangyuan-group",
      "name": "张远小队",
      "type": "survivor_group",
      "description": "以张远为核心的幸存者小团体，目前以五金店为据点。",
      "members": ["zhangyuan", "chenlei", "zhangjianguo", "laosun"],
      "territoryIds": ["hardware-store", "repair-shop"],
      "ideology": "互助生存，不主动害人但也不当圣人",
      "resources": {
        "food": 32,
        "fuel": 55,
        "medicine": 18
      },
      "relations": []
    },
    {
      "id": "blood-hand",
      "name": "血手帮",
      "type": "raiders",
      "description": "活跃在周边地区的掠夺者团伙。以武力抢夺他人物资。",
      "members": [],
      "territoryIds": [],
      "ideology": "末日是弱肉强食的世界，强者拥有一切",
      "resources": {},
      "relations": [
        { "targetId": "zhangyuan-group", "stance": "hostile", "description": "尚未直接接触，但迟早会有冲突" }
      ]
    }
  ]
}
```

---

## 四、环境初始参数 (`data/world/environment.json`)

```json
{
  "schemaVersion": "1.0",
  "startDate": "2025-12-03T18:00:00",
  "latitude": 28.2,
  "longitude": 113.04,
  "initialTemperature": -1.2,
  "globalCoolingRate": {
    "perDay": -0.08,
    "description": "每天降温约 0.08℃，10 年降至 -70℃",
    "curve": "linear"
  },
  "seasonTable": {
    "1": { "name": "一月", "baseTemp": 5, "daylightHours": 10.5 },
    "2": { "name": "二月", "baseTemp": 8, "daylightHours": 11.0 },
    "3": { "name": "三月", "baseTemp": 12, "daylightHours": 12.0 },
    "4": { "name": "四月", "baseTemp": 18, "daylightHours": 13.0 },
    "5": { "name": "五月", "baseTemp": 25, "daylightHours": 13.5 },
    "6": { "name": "六月", "baseTemp": 30, "daylightHours": 14.0 },
    "7": { "name": "七月", "baseTemp": 33, "daylightHours": 13.5 },
    "8": { "name": "八月", "baseTemp": 32, "daylightHours": 13.0 },
    "9": { "name": "九月", "baseTemp": 28, "daylightHours": 12.0 },
    "10": { "name": "十月", "baseTemp": 22, "daylightHours": 11.0 },
    "11": { "name": "十一月", "baseTemp": 15, "daylightHours": 10.5 },
    "12": { "name": "十二月", "baseTemp": 8, "daylightHours": 10.0 }
  },
  "weatherPatterns": {
    "winter": {
      "sunny": 0.15,
      "cloudy": 0.30,
      "snow": 0.35,
      "blizzard": 0.10,
      "freezing_rain": 0.10
    }
  }
}
```

---

## 五、剧情事件定义 (`data/plot/events.json`)

```json
{
  "schemaVersion": "1.0",
  "scheduledEvents": [
    {
      "id": "arrival-linshuang",
      "title": "林霜登场",
      "triggerTime": { "dayOffset": 7, "hour": 14, "minute": 0 },
      "description": "张远在废弃药店遇到独自搜索物资的林霜。这是她第一次登场。",
      "eventType": "character_arrival",
      "participants": ["zhangyuan", "linshuang"],
      "locationId": "clinic",
      "priority": "high",
      "oneShot": true
    },
    {
      "id": "zombie-first-sighting",
      "title": "首次目击丧尸",
      "triggerTime": { "dayOffset": 14, "hour": 10, "minute": 0 },
      "description": "有人在镇子边缘目击了疑似丧尸的生物。传言开始扩散。",
      "eventType": "plot_advance",
      "participants": [],
      "locationId": "supermarket",
      "priority": "high",
      "oneShot": true
    },
    {
      "id": "fuel-crisis",
      "title": "燃料危机",
      "triggerCondition": {
        "variable": "resources.hardware-store.fuel",
        "op": "less_than",
        "value": 5
      },
      "description": "五金店的燃料即将耗尽，必须尽快找到新的燃料来源。",
      "eventType": "resource_crisis",
      "priority": "major",
      "oneShot": false
    }
  ],
  "conditionTriggers": [
    {
      "id": "water-pipe-freeze",
      "condition": {
        "and": [
          { "variable": "environment.temperature", "op": "less_than", "value": -15 },
          { "variable": "environment.weather", "op": "not_equals", "value": "blizzard" }
        ]
      },
      "effect": {
        "type": "infrastructure_failure",
        "description": "水管冻裂，水源中断",
        "affectedLocations": ["hardware-store"],
        "message": "气温降到零下十五度，五金店的水管冻裂了。水从裂缝渗出来，在墙上结了冰。"
      },
      "oneShot": true
    }
  ]
}
```

---

## 六、前端 API 响应包装

所有 API 响应遵守统一格式：

```typescript
// 成功响应
interface ApiResponse<T> {
  status: 'ok'
  data: T
}

// 错误响应
interface ApiError {
  status: 'error'
  error: string       // 错误码
  message: string      // 人类可读描述
  details?: Record<string, unknown>
}

// WebSocket 事件
interface WsEvent {
  type: string
  timestamp: string    // ISO 8601
  data: Record<string, unknown>
}
```

---

## 七、TypeScript 接口前缀（前端开发参考）

```typescript
// 对应 Rust CharacterBase
interface CharacterBase {
  id: string
  name: string
  alias: string[]
  gender: string
  birthYear: number
  appearance: string
  corePersonality: string
  background: string
  speechStyle: string
  personalityTraits: PersonalityTraits
  initialSkills: Skill[]
  arc: string
  initialRelationships: Record<string, RelationshipDef>
}

interface PersonalityTraits {
  openness: number        // 0-1
  conscientiousness: number
  extraversion: number
  agreeableness: number
  neuroticism: number
  courage: number
  ruthlessness: number
  optimism: number
  loyalty: number
  curiosity: number
}

interface Skill {
  name: string
  level: number           // 0-1
  max: number             // 1.0
  category: 'technical' | 'social' | 'combat' | 'survival'
}

interface CharacterState {
  hp: number
  maxHp: number
  hunger: number          // 0-100
  warmth: number          // 0-100
  fatigue: number         // 0-100
  mental: number          // 0-100
  stress: number          // 0-100
  injuries: Injury[]
  skills: Skill[]
  currentGoal: string
  innerConflict: string
}

// 关系图
interface GraphNode {
  id: string
  name: string
  group: string
  gender: string
  status: string
  importance: number       // 0-1
  location?: string
}

interface GraphEdge {
  source: string
  target: string
  label: string
  trust: number            // -10 ~ 10
  familiarity: number      // 0-10
  sentiment: number        // -10 ~ 10
}

// 时间线事件
interface TimelineEvent {
  id: string
  timestamp: string
  eventType: string
  severity: 'critical' | 'major' | 'normal' | 'minor'
  title: string
  description: string
  participants: string[]
  location?: string
  novelSegment?: string
}

// 模拟控制
type SimSpeed = 'paused' | 'detailed' | 'fastforward'
type EngineState = 'paused' | 'running' | 'stopped'
```
