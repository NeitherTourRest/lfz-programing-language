# docs-writer — 工作状态
> 最后更新: 2026-09-27 by docs-writer
## 当前状态
**P7a 完成** — 人类向开发者文档（`docs/guide/`，评分项 4 一半）5 份文件已交付，全部示例经 `cargo run --quiet -- run` 实测。
## 进行中
- （无）
## 已交付（docs/guide/，均为 UTF-8）
| 文件 | 行数 | 内容 |
|---|---|---|
| `README.md` | 107 | 5 分钟上手：安装/构建、第一个程序、命令一览、常见错误一眼看懂 |
| `tutorial.md` | 457 | 教程：从零写「成绩分析器」，Step 1–12 覆盖 `#42`/let·var/控制流/函数递归/闭包/struct/管道/插值/`;;`/assert·check/测试 + 练习参考答案 |
| `reference.md` | 199 | 精简参考：词法/语句/表达式/优先级表/54 内置速查 + 每行标注 spec 章节号 |
| `errors.md` | 229 | 错误模型：12 类+基类、子消息表、输出格式、Traceback 折叠、退出码、`--json`、6 组实测示例 |
| `testing.md` | 187 | 测试：命令、T-R1…T-R4 契约、正向/负例写法、判定退出码、最小实测示例、覆盖矩阵/REPORT 位置 |
## 证据
- 实测输出（真实运行）：`cargo run --quiet -- run` → hello(`Hello, LFZ!`)、`grades.lfz`、`exercise.lfz` 等；`cargo run --quiet -- test` → **82 PASS/0 FAIL/0 ERROR, exit 0**；`test <mini>` → 2 PASS/1 FAIL, exit 1；`run --json` 错误 JSON 已验证。
- `#42` 审计：`docs/guide` 共 24 个 ```lfz 代码块，**22 个以 `#42` 开头**；2 个例外是**故意缺 `#42` 的负例**（`errors.md` e3、`testing.md` fixtures/no_header），均已显式标注。
- 结构：`Select-String '^# '` 逐文件标题清单见汇报。
## 阻塞 / 需要支持
- （无）
## 下一步计划
- 等待 team-lead 派发 P7 复审 / 与 ai-dx-engineer 交叉核对术语一致性。
- 工具链若后续新增子命令（`check`/`fmt`/REPL），需回补 `README.md` §5 命令表与 `reference.md`。
## 关键经验（写给未来的自己）
- **示例必须先跑再写**：本次全部输出均为 `cargo run --quiet -- run <file>` 实测（脚本与证据存 `C:\Users\19170\AppData\Local\Temp\opencode\lfz-docs-verify\`，含 `ev2.txt`/`ev-test.txt`/`ev-json.txt`/`ev-ref.txt`）。
- **PowerShell 捕获中文 stderr 会乱码**：用 `cmd.exe /c "... >> file 2>&1"` 重定向到文件、再用 Read 工具读，才能拿到干净 UTF-8（`Start-Process`/`2>&1 | ...` 会被 PS 5.1 的控制台编码破坏）。
- **`;;` dump 不走 `--json` 重定向**：`run --json`/`test --json` 只重定向 `print`/`input` 提示，`;;` 仍写 stdout（tooling 已知偏差，见 `tests/REPORT.md` §3.1 注意）——文档中只描述已验证行为。
- **spec 是唯一事实源**：所有消息/格式/退出码直接取自 `docs/spec/{syntax,semantics,interface-contract}.md`，未自造任何语法或概念。
- **负例文件本身就该缺 `#42`**：写作规范要求"所有代码块以 `#42` 开头"，但演示 `CosmosAnswerError` 的示例必须故意缺失——须显式注明，避免被当作违规。
