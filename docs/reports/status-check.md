# 现状测试报告（team-lead 亲测）

> 时间：2026-09-24 ｜ 基线：`main` = `f625150`（标签 `v0.1.0` / `v0.2.0`）
> 触发：用户要求「帮我进行目前进度的测试」。本报告为**现场重跑**（不复用旧报告结论）。

---

## 1. 构建与单测

| # | 命令 | 期望 | 实测 | 结论 |
|---|---|---|---|---|
| 1 | `cargo build` | 0 warning | `Finished`（无 warning） | ✅ |
| 2 | `cargo test` | 全绿 | **`361 passed; 0 failed; 0 ignored`**（lib）+ **9** + **7** = **377** | ✅ |

## 2. 端到端与错误模型（真实 CLI）

| # | 场景 | 期望 | 实测 | 结论 |
|---|---|---|---|---|
| 3 | `cargo run -- run examples/hello.lfz` | `Hello, LFZ!` / exit 0 | `Hello, LFZ!` / exit **0** | ✅ |
| 4 | `.lfz` 缺 `#42` 首行 | exit 2 + `CosmosAnswerError` | `File …, line 1` + `CosmosAnswerError: 你忘记了宇宙的答案` / exit **2** | ✅ |
| 5 | 语法错误 `let x = 1 $ 2` | exit 2 + `SyntaxError` + 插入符 | `let x = 1 $ 2` + `^` + `SyntaxError: 非法字符 '$'` / exit **2** | ✅ |
| 6 | 运行期错误 `s.missing` | exit 2 + `Traceback` + **正确行号** | `Traceback (most recent call last):` + `line 3, in <module>` + 源码行 + `^` + `FieldError: 结构体没有字段 'missing'` / exit **2** | ✅ |

## 3. 夹具全量压测（3 组 / 25 个 `.lfz`）

**判据**：正例 → 必须 exit 0；负例 → 必须 exit 2 **且错误类/消息符合预期**。

### 3.1 正例（16 个，全部 exit 0）
`fixtures-p3/01_arith` · `02_control` · `03_functions` · `04_containers` · `05_pipe` · `06_interp` · `07_dump` · `08_builtins`；
`fixtures-p3-rev2/bug01_multiline` · `bug01_leading_comment` · `bug01_leading_blank_block` · `bug02_format_spec` · `bug03_if_expr` · `bug07_stmt_brace` · `spec_9_4_refs_fixed`；
`fixtures-p3-rev3/spec_9_4_current` → **16/16 PASS**

### 3.2 负例（9 个，全部 exit 2 且错误类正确）

| 夹具 | exit | 实测错误（类 + 消息） | 期望 | 结论 |
|---|---|---|---|---|
| `nohdr.lfz` | 2 | `CosmosAnswerError: 你忘记了宇宙的答案` | 同 | ✅ |
| `bug04_span.lfz` | 2 | `FieldError: 结构体没有字段 'missing'`（**line 3**） | 同 | ✅ |
| `bug05_pipe_rhs.lfz` | 2 | `TypeError: 管道右侧必须是函数，得到 int` | 同 | ✅ |
| `bug06_let_rebind.lfz` | 2 | `TypeError: 不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容` | 同 | ✅ |
| `bug09_recursion.lfz` | 2 | `RecursionError: 递归深度超限（超过 10000 层）`，**输出 123 行**（截断折叠生效） | 同 | ✅ |
| `span_check_divzero.lfz` | 2 | `ZeroDivisionError: 除以零`（**line 2**） | 同 | ✅ |
| `span_check_name.lfz` | 2 | `NameError: 未定义的名字 'undefinedName'`（**line 2**） | 同 | ✅ |
| `span_check_nested_field.lfz` | 2 | `FieldError: 结构体没有字段 'missing'`（**line 3**） | 同 | ✅ |
| `spec_9_4_refs.lfz`（**已知陈旧副本**） | 2 | `SyntaxError: 语句之间必须有换行` | 同（该夹具是修订前 §9.4 的逐字拷贝，含被禁止的单个 `;`；规范已修，夹具按历史证据保留） | ✅（预期失败） |

**合计：25/25 行为符合预期**（16 正例全过 + 9 负例全中）。

## 4. 位置信息专项（`Span` 正确性）

| 夹具 | 报错行 | 源文件对应行 | 结论 |
|---|---|---|---|
| `bug04_span.lfz` | line 3 | 第 3 行 `s.missing` | ✅ |
| `span_check_nested_field.lfz` | line 3 | 第 3 行 `print(s.missing)` | ✅ |
| `span_check_divzero.lfz` | line 2 | 第 2 行 `div(1, 0)` | ✅ |
| `span_check_name.lfz` | line 2 | 第 2 行 `print(undefinedName)` | ✅ |

> 该项是 P3.11 缺陷 **bug-04（traceback 位置过期）** 的回归证据：修复后帧 `span` 指向"正在求值的最小 AST 节点"，不再停留在模块首语句。

## 5. 结论

- **P3 交付物（解释器核心）保持健康，无回归**：377 测试全绿、0 warning、端到端可用、错误模型（类/消息/位置/退出码/`Traceback`）全部符合 `docs/spec/`；
- P3.11 的三项阻塞缺陷与两项非阻塞缺陷修复**均被本轮实测复现验证**（多行块、`format_spec`、`if` 表达式、`let` 重绑定、traceback 截断与位置）；
- 唯一"失败"是**已知陈旧夹具**（`spec_9_4_refs.lfz`），按历史证据刻意保留，非缺陷。

**判定：可以继续开发（P4 工具链 / P5 黑盒测试集）。**
