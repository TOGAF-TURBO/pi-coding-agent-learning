# pi_agent_rust vs piso 技术对比评价

> 生成日期: 2026-05-09
> 对比版本: pi_agent_rust v0.1.15 vs piso (commit 3e32979)

---

## 1. 项目定位

| 维度 | pi_agent_rust (Dicklesworthstone) | piso (本项目) |
|------|-----------------------------------|---------------|
| 定位 | 生产级 Pi Agent Rust 替代品 | 学习型 Rust 重构项目 |
| 星数 | 885 ⭐ | 0 ⭐ |
| Fork | 104 | 0 |
| 开发者 | Jeffrey Emanuel (单人+AI) | 学习者+AI |
| 开发周期 | ~4 个月 (2026-02 起) | ~2 周 (2026-05) |
| 许可证 | MIT + Rider (自定义) | 学习用途 |
| 目标用户 | OpenClaw 企业用户 | 个人学习 |

## 2. 架构对比

| 维度 | pi_agent_rust | piso |
|------|---------------|------|
| Crate 结构 | 1 个巨型 crate (461 .rs 文件) | 8 crate workspace (84 .rs 文件) |
| 代码量 | ~80,000+ 行 (估算) | ~20,700 行 |
| 异步运行时 | asupersync (自建) | tokio |
| TUI 框架 | charmed-rust (BubbleTea 移植) | ratatui |
| 渲染库 | rich_rust (自建 Rich 移植) | 内建 markdown.rs |
| 会话存储 | SQLite + JSONL v2 sidecar | 纯 JSONL |
| 扩展运行时 | QuickJS 嵌入 + WASM + native | 无运行时 (仅 Rust trait) |
| HTTP 客户端 | asupersync 内置 | reqwest |
| 安全模型 | capability-gated hostcall + exec mediation | 无 (信任所有工具) |

## 3. 功能覆盖

| 功能 | pi_agent_rust | piso | 评价 |
|------|---------------|------|------|
| LLM Providers | 11 (含 Copilot/Cohere/GitLab) | 7 (Anthropic/OpenAI/Gemini/Azure/Bedrock/Vertex/Cloudflare) | 他们覆盖更多 |
| 工具 | 8 (含 hashline_edit) | 8 (bash/read/write/edit/find/grep/ls + extension_tool) | 持平 |
| 会话 | SQLite + JSONL + branching + v2 sidecar | JSONL + branching | 他们远超我们 |
| 扩展系统 | QuickJS + WASM + native + 224 扩展目录 | Rust trait + 注册表 | 完全不同级别 |
| TUI | BubbleTea + rich_rust | ratatui 4 区域 | 他们更成熟 |
| 模型选择器 | Ctrl+L + Ctrl+P/Ctrl+Shift+P | Ctrl+P | 我们简化版 |
| 并行工具 | 8-256 并行度自适应 | ExecutionMode 枚举 + futures::join_all | 他们更智能 |
| 压缩 | 后台 worker + 配额 + 结构化模板 | 结构化 Goal/Progress 模板 | 他们异步化 |
| 进程管理 | 完整 process tree + resource governor | setsid() + kill(-pgid) | 他们多了资源治理 |
| 权限系统 | capability-gated + trust lifecycle + kill switch | 无 | 他们独有 |
| @file 引用 | 支持 | 支持 + 图片 base64 + 1MB 限制 | 持平 |
| 安全检查 | exec mediation + shell pattern blocking | 无 | 他们独有 |
| 安装器 | curl + sigstore + TS 迁移 | 手动 cargo build | 他们远超我们 |

## 4. 工程质量

| 维度 | pi_agent_rust | piso |
|------|---------------|------|
| 测试文件 | 302 个 | 8 个 crate 内嵌 |
| 测试数量 | 数千 (302 文件 × 多个 test fn) | 337 |
| CI/CD | 完整 pipeline + evidence bundle + release gate | 无 |
| Benchmark | 完整对比框架 + 基准矩阵 | 无 |
| 扩展验证 | 3 阶段 pipeline (vendored/unvendored/live) | 无 |
| 文档 | 2500 行 README + docs/ 子目录 | 多个 .md 文件 |
| Clippy | ✅ | ✅ |
| unsafe | 禁止 (项目信条) | 最少使用 (libc::kill) |
| Release 优化 | opt-level="z" + LTO + strip | 默认 release |
| 二进制大小 | ~21.1 MiB | ~13 MiB |

## 5. 技术亮点对比

### pi_agent_rust 独有

1. **asupersync 自建运行时** — structured concurrency + 内置 HTTP/TLS，不依赖 tokio
2. **QuickJS 嵌入** — 真正运行 JS/TS 扩展，无需 Node.js
3. **WASM 扩展** — wasmtime 集成
4. **SQLite 会话** — O(index+tail) 大会话重开
5. **Extension 安全模型** — 3 层防护 (capability + exec mediation + trust lifecycle)
6. **Hostcall reactor mesh** — 确定性分片路由 + 背压遥测
7. **Benchmark 治理** — CI 级性能回归检测
8. **sigstore 签名** — 安装包签名验证
9. **ast-grep** — 代码搜索扩展
10. **VCR 测试** — 录制/回放 LLM 响应

### piso 独有

1. **8 crate workspace** — 真正的模块化，crates.io 可发布
2. **集中式 transform.rs** — 消除 6 个 driver 336 行重复
3. **dual-loop steering** — 内层 try_recv + 外层 recv.await，精确匹配 TS 版
4. **ModelChange JSONL entry** — 热切换记录，重放一致
5. **archived_range 压缩** — append-only JSONL 逻辑删除
6. **ExecutionMode 枚举** — 工具级并行/串行声明

## 6. 代码质量评价

### pi_agent_rust

- **复杂度过高**：单 crate 461 个 .rs 文件，耦合严重
- **过度工程**：hostcall_s3_fifo、hostcall_io_uring_lane、swarm_flight_recorder 等模块
  对 CLI 工具来说属于过度设计
- **AGENTS.md 令人担忧**：RULE 0 ("我说了算") + RULE 1 ("永不删文件")
  暗示 AI agent 失控问题严重
- **302 个测试文件过多**：许多是生成代码（ext_conformance_generated），实际
  覆盖率不如数量看起来那么高
- **命名不一致**：项目叫 pi_agent_rust 但二进制叫 pi，与原版冲突

### piso

- **简洁克制**：84 文件 20K 行，每个模块职责明确
- **学习目的明确**：不是为了生产使用
- **模块化好**：8 crate 真正解耦
- **测试覆盖不足**：337 个测试 vs 他们的数千个
- **无安全模型**：信任所有工具执行

## 7. 总结评分

| 维度 | pi_agent_rust | piso | 说明 |
|------|:---:|:---:|------|
| 功能完整度 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | 他们是企业级 |
| 架构质量 | ⭐⭐⭐ | ⭐⭐⭐⭐ | 我们模块化更好 |
| 代码简洁度 | ⭐⭐ | ⭐⭐⭐⭐⭐ | 我们更简洁 |
| 安全性 | ⭐⭐⭐⭐⭐ | ⭐⭐ | 他们有完整安全模型 |
| 测试覆盖 | ⭐⭐⭐⭐ | ⭐⭐⭐ | 他们数量碾压 |
| 扩展性 | ⭐⭐⭐⭐⭐ | ⭐⭐ | 他们有 3 种运行时 |
| 可维护性 | ⭐⭐ | ⭐⭐⭐⭐ | 我们小而清晰 |
| 生产就绪度 | ⭐⭐⭐⭐ | ⭐ | 他们可发布 |
| 文档质量 | ⭐⭐⭐⭐ | ⭐⭐⭐ | 他们更全面 |
| 学习价值 | ⭐⭐ | ⭐⭐⭐⭐⭐ | 我们是学习项目 |

## 8. 核心结论

**pi_agent_rust 是工业级产品**，适合需要替代 TS 版 Pi 的企业用户。它的安全模型、
扩展系统、性能优化和测试体系远超 piso。但它也存在过度工程问题——单 crate 461
文件、自建运行时、224 扩展验证管线——这些对一个 CLI 工具来说可能是过度设计。

**piso 是学习项目**，架构更清晰（8 crate），代码更简洁（20K 行），适合理解
coding agent 的核心原理。但缺少安全模型、扩展运行时、完善的 CI/CD，不适合生产使用。

两者目标不同，不应直接比较"谁更好"。piso 的价值在于**教学和实验**，
pi_agent_rust 的价值在于**生产替代**。
