# FairyField 转入 OMX 前迁移计划

> 状态：待执行  
> 适用仓库：`/Users/fallfield/Desktop/Projects/FairyField/FairyField`  
> 制定日期：2026-07-12  
> 目标：完整保存 V1，清除规则冲突和能力误报，在独立分支建立可由 OMX 持续开发的 V2 基线。

## 1. 迁移完成的定义

迁移不是重装 OMX，也不是一次大规模删库重写。完成迁移必须同时满足：

1. V1 的源码、文档和未提交工作都有可恢复的 Git 引用。
2. V1 的 `v1.0.0` 标签保持不动，不把当前工作区伪装成已发布版本。
3. V2 在独立分支开发，版本标记为预发布版本。
4. 项目只保留一套工作规则，OMX 是编排层，项目文档是产品与工程事实来源。
5. 所有“已实现”能力都由自动测试或可重复的人工证据支持。
6. 未完成、占位、仅有接口或缺少凭据的能力不出现在发布功能列表中。
7. V2 的首个里程碑能独立运行、取消、报告进度，并能证明任务是否真正完成。

## 2. 当前基线

迁移执行前必须重新确认下表，不可直接沿用本文日期后的旧结论。

| 项目 | 2026-07-12 已知状态 | 迁移要求 |
|---|---|---|
| 主分支 | `main` | 不在脏工作区直接重构 |
| 当前提交 | `60aac49` | 记录完整 SHA |
| 最近标签 | `v1.0.0`（`25ecb6c`，比 `main` 落后 5 个提交） | 不移动、不覆盖 |
| 描述 | `v1.0.0-5-g60aac49-dirty` | 说明标签后已有提交和未提交改动 |
| 版本号 | npm/Cargo/Tauri 均为 `1.0.0` | 建立 V2 后改为 `2.0.0-alpha.1` |
| 工作区 | 15 个已跟踪文件修改、4 组未跟踪路径，约 1,333 行新增/212 行删除 | 逐项归属，禁止 `git add .` |
| 本地备份 | 存在 pre/post Hermes loop stash | 把需要长期保存的内容转为分支和标签，不能只依赖 stash |
| 最近验证 | 前端 102 tests；Rust 默认 425、Sherpa 427；build/check/fmt/clippy 通过 | 迁移前重新运行并保存结果 |
| Release build | 未运行 | 迁移阶段禁止运行 Tauri release build |

### 2.1 当前主要风险

- `AGENTS.md` 是 OMX 生成的操作契约，`CLAUDE.md` 仍包含旧式强制多 Agent、Superpowers 和文档同步规则，两者会互相覆盖。
- 现有文档对“完成”“可用”“stub”“需配置”的定义不统一。
- 项目同时存在成熟模块、半成品集成、占位接口和未经端到端验证的功能，不能按文件数量判断完成度。
- `.codex/`、`.omx/`、构建产物、模型、日志和本地密钥的跟踪边界尚未正式裁定。
- 当前未提交代码与文档可能包含有价值修复，不能通过清空工作区来获得“干净”的假象。
- 当前属于“半接入 OMX”：plugin mode、本机 marketplace 配置和 setup 生成的 native agent 文件并存，需要由 `omx doctor` 和可重建性测试裁决。
- 现有 CI 使用的 Rust 版本与文档要求不一致，安全审计路径也需要确认是否实际覆盖 `src-tauri/Cargo.lock`。

## 3. 不可违反的迁移原则

1. **先保存，后清理。** 删除内容前必须有可推送的备份分支或标签。
2. **先验证，后宣称。** 编译通过不等于功能可用；UI 出现也不等于后端完成。
3. **小步替换。** 每个迁移批次必须可以测试、提交和回滚。
4. **核心优先。** Agent 闭环、工具安全、记忆可信度和语音链路优先于集成数量。
5. **配置与秘密分离。** 模型、令牌、日志、SQLite 数据库和本机状态不进入 Git。
6. **不盲目复制参考项目。** Hermes/MemPalace 只提供设计参考；采用前记录差异、许可证和 Rust/Tauri 适配方式。
7. **OMX 不替代工程责任。** Agent 可以并行执行，最终仍需要单一集成负责人和证据门槛。

## 4. 迁移阶段总览

| 阶段 | 目标 | 产物 | 退出条件 |
|---|---|---|---|
| P0 | 冻结现场 | 清单、测试记录、秘密扫描 | 所有改动已归属 |
| P1 | 创建恢复点 | 备份分支、快照标签、远端引用 | 能从新克隆恢复 |
| P2 | 裁决规则与文档 | 单一规则链、文档保留表 | 无冲突指令 |
| P3 | 盘点代码资产 | 保留/修复/重写/隔离/删除矩阵 | 每个公开能力有归属 |
| P4 | 完成技术决策 | 7 份 ADR 或明确结论 | V2 架构无关键悬案 |
| P5 | 建立 V2 基线 | 新分支、最小目录、CI、状态页 | 干净克隆可验证 |
| P6 | 迁移验收 | 迁移报告、首批任务 | 可以进入 OMX 开发手册 |

## 5. P0：冻结和审计现场

### P0.1 记录 Git 现场

保存以下输出到迁移临时记录，不要先修改工作区：

```bash
git rev-parse --show-toplevel
git branch --show-current
git rev-parse HEAD
git describe --tags --always --dirty
git status --short
git diff --stat
git diff --cached --stat
git diff --name-status
git stash list
git remote -v
```

### P0.2 对未提交文件逐项分类

每个文件只能进入一个类别：

| 类别 | 含义 | 处理方式 |
|---|---|---|
| A：V1 修复 | 已验证、应保留的源码或测试 | 进入 V1 收尾提交 |
| B：迁移文档 | 本次 OMX 迁移需要的规则和计划 | 独立文档提交 |
| C：实验 | 有价值但未验证 | 保存到实验分支，不合入 V1 发布事实 |
| D：本地状态 | 日志、缓存、数据库、模型、生成状态 | 加入 `.gitignore`，不提交 |
| E：秘密 | API key、token、个人路径敏感内容 | 立即轮换并从历史候选中排除 |
| F：无用 | 空文件、重复生成物、可重建产物 | 在快照后删除 |

必须使用 `git diff -- <file>` 审查，不使用 `git add .`、`git add -A` 或一次性全选。

### P0.3 运行迁移前验证

只执行开发与测试命令：

```bash
npm run test
npm run build
cd src-tauri
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
cargo check --features sherpa-onnx
cargo test --features sherpa-onnx
cargo clippy --features sherpa-onnx -- -D warnings
```

禁止在本阶段运行 `npm run tauri build`。桌面人工冒烟测试使用 `npm run tauri dev`，结束后确认前端、Rust 和音频子进程全部退出。

### P0.4 秘密和大文件检查

检查范围包括已跟踪文件、待提交 diff 和 Git 历史候选：

- OpenAI、DeepSeek、Discord、GitHub、OAuth 等令牌。
- `~/.fairyfield` 的真实配置、SQLite 数据库和对话内容。
- `.env`、模型权重、音频样本、构建产物和崩溃日志。
- 绝对用户路径。文档中的仓库路径可以保留，个人令牌路径不可保留。

若发现曾暴露的令牌，删除文本不等于修复；必须先轮换令牌，再决定是否清理 Git 历史。

## 6. P1：创建可恢复的 V1 快照

### P1.1 保存当前主线引用

推荐命名：

```text
archive/v1-source-candidate-20260712
snapshot/pre-omx-v2-20260712
v2/omx-redevelopment
```

执行顺序：

1. 从 `main` 建立 `archive/v1-source-candidate-20260712`。
2. 记录基线文件哈希和完整 diff；不能只记录“启动时已经脏的文件名”。
3. 在该分支按 A/B/C 分类形成多个可审阅提交，禁止把秘密和本地状态提交进去。
4. 完整测试通过后，在归档分支创建带说明的标签 `snapshot/pre-omx-v2-20260712`。
5. 推送归档分支和快照标签，确认 GitHub 可见。
6. 从确认过的快照建立 `v2/omx-redevelopment`。
7. 保留 `main` 和 `v1.0.0` 原样，直到 V2 达到替换门槛。

### P1.2 提交拆分建议

```text
fix(v1): preserve verified agent and tool-loop corrections
docs(v1): record honest release state and known limitations
chore(omx): add migration plans and project OMX contract
chore(snapshot): freeze pre-OMX V1 source candidate
```

每个提交都必须可单独解释和回滚。提交前使用 `git diff --cached` 检查完整内容。

### P1.3 恢复演练

在临时目录重新克隆远端，检出快照标签，至少完成：

- 依赖安装。
- 前端测试和构建。
- Rust 默认特性检查与测试。
- 确认没有依赖未提交的本机文件才能启动。

恢复演练失败时不得进入 P2。

## 7. P2：统一规则和文档

### P2.1 规则优先级

迁移后规则按以下顺序解释：

1. 平台和安全限制。
2. 仓库根目录 `AGENTS.md`：OMX 工作方式与通用工程约束。
3. V2 产品、架构、路线图和状态文档。
4. 具体任务说明和 ADR。
5. 历史文档只作参考，不产生强制规则。

### P2.2 冲突裁决

| 旧规则或习惯 | V2 裁决 |
|---|---|
| 每个任务自动启动所有相关 Agent | 取消。默认直接执行；只有可独立并行且收益明确时才组建 OMX team |
| 所有 Agent 必须独立 worktree | 取消绝对要求。并行写代码时必须隔离；只读审计和单人小改可不建 |
| Phase 完成后同步修改多份重复文档 | 取消。日常只更新 `STATUS.md`；里程碑更新路线图；发布更新 README/CHANGELOG |
| 实现任何功能前必须查 Hermes/MemPalace | 改为相关性规则。Agent、工具、记忆功能必须查；无关 UI 或构建修复不强制 |
| Superpowers 是强制流程 | 移除。项目不再依赖 Superpowers 文档或插件 |
| “零 Python”是永久目标 | 改为 ADR 决策。发布产物默认零 Python；开发/模型服务是否使用由体积、延迟和维护成本决定 |
| “所有功能无 bug”才算完成 | 改为明确发布门槛、已知限制和严重级别；禁止承诺不可证明的零缺陷 |
| 文件存在即表示能力完成 | 取消。必须有可重复的行为证据和失败路径测试 |
| 手工编辑 `.codex/agents` 中 OMX 生成角色 | 禁止。生成目录由 `omx setup` 管理，项目只定义使用哪些角色 |

### P2.3 文档收敛

V2 只维护以下权威文档：

| 文档 | 作用 | 更新时机 |
|---|---|---|
| `README.md` | 对外介绍、真实能力、安装入口 | 发布候选 |
| `AGENTS.md` | OMX 和工程工作契约 | 工作流变化 |
| `docs/v2/PRODUCT.md` | 用户、目标、非目标、产品验收 | 产品决策变化 |
| `docs/v2/ARCHITECTURE.md` | 模块边界、接口、ADR 索引 | 架构决策变化 |
| `docs/v2/ROADMAP.md` | 里程碑、依赖、优先级 | 里程碑调整 |
| `docs/v2/STATUS.md` | 当前事实、阻塞、下一任务 | 每个集成批次 |
| `CHANGELOG.md` | 已发布版本历史 | 正式发布 |

现有指南在快照后按以下方式处理：

- 有效且仍面向用户的内容，在对应功能重新验收后迁入新用户指南。
- 纯历史 Phase 记录不复制到 V2 主线，Git 归档分支已承担历史保存。
- `docs/superpowers/` 在确认无唯一信息后删除。
- `CLAUDE.md` 缩减为指向 `AGENTS.md` 的兼容说明，或在确认没有工具依赖后删除。
- 不建立 `docs/archive/` 来重复保存 Git 已经保存的内容。

### P2.4 OMX 文件跟踪边界

执行 `omx setup` 和 `omx doctor` 后确认哪些内容可以重建：

- 应跟踪：项目需要的最小 `.codex/config.toml`、根 `AGENTS.md`、项目自定义且不可重建的模板。
- 不应跟踪：`.omx/` 运行状态、日志、会话、tmux 状态、备份、缓存和临时报告。
- `.codex/agents/` 与 `.codex/.omx/native-agents.json` 若完全由 setup 生成，默认不手工维护；是否跟踪由恢复演练决定。
- 本机 marketplace 的绝对路径不能作为其他贡献者的唯一安装方式，README 必须提供可移植的安装说明。
- 当前 `.omx/setup-scope.json` 所示 plugin mode 与仍存在的 setup-owned native agent 集合需要专门检查，避免旧角色 shadow plugin 角色。

## 8. P3：代码资产分级

所有模块必须标注为 `Keep`、`Repair`、`Rewrite`、`Quarantine` 或 `Remove`。以下是迁移起点，不替代逐文件审计。

| 领域 | 初始分类 | 理由与动作 |
|---|---|---|
| Tauri/Vue 应用壳 | Keep + Repair | 已能构建；保留窗口、IPC 和组件骨架，清理跨层耦合 |
| Three.js/VRM 渲染 | Keep + Repair | 保留模型、表情、口型和待机动画；补资源失败、帧率和设备测试 |
| LLM provider | Repair | 统一流式事件、错误、超时、取消和 function calling 契约 |
| Primary Agent loop | Rewrite around tested pieces | 保留已验证的 UTF-8/工具循环修复；重建状态机、预算、取消和恢复 |
| Web/File/Git 核心工具 | Repair | 保留现有安全修复；补根目录策略、产物验证和真实 E2E |
| 大量第三方工具 | Quarantine | 未有凭据和 E2E 证据时不进入公开 registry |
| Coding Agent Manager | Rewrite | 从“启动 CLI”提升为工作区、权限、事件、diff、测试和取消的完整运行时 |
| SQLite/FTS 记忆基础 | Keep + Repair | 保留原文存储和索引；重做召回可信度、来源、去重和删除策略 |
| 向量/Embedding 占位路径 | Quarantine | 真实模型、维度、迁移和降级未验证前不宣传 |
| MCP Server | Repair | 采用标准协议、鉴权、生命周期和兼容测试，移除“自称 MCP”的假接口 |
| ASR/VAD/TTS | Rewrite after benchmark | 以中英双语、流式延迟、打断和自然度实测选型，不受旧方案绑定 |
| Discord/Gateway/Cron | Quarantine | 核心闭环稳定后再恢复，避免扩大安全面 |
| Growth/自修改技能 | Quarantine | V2 发布前不允许自主修改生产代码或权限规则 |
| 安全 Guard/Redaction | Keep + Rewrite boundaries | 保留测试资产；把权限、秘密、URL、命令和审计统一到执行边界 |

必须保留并建立兼容测试的 V1 边界：

- Tauri 应用标识 `com.fallfield.fairyfield`，除非明确接受升级断裂。
- `~/.fairyfield/config.json`、`user.json`、`secrets.json`、`fairyfield.db` 和 `models/` 的读取或迁移路径。
- SQLite/FTS 原文记忆、时序事实的失效语义、用户纠正能力和已有 IPC 合约。
- `soul/SOUL.md`、`soul/identity.txt`、默认角色资产、图标、许可证、lockfile 和模型校验逻辑。

当前 `lib.rs` 中可能存在执行 Guard 的重复状态实例。V2 必须验证批准状态与真正执行路径使用同一个安全状态，不能只修 UI 侧实例。

### P3.1 能力真实性标签

每个功能必须使用一个标签：

- `Verified`：自动测试和真实环境验收通过。
- `Beta`：主路径可用，有明确限制和回退。
- `Experimental`：默认关闭，不进入发布承诺。
- `Scaffold`：只有接口或样例，不能注册为用户可用能力。
- `Removed`：不再进入 V2。

## 9. P4：迁移前必须完成的架构决策

每份 ADR 必须包含背景、选项、数据、决定、后果和撤销条件。

| ADR | 决策问题 | 必须获得的证据 |
|---|---|---|
| ADR-001 | 单进程 Rust 还是本地服务边界 | 启动时间、打包体积、崩溃隔离、跨平台成本 |
| ADR-002 | LLM provider 与流式协议 | OpenAI-compatible、Codex 路径、tool call、取消和重试实测 |
| ADR-003 | 工具权限和产物证明 | 路径策略、命令分级、用户批准、审计日志、假成功防护 |
| ADR-004 | 长期记忆最小模型 | 原文、来源、检索、时序事实、删除/导出、隐私测试 |
| ADR-005 | 中英语音技术路线 | ASR WER、TTS 首包延迟、自然度、CPU/内存、打断和模型许可证 |
| ADR-006 | OMX/Codex 工作树模型 | 并发冲突、集成成本、状态恢复和失败清理演练 |
| ADR-007 | V2 平台和发布范围 | macOS 优先还是跨平台；签名、权限、自动更新和 CI 成本 |

ADR-005 未完成前不得继续堆叠 Kokoro/Matcha/系统 `say` 的补丁。目标是选出满足产品指标的链路，不是维护旧技术名称。

## 10. P5：建立 V2 基线

### P5.1 分支与版本

- 开发分支：`v2/omx-redevelopment`。
- 首个可运行骨架版本：`2.0.0-alpha.1`。
- `main` 在 V2 达到发布候选前继续代表 V1 稳定线。
- 不创建 `v2.0.0` 标签，直到全部 release gate 通过。

### P5.2 最小模块边界

V2 先建立以下边界，不先恢复 135+ 工具：

```text
frontend: shell / chat / character / settings / diagnostics
backend: runtime / providers / tools / memory / voice / security / platform
contracts: request events / tool events / voice events / error taxonomy
tests: unit / contract / integration / desktop-e2e
```

每个长任务必须有 `request_id`、状态、取消句柄、时间预算和最终证据。前端只消费统一事件，不根据文本猜测工具状态。

### P5.3 首批基础设施

1. 建立统一错误类型和 request/tool event schema。
2. 建立可取消的运行时和结构化进度事件。
3. 建立最小 provider mock、tool mock 和 deterministic test harness。
4. CI 执行前端测试、Rust 测试、格式、lint、秘密扫描和许可证检查。
5. 建立 `docs/v2/STATUS.md`，记录真实能力、阻塞和下一任务。
6. 建立功能注册门：没有 manifest、权限、测试和失败语义的工具不能注册。

## 11. P6：迁移验收清单

只有全部勾选后才进入日常 OMX 开发：

- [ ] 当前脏工作区每个文件已有分类和负责人。
- [ ] V1 归档分支和快照标签已推送。
- [ ] 从远端快照完成恢复演练。
- [ ] 已轮换或排除所有发现的秘密。
- [ ] `v1.0.0` 标签未被移动。
- [ ] OMX `setup` 和 `doctor` 通过，其他机器有可移植安装说明。
- [ ] plugin mode 不再被陈旧 native-agent 生成文件遮蔽。
- [ ] `AGENTS.md` 成为唯一工作契约，旧规则不再产生冲突。
- [ ] Superpowers 文档和依赖已退出活动工作流。
- [ ] 代码资产分级完成，公开能力列表不含 Scaffold。
- [ ] 7 个 ADR 已决定，或有明确负责人、截止时间和阻塞关系。
- [ ] V2 分支版本为预发布版本。
- [ ] 新克隆可完成基础测试，且不依赖本机秘密或未跟踪源码。
- [ ] V1 配置和数据库副本能在 V2 迁移测试中打开，且能回滚。
- [ ] CI 使用的 Rust 版本与 `rust-version` 一致，并实际审计 `src-tauri/Cargo.lock`。
- [ ] `STATUS.md` 列出第一个可执行任务及验收标准。

## 12. 回滚和停止条件

遇到以下情况立即停止迁移，不继续清理：

- 发现未备份且无法确认归属的源码修改。
- 快照无法从远端恢复。
- 测试结果比记录基线退化且原因不明。
- 发现秘密进入历史但尚未轮换。
- OMX 生成文件与项目手写文件边界不清。
- 关键 ADR 仍在互相矛盾，但实现已经依赖其中一个结论。

回滚时检出 `snapshot/pre-omx-v2-20260712` 或归档分支。禁止用 `git reset --hard` 清理不明改动；先保存补丁或新分支。

## 13. 迁移职责

| 角色 | 职责 |
|---|---|
| 项目所有者 | 确认产品范围、秘密轮换、是否推送和发布 |
| 迁移负责人 | 维护清单、提交边界、规则裁决和最终验收 |
| OMX Planner/Architect | 拆解 ADR、依赖和模块边界 |
| Executor | 在明确文件范围内完成迁移任务 |
| Test Engineer | 维护基线、恢复演练和回归矩阵 |
| Code Reviewer | 检查行为退化、安全和不可逆删除 |
| Verifier | 根据证据判断是否达到退出条件，不以口头总结代替测试 |

## 14. 迁移结束后的入口

迁移通过后，所有新开发按 `docs/omx/DEVELOPMENT_PLAYBOOK.md` 执行。本文保留为迁移审计记录，不继续扩展为开发路线图。
