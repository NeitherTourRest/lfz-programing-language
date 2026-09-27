# ai-dx-engineer — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-27 23:10] P7c — lfz-programming skill 包「可直接交付」升级（增量）
- 来源: team-lead 任务书「升级 `.opencode/skills/lfz-programming/` 为可直接交付给编程 Agent 使用的 Skill 包」（对应 `task-info.md` 第 4 条 + `interface-contract.md` §11.1）
- 完成（四件事，**未重做 P7b**）:
  1. 对照表：`task-info` 第 4 条拆 (a)–(d) + §11.1 五条 → `SKILL.md` 落点，写入 `VERIFICATION.md` §1；结论**达标、无缺口**。
  2. 新增 `README.md`：是什么/给谁/何时；**三种安装**（本项目重启生效 / 全局 `%USERPROFILE%\.config\opencode\skills\` / DSH `dsh-skill-filesystem` 配置片段）；运行前提（cargo PATH 坑、`dist\lfz.exe`、`cargo run --quiet -- run`）；入口限制；不做什么。
  3. `SKILL.md` 强化：顶部 🚫 硬纪律「不要凭记忆写 LFZ」；新增 §0.5 **Agent 标准工作流（5 步）**（未步强制实跑、须 exit 0）；新增 §4.5 **错误类→原因→修法**（26 行）；§7 新增 **L4**；§10 增补 README 指针。`prompt-template.md` 模板 A 增补「不凭记忆 + 按 §0.5 实跑」。
  4. **第 4 次实测**：新写 `docs/guide/ai/examples/04_wordcount.lfz`（struct + 带参方法 + `while` + 管道 + 插值）→ **exit 0**；记录进 `VERIFICATION.md` §7。
- 产出:
  - `.opencode/skills/lfz-programming/`：`SKILL.md`（28748 B / 554 行）、`README.md`（5596 B / 121 行，**新**）、`VERIFICATION.md`（15028 B / 311 行）、`prompt-template.md`（2721 B）
  - `docs/guide/ai/examples/04_wordcount.lfz`（759 B）
- 证据:
  - 4 示例全部 exit 0：`cargo run --quiet -- run docs\guide\ai\examples\{01_hello,02_basics,03_students,04_wordcount}.lfz`
  - `04_wordcount` 输出：`total=13, distinct=8` / `top 3 words (desc):` / `  the:  4` / `  fox:  2` / `  jumps:  2`；`--json` → stdout `{"ok":true}`，exit 0
  - 严格 `E-[0-9]`（区分大小写）检索 SKILL/README/VERIFICATION → **0 命中**（无旧编号错误码）
- 决策: 见 DECISIONS `[2026-09-27 23:10] [ai-dx-engineer]`（skill 包交付形态 + 三种安装方式 + 事实源边界）。
- 下一步: 与 app-dev 闭环回填卡点；spec 变更时同步 SKILL 并重跑 4 示例 + 最小基线。
- 阻塞: 无。

## [2026-09-27 22:00] P7b — AI 向开发指南 + lfz-programming SKILL
- 来源: team-lead 任务书「P7b — AI 向开发指南 + `lfz-programming` SKILL」（交付物 5 的另一半，评分项 4）
- 完成:
  1. 读取权威输入 `docs/spec/interface-contract.md` §11.1（5 条硬性要求）与 `docs/spec/{syntax,semantics}.md`、`docs/tooling/runner-contract.md`、`tests/REPORT.md`。
  2. 产出 `.opencode/skills/lfz-programming/SKILL.md`（frontmatter `name: lfz-programming` + `description`；§11.1 五条逐条写死：`#42` 置顶、12 错误类清单、示例含首行、禁编号码、check/assert 语义）。
  3. 产出 `docs/guide/ai/README.md`（人类可读正文版，与 SKILL 内容一致）+ 可运行示例 `docs/guide/ai/examples/{01_hello,02_basics,03_students}.lfz`。
  4. 产出 `prompt-template.md`、`VERIFICATION.md`（实测记录 + §11.1 对照表 + 修正清单）。
  5. **实测**：3 个示例程序 + 最小任务基线全部 `cargo run --quiet -- run` 跑通（exit 0）；13 类错误场景实测类名/消息/退出码；头部规则边界实测（`#42 ` 尾随空格/仅 3 字节/空文件 → CosmosAnswerError；`.LFZ` 大小写不敏感；BOM 豁免；程序中间 `#` → SyntaxError）。
- 产出:
  - `.opencode/skills/lfz-programming/SKILL.md`（~21 KB）
  - `.opencode/skills/lfz-programming/prompt-template.md`、`VERIFICATION.md`
  - `docs/guide/ai/README.md`、`docs/guide/ai/examples/*.lfz`（3 个）
  - 证据：`03_students.lfz` → `ranking (desc): / Alice: 93 / Cara: 88 / Bob: 67 / members: Alice, Bob, Cara / average: 82.67 / top: Alice (93)`，exit 0
- 决策: SKILL 中如实写明「本 CLI v1 只有 `run <file>` / `test`，无 `-e`/stdin/REPL」——与 spec §11.1「REPL/stdin/-e 豁免」存在实现间隙，已向 team-lead 报告（不在 `src/**` 改动授权内）。
- 实测驱动修正 6 处（详见 VERIFICATION.md §4）：`${}` 内 `\"` 非法；`10 |> push([1,2])` 末参语义；`let a=1; a=2` 实为 SyntaxError；CLI 无 `-e`/stdin；`--json` 程序输出走 stderr；`min([])`/`int(NaN)`/`int(±Inf)` 边界。
- 下一步: 与 app-dev 闭环收集 P8 卡点并回填 SKILL；spec 变更时同步 SKILL 并重跑最小基线。
- 阻塞: 无
## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/ai-dx-engineer.md`
- 下一步: 等待 team-lead 调度
