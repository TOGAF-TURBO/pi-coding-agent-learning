# pi_agent_rust vs TypeScript pi 原版 深度对比分析

> 生成日期: 2026-05-09
> 对比版本: TypeScript pi v0.74.0 vs pi_agent_rust v0.1.15

---

## 1. 项目概况

| 维度 | TypeScript pi (原版) | pi_agent_rust (移植) |
|------|---------------------|---------------------|
| 版本 | 0.74.0 | 0.1.15 |
| 开发者 | Mario Zechner (badlogic) | Jeffrey Emanuel (Dicklesworthstone) |
| 开发周期 | 1.5+ 年 | ~4 个月 |
| 源代码 | **309 文件, 114,214 LOC** (src/) | **461 文件, ~250,000 LOC** (估算) |
| 测试代码 | 219 文件, 58,741 LOC | 302 文件, ~10MB |
| 包结构 | **5 个 npm 包**（monorepo） | **1 个 crate** |
| 运行时依赖 | Node.js 18+ | **无**（单二进制） |
| node_modules | 586 MB | 0 |
| 许可证 | Apache-2.0 | MIT + Rider |

## 2. 架构对比

### TypeScript pi 的 5 包拆分

```
@earendil-works/pi-ai         (50 文件, 31,716 LOC) — LLM streaming, providers, types
@earendil-works/pi-agent-core (26 文件,  8,110 LOC) — agent loop, session, compaction
@earendil-works/pi-tui        (25 文件, 11,971 LOC) — TUI components, editor, markdown
@earendil-works/pi-web-ui     (71 文件, 14,971 LOC) — Web UI components
@earendil-works/pi-coding-agent (137 文件, 47,446 LOC) — CLI, tools, extensions, modes
```

### pi_agent_rust 的单 crate

```
pi_agent_rust (461 文件, ~250K LOC) — 全部功能合在一起
```

**关键差异**：TS 版的 5 包拆分使每个包可独立发布、独立版本化、独立测试。pi_agent_rust 选择单 crate 便于内部共享，但失去了模块化边界。

## 3. 核心模块对比

| 模块 | TS pi (LOC) | pi_agent_rust (LOC) | 比率 | 评价 |
|------|-------------|---------------------|------|------|
| Agent Loop | 660 | 10,224 | **15x** | Rust 版膨胀严重 |
| Tools | ~2,000 (15 文件) | 11,251 | **5.6x** | Rust 版含 hashline_edit + 大量安全检查 |
| Interactive Mode | 5,787 + 14,822 组件 | 3,276 + 14,375 子目录 | ~1x | 两者相当 |
| Session | ~800 (JSONL) | 12,104 + 955 (SQLite) + 2,125 (v2) | **19x** | Rust 版加了 SQLite + v2 sidecar |
| Provider | ~3,000 (17 文件) | 829 + 3,002 (11 providers) | ~1.3x | 持平 |
| Compaction | 2,769 | 2,489 | ~1x | 持平 |
| Extensions | 3,505 | 28,672 (JS) + 14 个 extension_*.rs | **10x+** | Rust 版有完整 JS 运行时+安全+评分 |
| Auth | 552 | 10,021 | **18x** | Rust 版含完整 OAuth refresh + 多 provider |
| RPC | 754 | 7,838 | **10x** | Rust 版协议更复杂 |
| Config | 1,074 | 3,456 | 3.2x | Rust 版多了安全策略配置 |

**结论**：pi_agent_rust 的代码量是 TS 原版的 **2-15 倍**（按模块），主要膨胀来自安全子系统（hostcall、exec mediation、trust lifecycle）和扩展验证管线。

## 4. 功能覆盖对比

### 4.1 LLM Providers

| Provider | TS pi | pi_agent_rust |
|----------|:-----:|:-------------:|
| Anthropic (Messages) | ✅ | ✅ |
| OpenAI (Completions) | ✅ | ✅ |
| OpenAI (Responses) | ✅ | ✅ |
| Google Gemini | ✅ | ✅ |
| Azure OpenAI | ✅ | ✅ |
| Amazon Bedrock | ✅ | ✅ |
| Google Vertex | ✅ | ✅ |
| Cloudflare Workers AI | ✅ | ❌ |
| Mistral | ✅ | ❌ |
| GitHub Copilot | ✅ (headers) | ✅ (完整 provider) |
| OpenAI Codex | ✅ (Responses) | ❌ |
| Cohere | ❌ | ✅ |
| GitLab | ❌ | ✅ |

**TS 版 10 个 provider，Rust 版 11 个**。各有独家 provider。

### 4.2 工具

| 工具 | TS pi | pi_agent_rust |
|------|:-----:|:-------------:|
| read | ✅ | ✅ |
| write | ✅ | ✅ |
| edit | ✅ | ✅ |
| bash | ✅ | ✅ |
| grep | ✅ | ✅ |
| find | ✅ | ✅ |
| ls | ✅ | ✅ |
| hashline_edit | ❌ | ✅ |

**持平**（7 共同 + Rust 多了 hashline_edit）。

### 4.3 会话

| 特性 | TS pi | pi_agent_rust |
|------|:-----:|:-------------:|
| JSONL 持久化 | ✅ | ✅ |
| 会话分支 (tree) | ✅ | ✅ |
| SQLite 索引 | ❌ | ✅ |
| v2 sidecar (segmented log) | ❌ | ✅ |
| O(index+tail) 大会话重开 | ❌ | ✅ |
| 会话 metrics | ❌ | ✅ |

**Rust 版会话系统远超 TS 版**。SQLite + v2 sidecar 解决了 JSONL 在超长会话中的性能问题。

### 4.4 扩展系统

| 特性 | TS pi | pi_agent_rust |
|------|:-----:|:-------------:|
| JS/TS 扩展运行时 | ✅ (Node.js 进程) | ✅ (**QuickJS 嵌入**) |
| WASM 扩展 | ❌ | ✅ (wasmtime) |
| Native descriptor | ❌ | ✅ |
| 事件钩子 | 32 种 | 类似+更多 |
| registerTool | ✅ | ✅ |
| registerCommand | ✅ | ✅ |
| registerShortcut | ✅ | ✅ |
| Capability gates | ❌ | ✅ |
| Exec mediation | ❌ | ✅ |
| Trust lifecycle | ❌ | ✅ |
| Kill switch | ❌ | ✅ |
| 扩展目录 | 无 | 224 个 |
| 验证管线 | ❌ | 3 阶段 pipeline |

**这是最大差异**。TS 版的扩展运行在独立 Node.js 进程中，没有安全模型。Rust 版用 QuickJS 嵌入（无需 Node.js），加了 3 层安全模型和完整的验证管线。

### 4.5 TUI

| 特性 | TS pi | pi_agent_rust |
|------|:-----:|:-------------:|
| 框架 | 自建 (@earendil-works/pi-tui) | charmed-rust (BubbleTea 移植) |
| Markdown 渲染 | 内建 | rich_rust + glamour |
| 模型选择器 | ✅ | ✅ |
| 会话树导航 | ✅ | ✅ |
| 自动补全 | ✅ (@ 引用 + / 命令) | ✅ |
| Diff 查看器 | ✅ | ✅ |
| 图片渲染 | ✅ (终端 Sixel) | ✅ |

**持平**，但实现方式完全不同。TS 版自建 TUI 库，Rust 版用 BubbleTea 移植。

### 4.6 安全模型

| 特性 | TS pi | pi_agent_rust |
|------|:-----:|:-------------:|
| 工具执行沙箱 | ❌ | ✅ (capability-gated) |
| 命令阻止 | ❌ | ✅ (exec mediation) |
| 危险 shell 模式检测 | ❌ | ✅ (AST 分析) |
| 信任状态机 | ❌ | ✅ (pending→trusted→killed) |
| Kill switch | ❌ | ✅ |
| 资源配额 | ❌ | ✅ (resource_governor) |
| 审计日志 | ❌ | ✅ (swarm_activity_ledger) |

**Rust 版安全模型是 TS 版完全没有的维度**。这也是 Rust 版代码膨胀的主要原因。

## 5. pi_agent_rust 独有模块（TS 中不存在）

以下模块在 TS pi 原版中**完全没有对应物**：

| 模块 | LOC | 用途 |
|------|-----|------|
| hostcall_amac.rs | 1,460 | AMAC 并发控制 |
| hostcall_io_uring_lane.rs | 1,033 | io_uring 通道抽象 |
| hostcall_s3_fifo.rs | 1,288 | S3FIFO 缓存替换策略 |
| hostcall_superinstructions.rs | 859 | 超级指令优化 |
| hostcall_trace_jit.rs | 1,364 | Trace JIT 编译 |
| swarm_activity_ledger.rs | 3,560 | Swarm 活动账本 |
| swarm_flight_recorder.rs | 559 | 飞行记录器 |
| resource_governor.rs | 3,916 | 资源治理器 |
| scheduler.rs | 4,347 | 任务调度器 |
| conformance.rs | 4,371 | 一致性验证 |
| conformance_shapes.rs | 2,056 | 一致性形状定义 |
| vcr.rs | 2,786 | VCR 录制/回放 |
| 14 个 extension_*.rs | ~8,000+ | 扩展评分/许可证/流行度/预检 |
| **合计** | **~35,000+ LOC** | **约占总量 14%** |

这些模块大多是**扩展安全子系统和测试基础设施**，与 coding agent 的核心功能无关。

## 6. TS pi 独有能力（Rust 中不存在）

| 特性 | TS pi | pi_agent_rust |
|------|:-----:|:-------------:|
| Web UI (@earendil-works/pi-web-ui) | ✅ | ❌ |
| Mistral provider | ✅ | ❌ |
| Cloudflare provider | ✅ | ❌ |
| OpenAI Codex provider | ✅ | ❌ |
| npm 包独立发布 | ✅ | ❌ (单二进制) |
| 自定义 TUI 库 (pi-tui) | ✅ | ❌ (依赖 charmed-rust) |
| 模型自动发现 | ✅ | 部分实现 |

## 7. 代码质量对比

| 维度 | TS pi | pi_agent_rust |
|------|-------|---------------|
| 单文件最大 | interactive-mode.ts 5,787 行 | extensions_js.rs 28,672 行 |
| 平均文件大小 | ~370 LOC/src/ | ~540 LOC/文件 |
| 模块耦合 | 包边界隔离 | 全部耦合在 1 个 crate |
| 类型安全 | TypeScript strict | Rust borrow checker |
| 错误处理 | try/catch + Result | anyhow + thiserror |
| 测试模式 | vitest 单元测试 | 自建 harness + conformance + VCR |

**最惊人的数字**：`extensions_js.rs` 单文件 28,672 行——比 TS 版整个扩展系统（3,505 行）大 **8 倍**。

## 8. 性能对比（声明 vs 实际）

| 指标 | TS pi | pi_agent_rust (声明) |
|------|-------|---------------------|
| 启动时间 | ~500ms (Node.js) | <100ms |
| 内存占用 (空闲) | ~200MB+ | <50MB |
| 二进制大小 | ~100MB+ (含 runtime) | ~21 MiB |
| 流式解析 | Node stream | 自建 SSE parser |

Rust 版的启动和内存优势是真实的（单二进制 vs Node.js 运行时），但**不可忽视的代价是 2-15 倍的代码复杂度**。

## 9. 开发哲学对比

| 维度 | TS pi | pi_agent_rust |
|------|-------|---------------|
| 设计理念 | **极简主义** — 只做必要的事 | **防御性工程** — 假设一切可能出错 |
| 代码增长 | 按需添加 | 预先过度建设 |
| 扩展安全 | 信任扩展作者 | 0 信任 + 审计 |
| 测试策略 | 覆盖核心路径 | 数千测试覆盖边界 |
| 依赖管理 | npm 生态 | 自建关键库 (asupersync, rich_rust) |
| 模块化 | 5 包独立发布 | 1 crate 全耦合 |

## 10. 总结评价

### TS pi 原版的优点

1. **干净利落** — 114K 行做了该做的事，没有多余
2. **模块化好** — 5 个包独立发布，依赖关系清晰
3. **生态成熟** — npm 包、Web UI、0.74 版本经过实战检验
4. **可扩展** — 包设计允许第三方只用 pi-ai 或只用 pi-tui

### TS pi 原版的不足

1. **Node.js 依赖** — 启动慢、内存大、安装复杂
2. **无安全模型** — 扩展运行在 Node 进程中，无沙箱
3. **JSONL 瓶颈** — 超长会话性能差
4. **单文件过大** — interactive-mode.ts 5,787 行

### pi_agent_rust 的优点

1. **单二进制** — 21MB，无运行时依赖，启动快
2. **安全模型** — 3 层防护，TS 版完全没有
3. **SQLite 会话** — 解决了超长会话性能问题
4. **QuickJS 嵌入** — 无需 Node.js 运行扩展
5. **WASM 支持** — 开辟了新的扩展可能性

### pi_agent_rust 的问题

1. **过度工程** — hostcall_s3_fifo、swarm_flight_recorder、hostcall_trace_jit 等模块对 CLI 工具来说不必要，增加了 ~35K LOC
2. **代码膨胀** — 250K+ 行做 TS 版 114K 行做的事（核心功能），膨胀来自安全子系统
3. **单 crate 耦合** — 461 文件全在一个包里，失去了模块化
4. **extensions_js.rs 28,672 行** — 单文件比 TS 版整个扩展系统大 8 倍
5. **自建依赖多** — asupersync、rich_rust 都是自建的，维护成本高
6. **302 测试文件中大量是生成代码** — 实际手工测试覆盖率不明

### 最终判断

**pi_agent_rust 是 TS pi 的"过度防御版"**。它在安全模型、会话性能、扩展运行时上确实超越了原版。但代价是代码量膨胀 2-3 倍（核心功能），加上 ~35K LOC 的"额外子系统"。单文件 28K 行的 extensions_js.rs 说明项目缺乏拆分纪律。

TS pi 的设计哲学是"做必要的事"。pi_agent_rust 的设计哲学是"假设一切可能出错并预先防御"。前者适合快速迭代，后者适合高安全要求场景。**对于一个 CLI 编码助手，TS 版的平衡点可能更合理。**
