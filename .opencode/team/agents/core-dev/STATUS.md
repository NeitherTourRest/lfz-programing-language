# core-dev — 工作状态
> 最后更新: 2026-09-23 23:58 by core-dev

## 当前状态
**P3.3a（`src/lexer.rs` 第一批：CODE 模式核心记号）✅ 完成** —— `cargo build` **0 warning**、`cargo test` **113 passed / 0 failed**（新增 31 项 lexer 测试）。待 team-lead 核验 + release-manager 提交 `feat(p3): lexer code-mode`。
- P3.1 基座（`span.rs` + `error.rs`）✅、P3.2 加载器（`loader.rs`）✅ 已交付。
- core-dev 链下一步：**P3.3b `lexer.rs` 第二批**（字符串 / 插值：模式栈 CODE/STR/INTERP、`"`→`StrBegin`、`${`→`InterpBegin`、`Text`/`FormatSpec`、转义、字符串未闭合错误）。

## 进行中
- （无）

## 最近完成
- **P3.3a lexer 第一批**（2026-09-23）
  - `src/lexer.rs`（**27396 B / 824 行**，含测试），公开 API：
    - `pub struct Token { pub kind: TokenKind, pub span: Span }`（`Clone, PartialEq, Eq, Debug`）。
    - `pub enum TokenKind`（`Clone, PartialEq, Eq, Debug`）—— 一次定义到位，含 P3.3b 的 8 个字面量/字符串变体。
    - `pub fn lex(text: &str, line_base: u32) -> R<Vec<Token>>`：返回以 `Eof` 结尾的记号流；失败返回**带位置**的 `SyntaxError`。
  - **本批实现**：空白（空格/Tab/`\r`）跳过；`//` 行注释（不含行尾 `\n`）；`/* */` 块注释（不可嵌套、等价一个空格、**不产 `Newline`**）；`\n`→`Newline`；标识符/16 关键字/14 保留字/单独 `_`→`Placeholder`；数值（十/`0x`/`0b`/`0o`、浮点 `.digit` 与 `[eE][+-]?digit`、**原文保留**）；运算符与分隔符（严格按 §2.3 最大匹配表）；`#`→`SyntaxMsg::HashPosition`；其余未识别字符→`SyntaxMsg::IllegalChar { c }`。
  - **回退规则（消歧）**：`1e`→`Int("1")`+`Ident("e")`；`1.`→`Int("1")`+`Dot`（A23）；`0x`（无位）→`Int("0")`+`Ident("x")`。
  - **位置语义**：`line = 本地行号(1-based) + line_base`；`col` = 1-based **Unicode 标量**计数（非字节）。
  - **本批未做（如约兜底）**：字符串/插值（`"` 暂按 `IllegalChar('"')`，P3.3b 替换）。
  - 证据：
    - `Get-ChildItem src\lexer.rs` → **Length 27396**（原桩 143 B，真落盘）。
    - `cargo build`（改 `LastWriteTime` 强制重编）→ `Finished dev profile ... `，**WARN=0**，exit 0。
    - `cargo test` → `test result: ok. 113 passed; 0 failed; 0 ignored`（lib harness；main/doc 各 0），exit 0；其中 **31** 个 `lexer::tests::*`。
  - 31 个新增测试：`sixteen_keywords, reserved_words, identifiers_only_ascii, lone_underscore_is_placeholder, non_ascii_letter_is_illegal, single_char_operators_and_delimiters, multi_char_operators, maximal_munch_pipe_vs_or, maximal_munch_ampersand, maximal_munch_shift_and_dotdot_split, maximal_munch_assign_family, maximal_munch_semi_vs_dump, line_comment_ends_before_newline, line_comment_at_eof_emits_no_token, block_comment_acts_as_space_without_newline, hash_inside_comment_is_plain, spaces_tabs_cr_are_skipped, newline_is_a_token, integer_literals_preserve_source, float_literals_preserve_source, incomplete_exponent_falls_back, trailing_dot_is_int_then_dot, radix_without_digits_falls_back, huge_integer_text_is_preserved_no_range_check, hash_anywhere_in_code_is_error, illegal_chars, error_carries_position_of_offending_char, line_base_offsets_absolute_line, col_counts_unicode_scalars_not_bytes, multibyte_inside_comment_does_not_shift_following_marks, empty_input_yields_only_eof`。

## 阻塞 / 需要支持
- 无硬阻塞。**1 处本次「契约缺口」**（已按最小读法实现，请 team-lead 转 language-architect 裁定；不阻塞 P3.3b）：
  - **未闭合块注释 `/*`（至 EOF 仍无 `*/`）**：`SyntaxMsg` 16 变体中**无对应条目**（最接近的只有 `UnterminatedString`）。**我的口径**：消费至 EOF、等价一个空白、**不报错**（不自造错误消息）。若 architect 期望报错，请指定错误码/文案。
- （沿用 P3.2）3 处「契约待确认」（`NotUtf8.span` / `#42` 消费后 `text` 边界 / `ext` 返回形态）仍未获裁定；均按最小合理读法实现，不阻塞。

## 下一步计划
- 等待 team-lead 派发 **P3.3b `lexer.rs` 第二批**（§2.8 模式栈 CODE/STR/INTERP、`STRING_BEGIN`/`TEXT`/`INTERP_BEGIN`/`FORMAT_SPEC`/`INTERP_END`/`STRING_END`、转义 `\n \t \r \\ \" \e \$`、未列转义→`UnknownEscape`、字符串未闭合→`UnterminatedString`、插值内 `\n`→`InterpolationNewline`、`:` 后的 FORMAT_SPEC 原始文本）。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **强制看 warning**：改 `(Get-Item <file>).LastWriteTime = Get-Date` 再 `cargo build`，否则 cargo 缓存跳过、看不到 warning。
- lexer 用 `Vec<char>` + `pos` 统一推进；`bump()` 内维护 `line/col`（`\n`→`line+=1,col=1`，否则 `col+=1`），天然保证 **col 按 Unicode 标量**。
- **最大匹配表易漏单字符**：本次 `;` 一度只写了 `;;`（`Dump`）而漏了单 `;`（`Semi`），被 `single_char_*` 测试逮住（`IllegalChar(';')`）。写 `match` 单字符表时逐条对照 §2.3 末行 `= < > ! + - * / % ;`。
- 数值回退三例（`1e` / `1.` / `0x`）务必用**前瞻**判断再消费，切勿先 `bump` 后判断。
- 测试里"多字节"用例必须让多字节字符处于**合法上下文**（注释内）；直接 `"😀@"` 会在 `😀` 处先报 `IllegalChar`，无法验证 `@` 的列号。
- 不要用 `cargo init`（会覆盖/追加 `.gitignore`/`README.md`/`LICENSE`）。
