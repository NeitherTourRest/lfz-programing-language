# core-dev — 工作状态
> 最后更新: 2026-09-24 02:05 by core-dev

## 当前状态
**P3.3b（`src/lexer.rs` 第二批：STR / INTERP 模式 + 未闭合块注释裁定落地）✅ 完成** —— `cargo build` **0 warning**、`cargo test` **135 passed / 0 failed**（+18 新测试）。待 team-lead 核验 + release-manager 提交。
- 依据：`docs/spec/syntax.md` §2.8（字符串与插值）+ §2.1/§2.2/§2.4；`DECISIONS.md` ADR「[2026-09-24 01:00] [language-architect] 未闭合块注释（/* 至 EOF）裁定」+ `semantics.md` §8.1（细分表 16 → 17 行）。
- `src/lexer.rs`：**27396 B → 45149 B**（+566/-27 行）。新增模式栈 `Mode { Str, Interp(u32), FormatSpec }`（空栈 = CODE）；`StrBegin/StrEnd/Text/InterpBegin/InterpEnd/FormatSpec` **全部开始产出**；`"` 不再走 `IllegalChar` 兜底。
- `src/error.rs`：新增 `SyntaxMsg::UnterminatedBlockComment`（无字段）→ `块注释在此处未闭合（缺少 '*/'）`；文档注释「16 条」→「17 条」；测试改名 `syntax_msg_covers_all_seventeen_rows`。
- core-dev 链下一步：**P3.4a AST / P3.5 parser** 待 team-lead 派发（lexer 已完整）。

## 进行中
- （无）

## 最近完成
- **P3.3b lexer 第二批**（2026-09-24）—— 先落盘再测试（遵守轻量启动纪律）。
  - **A. `src/error.rs`**：
    - `SyntaxMsg` 16 → **17 变体**，新增 `UnterminatedBlockComment`（无字段，附文档注释）。
    - `message()` 新增分支 → `"块注释在此处未闭合（缺少 '*/'）"`（全角括号，与 `NotUtf8` 同风格）。
    - 测试 `syntax_msg_covers_all_sixteen_rows` → **`syntax_msg_covers_all_seventeen_rows`** 并加断言；`class_name()` 不变（仍 `"SyntaxError"`），`LzError` 仍 12 变体、无 `E-xxx`。
  - **B. 块注释裁定落地**：`block_comment()` 由 `()` 改 `R<()>`；`/*` 至 EOF 仍无 `*/` → `Err(syntax(UnterminatedBlockComment, sp))`，`sp` = `/*` 的 `/` 位置。**已闭合**块注释行为不变（等价空格、不产 `Newline`）。
  - **C. STR / INTERP 模式**（严格照 §2.8）：
    - `run()` 改为按模式栈分发：空栈/`Interp`→`code_token()`、`Str`/`FormatSpec`→`str_text(bool)`；循环结束若栈非空 → `UnterminatedString`。
    - **STR**：累积文本 → 非空才发 `Text`；转义 `\n \t \r \\ \" \e \$` 解码（`\e`=`\u{1B}`）；未列举转义 → `UnknownEscape { c }`（span 指向 `\`）；裸 `\n` → `UnterminatedString`；`${` → `InterpBegin` + 压 `Interp(1)`；`"` → `StrEnd` + 弹栈；`{` `}` `#` `$`（非 `${`）按字面。
    - **INTERP**：与 CODE 共用 `code_token()`；`{`→depth+1 发 `LBrace`；`}`→depth−1，为 0 发 `InterpEnd` 并弹栈、否则发 `RBrace`；depth==1 的 `:`→发 `Colon` 并压 `FormatSpec`；裸 `\n`→`InterpolationNewline`；`#`→`HashPosition`（复用 `punct_or_error`）；`"`→嵌套压 `Str`；其余按 CODE 记号走。
    - **FormatSpec 原始模式**：自 `:` 后原样读到**首个** `}`（不解析 `#`/`{}`/转义，且 `\n`→`InterpolationNewline`）；发 `FormatSpec(text)`（**可为空串**）后发 `InterpEnd`，弹 `FormatSpec` + `Interp`。
  - **D. 单测**（`#[cfg(test)]`，lexer 测试 31 → 48，新增 17 项 + error 加断言）：
    - `empty_string_is_begin_end`、`plain_string_is_one_text_fragment`、`every_escape_decodes`（7 转义各 1 例）、`escaped_dollar_is_literal_not_interp`（`\$` 使 `${` 成字面）、`unknown_escape_is_error_at_backslash`（`\q`/`\{`/`\'`/`\0`）、`unterminated_string_at_eof_and_newline`、`interpolation_single_and_multi_segment`（`"a${x}b"`、`"${x}"`）、`nested_interpolation_and_nested_string`（`"${{a}}"`、`"a${ "b" }c"`、`"a${ "b${y}" }c"`）、`format_spec_is_raw_until_brace`（`"${x:>3}"`→`… Ident Colon FormatSpec InterpEnd StrEnd`；空 spec；`#{a` 原始；depth>1 的 `:` 仍是 `Colon`）、`interpolation_bare_newline_is_error`、`hash_is_literal_in_str_but_error_in_interp`、`dollar_without_brace_is_literal`、`braces_are_literal_in_str`、`unmatched_interp_brace_at_eof_is_unterminated`（含 `"${x:`）、`unterminated_block_comment_is_error_at_slash`、`closed_block_comment_still_acts_as_space`、`line_base_holds_across_string_interp`。
    - `illegal_chars` 更新：`"` 不再断言 `IllegalChar`，改为 `UnterminatedString`。
  - 证据：
    - `Get-ChildItem src\lexer.rs,src\error.rs` → `lexer.rs 45149`、`error.rs 37507`。
    - `git diff --stat -- src/lexer.rs src/error.rs` → `error.rs | 13 +-`、`lexer.rs | 566 ++++…`，`2 files changed, 552 insertions(+), 27 deletions(-)`。
    - `cargo build`（改 mtime 强制重编）→ `Finished dev profile [unoptimized + debuginfo] target(s) in 1.46s`，**WARNCOUNT=0**，exit 0。
    - `cargo test` → `test result: ok. 135 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`，exit 0（117 → 135，+18）。
  - 边界纪律：**未改** `src/span.rs`、`src/loader.rs`、`src/value.rs`、`src/env.rs`、`src/builtins.rs`、`docs/spec/`、`Cargo.toml`；**未** commit / tag / push。
- **P3.9a `ValueMsg` 新增 2 变体**（2026-09-24）✅：`EmptyExtremum { func }` / `BadRange { lo, hi }` + `message()` 分支 + 4 测试。
- **P3.3a lexer 第一批**（2026-09-23）✅：CODE 模式核心记号（27396 B / 31 测）。
- **P3.1 基座**（`span.rs` + `error.rs`）✅、**P3.2 加载器**（`loader.rs`）✅ 已交付。

## 阻塞 / 需要支持
- **无阻塞**。历史契约缺口（未闭合块注释）已由 architect ADR 裁定并在本批落地（`SyntaxMsg::UnterminatedBlockComment`）。
- （沿用 P3.2）3 处「契约待确认」（`NotUtf8.span` / `#42` 消费后 `text` 边界 / `ext` 返回形态）仍未获裁定；均按最小合理读法实现，不阻塞。

## 下一步计划
- 等待 team-lead 派发 **P3.4a AST / P3.5 parser**（消费本批完整 lexer 记号流：`StrBegin…StrEnd`、`InterpBegin…FormatSpec?…InterpEnd`）。
- runtime-dev 侧：按 P3.9a 裁定用 `ValueMsg::EmptyExtremum` / `ValueMsg::BadRange` 改造 `min`/`max`/`minBy`/`maxBy` 与 `randInt`（其职责）。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **强制看 warning**：改 `(Get-Item <file>).LastWriteTime = Get-Date` 再 `cargo build`，否则 cargo 缓存跳过、看不到 warning。
- **PowerShell 抓 `cargo` 输出**：`cargo ... 2>&1 | Tee-Object -Variable out` 再 `Select-String` 统计；cargo 往 stderr 写正常进度会被记成 `NativeCommandError`，**以 `$LASTEXITCODE` 为准**。
- **模式栈设计（P3.3b）**：单个 `Vec<Mode>`（空栈=CODE）即可覆盖 CODE/STR/INTERP/FormatSpec；`Str` 与 `Interp` 交替压栈天然支持「嵌套字符串 × 嵌套插值」多层递归。
- **`code_token()` 复用**：顶层 CODE 与 INTERP 共用一个扫描函数，用 `interp_depth()` 做条件分支（`\n` 报错 / `{}` 记账 / depth==1 `:`）。这样 P3.3a 的数值/标识符/最大匹配逻辑零改动被复用。
- **格式说明符读「首个 `}`」**：`"${x:#{}}"` 的 spec 是 `#{` 而非 `#{}`——写测试时务必注意（我最初就写错了一例，被测试逮住）。
- **EOF 统一处理**：`run()` 循环结束若 `modes` 非空 → 一把报 `UnterminatedString`，无需在 STR/INTERP/FormatSpec 各自重复 EOF 逻辑。
- **最大匹配表易漏单字符**：P3.3a 时 `;` 一度只写了 `;;`（`Dump`）而漏了单 `;`（`Semi`），被 `single_char_*` 测试逮住。
- 测试里"多字节"用例必须让多字节字符处于**合法上下文**（注释 / 字符串内）。
- 不要用 `cargo init`（会覆盖/追加 `.gitignore`/`README.md`/`LICENSE`）。
