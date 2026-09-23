# core-dev — 工作状态
> 最后更新: 2026-09-24 00:35 by core-dev

## 当前状态
**P3.9a（`src/error.rs`：`ValueMsg` 新增 2 变体）✅ 完成** —— `cargo build` **0 warning**、`cargo test` **117 passed / 0 failed**（+4 新测试）。待 team-lead 核验 + release-manager 提交。
- 依据：`DECISIONS.md` ADR「[2026-09-24 00:20] [language-architect] P3.9a 契约缺口闭合（6 项）」第 1/2 项 + `docs/spec/semantics.md` §8.1（已补钉模板）。
- 新增：`ValueMsg::EmptyExtremum { func: String }` → `空数组没有极值（{func}）`；`ValueMsg::BadRange { lo: i64, hi: i64 }` → `区间非法：{lo} >= {hi}`。`class_name()` 映射不变（仍 `"ValueError"`），`LzError` 仍 12 变体、无 `E-xxx`。
- core-dev 链下一步：**P3.3b `lexer.rs` 第二批**（字符串 / 插值：模式栈 CODE/STR/INTERP、`"`→`StrBegin`、`${`→`InterpBegin`、`Text`/`FormatSpec`、转义、字符串未闭合错误）。

## 进行中
- （无）

## 最近完成
- **P3.9a `ValueMsg` 新增 2 变体**（2026-09-24）
  - `src/error.rs`（+152 / -1 行）：
    - `ValueMsg` 由「双变体」扩展为 **四变体**，新增 `EmptyExtremum { func: String }`、`BadRange { lo: i64, hi: i64 }`（附文档注释指向 `semantics.md` §8.1）。
    - `ValueMsg::message()` 新增两条分支：
      - `ValueMsg::EmptyExtremum { func } => format!("空数组没有极值（{func}）")`
      - `ValueMsg::BadRange { lo, hi } => format!("区间非法：{lo} >= {hi}")`
    - **未改** `class_name()`（`EmptyExtremum` / `BadRange` 仍 → `"ValueError"`）；**未新增**错误类；**无** `E-xxx`。
    - 枚举文档注释由「两条/双变体」更正为「四条/四变体」。
  - 新增 4 项测试（原有单测全部保留）：
    - `value_msg_covers_all_four_rows`（4 条消息模板全覆盖）
    - `empty_extremum_message_char_by_char`（逐字符断言，`min`/`max`/`minBy`/`maxBy`）
    - `bad_range_message_char_by_char`（逐字符断言，`3 >= 3`、`5 >= 2`、负数、`0 >= i64::MIN`）
    - `new_value_variants_still_map_to_value_error`（新变体仍归 `ValueError`，含 `to_string()`）
    - 辅助 `assert_chars_eq(got, expected)`（逐字符比对，失败指出首个不符下标）
  - 证据：
    - `git diff --stat -- src/error.rs` → `1 file changed, 152 insertions(+), 1 deletion(-)`。
    - `cargo build` → `Finished dev profile [unoptimized + debuginfo] target(s) in 1.75s`，**WARNCOUNT=0**，exit 0。
    - `cargo test` → `test result: ok. 117 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`，exit 0（此前 113 → 117，+4）。
  - 边界纪律：**未改** `docs/spec/`、`src/builtins.rs`、`src/span.rs`、`src/loader.rs`、`src/lexer.rs`、`Cargo.toml`；**未** commit / tag / push。
- **P3.3a lexer 第一批**（2026-09-23）
  - `src/lexer.rs`（**27396 B / 824 行**，含测试），公开 API：
    - `pub struct Token { pub kind: TokenKind, pub span: Span }`（`Clone, PartialEq, Eq, Debug`）。
    - `pub enum TokenKind`（`Clone, PartialEq, Eq, Debug`）—— 一次定义到位，含 P3.3b 的 8 个字面量/字符串变体。
    - `pub fn lex(text: &str, line_base: u32) -> R<Vec<Token>>`：返回以 `Eof` 结尾的记号流；失败返回**带位置**的 `SyntaxError`。
  - **本批实现**：空白（空格/Tab/`\r`）跳过；`//` 行注释（不含行尾 `\n`）；`/* */` 块注释（不可嵌套、等价一个空格、**不产 `Newline`**）；`\n`→`Newline`；标识符/16 关键字/14 保留字/单独 `_`→`Placeholder`；数值（十/`0x`/`0b`/`0o`、浮点 `.digit` 与 `[eE][+-]?digit`、**原文保留**）；运算符与分隔符（严格按 §2.3 最大匹配表）；`#`→`SyntaxMsg::HashPosition`；其余未识别字符→`SyntaxMsg::IllegalChar { c }`。
  - **回退规则（消歧）**：`1e`→`Int("1")`+`Ident("e")`；`1.`→`Int("1")`+`Dot`（A23）；`0x`（无位）→`Int("0")`+`Ident("x")`。
  - **位置语义**：`line = 本地行号(1-based) + line_base`；`col` = 1-based **Unicode 标量**计数（非字节）。
  - **本批未做（如约兜底）**：字符串/插值（`"` 暂按 `IllegalChar('"')`，P3.3b 替换）。
  - 31 个新增测试：`sixteen_keywords, reserved_words, identifiers_only_ascii, lone_underscore_is_placeholder, non_ascii_letter_is_illegal, single_char_operators_and_delimiters, multi_char_operators, maximal_munch_pipe_vs_or, maximal_munch_ampersand, maximal_munch_shift_and_dotdot_split, maximal_munch_assign_family, maximal_munch_semi_vs_dump, line_comment_ends_before_newline, line_comment_at_eof_emits_no_token, block_comment_acts_as_space_without_newline, hash_inside_comment_is_plain, spaces_tabs_cr_are_skipped, newline_is_a_token, integer_literals_preserve_source, float_literals_preserve_source, incomplete_exponent_falls_back, trailing_dot_is_int_then_dot, radix_without_digits_falls_back, huge_integer_text_is_preserved_no_range_check, hash_anywhere_in_code_is_error, illegal_chars, error_carries_position_of_offending_char, line_base_offsets_absolute_line, col_counts_unicode_scalars_not_bytes, multibyte_inside_comment_does_not_shift_following_marks, empty_input_yields_only_eof`。
- **P3.1 基座**（`span.rs` + `error.rs`）✅、**P3.2 加载器**（`loader.rs`）✅ 已交付。

## 阻塞 / 需要支持
- 无硬阻塞。**1 处历史「契约缺口」**（已按最小读法实现，请 team-lead 转 language-architect 裁定；不阻塞 P3.3b）：
  - **未闭合块注释 `/*`（至 EOF 仍无 `*/`）**：`SyntaxMsg` 16 变体中**无对应条目**（最接近的只有 `UnterminatedString`）。**我的口径**：消费至 EOF、等价一个空白、**不报错**（不自造错误消息）。若 architect 期望报错，请指定错误码/文案。
- （沿用 P3.2）3 处「契约待确认」（`NotUtf8.span` / `#42` 消费后 `text` 边界 / `ext` 返回形态）仍未获裁定；均按最小合理读法实现，不阻塞。

## 下一步计划
- 等待 team-lead 派发 **P3.3b `lexer.rs` 第二批**（§2.8 模式栈 CODE/STR/INTERP、`STRING_BEGIN`/`TEXT`/`INTERP_BEGIN`/`FORMAT_SPEC`/`INTERP_END`/`STRING_END`、转义 `\n \t \r \\ \" \e \$`、未列转义→`UnknownEscape`、字符串未闭合→`UnterminatedString`、插值内 `\n`→`InterpolationNewline`、`:` 后的 FORMAT_SPEC 原始文本）。
- runtime-dev 侧：按 P3.9a 裁定用 `ValueMsg::EmptyExtremum` / `ValueMsg::BadRange` 改造 `min`/`max`/`minBy`/`maxBy` 空数组与 `randInt` 非法区间（其职责）。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **强制看 warning**：改 `(Get-Item <file>).LastWriteTime = Get-Date` 再 `cargo build`，否则 cargo 缓存跳过、看不到 warning。（本次改动文件后 `cargo build` 已重编，警告数用 `Select-String "^warning"` 计数。）
- **新增 `ValueMsg` 变体后**：`message()` 的 `match` 是**穷尽**的，漏分支会编译失败——这正好充当「忘记加分支」的编译期保险。
- **PowerShell 抓 `cargo` 输出**：`cargo ... 2>&1 | Tee-Object -Variable out` 再 `Select-String` 统计；`cargo` 往 stderr 写正常进度会被 PowerShell 记成 `NativeCommandError`，**以 `$LASTEXITCODE` 为准**，勿被红字误导。
- lexer 用 `Vec<char>` + `pos` 统一推进；`bump()` 内维护 `line/col`（`\n`→`line+=1,col=1`，否则 `col+=1`），天然保证 **col 按 Unicode 标量**。
- **最大匹配表易漏单字符**：P3.3a 时 `;` 一度只写了 `;;`（`Dump`）而漏了单 `;`（`Semi`），被 `single_char_*` 测试逮住。写 `match` 单字符表时逐条对照 §2.3 末行 `= < > ! + - * / % ;`。
- 数值回退三例（`1e` / `1.` / `0x`）务必用**前瞻**判断再消费，切勿先 `bump` 后判断。
- 测试里"多字节"用例必须让多字节字符处于**合法上下文**（注释内）；直接 `"😀@"` 会在 `😀` 处先报 `IllegalChar`，无法验证 `@` 的列号。
- 不要用 `cargo init`（会覆盖/追加 `.gitignore`/`README.md`/`LICENSE`）。
