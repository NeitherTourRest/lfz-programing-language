# docs-writer — 工作状态
> 最后更新: 2026-09-27 by docs-writer

## 当前状态
**T11-③（人类向文档）完成** — 把 7 条盲测缺口（D1–D7）+ 架构师裁定修正 + 3 条新增错误消息同步进 `docs/guide/**`，全部示例经 `dist\lfz.exe` 实跑复核。

## 进行中
- （无）

## 已交付（T11-③，均为 no-BOM UTF-8）
| 文件 | 字节数 | 行数 | 本轮改动 |
|---|---|---|---|
| `docs/guide/README.md` | 7153 | 150 | §6 新增 4 行常见错误（字符串下标/容量溢出/嵌套/动态宽度）+「两个最常问的点」（D1、D4） |
| `docs/guide/tutorial.md` | 18271 | 622 | Step 3 补充（`else if`、循环 `let` 每轮新绑定 D3/D7）；Step 8 对齐 `<`/`>`/`^`+fill 表 + 动态宽度警告（D5）；**新增 Step 13** 字符串处理（D1/D6/O(n) 构建）；Step 11 用例数 82→90 |
| `docs/guide/reference.md` | 20825 | 338 | §3 表 + 注：字符串不可下标/零参 print；§3.3 三种对齐+fill 表 + 动态宽度；§3.4 循环每轮新绑定 + float 显示裁定；**新增 §3.5** 字符串惯用法（D1/D6/O(n)/padRight）、**§3.6** 隐性语法正面示例（D7）；§5.1 `len`/`range` 行；§5.3 v1.1 方法提示；§6 清单 |
| `docs/guide/errors.md` | 16595 | 317 | §1 零帧 Traceback 例外（obs-B-01）+ OverflowError 容量消息；§2 新增两条 SyntaxError 细分（D10）+ OverflowError 细分表；§3.2 三种结构（含零帧）；§3.4 `assert` 退出码语境（D4）；§4 示例 6 增行 + 示例 7（零帧 IOError）+ 示例 8（嵌套超限）；§5 增 6 行 |
| `docs/guide/testing.md` | 9696 | 231 | §5 退出码语境补充（D4）；计数 82→90（§1/§7/§7.1） |

## 证据
- 一手证据报告：`docs/reports/T11-03-docs-evidence.md`（5282 B，含全部命令 + 逐字输出 + 退出码 + `git diff --stat`）。
- `git diff --stat docs/guide/`：5 files changed, **265 insertions(+), 16 deletions(-)**。
- 实跑：`dist\lfz.exe run` 逐条探针（`s[0]`→exit 2；`split`→0；`range(1,4)`→exit 2；对齐→0；动态宽度→exit 2；`len(str)`→0；隐性语法→0；`push+join`→0；`repeat` 容量→exit 2；`(`×1001 / `1+`×10000→exit 2；文件不存在→零帧 `IOError`）。
- 文档内 6 个新增示例原文逐字复跑（`doc_s13/s3b/s8b/padRight/ref35_1/ref36`）全部 exit 0、输出逐字一致。
- `lfz test` 实测 **90/90 exit 0**；`#42` 审计：32 个 `lfz` 块 / 30 带 `#42` / 2 个**故意缺前导的负例**（已标注）。

## 阻塞 / 需要支持
- （无）

## 下一步计划
- 若 v1.1 语言补强落地（`range(lo,hi)`、字符串方法族 `indexOf/padEnd/substring` 等），需回补 `reference.md` §5 与教程 Step 13。
- 已在文档标注「v1.1 计划、当前未实现」，待实现后改为正式条目。

## 关键经验（写给未来的自己）
- **PowerShell 读文件绝不能 `Get-Content -Raw` 后再写回**：PS 5.1 按 ANSI(GBK) 解码 UTF-8 → 中文全变乱码（本轮 `testing.md` 曾因此损坏，已 `git checkout` 恢复）。改文档**只用 Edit 工具**；确需脚本处理则用 `[System.IO.File]::ReadAllText/WriteAllText` + `UTF8Encoding($false)`。
- **新增 lfz 代码块后必跑 `#42` 审计**：本轮为演示"嵌套超限"曾写入一个含 `…` 的**不可运行** `lfz` 块——违反"示例必须可运行"，已改为 `text` 说明式描述（不要给假可运行块）。
- **只写当前已实现行为**：`padEnd`/`indexOf`/`range(lo,hi)` 均属 v1.1 **未实现**，只能作为"v1.1 计划"标注，不能当可用 API 写。
- **分工边界**：只改 5 个人类向文件；`docs/guide/ai/**`（AI 向）与 `.opencode/skills/**` 归 ai-dx-engineer，本轮未触碰。
- **计数一致性**：`lfz test` 已 90（非 82），guide 旧计数须同步；根 README 由 release-manager 维护。
