# FairyField OMX 开发执行手册

> 状态：迁移完成后启用  
> 依赖：`docs/omx/PRE_MIGRATION_PLAN.md` 的验收清单全部通过  
> 目标：用 OMX 将 FairyField 从 V2 骨架逐步交付为真实、可测、可取消、可发布的桌面 AI 伴侣。

## 1. 产品目标

FairyField V2 的核心不是工具数量，也不是“像某个项目”的宣传，而是以下用户闭环：

1. 用户可以用文字或语音提出一个目标。
2. Fairy 能理解目标、说明计划、在权限边界内调用工具并持续报告进度。
3. Fairy 能创建或修改真实产物，并在回复前验证产物存在且符合要求。
4. 用户可以随时取消、拒绝高风险操作或纠正计划。
5. Fairy 能在用户授权下保存有来源、可查看、可删除的长期记忆。
6. 角色通过表情、口型和身体动画表达状态，但动画不能掩盖任务错误。
7. 对代码任务，Fairy 能委托 Codex/OpenCode 类运行时，在隔离工作区完成 diff、测试和审阅闭环。

### 1.1 V2 明确非目标

- 首个版本不追求 135+ 工具。
- 不宣称与 ChatGPT Voice 的闭源服务完全相同。
- 不在 V2 MVP 中开放自主修改自身权限或生产代码。
- 不以第三方集成数量替代核心任务成功率。
- 不在没有签名、权限和安装测试时宣称跨平台发布。
- 不把 Hermes/MemPalace 源码直接复制为产品能力。

## 2. 成功标准

### 2.1 产品级标准

| 指标 | V2 Alpha 目标 | V2 Release 目标 |
|---|---:|---:|
| 20 个固定文字任务脚本成功率 | >= 90% | 100% |
| 工具“假成功” | 0 | 0 |
| 用户取消后仍继续执行 | 0 | 0 |
| 严重数据损坏/越权 | 0 | 0 |
| 应用崩溃恢复 | 会话可解释失败 | 可恢复未完成任务状态 |
| 记忆来源可追溯 | 100% 新记忆 | 100% |
| 中英 ASR 固定集准确度 | ADR-005 基线后确定 | 达到已批准阈值 |
| TTS 首段音频延迟 | 中位数 <= 1.5s | 中位数 <= 1.0s |
| 语音打断生效 | <= 500ms | <= 300ms |

外部 API 排队时间必须单独记录，不能混入本地运行时延迟。指标未达到时如实标记 Beta，不通过修改文档数字来“完成”。

### 2.2 工程级标准

- 每个请求有唯一 `request_id`。
- 每个工具调用有开始、进度、结果、错误和取消事件。
- 所有超时均有明确错误类型和可恢复提示。
- 所有高风险操作在执行前获得用户批准。
- 所有文件型任务在最终回复前重新读取或检查产物。
- 默认测试、Sherpa 可选特性测试、前端构建和 lint 均由 CI 执行。
- 发布列表只包含 `Verified` 或经批准的 `Beta` 能力。

## 3. 优先级规则

| 优先级 | 定义 | 示例 |
|---|---|---|
| P0 | 安全、数据、核心闭环或发布阻塞 | 越权写文件、无法取消、假成功、密钥泄漏、崩溃 |
| P1 | MVP 用户路径所需 | 流式聊天、核心工具、记忆、语音、中英识别、Coding Runtime |
| P2 | 质量、可观察性和有限集成 | 性能面板、Discord、GitHub OAuth、模型管理 |
| P3 | 实验和扩展 | 自成长、全息、社区插件市场、大规模第三方工具 |

排序公式：`用户影响 × 失败严重度 × 发生概率 ÷ 工作量`。P0 永远优先于新功能；P1 之间按依赖顺序执行。

## 4. OMX 角色与使用边界

OMX 可以生成很多角色文件，但日常工作只使用一组小而稳定的角色。不要手工删除或修改 setup 管理的角色目录。

| 工作角色 | OMX 角色 | 责任 | 不负责 |
|---|---|---|---|
| Manager | `planner` | 目标、依赖、任务图、状态和集成顺序 | 代替验证宣布完成 |
| Architect | `architect` | 接口、ADR、模块边界和迁移方案 | 日常小修的过度设计 |
| Builder | `executor` | 在限定文件范围实现任务 | 修改其他工作树或扩大范围 |
| Tester | `test-engineer` | 测试设计、失败复现、E2E 证据 | 为了通过而弱化断言 |
| Reviewer | `code-reviewer` | Bug、安全、回归、维护风险 | 只总结改动 |
| Goal Verifier | `verifier` | 对照用户结果和 release gate 独立验收 | 接受“应该可以” |

按需角色：复杂故障用 `debugger`，依赖升级用 `dependency-expert`，文档一致性用 `writer`，探索未知代码用 `explore`。

### 4.1 项目所有权通道

OMX 角色是临时能力，不是第二套永久模块组织。所有任务最终归入以下五个项目通道：

| 项目通道 | 长期所有权 | 常用 OMX 能力 |
|---|---|---|
| Lead/Integrator | 范围、里程碑、共享合约、集成主线 | planner、architect、verifier |
| Agent/Tools | `agent/`、`llm/`、工具执行、Coding Runtime | executor、debugger、code-reviewer |
| Companion Experience | Vue、Three.js、VRM、voice、可访问性 | executor、test-engineer、vision |
| Memory/Connectivity | memory、MCP、gateway、外部集成 | executor、researcher、dependency-expert |
| Assurance/Release | security、测试、Goal 证据、CI、发布文档 | test-engineer、code-reviewer、verifier |

共享 IPC、事件 schema、manifest、版本文件和 release 文件只由 Lead/Integrator 合入。模块 Agent 如需修改共享合约，先提交接口提案。

### 4.2 何时直接执行

满足以下条件时由一个 `executor` 完成：

- 目标清楚。
- 改动集中在一个模块。
- 不涉及架构决策或高风险数据迁移。
- 预计一个工作批次内完成。

### 4.3 何时使用 OMX Team

只有以下情况才组建 team：

- 至少两个可独立写入、文件所有权不重叠的任务。
- 测试设计可与实现并行。
- 需要独立 reviewer/verifier 形成质量闭环。
- 并行收益大于 worktree 和集成成本。

默认最多 4 个同时运行的写入 Agent。复杂里程碑最多 6 个，但必须有 Manager。非 team 模式不要使用 `worker`，使用 `executor`。

### 4.4 OMX 工作流选择

| 情况 | 工作流 |
|---|---|
| 需求或架构仍模糊 | `$ralplan`，先形成可审阅计划 |
| 多个独立模块并行 | `$team`，分配 worktree 和文件所有权 |
| 单一任务需要持续修复直到通过 | `$ralph`，设置明确退出条件 |
| 小型明确改动 | 直接执行，不启动编排仪式 |

CLI team 需要附着的 tmux 环境。无法满足时退回普通 Codex 子 Agent/串行执行，不伪造 team 状态。

## 5. 标准任务闭环

每个任务按以下顺序完成：

1. **Intake**：把用户需求改写为一个可观察结果。
2. **Reality check**：检查当前代码、测试、状态文档和相关参考实现。
3. **Plan**：列出范围、非范围、依赖、风险、文件所有权和验收。
4. **Test first**：先复现 bug，或先定义会失败的契约/E2E。
5. **Build**：在独立分支或 worktree 中实现最小完整行为。
6. **Self-check**：运行局部测试、格式和静态检查。
7. **Review**：独立 Reviewer 检查行为退化、安全和缺失测试。
8. **Verify**：Verifier 按用户路径运行，不读取实现者的主观结论作为证据。
9. **Integrate**：Manager 处理冲突，运行完整测试矩阵。
10. **Record**：更新 `STATUS.md`、任务状态和提交；里程碑完成才更新路线图。

状态只能是：`ready`、`in_progress`、`blocked`、`review`、`verified`、`integrated`。没有证据时不能从 `review` 跳到 `integrated`。

任务开始时保存 base commit、工作区状态、相关文件哈希和 diff。仅记录“原本已脏的文件名”会让后续修改逃过检测，因此不能作为隔离证据。

## 6. 任务规格模板

每个任务都使用以下结构，不接受“优化语音”“完善 Agent”之类无法验收的标题。

```markdown
# TASK-ID: 用户可观察结果

Priority: P0/P1/P2/P3
Milestone: M0-M6
Owner: executor name
Worktree: path/branch
Dependencies: TASK-ID list

## User outcome
用户完成什么，界面或文件会出现什么结果。

## Scope
- 要修改的行为
- 拥有的文件或模块

## Non-goals
- 本任务明确不处理的事项

## Failure cases
- 超时、取消、无权限、离线、无模型、坏输入

## Acceptance evidence
- 自动测试名称
- 人工/E2E 步骤和期望结果
- 性能或安全指标
- 每条需求的 pass/fail/unverified 状态

## Done
- [ ] 实现
- [ ] 局部测试
- [ ] Reviewer
- [ ] Verifier
- [ ] 完整回归
- [ ] STATUS 更新
```

一个任务尽量只改变一个用户行为或一个接口契约。超过两天仍无法形成可审阅 diff 时必须重新拆分。

Goal 验证必须输出“需求 -> 证据”矩阵。成功的测试进程、受控路径中的改动文件或 coding stage 成功都不能单独证明用户需求完成。

## 7. 里程碑路线

```text
M0 工程基线
  -> M1 文字 Agent 闭环
     -> M2 可信记忆
     -> M3 实时语音与角色反馈
     -> M4 Coding Agent 闭环
        -> M5 有限工具与集成
           -> M6 开源发布
```

M2 与 M3 在 M1 稳定后可并行。M4 必须复用 M1 的任务状态、取消和权限协议。M5 只能在核心 registry 和安全边界稳定后开始。

## 8. M0：工程基线

**目标：** 新克隆可复现，规则唯一，系统可以被测试和观察。

### 必做任务

| ID | P | 任务 | 依赖 | 验收 |
|---|---|---|---|---|
| GOV-001 | P0 | 完成 pre-migration 验收 | 无 | 快照可远端恢复 |
| GOV-002 | P0 | 建立 V2 权威文档和 `STATUS.md` | GOV-001 | 无冲突规则 |
| ARC-001 | P0 | 定义 request/tool/voice event schema | GOV-002 | Rust/TS contract tests |
| ARC-002 | P0 | 定义统一错误 taxonomy | ARC-001 | UI 可区分错误、取消、超时 |
| RUN-001 | P0 | 建立取消 token 和任务生命周期 | ARC-001 | 取消后无后续副作用 |
| CI-001 | P0 | 建立默认与可选特性 CI 矩阵 | GOV-001 | 新克隆全绿 |
| CI-002 | P0 | 对齐 Rust toolchain 与 lockfile 审计路径 | CI-001 | CI 与项目 `rust-version` 一致，Cargo audit 覆盖真实 lockfile |
| SEC-001 | P0 | 秘密、路径和生成物边界 | GOV-001 | 扫描无秘密，ignore 测试通过 |
| OBS-001 | P1 | 结构化本地诊断日志 | ARC-002 | 可按 request_id 追踪，不记录秘密 |

### 退出条件

- 统一事件契约已被前后端共同使用。
- 取消是运行时能力，不是 UI 隐藏按钮。
- CI 和本地命令一致。
- 一个新贡献者只按 README 即可完成开发启动。

## 9. M1：文字 Agent MVP

**用户结果：** 用户要求搜索资料并在桌面创建 Markdown，Fairy 会计划、搜索、写文件、验证文件并返回真实路径；失败时说明失败步骤。

### 必做任务

| ID | P | 任务 | 依赖 | 验收 |
|---|---|---|---|---|
| LLM-001 | P0 | Provider trait：流式文本、tool call、错误、取消 | ARC-001 | mock + 一个真实 provider contract test |
| LLM-002 | P1 | 配置和模型探测 | SEC-001 | 缺 key/坏 endpoint 有明确错误 |
| AGT-001 | P0 | 显式 Agent 状态机 | LLM-001,RUN-001 | 无隐藏递归和无限循环 |
| AGT-002 | P0 | 工具轮次、预算和上下文控制 | AGT-001 | 达到预算后可解释终止 |
| AGT-003 | P1 | 计划与最终答复分离 | AGT-001 | 工具结果不会被当作最终答案 |
| TOOL-001 | P0 | Registry 只注册可用工具 | ARC-002 | Scaffold 无法进入生产列表 |
| TOOL-002 | P0 | JSON schema 校验和参数修正 | TOOL-001 | 坏参数不崩溃且有修复上限 |
| TOOL-003 | P0 | 权限、工作目录和批准协议 | SEC-001 | 路径逃逸和高风险命令被阻止 |
| WEB-001 | P1 | 可取消 web search | TOOL-002 | UTF-8、超时、无结果、慢响应测试 |
| WEB-002 | P1 | 可取消 web fetch 和内容压缩 | WEB-001 | 大页面、编码和恶意内容测试 |
| FILE-001 | P0 | 统一路径解析（含 Desktop） | TOOL-003 | `~/Desktop`、绝对路径、沙箱路径测试 |
| FILE-002 | P1 | 原子写文件与覆盖策略 | FILE-001 | 中断不留下损坏文件 |
| ART-001 | P0 | 产物验证器 | FILE-002 | 最终回复前确认文件存在和内容摘要 |
| UI-001 | P1 | 展示计划、工具进度、批准、错误和取消 | ARC-001 | 不靠解析自然语言判断状态 |
| E2E-001 | P0 | 搜索后创建 Markdown 全链路 | 全部以上 | 固定脚本连续通过 10 次，无假成功 |

### Agent 循环约束

- 默认最大工具轮次由任务预算决定，不能用固定 5 轮作为所有任务的产品逻辑。
- 每轮记录原因、工具、时间、结果大小和剩余预算。
- 工具失败后最多进行有限修正；相同错误重复两次则停止并报告。
- 最终回复必须引用已验证产物，不得只复述搜索结果。
- 用户取消后立即停止后续 LLM 请求、工具和文件写入。

### 退出条件

- 20 个固定脚本达到 Alpha 成功率。
- Web 搜索没有 UTF-8 截断 panic。
- Desktop 路径写入在用户批准下可用。
- 所有失败都能定位到 provider、planner、tool、permission 或 verification 阶段。

## 10. M2：可信长期记忆

**用户结果：** Fairy 能在重启后准确回忆用户授权保存的信息，显示来源，并允许纠正、导出和删除。

| ID | P | 任务 | 依赖 | 验收 |
|---|---|---|---|---|
| MEM-001 | P0 | 定义记忆 schema、来源和版本迁移 | M1 | migration 可回滚 |
| MEM-002 | P1 | 原文存储、去重和时序失效 | MEM-001 | 不用摘要替换用户原文 |
| MEM-003 | P1 | FTS 中英检索基线 | MEM-001 | 固定语料 precision/recall 报告 |
| MEM-004 | P1 | 向量检索 spike 和真实模型决策 | MEM-003 | 无模型时明确降级 |
| MEM-005 | P0 | 召回来源、置信度和 prompt 隔离 | MEM-002 | 注入语料不能变成系统指令 |
| MEM-006 | P1 | 用户查看、纠正、导出、删除 | MEM-002 | UI 和数据库一致 |
| MEM-007 | P1 | 对话挖掘需用户策略和审计 | MEM-005 | 敏感内容默认不自动保存 |
| E2E-002 | P0 | 重启后的记忆闭环 | MEM-001..007 | 误召回、冲突事实和删除测试通过 |

知识图谱不是 MVP 前置条件。只有固定语料证明时序关系确实优于普通检索后，才从 Experimental 升级。

## 11. M3：实时语音与角色反馈

**用户结果：** 用户可自然说中文和英文，看到部分识别，听到连续自然回复，可随时打断；角色同步口型、表情和轻量身体动作。

### 先完成 ADR-005 Benchmark

不得先指定 Kokoro、Matcha 或系统 `say` 为答案。对候选链路统一测量：

- 中文、英文、混合语句和日语基础覆盖。
- 固定音频集 WER/CER。
- 首个 partial transcript 延迟。
- TTS 首音频延迟和实时系数。
- CPU、内存、模型体积、许可证和 macOS 打包方式。
- 标点、Markdown、代码符号和 emoji 清洗。
- 语音打断、设备切换、无麦克风和模型缺失。

| ID | P | 任务 | 依赖 | 验收 |
|---|---|---|---|---|
| VOI-001 | P0 | 真实设备枚举、权限和音频输入 | M0 | 拒绝权限/拔设备不崩溃 |
| VOI-002 | P1 | VAD 状态机和噪声基线 | VOI-001 | 安静/噪声/连续语音集 |
| VOI-003 | P0 | 中英流式 ASR | VOI-002,ADR-005 | 不再固定输出 `oah`，有 partial/final |
| VOI-004 | P0 | TTS 文本清洗和语言分段 | ADR-005 | 不朗读 Markdown/奇怪符号 |
| VOI-005 | P1 | 流式 TTS 队列 | VOI-004 | 边生成边播放，无句间长停顿 |
| VOI-006 | P0 | Barge-in 打断 | VOI-002,VOI-005,RUN-001 | 指标内停止播放和生成 |
| VOI-007 | P1 | 模型安装、校验和回退 | ADR-005 | 缺模型不静默回退到 Mac `say` |
| VRM-001 | P1 | PCM/RMS 驱动口型 | VOI-005 | speaking 时自然开合，结束归零 |
| VRM-002 | P2 | 状态驱动表情 | AGT-001 | 状态切换平滑、无标签泄漏 |
| VRM-003 | P2 | 可打断随机待机身体动画 | VRM-001 | 无动作叠加抖动和骨骼漂移 |
| E2E-003 | P0 | 中英连续对话 10 分钟 | 全部以上 | 无崩溃、可打断、记录延迟 |

### 语音文本清洗规则

- TTS 使用语义文本，不直接朗读 Markdown 源码。
- 保留中文、英文、日文、数字和必要标点。
- URL、代码块、表格、路径和工具日志改为简短口语提示，不逐字符朗读。
- 清洗必须保留语言边界，不能用字节索引截断 UTF-8。
- 屏幕文本保留完整信息，语音文本是独立派生结果。

## 12. M4：Coding Agent 闭环

**用户结果：** 用户给出代码目标后，Fairy 能创建隔离工作区、委托 coding runtime、展示计划和 diff、运行测试、请求批准并提交可审阅结果。

| ID | P | 任务 | 依赖 | 验收 |
|---|---|---|---|---|
| CODE-001 | P0 | 定义 Coding Runtime trait | M1 | Codex/OpenCode 适配不污染核心 Agent |
| CODE-002 | P0 | 工作区和 worktree 生命周期 | CODE-001 | 不修改项目外文件，不丢用户改动 |
| CODE-003 | P0 | 结构化进度、stdout 限制和取消 | CODE-001,RUN-001 | 超时后无孤儿进程 |
| CODE-004 | P0 | diff、测试和产物收集 | CODE-002 | 不能仅凭 CLI 退出码宣称完成 |
| CODE-005 | P0 | 命令权限和人工批准 | TOOL-003 | 破坏性命令必须阻止或确认 |
| CODE-006 | P1 | Reviewer/Verifier 回路 | CODE-004 | 修复后重新测试，证据可追踪 |
| CODE-007 | P1 | 失败恢复和继续任务 | CODE-003 | 重启后能解释未完成状态 |
| MCP-001 | P1 | 标准 MCP 记忆/工具服务 | M2 | 官方兼容客户端 contract test |
| E2E-004 | P0 | 真实小型仓库修 bug | 全部以上 | 复现失败、修改、测试、diff、取消均通过 |

Coding Runtime 的首个目标是可靠完成一个受控仓库的小任务，不是立即达到 Codex 的所有能力。模型负责推理，Fairy 负责权限、工作区、状态、记忆和用户体验。

## 13. M5：有限工具与集成

每次只新增一个通过完整门槛的集成。优先顺序：

1. Git/GitHub。
2. 浏览器或系统级可控自动化。
3. 日历和提醒。
4. Discord 移动通信。
5. Notion/Linear/Slack/Gmail。
6. 社区插件。

### 工具上线门槛

每个工具必须具有：

- 稳定唯一 ID、版本和 JSON Schema。
- 明确权限、作用域、速率限制、超时和取消。
- 秘密存储与刷新策略。
- 成功、失败、部分成功和重试语义。
- mock contract test 与至少一个真实环境测试。
- 产物或远端状态的执行后验证。
- 文档和卸载/撤销授权路径。

不满足任一项时只能标记 `Experimental`，默认关闭且不计入工具数量。

## 14. M6：开源发布

### Release Candidate 门槛

- [ ] M0-M4 的 P0/P1 任务全部 integrated。
- [ ] P0/P1 已知缺陷为 0；P2 缺陷有公开限制和负责人。
- [ ] 前端、Rust 默认和可选特性测试全绿。
- [ ] macOS 新用户安装、升级、卸载和权限流程通过。
- [ ] 真实中英文字/语音/Coding E2E 通过。
- [ ] 内存、CPU、启动和延迟基线达标。
- [ ] 无硬编码秘密、个人路径、用户数据、模型或构建产物。
- [ ] 第三方模型、模型声音、VRM 和依赖许可证完成审计。
- [ ] README 只描述 Verified/Beta 能力。
- [ ] 数据导出、删除、隐私和安全边界有用户文档。
- [ ] GitHub Actions 在干净 runner 上通过。
- [ ] 只有此时才执行签名的 Tauri release build。

### 版本推进

```text
2.0.0-alpha.1  M0 基线
2.0.0-alpha.2  M1 文字 Agent
2.0.0-alpha.3  M2 记忆
2.0.0-alpha.4  M3 语音与角色
2.0.0-beta.1   M4 Coding Agent
2.0.0-rc.1     M5 有限集成 + 全面验收
2.0.0          M6 发布
```

版本号代表证据门槛，不按日期自动晋级。

## 15. 第一批 25 个任务的执行顺序

以下顺序可直接作为 OMX backlog。只有标注“可并行”的任务才能并行启动。

| 顺序 | ID | P | 任务 | 并行关系 |
|---:|---|---|---|---|
| 1 | GOV-001 | P0 | 完成迁移快照和恢复演练 | 串行入口 |
| 2 | GOV-002 | P0 | 统一规则和状态文档 | 与 SEC-001 可并行 |
| 3 | SEC-001 | P0 | 秘密/生成物/路径边界 | 与 GOV-002 可并行 |
| 4 | ARC-001 | P0 | 事件契约 | 串行架构入口 |
| 5 | ARC-002 | P0 | 错误 taxonomy | 与 RUN-001 设计并行 |
| 6 | RUN-001 | P0 | 可取消任务生命周期 | 与 ARC-002 协调接口 |
| 7 | CI-001 | P0 | CI 测试矩阵 | 与 OBS-001 可并行 |
| 8 | CI-002 | P0 | 对齐 Rust toolchain 和 lockfile 审计 | 与 OBS-001 可并行 |
| 9 | OBS-001 | P1 | request_id 诊断 | 与 CI-002 可并行 |
| 10 | LLM-001 | P0 | Provider 流式契约 | M1 入口 |
| 11 | AGT-001 | P0 | Agent 状态机 | 依赖 LLM-001/RUN-001 |
| 12 | TOOL-001 | P0 | 真实工具 Registry | 与 AGT-001 可并行 |
| 13 | TOOL-002 | P0 | Schema 和参数校验 | 依赖 TOOL-001 |
| 14 | TOOL-003 | P0 | 权限和批准 | 与 TOOL-002 可并行 |
| 15 | FILE-001 | P0 | 统一路径解析 | 依赖 TOOL-003 |
| 16 | FILE-002 | P1 | 原子文件写入 | 依赖 FILE-001 |
| 17 | WEB-001 | P1 | 可取消、UTF-8 安全搜索 | 与 FILE-002 可并行 |
| 18 | WEB-002 | P1 | Fetch 和内容压缩 | 依赖 WEB-001 |
| 19 | AGT-002 | P0 | 工具轮次、预算、修正 | 依赖 Tool/Web/File 契约 |
| 20 | AGT-003 | P1 | 计划和最终答案分离 | 与 ART-001 可并行 |
| 21 | ART-001 | P0 | 产物验证器 | 依赖 FILE-002 |
| 22 | UI-001 | P1 | 进度、批准、错误、取消 UI | 依赖事件契约稳定 |
| 23 | E2E-001 | P0 | 搜索后写 Markdown | 集成检查点 |
| 24 | MEM-001 | P0 | 记忆 schema 和迁移 | M1 通过后启动 |
| 25 | VOI-001/ADR-005 | P0 | 语音设备基线和技术 benchmark | 可与 MEM-001 并行 |

完成第 23 项之前，不开始第三方 OAuth、社区插件、自成长或全息功能。

## 16. 测试策略

### 16.1 测试层级

| 层级 | 目的 | 示例 |
|---|---|---|
| Unit | 纯逻辑和边界 | UTF-8、安全路径、参数修正、文本清洗 |
| Contract | 跨层协议 | Rust/TS 事件、provider、MCP、tool schema |
| Integration | 真实模块组合 | Agent + mock LLM + real filesystem |
| Desktop E2E | Tauri 用户路径 | 输入、批准、取消、模型、麦克风、VRM |
| Soak/Performance | 长时间和延迟 | 10 分钟语音、长任务、内存增长 |
| Security | 越权和注入 | 路径逃逸、shell 注入、恶意网页、秘密脱敏 |

### 16.2 Bug 修复规则

每个 bug 必须包含：

1. 能稳定复现的失败测试或脚本。
2. 根因说明，不以症状补丁作为结束。
3. 最小修复。
4. 同类边界检查。
5. 回归测试。
6. 若用户可见，更新 `STATUS.md` 的已知问题。

### 16.3 人工测试证据

桌面 E2E 必须记录：日期、commit、macOS 版本、模型、设备、步骤、预期、结果、日志 request_id 和截图/录屏位置。只写“手测通过”不构成证据。

## 17. Git 和 Worktree 规范

### 分支命名

```text
omx/m0-event-contract
omx/m1-file-verification
omx/m3-streaming-asr
fix/p0-path-escape
```

### 文件所有权

- 每个并行 Builder 有明确目录或文件范围。
- 每个任务一个临时 worktree，而不是每个角色一个永久 worktree。
- 记录 worktree 的 base commit，并要求任务 worktree 初始干净；保留原有脏 checkout 不动。
- 跨模块接口由 Architect 先合入契约提交，Builder 再各自实现。
- Agent 不得重置、覆盖或整理不属于自己的修改。
- 发现用户并行修改时停止相关文件写入，先重新读取和协调。

### 提交规范

- 一个提交表达一个行为变化。
- 测试与实现进入同一提交或紧邻提交。
- 禁止提交日志、模型、数据库、密钥和构建输出。
- 合并前 Reviewer 检查 diff，Verifier 检查运行结果。
- 不为“保持整洁”而改写已共享历史。
- 失败 worktree 保留用于诊断，只有合并完成或明确取消后才移除。

## 18. 文档更新规则

避免再次产生大量冲突文档：

- 每个集成批次只更新 `docs/v2/STATUS.md`。
- 里程碑范围变化才更新 `docs/v2/ROADMAP.md`。
- 架构决定写 ADR，并在 `ARCHITECTURE.md` 建索引。
- 用户可用功能通过验收后才更新 README 和用户指南。
- 正式发布才更新 CHANGELOG。
- 历史任务、Agent 临时报告和测试日志不进入权威文档。

`STATUS.md` 固定包含：当前版本、Verified/Beta/Experimental 能力、通过的测试、已知问题、正在进行、阻塞、下一批 ready 任务。

## 19. 每次 OMX 会话的操作清单

### 开始

1. 确认分支、工作区和最新 `STATUS.md`。
2. 运行 `omx doctor`；team 工作前确认 tmux 条件。
3. 从 `ready` 中选择最高优先级且依赖满足的任务。
4. 读取相关代码、测试、ADR；只在相关功能上查 Hermes/MemPalace。
5. 写任务规格，确定直接执行、`$ralplan`、`$team` 或 `$ralph`。
6. 若并行，先建立 worktree 和文件所有权表。

### 进行中

1. Manager 保持一个集成主线，不同时自己修改 Builder 拥有的文件。
2. 每个 Builder 先复现或写测试，再实现。
3. 每 30 分钟或关键状态变化更新进度和阻塞。
4. 相同失败重复两次，转 debugger 或重新规划，不盲目重跑。
5. 发现范围扩大时创建后续任务，不偷偷纳入当前 diff。

### 结束

1. Builder 完成局部测试和自审。
2. Reviewer 输出按严重度排序的问题。
3. Tester/Verifier 独立运行验收。
4. Manager 集成并运行完整矩阵。
5. 更新 `STATUS.md` 和任务状态。
6. 提交清晰 diff，清理已结束的 worktree 和进程。
7. 未达到门槛的任务保持 `blocked` 或 `review`，不改成完成。

## 20. 禁止事项

- 禁止在未保存快照前删除旧项目内容。
- 禁止把当前 V1 标签移动到新代码。
- 禁止为了数量注册未完成工具。
- 禁止把系统 `say` 的静默回退当作本地 TTS 成功。
- 禁止在没有真实模型测试时宣称 ASR/TTS 已完成。
- 禁止通过字节索引截断 UTF-8 字符串。
- 禁止把网页、记忆或工具输出直接当作系统指令。
- 禁止工具只返回文本后就声称文件或远端对象已创建。
- 禁止 Agent 越过工作目录、绕过批准或留下孤儿进程。
- 禁止将“测试通过”写进文档但不记录实际命令和 commit。
- 禁止让 Goal Agent 用“coding stage 成功 + tests 通过 + 有文件改动”替代逐条验收证据。
- 禁止在日常开发运行 Tauri release build；只在 M6 明确执行。
- 禁止恢复 Superpowers 作为项目依赖，除非未来有新的、独立批准的 ADR。

## 21. Definition of Done

一个任务只有同时满足以下条件才是 Done：

- 用户可观察结果已经实现。
- 失败、超时、取消、无权限和坏输入均有行为定义。
- 自动测试覆盖关键逻辑和回归。
- 真实环境验收有证据。
- 没有新增秘密、越权、崩溃或假成功风险。
- Reviewer 的 P0/P1 问题为 0。
- Verifier 对照原始需求签字通过。
- 完整测试矩阵无回归。
- `STATUS.md` 反映真实状态。
- 提交可审阅、可回滚、没有无关改动。

里程碑只有在其所有 P0/P1 任务都达到上述标准后才能完成。未实现的能力保留在 backlog，不通过更改措辞提前发布。
