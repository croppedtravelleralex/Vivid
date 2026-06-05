# Vivid — 小说世界模拟引擎

基于 Rust + Vue 3 的小说世界时间线模拟引擎。让角色在无人干预的情况下自发行动、感知、思考，产出可用于写作的素材。

## 快速开始

### 后端
```bash
cd backend
cp .env.example .env
# 编辑 .env 填入 LLM_API_KEY
cargo run
```

### 前端
```bash
cd frontend
npm install
npm run dev
```

## 目录结构
- `backend/` — Rust 模拟引擎
- `frontend/` — Vue 3 Dashboard
- `data/` — 世界设定与角色卡
- `docs/` — 设计文档

## 设计文档
详见 `design/` 目录。
