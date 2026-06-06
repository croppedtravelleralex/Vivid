# 零下家园 · 小说世界模拟引擎 — 极致优化与 LLM 编排架构

> 本文档定义引擎的 LLM 调用优化策略、三层编排架构、Token/RPM 预算管理、认知层级体系，以及 GPU 加速方案。
> 目标：200 角色 + 世界并行运转，RPM 消耗控制在 3-5，Token 消耗 1500-3000/tick。

---

## 一、优化哲学：逼近信息论下限

### 1.1 核心思想

每次 LLM 调用本质是传输信息。hook 的"不确定性"可以用信息熵 H 衡量：

```rust
/// hook 的信息熵：H = -Σ p·log₂(p)
/// H=0 → 完全可预测，不需要 LLM
/// H=1 → 二选一，需要 1 bit 信息
/// H>3 → 高度不确定，需要 LLM
fn hook_entropy(hook: &Hook, world: &WorldState) -> f64 {
    let p = predict_outcome_distribution(hook, world);
    -p.iter().filter(|&&x| x > 0.0)
        .map(|&x| x * x.log2())
        .sum::<f64>()
}
```

**理论上限**：如果预测模型能 100% 准确预测 LLM 的输出，零次调用。
**逼近上限**：让引擎学习 LLM 的行为模式，逐步替代它。

### 1.2 优化全景

| 层级 | 技术 | RPM 节省 | Token 节省 | 延迟改善 | 实现复杂度 |
|------|------|---------|-----------|---------|-----------|
| L0 | 优先级预算分配 | -40% | - | - | 低 |
| L1 | 全局协调 diff 压缩 | -80%(L1) | -80% | - | 低 |
| L2 | Hook 精确缓存 | -50%(L2) | -50% | -40~60% | 中 |
| L3 | 依赖图最大波次并行 | - | - | -65% | 中 |
| L4 | Token 预算管理 | - | -60% | - | 低 |
| L5 | 语义缓存 | -15~25% | -15~25% | - | 高 |
| L6 | 推测执行 + 预取 | - | - | 感知 0 延迟 | 中 |
| L7 | 上下文复用 | -66% | -66% | - | 中 |
| L8 | Hook 链剪枝 | - | Token -50% | - | 中 |
| L9 | 自适应上下文窗口 | - | -35% | - | 低 |
| L10 | 优先级反转防护 | 保证公平 | - | - | 低 |
| L11 | 微批调度 | 吞吐 +100% | 成本 -60% | - | 中 |
| L12 | 模仿学习 | -40% | -40% | - | 高 |
| L13 | 神经缓存 | -60~70% | -60~70% | - | 极高 |
| L14 | 认知层级 | -90% | -90% | 0 感知 | 高 |
| L15 | 差分 LLM | - | -95% | - | 中 |
| L16 | 注意力稀疏 | -90% | -90% | - | 低 |
| L17 | KV-Cache 共享 | - | Prefill -50% | -50% | 低 |

---

## 二、三层 LLM 编排架构

### 2.1 三种 LLM 调用模式

| 模式 | 抽象 | 特点 | 并行度 | RPM 消耗 |
|------|------|------|--------|---------|
| **点→全** | `1 LLM call → N 角色` | 一次性审阅全局状态，输出所有角色的校正 | 串行（1 次调用） | 0.2/tick |
| **点→面** | `1 LLM call → 1 事件链` | 推演单一事件链，决定触发/分支/遗忘 | 可并行 | 1-5/tick |
| **Hook 牵引多个** | `M calls → K 因果链` | 多个因果链同时需要 LLM 决策 | 高并行 | 3-15/tick |

### 2.2 架构图

```
┌────────────────────────────────────────────────────────────┐
│                   CPU 主循环 (tokio)                        │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Layer 1: 全局协调器 (点→全)                          │  │
│  │  每 3-5 tick 1 次 LLM 调用                            │  │
│  │  输入：全局状态 diff（~500 tokens）                     │  │
│  │  输出：一致性校正 + 叙事节奏 + 焦点角色                  │  │
│  │  RPM: 0.2-0.3 / tick                                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                           │                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Layer 2: Hook 事件引擎 (点→面)                       │  │
│  │  输入：事件上下文 + 关联角色压缩状态                     │  │
│  │  输出：是否触发→下步连接谁→概率修正→遗忘判定             │  │
│  │  缓存命中率 40-60% + 语义缓存额外 +15~25%              │  │
│  │  RPM: 1-3 / tick                                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                           │                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Layer 3: 多链并行器 (hook 牵引多个)                   │  │
│  │  依赖图解析 → 最大并行波次 → 并发发 LLM                │  │
│  │  Semaphore 控制并发上限 + 优先级排队                     │  │
│  │  推测执行：预测未来 hook，提前发 LLM 缓存结果            │  │
│  │  RPM: 3-5 / tick（峰值 8）                            │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  GPU Compute Layer（可选，无 GPU 时 CPU 降速运行）          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  GPU: 概率扩散 + 因果链分支指数计算                    │  │
│  │  输入：LLM 输出的事件链概率分布                          │  │
│  │  计算：蒙特卡洛展开 512 条分支路径                     │  │
│  │  输出：概率排序——"这条链 70% 走向 X，20% 走向 Y"       │  │
│  │  无 GPU 降级：只展开前 16 条最高概率路径                 │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

---

## 三、Hook 优先级预算分配（L0）

### 3.1 三级优先级

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum HookPriority {
    Critical,   // 生死相关——必须立即 LLM 决策（战斗/逃跑/重大事件）
    Normal,     // 剧情推进——尽可能快 LLM 决策（对话触发/任务推进）
    Background, // 后台演化——可延迟/合并/用规则替代（物资消耗确认/日常巡逻）
}

struct HookBudget {
    total_rpm: usize,           // 总配额，如 60
    priority_weights: [f64; 3], // [Critical: 0.5, Normal: 0.3, Background: 0.2]
    
    fn rpm_for(&self, priority: HookPriority) -> usize {
        (self.total_rpm as f64 * self.priority_weights[priority as usize]) as usize
    }
}
```

### 3.2 自适应降级

当 RPM 预算不足时，按优先级降级：

```rust
async fn resolve_hook_with_budget(
    hook: Hook,
    remaining_budget: f64,
) -> HookResult {
    match hook.priority {
        Critical => {
            // 必须 LLM，但可以压缩 prompt
            call_llm_compressed(hook).await
        }
        Normal => {
            if remaining_budget > 0.3 {
                call_llm(hook).await
            } else {
                rule_engine_resolve(hook)  // 降级为规则
            }
        }
        Background => {
            if hook.has_new_context() {
                call_llm(hook).await
            } else {
                rule_engine_resolve(hook)
            }
        }
    }
}
```

---

## 四、全局协调器（L1）

### 4.1 三级压缩 + Diff 传递

全局协调器不发送全量状态，只发送自上次协调以来的变化：

```rust
struct GlobalCoordinator {
    last_full_check: u64,
    diff_accumulator: Vec<Event>,
    current_focus: Option<Uuid>,
}

impl GlobalCoordinator {
    fn build_prompt(&self, world: &WorldState) -> Vec<Message> {
        let diff_summary = self.diff_accumulator
            .iter()
            .map(|e| format!("[tick {}] {}: {}", e.tick, e.actor, e.summary))
            .take(10)
            .collect::<Vec<_>>()
            .join("\n");
            
        vec![
            Message::system("你是世界叙事协调器。基于最近的事件 diff，判断：1) 当前世界运行是否正常 2) 下一个应该聚焦哪个角色 3) 是否有潜在危机苗头"),
            Message::user(diff_summary),
        ]
    }
}
```

**频率**：每 3-5 tick 一次，而非每 tick。
**效果**：L1 层 RPM 降低 80%（从 1/tick 到 0.2/tick）。

---

## 五、Hook 缓存系统（L2 + L5）

### 5.1 精确缓存

高频重复的 hook（巡逻完成、物资搬运等）直接缓存结果：

```rust
struct HookCache {
    cache: LruCache<HookCacheKey, HookCacheValue>,
    stats: HashMap<HookType, (u64, u64)>,  // (命中, 总请求)
}

impl HookCache {
    fn try_cached(&mut self, hook: &Hook) -> Option<HookResult> {
        let key = HookCacheKey {
            hook_type: hook.hook_type,
            actor_id: hook.actor,
            context_hash: hook.context_light_hash(),
        };
        
        if let Some(cached) = self.cache.get(&key) {
            if cached.world_version == self.world_version {
                return Some(cached.result.clone());
            }
        }
        None
    }
}
```

**预期命中率**：40-60%。

### 5.2 语义缓存

语义相似的 hook 共享结果：

```rust
struct SemanticCache {
    entries: Vec<SemanticEntry>,
    embedder: MiniEmbedder,
    similarity_threshold: f64, // 0.92
}

impl SemanticCache {
    fn lookup(&self, hook: &Hook) -> Option<HookResult> {
        let embedding = self.embedder.embed(hook.context_summary());
        
        for entry in &self.entries {
            let sim = cosine_similarity(&embedding, &entry.embedding);
            if sim > self.similarity_threshold {
                return Some(entry.result.clone());
            }
        }
        None
    }
}
```

**嵌入器**：用 `simhash` 或 `sentence-transformers`，CPU 上 50μs 完成。
**预期命中率**：额外 15-25%。

---

## 六、依赖图并行（L3）

### 6.1 最大并行波次求解

把 hook 链解析为 DAG，求拓扑排序的最大并行波次：

```rust
/// 每波内的 hook 互不依赖，可全并行
fn max_parallel_waves(hooks: &[Hook]) -> Vec<Vec<&Hook>> {
    let graph = build_dependency_graph(hooks);
    
    // Kahn 算法求拓扑排序
    let mut in_degree = HashMap::new();
    let mut adj = HashMap::new();
    
    let mut waves = Vec::new();
    let mut queue: Vec<_> = graph.nodes()
        .filter(|n| in_degree[n] == 0)
        .collect();
    
    while !queue.is_empty() {
        waves.push(queue.clone());
        for node in &queue {
            for neighbor in adj[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push(neighbor);
                }
            }
        }
        queue.retain(|n| !waves.last().unwrap().contains(n));
    }
    
    waves
}
```

**效果**：串行 3 波的 hook 链 → 并行 1 波完成。延迟降低 60-70%。

---

## 七、Token 预算管理（L4）

### 7.1 按优先级分 Token

```rust
struct TokenBudget {
    max_tpm: usize,          // DeepSeek TPM 限制，如 1,000,000
    tick_interval_sec: f64,  // 每 tick 真实时间，如 5s
    priority_allocation: [f64; 3], // [Critical: 0.6, Normal: 0.3, Background: 0.1]
}

impl TokenBudget {
    fn per_tick(&self) -> usize {
        let ticks_per_min = 60.0 / self.tick_interval_sec;
        (self.max_tpm as f64 / ticks_per_min) as usize
    }
    
    fn per_hook(&self, priority: HookPriority, active_count: usize) -> usize {
        let tick_budget = self.per_tick();
        let priority_share = self.priority_allocation[priority as usize];
        (tick_budget as f64 * priority_share) as usize / active_count.max(1)
    }
}
```

### 7.2 自适应上下文窗口

根据 hook 复杂度和优先级动态调整上下文长度：

```rust
enum ContextSize {
    Minimal,    // 只有角色名 + hook 描述（~50 tokens）
    Compact,    // + 状态摘要 + 2 条相关记忆（~200 tokens）
    Standard,   // + 人格概要 + 5 条记忆 + 关系摘要（~500 tokens）
    Full,       // 完整角色卡 + 历史（~1500 tokens）
}

fn determine_context_size(hook: &Hook, priority: HookPriority) -> ContextSize {
    match (hook.complexity(), priority) {
        (Simple, _) => ContextSize::Minimal,
        (Medium, Background) => ContextSize::Compact,
        (Medium, Normal) => ContextSize::Compact,
        (Complex, Background) => ContextSize::Compact,
        (Complex, Normal) => ContextSize::Standard,
        (_, Critical) => ContextSize::Full,
    }
}
```

**效果**：~40% 的 hook 用 Minimal/Compact 上下文，Token 消耗再降 35%。

---

## 八、推测执行（L6）

### 8.1 马尔可夫预测器

基于历史事件预测未来最可能发生的 hook，提前发 LLM：

```rust
struct SpeculativeEngine {
    prediction_model: MarkovPredictor,
    prefetch_queue: Vec<PendingHook>,
    cache: HashMap<Uuid, HookResult>,
}

impl SpeculativeEngine {
    fn predict_and_prefetch(&mut self, world: &WorldState) {
        let predictions = self.prediction_model.predict_next(world, top_k: 5);
        
        for pred in predictions {
            if pred.probability > 0.4 {
                tokio::spawn(async {
                    let result = call_llm(pred.hook).await;
                    self.cache.insert(pred.hook.id, result);
                });
            }
        }
    }
}

struct MarkovPredictor {
    transition_matrix: HashMap<(EventType, EventType), f64>,
    window: VecDeque<EventType>,
}
```

**效果**：~30% 的 hook 在触发时 LLM 调用已完成。用户感知延迟降为 0。

---

## 九、上下文复用（L7）

同一 tick 内、同一角色的多个 hook 合并为一次 LLM 调用：

```rust
async fn batch_character_hooks(
    char: &Character,
    hooks: Vec<Hook>,
) -> Vec<HookResult> {
    let combined_prompt = format!(
        "角色：{}\n当前状态：{}\n\n需要同时决策的事件：\n{}",
        char.name,
        char.state_shorthand(),
        hooks.iter()
            .enumerate()
            .map(|(i, h)| format!("{}. {}", i + 1, h.description))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    let response = llm.chat_json(combined_prompt).await;
    response.results
}
```

**效果**：相同角色的 3 个独立 hook → 3 次 LLM 变为 1 次。Token 节省 66%，RPM 节省 66%。

---

## 十、Hook 链剪枝（L8）

用静态分析提前剪掉不可能路径：

```rust
fn prune_hook_branches(hook: &Hook, world: &WorldState) -> Vec<HookBranch> {
    let all_branches = enumerate_all_branches(hook);
    
    all_branches.into_iter().filter(|branch| {
        branch.preconditions().iter().all(|cond| {
            match cond {
                Precondition::ResourceExists(res, min) => {
                    world.resources.get(res) >= *min
                }
                Precondition::CharacterAlive(id) => {
                    world.characters.iter().any(|c| c.id == *id && c.is_alive)
                }
                Precondition::RelationshipAbove(a, b, t) => {
                    world.relationship_graph.trust(a, b) >= *t
                }
            }
        })
    }).collect()
}
```

**效果**：分支数减少 40-60%，LLM 决策空间缩小，Token 节省 50%。

---

## 十一、微批调度（L11）

收集多个 hook 请求，用 DeepSeek batch API 一次发送：

```rust
struct BatchScheduler {
    batch_window: Duration,     // 收集窗口，如 200ms
    max_batch_size: usize,      // 最大批大小，如 20
    pending: Vec<PendingHook>,
}

impl BatchScheduler {
    async fn schedule(&mut self, hook: PendingHook) -> HookResult {
        self.pending.push(hook);
        
        if self.pending.len() >= self.max_batch_size {
            return self.flush().await;
        }
        tokio::time::sleep(self.batch_window).await;
        self.flush().await
    }
    
    async fn flush(&mut self) -> HookResult {
        let batch = std::mem::take(&mut self.pending);
        let response = llm.batch_chat(
            batch.iter().map(|h| h.build_prompt()).collect()
        ).await;
        
        batch.into_iter().zip(response.results)
            .map(|(hook, result)| hook.resolve(result))
            .collect()
    }
}
```

**效果**：DeepSeek batch API 折扣 50-70%，有效吞吐量翻倍。

---

## 十二、模仿学习（L12）

让规则引擎学会 LLM 的决策模式，逐步替代 routine 决策：

```rust
struct ImitationLearner {
    decision_history: HashMap<DecisionKey, Vec<HistoricalDecision>>,
    decision_tree: Option<DecisionForest>,
    rebuild_interval: u64,
}

impl ImitationLearner {
    fn record(&mut self, hook: &Hook, result: &HookResult) {
        let key = DecisionKey {
            hook_type: hook.hook_type,
            context: hook.context_light_hash(),
        };
        self.decision_history.entry(key).or_default()
            .push(HistoricalDecision { result: result.clone(), tick: current_tick });
    }
    
    fn try_imitate(&self, hook: &Hook) -> Option<HookResult> {
        let tree = self.decision_tree.as_ref()?;
        tree.predict(hook.context_features())
    }
    
    fn rebuild(&mut self) {
        let features = self.extract_features();
        let labels = self.extract_labels();
        self.decision_tree = Some(build_random_forest(&features, &labels));
    }
}
```

**效果**：1000+ 次 LLM 调用后，~40% 的 routine hook 被决策树替代。推理耗时 50μs。

---

## 十三、认知层级（L14）

### 13.1 四级认知架构

不是所有决策都需要 LLM：

```rust
enum CognitionLevel {
    /// 反射——零计算，零 LLM。纯生理反应，如冷了缩脖子
    Reflex,
    /// 习惯——决策树，零 LLM。历史中已见过 10+ 次相似场景
    Habit,
    /// 规则——if-then 引擎，零 LLM。有明确规则可循
    Rule,
    /// 规划——LLM 调用。以上都不满足
    Deliberate,
}

fn classify_cognition_level(hook: &Hook, learner: &ImitationLearner) -> CognitionLevel {
    if hook.is_physiological_urge() { return CognitionLevel::Reflex; }
    if learner.seen_count(hook) > 10 { return CognitionLevel::Habit; }
    if rule_engine.match_rule(hook).is_some() { return CognitionLevel::Rule; }
    CognitionLevel::Deliberate
}
```

### 13.2 各层级占比

| 层级 | 决策占比 | 单次耗时 | LLM 消耗 |
|------|---------|---------|---------|
| Reflex | ~50% | <1μs | 零 |
| Habit | ~30% | ~50μs | 零 |
| Rule | ~10% | ~10μs | 零 |
| Deliberate | ~10% | ~300ms | 3-5 RPM |

**效果**：90% 的 hook 不消耗 LLM。200 角色的 RPM 需求从 60 降至 3-5。

---

## 十四、差分 LLM（L15）

不每次都让 LLM 从零推理，只输出相对上次的变化量：

```rust
fn delta_prompt(prev_decision: &Decision, actual_outcome: &Outcome) -> Vec<Message> {
    vec![
        Message::system("你是微调决策器。上周期的决策效果已反馈。判断是否需要修正，输出增量调整。"),
        Message::user(format!(
            "上次决策：{}\n预期结果：{}\n实际结果：{}\n判断：继续 修正 终止？",
            prev_decision.summary,
            prev_decision.expected_outcome,
            actual_outcome
        )),
    ]
}
```

**效果**：增量决策输出通常只需 1-3 个 token。Token 消耗降低 95%。

---

## 十五、注意力稀疏（L16）

每 tick 只关注 top-K 个最重要的 hook：

```rust
fn select_top_k_hooks(world: &WorldState, k: usize) -> Vec<Hook> {
    let all_hooks = collect_active_hooks(world);
    
    let mut scored: Vec<_> = all_hooks.into_iter()
        .map(|hook| {
            let score = hook.narrative_importance(world)
                * hook.urgency()
                * (1.0 + hook.cascade_potential(world));
            (hook, score)
        })
        .collect();
    
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.into_iter().take(k).map(|(hook, _)| hook).collect()
}
```

**效果**：200 角色 → 每 tick 只处理 top-5 hook。RPM 从 60 降至 5。

---

## 十六、GPU 加速方案

### 16.1 适用场景

| 场景 | CPU | GPU | 加速比 | 建议 |
|------|-----|-----|--------|------|
| 200 agent 状态更新 | ~3μs | ~55μs(含启动) | 0.05x | CPU |
| 全对全社会 O(n²) | ~12μs | ~55μs | 0.2x | CPU |
| 1024 集合预报 | ~2s | ~2ms | 1000x | **GPU** |
| Hook 分支 512 路展开 | ~500ms | ~2ms | 250x | **GPU** |
| 神经缓存推理 (1M param) | ~2ms | ~0.1ms | 20x | 可选 |
| 本地 LLM 推理 (7B q4) | 5-8 tok/s | 35-105 tok/s | 10-15x | 如果离线 |

### 16.2 架构

```rust
enum SimulationMode {
    /// 纯 CPU，适用 5-200 agent
    CpuOnly,
    
    /// GPU 用于集合预报 + 分支展开
    GpuAccelerated {
        forecast_count: u32,     // 集合预报数
        branch_expansion: u32,   // 因果链展开路径数
    },
    
    /// GPU 用于所有数值计算 + 本地 LLM
    FullGpu {
        forecast_count: u32,
        branch_expansion: u32,
        local_model_path: String,
        gpu_layers: u32,
    },
}
```

### 16.3 无 GPU 降级策略

| GPU 函数 | 有 GPU | 无 GPU 替代 |
|---------|--------|------------|
| Hook 分支 512 路展开 | 2ms | 展开前 16 路，20ms |
| 集合预报 1024 世界 | 2ms | 只跑 1 条，不预报 |
| SOC 压力计算 | 全量并行 | 串行，差异可忽略 |
| 神经缓存 | 0.1ms/推理 | 2ms/推理，CPU |

**影响**：无 GPU 丢的是"预测能力"和"大规模并行概率展开"，不影响核心执行。降速约 30-50%，不降功能。

---

## 十七、全局 RPM 账本

```rust
#[derive(Default)]
struct RpmLedger {
    layers: HashMap<Layer, LayerBudget>,
    stats: RpmStats,
}

impl RpmLedger {
    fn allocate(&mut self, active_hooks: &[ActiveHook]) -> Allocation {
        let critical_count = active_hooks.iter()
            .filter(|h| h.priority == Critical).count();
        let normal_count = active_hooks.iter()
            .filter(|h| h.priority == Normal).count();
        
        let critical_budget = (critical_count as f64 * 1.0)
            .min(self.layers[Layer::L3].max_rpm as f64);
        
        let normal_budget = (self.layers[Layer::L3].max_rpm as f64 - critical_budget)
            .max(0.0) / (normal_count.max(1) as f64);
        
        Allocation { critical_budget, normal_budget }
    }
}
```

---

## 十八、损益表

| 指标 | 朴素实现 | 全量优化 | 改善 |
|------|---------|---------|------|
| RPM/tick | ~60 | 3-5 | **-92~95%** |
| Token/tick | ~48000 | 1500-3000 | **-94~97%** |
| 延迟/tick | 3-5s | 0s(感知) / 30ms(LLM) | **~0 感知** |
| API 成本 | 1x | ~0.03x | **-97%** |
| 200 角色并发 | 卡顿 | 流畅 | **质变** |

### 最终一句话

200 角色 × 世界并行运转，到了这个级别，90% 的决策是反射和习惯，9% 是规则，1% 才需要 LLM。把这 99% 砍掉，剩下的自然跑得动。
