# app-dev — 工作状态
> 最后更新: 2026-09-27 by app-dev

## 当前状态
**P8 应用开发完成**：交付物 6（评分项 5，30 分）——LFZ 应用 `app/sortviz.lfz`「排序算法可视化」已实现并实测跑通，配套 `app/README.md` + `app/DEV_RECORD.md` 完成。

## 进行中
- （无）

## 已完成（本轮）
- **应用源码** `app/sortviz.lfz`：**341 行 / 有效行 290**（UTF-8 无 BOM，LF；首行恰好 `#42`，前 5 字节 `23 34 32 0A 2F`）。
  - 5 种排序：冒泡 / 选择 / 插入 / 归并 / 快速（Lomuto）。
  - 可视化：`repeat`/插值打印字符条形图，逐帧快照（`snap` 回调）。
  - 统计：`struct Stats`（比较 / 交换 / 步数，`self.x += 1` 原地累加）。
  - 交互/参数：规模 `N_SMALL`/`N_BIG`、种子 `SEED`（`seed(n)` 可复现）、`CELLW`、`SHOW_FRAMES`（因 CLI v1 无参数，用源码常量）。
  - 校验：`assert` + 与内置 `sort()` 对比；另有 150 次随机数组属性自测（`check` 非致命）。
  - 5 特色全用：管道 `|>`、结构体、富插值、`check`/`assert`、`;;`。
- **文档**：`app/README.md`（功能/运行方法/交互示例/设计结构/5 特色对照）；`app/DEV_RECORD.md`（选题理由/需求分析/**6 轮提示词与迭代**/依据 SKILL 的决策/踩坑与修复/测试验证/演示脚本/给 ai-dx-engineer 的反馈）。
- **实测证据**：
  - `cargo run --quiet -- run app/sortviz.lfz` → 退出码 **0**（逐帧 + 统计 + 自测 0 失败）
  - `cargo run --quiet -- run --json app/sortviz.lfz` → stdout `{"ok":true}`，退出码 **0**
  - `SHOW_FRAMES=false` 复跑（改一行常量）→ 仅统计模式，退出码 **0**
  - `cargo build` → **0 warning**；`cargo run --quiet -- test` → **82/82 通过**
  - `git status --porcelain` → 仅 `?? app/`（未 commit / tag / push）

## 阻塞 / 需要支持
- （无）

## 下一步计划
- 可选：与 **ppt-presenter** 对齐现场演示脚本（演示路径见 `app/DEV_RECORD.md` §7，3 分钟版）。
- 等待 **verifier** 对评分项 5 独立验收；运行命令与行数证据已备齐。
- 已向 **ai-dx-engineer** 提 1 条指南补充建议（`DEV_RECORD.md` §8）。

## 关键经验（写给未来的自己）
- **LFZ v1 无模块/import**（`import` 是保留字，用即 `SyntaxError`）→ 应用只能**单文件 + 分节**；**无 CLI 参数 / 环境变量 / 文件 IO** → 配置只能写源码常量。
- **`input()` 在无 stdin 环境抛 `IOError`（退出码 2）** → 默认运行路径**绝不调用 `input`**。
- **array / struct 是引用类型、传参按引用**：原地 `xs[j] = v`、struct 方法 `self.x += 1` 都能改到调用方容器；但 `push`/`sort`/`slice`/`map` 等内置**返回新值、不改原容器**（A1）。
- 取整一律用 **`div(a, b)`**（`/` 恒返回 `float`）。
- **工具坑**：Windows PowerShell `Get-Content` **不带 `-Encoding UTF8`** 会按 GBK 解码 UTF-8 源文件，导致行数失真（报 327，真实 **341**）；计数务必加 `-Encoding UTF8` 或用 `ReadAllLines`/数 LF 字节。
- 按 **SKILL.md §8 自检清单**逐项打勾后再跑，首版**真跑即通过**（唯一缺陷 `stHead` 未定义在运行前被清单拦下）——**AI 指南有效**。
