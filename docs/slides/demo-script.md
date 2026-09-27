# LFZ 答辩 · 现场演示脚本

> 交付物 8（系统介绍 PPT 配套）｜ 作者：ppt-presenter ｜ 最后更新：2026-09-27
> 适用：LFZ 线下答辩现场演示。**本文件中每条命令均由 ppt-presenter 在 2026-09-27 亲自跑通**（输出为真实转录）。
> 事实源：`app/README.md` / `app/DEV_RECORD.md` §7、`docs/guide/README.md`、`docs/reports/status-check.md`、`tests/REPORT.md`。

---

## 0. 使用说明与总时长

- **总时长预算：8–12 分钟**（以实际答辩时限为准；超时优先砍 §5 第 4/5 步，见 §6 压缩策略）。
- **演示叙事线**：hello（这是什么）→ 缺 `#42` 负例（它的签名与错误模型）→ `--json`（机器可读）→ `lfz test`（怎么证明它对）→ `app/sortviz.lfz`（我们用它能做什么，**高光时刻**）→ 现场手写小程序（现学现用，**高光时刻**）。
- **终端编码提示（重要）**：本机 PowerShell 默认 GBK 控制台。直接运行 `cargo run` 中文显示正常；**不要把输出再经管道**（`| Select-Object` 等）否则可能乱码——这是终端编码问题，非 LFZ 缺陷。整场演示**直接运行、不接管道**。
- 全程在**仓库根目录**执行；先设一次 PATH。

> 环境初始化（每场开始前一次，约 3 秒）：
> ```powershell
> $env:Path += ";$env:USERPROFILE\.cargo\bin"
> ```

---

## 1. 演示步骤总表

| 步骤 | 命令 | 预期输出（摘要） | 时长 | 失败预案 |
|---|---|---|---|---|
| 0 预热构建 | `cargo build` | `Finished` 0 error | 0–30s | 跳过（已构建）；若失败见 §5-F0 |
| 1 hello world | `cargo run --quiet -- run examples/hello.lfz` | `Hello, LFZ!` / exit 0 | 20s | 见 §5-F1 |
| 2 缺 `#42` 负例 | `cargo run --quiet -- run tests/fixtures/missing_preamble.lfz` | `CosmosAnswerError` / exit 2 | 30s | 见 §5-F2 |
| 3 `--json` 机器可读 | `cargo run --quiet -- run --json app/sortviz.lfz` | 末行 `{"ok":true}` / exit 0 | 20s | 见 §5-F3 |
| 4 `lfz test` 全绿 | `cargo run --quiet -- test` | `汇总：共 82 个用例，通过 82，失败 0，错误 0` / exit 0 | 60–90s | 见 §5-F4 |
| 5 应用现场跑（**高光**） | `cargo run --quiet -- run app/sortviz.lfz` | 三阶段 + `5 种算法全部通过正确性校验` / exit 0 | 120–180s | 见 §5-F5 |
| 6 现场手写小程序（**高光**） | 见 §4 | `evens^2 = [4, 16, 36, 64, 100]` / `sum = 220 n = 5` / exit 0 | 90s | 见 §5-F6 |

---

## 2. 逐步脚本（命令 → 期望输出 → 失败备选）

### 步骤 1 — hello world（20s）

**操作**：
```powershell
cargo run --quiet -- run examples/hello.lfz
```

**期望输出（真实转录）**：
```
Hello, LFZ!
```
退出码 `0`。

**旁白**：「`examples/hello.lfz` 只有两行：第一行固定是 `#42`，第二行 `print("Hello, LFZ!")`。LFZ 的所有 `.lfz` 文件都必须以 `#42` 起手。」

**若失败**：`examples/hello.lfz` 缺失或损坏 → 改跑内置指南样例 `docs/guide/ai/examples/01_hello.lfz`（内容相同）。

---

### 步骤 2 — 缺 `#42` 的负例（30s）★

**操作**：
```powershell
cargo run --quiet -- run tests/fixtures/missing_preamble.lfz
```

**期望输出（真实转录）**：
```
File "tests/fixtures/missing_preamble.lfz", line 1
CosmosAnswerError: 你忘记了宇宙的答案
```
退出码 `2`。**注意**：`CosmosAnswerError` 属**加载期错误，没有 `Traceback` 头**（与运行期错误的区别，见步骤 5 之外可补充说明）。

**旁白**：「这就是 `#42` 铁律的现场证明——文件缺了首行，程序在**加载阶段**就停下，报 `CosmosAnswerError`；不存在任何变体（`# 42`、`#42 ` 都会失败）。LFZ 的错误输出只有**类名 + 中文消息 + 位置**，没有编号错误码。」

**若失败**：夹具路径不存在 → 现场写一个临时文件（见 §5-F2）。

---

### 步骤 3 — `--json` 机器可读输出（20s）

**操作**：
```powershell
cargo run --quiet -- run --json examples/hello.lfz
```

**期望输出（真实转录）**：
```
Hello, LFZ!
{"ok":true}
```
**注意**：`--json` 模式下 **stdout 恒为一行 JSON**（`{"ok":true}`），被运行程序的 `print` 输出重定向到 **stderr**。因 PowerShell 把两股流都汇总到终端，肉眼会先看到程序输出、再看到 JSON；**JSON 恒在最后一行**。退出码 `0`。

**旁白**：「`--json` 让 LFZ 能被脚本/CI 机器消费：成功给 `{"ok":true}`，失败给结构化错误字段，退出码仍严格区分 0/1/2。」

**若失败**：JSON 行不在最后 → 改用 `cargo run --quiet -- run --json examples/hello.lfz 2>$null` 只看 stdout。

---

### 步骤 4 — `lfz test` 一键全绿（60–90s）★

**操作**：
```powershell
cargo run --quiet -- test
```

**期望输出（真实转录，末尾关键行）**：
```
PASS  tests/lfz/test_structs.lfz

汇总：共 82 个用例，通过 82，失败 0，错误 0
```
退出码 `0`。**评估要点**：这是评分项 2「一个命令执行所有测试」的直接证据；82 个用例含 25 个正向 `.lfz`（547 个 `assert`）+ 56 个负例夹具 + 1 个非 `.lfz` 豁免正向。

**旁白**：「一条命令跑完整套黑盒测试：正向用例在 LFZ 里用 `assert`/`check` 表达断言，负例只断言**错误类**（12 类之一），不逐字比对中文消息——理由见答辩预案『不可断言项』。」

**若失败**：某用例 FAIL/ERROR（应不会，基线全绿）→ 记下用例名，切到预录截图（见 §6 备份），会后定位；**不要现场改解释器**。

---

### 步骤 5 — 应用现场运行（120–180s）★★ 高光时刻

**操作**：
```powershell
cargo run --quiet -- run app/sortviz.lfz
```

**期望输出（真实转录，节选）**：
```
APP_NAME ： LFZ 排序算法可视化
N_SMALL ： 8
...
== 阶段 1：小规模逐帧可视化（N=8） ==
原始数据: [13, 91, 58, 64, 50, 62, 25, 8]
数据摘要: min=8  max=91  sum=371  n=8
-- 冒泡排序（逐趟）--
bubble t=0    |      |######|###   |####  |###   |####  |#     |      |
bubble pass 1 |      |###   |####  |###   |####  |#     |      |######|
...
  bubble     比较=    28  交换=    19  步数=    28  [校验通过]

== 阶段 2：大规模仅统计对比（N=600，不输出逐帧） ==
  bubble     比较=179510  交换= 93045  步数=179510  [校验通过]
  selection  比较=179700  交换=   589  步数=179700  [校验通过]
  insertion  比较= 93640  交换= 93045  步数= 93640  [校验通过]
  merge      比较=  4803  交换=     0  步数=  5576  [校验通过]
  quick      比较=  6473  交换=  2433  步数=  6473  [校验通过]

== 阶段 3：属性自测（随机数组 × 5 算法，check 非致命） ==
  自测 150 次排序，失败 0 次（check 非致命）
== 完成：5 种算法全部通过正确性校验 ==
```
退出码 `0`。

**演示解说（配合输出滚动）**：
1. 开头 `;;` dump 打印 6 个配置常量 → 展示 `;;` 变量 dump 特色。
2. 阶段 1 逐帧条形图 → **「算法长什么样」**：每个数值占一个单元，`#` 数 = 数值比例。
3. 阶段 2 统计表 → **「算法有多快」**：同一个 600 元数组，bubble 比较 **179510** 次，quick 只要 **6473** 次。`insertion` 与 `bubble` 交换次数相同（都等于逆序对数 93045），计数自洽。
4. 阶段 3 → 150 次属性自测 0 失败 → 正确性。
5. 收尾 `assert` + 与内置 `sort()` 对比全通过。

**一句话点睛**：「LFZ 用管道 `|>` + 结构体 `Stats` + 富插值字符画，把排序算法的行为**直接跑给人看**。」

**若失败**：
- 若因终端编码出现乱码 → 见 §5-F5。
- 若某 `assert` 失败（exit 2）→ 切预录输出，会后定位；不现场调试。
- 若耗时过长 → 跳过阶段 1 逐帧（`SHOW_FRAMES=false` 的预跑版本，见 §6）。

---

### 步骤 6 — 现场手写一个小程序（90s）★★ 高光时刻

**操作**：在编辑器中新建 `live_demo.lfz`，当场键入（或粘贴）以下 4 行：
```lfz
#42
let xs = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let ys = xs |> filter((x) => x % 2 == 0) |> map((x) => x * x)
print("evens^2 = ${ys}")
print("sum = ${ys |> sum()}  n = ${len(ys)}")
```
运行：
```powershell
cargo run --quiet -- run live_demo.lfz
```

**期望输出（真实转录）**：
```
evens^2 = [4, 16, 36, 64, 100]
sum = 220  n = 5
```
退出码 `0`。

**旁白**：「4 行代码用上了三大特色——管道 `|>` 的 **data-last**（`xs` 自动注入末参）、lambda `(x) => x * x`、富字符串插值 `${...}`。第一行必须是 `#42`。这就是『值皆可流』。」

**若失败**：手滑打错（如忘 `#42`、用了单引号、误用 `/`）→ **正好用作反面教材**：现场展示对应错误（`CosmosAnswerError` / `SyntaxError`），再改正。这反而是「错误模型清晰」的加分演示。

---

## 3. 关键命令速查（可整段复制）

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"

# 1 构建（应 0 warning / 0 error）
cargo build

# 2 hello
cargo run --quiet -- run examples/hello.lfz

# 3 缺 #42 负例（应 exit 2 + CosmosAnswerError）
cargo run --quiet -- run tests/fixtures/missing_preamble.lfz

# 4 机器可读
cargo run --quiet -- run --json examples/hello.lfz

# 5 黑盒全绿（应 82/82，exit 0）
cargo run --quiet -- test

# 6 应用（应 exit 0）
cargo run --quiet -- run app/sortviz.lfz

# 7 现场小程序（先写 live_demo.lfz，见步骤 6）
cargo run --quiet -- run live_demo.lfz
```

---

## 4. 时间分配与压缩策略

| 章节 | 用时 | 超时压缩 |
|---|---|---|
| 开场 + PPT 前 5 页（背景/语言/架构） | 2.5 min | 语言 5 特色压缩成一句话 |
| 演示步骤 1–3（hello / 负例 / `--json`） | 1.5 min | 步骤 3 可跳过（口头带过） |
| 演示步骤 4（`lfz test`） | 1.5 min | 只念汇总行，不逐条滚 PASS |
| **演示步骤 5（应用）** | **2.5–3 min** | **必留**；若超时改 `SHOW_FRAMES=false` 预跑 |
| **演示步骤 6（手写小程序）** | **1.5 min** | **必留**（最能证明语言可用） |
| PPT 质量/性能/团队/结论页 | 2.5 min | 性能页只讲两处 O(N²) 与结论 |
| **合计** | **≈12 min** | 压缩后 ≈8 min |

**永远不砍**：步骤 5、步骤 6、性能页的诚实结论、收尾 30 秒总结。
**可砍顺序**：步骤 3 → 步骤 1 的构建 → 质量页细节 → 团队页细节。

---

## 5. 失败预案（写实，逐条可执行）

- **F0 构建失败（`cargo build` 报错）**：说明环境被改动。立即执行 `cargo clean -p lfz; cargo build`；仍失败则**跳过构建**直接 `cargo run`（若 `target/` 已有二进制）或改用**预录输出**（§6）。不在现场调解释器源码。
- **F1 hello 输出为空/乱码**：检查是否误接管道；若 `examples/hello.lfz` 缺失，改跑 `docs/guide/ai/examples/01_hello.lfz`。
- **F2 负例夹具不存在**：现场新建临时文件，内容仅一行 `print("no header")`（**故意不含 `#42`**），运行它即得 `CosmosAnswerError`。演示后删除临时文件。
- **F3 `--json` 未见 JSON 行**：加 `2>$null` 只看 stdout；JSON 恒在 stdout 最后一行。
- **F4 测试非全绿**：概率极低（基线 82/82）。记下 FAIL/ERROR 用例名，**切预录截图**，诚实说明「现场环境与该用例相关，会后定位」，继续演示——不掩盖。
- **F5 中文乱码**：确认是**直接运行**、未接管道；把控制台换 UTF-8 代码页 `chcp 65001` 后重跑；仍乱码则用预录 GIF/文本。
- **F6 手写小程序报错**：多为漏 `#42`、用单引号、把 `/` 当整除。**当作反面演示**：读出错误类（`CosmosAnswerError`/`SyntaxError`/`TypeError`），改正后重跑成功——展示错误模型的清晰度。
- **F7 终端卡死/无响应**：`Ctrl+C` 中断，切预录输出，继续下一节。绝不现场调试解释器。

---

## 6. 备份方案（答辩前必备齐）

| 备份 | 内容 | 位置建议 |
|---|---|---|
| ① PPT 备份 | `LFZ-defense.pptx` 的 **PDF 版**（防字体/排版差异） | U 盘 + 本机 |
| ② 命令备份 | 本文件 §3 的可复制命令块 | 打印稿 + 记事本 |
| ③ 输出备份 | 步骤 4/5/6 的真实输出**预录文本 + 截图/GIF** | 本机 `docs/slides/` 或 U 盘 |
| ④ 预跑版本 | `SHOW_FRAMES=false` 的 sortviz 快速版（阶段 1 不逐帧） | 临时副本 |
| ⑤ 兜底话术 | 「这点我们记录在 <对应文件> 中，会后可补充」 | qa-prep.md 末尾 |

**答辩前 5 分钟环境核对清单**：
- [ ] `$env:Path += ";$env:USERPROFILE\.cargo\bin"` 已设。
- [ ] `cargo run --quiet -- run examples/hello.lfz` 已预跑一遍（防环境差异）。
- [ ] `cargo run --quiet -- test` 已预跑一遍（防中文乱码/缺夹具）。
- [ ] `app/sortviz.lfz` 环境里能跑出 `EXIT=0`。
- [ ] PPT 打开确认字体与排版正常（无乱码/无溢出）。
- [ ] 预录输出、PDF、命令打印稿就位。

---

## 7. 可核证据（本次实测记录，2026-09-27）

| 命令 | 实测结果 | 退出码 |
|---|---|---|
| `cargo run --quiet -- run examples/hello.lfz` | `Hello, LFZ!` | 0 |
| `cargo run --quiet -- run tests/fixtures/missing_preamble.lfz` | `File …, line 1` + `CosmosAnswerError: 你忘记了宇宙的答案` | 2 |
| `cargo run --quiet -- run --json examples/hello.lfz` | 程序输出 + `{"ok":true}` | 0 |
| `cargo run --quiet -- test` | `汇总：共 82 个用例，通过 82，失败 0，错误 0` | 0 |
| `cargo run --quiet -- run app/sortviz.lfz` | 三阶段 + `5 种算法全部通过正确性校验` | 0 |
| `cargo run --quiet -- run live_demo.lfz` | `evens^2 = [4, 16, 36, 64, 100]` / `sum = 220 n = 5` | 0 |
| `cargo test` | `431 passed / 0 failed / 0 ignored`（361+42+16+12） | 0 |
