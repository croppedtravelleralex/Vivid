# Vivid · 小说世界模拟引擎 — AI 接手文档

> 任何人/任何 AI 读完本文档即可完全接手本项目。
> 位置：`D:\SelfMadeTool\小说\vivid\`

---

## 一、项目是什么

Rust + Vue 3 的小说世界时间线模拟引擎。角色在无人干预下自发行动、感知、思考，产出可用于写作的素材。

核心理念：**角色驱动，而非剧情驱动**。引擎不预设剧情，剧情是角色在环境中按人设自发行动的涌现结果。

---

## 二、技术栈

**后端**：Rust (edition 2021), axum 0.8, tokio, hecs (ECS), petgraph, serde, bincode, redb 4.1, chrono, uuid v7, reqwest, governor 0.10, minijinja, tracing

**前端**：Vue 3 (Composition API), Vite 7, vue-router 4, axios, chart.js + vue-chartjs, d3-force + d3-zoom, 纯 CSS

---

## 三、项目结构

```
vivid/
├── Cargo.toml              # workspace 根
├── backend/
│   ├── Cargo.toml
│   ├── .env.example
│   └── src/
│       ├── main.rs         # 入口：加载→创建引擎→启动服务
│       ├── lib.rs          # 模块导出
│       ├── config.rs       # 全局配置 (yaml 加载+验证)
│       ├── telemetry.rs    # tracing 日志
│       ├── api/            # 12 文件, 28 REST 端点
│       ├── engine/         # 核心 + M2 子模块 + tag 系统
│       ├── council/        # M4: 叙事议会
│       ├── llm/            # LLM 网关 + M4 优化栈
│       ├── models/         # 数据结构 + M3 数学模型
│       └── storage/        # 检查点 + WAL
│   └── tests/
│       ├── api_tests.rs    # 37 集成测试
│       ├── engine_tests.rs # 5 基础测试
│       ├── scenarios/      # 16 场景测试
│       └── models/         # 13 向量 + 23 M3 测试
├── frontend/               # Vue 3 前端 (43 文件)
│   └── src/
│       ├── api/            # 5 个 API 层
│       ├── views/          # 6 个页面
│       ├── components/     # 18 组件
│       ├── composables/    # 3 状态管理
│       └── styles/         # 暗色主题
├── data/                   # 运行数据
│   ├── config.yaml         # 全局配置
│   ├── characters/         # 5 个角色 (v1 格式)
│   ├── world/              # 4 个地点 + 环境
│   └── plot/               # 种子事件
├── design/                 # 18 份原始设计文档
└── docs/                   # README + 设计索引 + 本文
```

---

## 四、当前里程碑状态

### M1 — 核心引擎 + 基础 Dashboard ✅ COMPLETE
- 26 REST API 端点 + WebSocket 全部实现并通过测试
- 三态主循环 (Paused/Detailed/FastForward) 正常运行
- ECS 角色系统 + 关系图 + 环境演化
- LLM 网关 (Semaphore + 断路器 + led rate limiter)
- 检查点 + WAL + 优雅关闭
- 5 角色 + 4 地点端到端启动验证通过
- 前端 43 文件, vite build 通过

### M2 — 事件子系统 ✅ COMPLETE
- Storyteller (叙事节奏控制)
- EmergentDetector (涌现事件检测 + SOC 沙堆)
- CascadeEngine (社交网络级联传播)
- EventMemory (Ebbinghaus 遗忘曲线)
- ProbabilityTree (条件分支展开)
- NarrativeFilter (叙事价值评分)

### M3 — 数学模型 ✅ COMPLETE
- Social: 68 社会学公式 (意见动力学/规范/互惠/地位/谣言/记忆/认同/网络/集体行为/信任)
- Chaos: 28 混沌模型 (逻辑斯蒂/李雅普诺夫/洛伦兹/分岔/SOC/幂律/Lotka-Volterra/Langton/级联)
- Economics: 23 经济模型 (效用曲线/前景理论/享乐适应/满意化/时间贴现/弹性/生产函数/易货/Ostrom)

### M4 — 叙事议会 + 标签 + LLM 优化栈 ✅ COMPLETE
- 叙事议会: 5 路成员 (auditor/architect/assessor/analyst/summarizer)
- 标签系统: TagIndex + 自动打标 + 新鲜度生命周期 + 线索管理
- L0-L17 LLM 优化栈 (18 层管线: 优先级/缓存/信号量/自适应token/去重/频率控制/相似路由/预合成/批处理/生成缓存/语义去重/截止时间/优先级队列/压缩/降级链/熔断v2/议会优化)

### M5 — 专业前端 🔲 PENDING
未开始: Undo/Redo/搜索/快捷键/演化曲线/评分时间线/What-if/备份/工作区/无障碍/虚拟滚动/Web Worker/性能监控

---

## 五、关键路径

| 文件 | 重要性 | 影响 |
|------|--------|------|
| `engine/simulation_loop.rs` | 核心 | 主循环 Paused/Detailed/FF |
| `engine/mod.rs` | 核心 | SimulationEngine, EngineEvent, EngineState |
| `models/world.rs` | 核心 | WorldState, CharacterState, EnvironmentState |
| `models/event.rs` | 核心 | EventType, EventQueue, ScheduledEvent |
| `models/character.rs` | 核心 | 角色卡 v1/v3 定义 + 迁移函数 |
| `api/mod.rs` | 核心 | 路由注册 |
| `llm/gateway.rs` | 关键 | LLM HTTP 调用 + 断路器 + 限速 |
| `config.rs` | 关键 | 全局配置 (唯一 EngineConfig) |

---

## 六、编译 + 运行

```bash
# 编译检查
cd D:\SelfMadeTool\小说\vivid
cargo check

# 运行测试
cargo test

# 启动后端 (需 LLM_API_KEY)
LLM_API_KEY=sk-xxx cargo run

# 前端 (另一个终端)
cd frontend
npm install
npm run dev
```

**当前状态**：
- ✅ `cargo check`: 0 errors, ~17 warnings (dead_code, M4 预留)
- ✅ `cargo test`: 172 pass, 0 fail, 1 ignored (round counter 竞态)
- ✅ 端到端: 5 角色 + 4 地点正常启动, API 响应正常

---

## 七、测试架构

```
backend/tests/
├── api_tests.rs          # 37 个 API 集成测试 (axum::test)
├── engine_tests.rs       # 5 个引擎基础测试
├── scenarios/
│   └── engine_scenarios.rs  # 16 个场景测试 (并发/时间推进/状态转换)
└── models/
    ├── test_vectors.rs   # 13 数学向量测试
    ├── social_tests.rs   # 社会模型测试
    ├── chaos_tests.rs    # 混沌模型测试
    └── economics_tests.rs # 经济学模型测试
```

**单元测试**在各模块的 `#[cfg(test)] mod tests` 中。

---

## 八、主要数据流

```
用户 POST /start
  → EngineState → Running
  → 主循环 (simulation_loop.rs)
    → Detailed tick (每 5 分钟模拟时间)
      → 环境更新 (温度/天气/季节)
      → 检查事件队列触发种子事件
      → 检查条件触发器 (资源阈值)
      → 随机选角色调 LLM (Perceive→Think→Act)
      → 应用决策到 WorldState
    → FastForward tick (每 1 小时模拟时间)
      → 快速消耗资源, 不调 LLM
    → 每 50 tick 触发叙事议会 (异步)
  → WebSocket 推送事件到前端
```

---

## 九、API 路由 (28 个端点)

所有端点挂载于 `/api/v1/`:

| 分组 | 端点 |
|------|------|
| world | GET /world, /world/environment, /world/locations |
| characters | GET /characters, /characters/:id, /characters/:id/memory, /characters/:id/relationships |
| simulation | POST start/pause/speed/step/stop, GET status/stats |
| graph | GET /graph/relationships, /graph/locations |
| timeline | GET /timeline, /timeline/events |
| checkpoint | POST /checkpoint/save, /checkpoint/load, GET /checkpoint/list |
| events | POST /events, GET /events/:id |
| tags | GET /tags/heatmap, /tags/threads |
| dashboard | GET /dashboard/summary |
| search | GET /search?q= |
| ws | WS /ws |

---

## 十、已知坑位 (接手者必读)

1. **council :: ROUND_COUNTER 是全局 AtomicU64** — 并行测试时 1 个被标记 ignore。不影响生产运行。
2. **TagIndex 在 engine/tags.rs 和 engine/tags/mod.rs 是两套** — 当前最终版是 engine/tags.rs（单文件），mod.rs 目录已被移除。如果看到 `engine/tags/` 目录是旧版残留。
3. **EngineConfig 已在 config.rs 统一** — 不要再去 engine/mod.rs 找 EngineConfig，它是 `pub use config::EngineConfig`。
4. **检查点恢复丢失角色数据历史** — 已在 2 月修复，使用 `collect_entities_json()` 序列化 ECS 实体。但不用时需注意。
5. **条件触发器 loaded but never evaluated** — 已在 2 月修复，每 detailed tick 末尾评估。
6. **17 warnings 全是 M4+ dead_code 预留** — HalfOpen, max_concurrent, current_version 等，不影响功能。
7. **前端 `ConfirmDialog` 已被 removed 死代码** — provide('confirm') 已删，但组件模板还在。如需恢复需重接 provide/inject。
8. **`design/` 目录是原始设计文档，部分已过时** — 比如目录树不匹配当前实现。以实际代码结构为准，设计文档只作概念参考。
9. **`backend/write_tags.py`** — 如果存在可删除，Python 原型脚本残留。

---

## 十一、下一步推荐

| 优先级 | 任务 | 文件 |
|--------|------|------|
| 立即 | 清理 17 个 dead_code warnings (加 `#[allow(dead_code)]` 或 #[expect]) | 各模块 |
| 立即 | 设 LLM_API_KEY 跑真实角色对话验证 | .env |
| 立即 | 前端 npm run dev 验证 Dashboard 渲染 | frontend/ |
| M5 | 撤销/重做 + 快捷键 + What-if 分支 | 前端 |
| M5 | 工作区 + 备份 + 命令面板 | 前端 |
| 优化 | 集成 cargo-audit / cargo-deny 依赖安全检查 | CI |
| 扩展 | 接入真实 DeepSeek API 测试三级 Prompt 链 | llm/ |

---

## 十二、跨文档索引

- 18 份设计文档在 `design/` 目录
- 设计索引: `docs/design-links.md`
- 本文件: `docs/HANDOFF.md`

---

## 自维护规则

当接手 AI 完成以下任一操作后，应更新本文档：

1. 修改了关键路径中的文件（如上所列）
2. 新增或删除了模块级目录
3. 里程碑状态发生变化（M5 开始/完成等）
4. 发现了新的已知坑位
5. 测试通过数变化超过 10

更新方法：`git diff` 看变更，逐段检查本文档，过期的更新、缺失的补充。

最后更新：2026-06-06
