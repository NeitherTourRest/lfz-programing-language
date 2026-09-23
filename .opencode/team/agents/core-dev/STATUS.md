# core-dev — 工作状态
> 最后更新: 2026-09-23 22:02 by core-dev

## 当前状态
**P3.2（`src/loader.rs`）✅ 完成** —— `cargo build` 0 warning、`cargo test` 27 passed / 0 failed（新增 12 项 loader 测试）。待 team-lead 核验 + release-manager 提交 `feat(p3): loader`。
P3.1 基座（`span.rs` + `error.rs`）已交付（15 测试）。core-dev 链下一步：**P3.3 `lexer.rs`**。

## 进行中
- （无）

## 最近完成
- **P3.2 加载器**（2026-09-23）
  - `src/loader.rs`（**15873 B / 384 行**，内含测试）：
    - `pub struct Loaded { pub text: String, pub line_base: u32 }`（契约 §10.1）。
    - `pub fn load_file(path: &str) -> R<Loaded>`：读文件 → UTF-8 校验 → 跳 BOM → 归一化行终止符 → `is_lfz` 分流（`.lfz` 要求并**消费** `#42` 前导，否则 `line_base = 0`）。
    - `pub fn load_source(text: &str) -> R<Loaded>`：**不要求/不消费**前导；仍跳 BOM + 归一化；`line_base = 0`（REPL/stdin/`-e`）。
    - `pub fn ext(path: &str) -> Option<&str>`（`syntax.md` §2.2.0 步骤 1–3）+ `pub fn is_lfz(path: &str) -> bool`（步骤 4，**仅 ASCII** 折叠）。
    - 私有 `normalize()`（BOM 仅一个 + `\r\n`/`\r`→`\n`）、`consume_preamble()`（前 4 字节须恰为 `#42\n`；否则 `CosmosAnswerError`）。
  - **§10.1 四要点落点**：
    1. 模块职责（读→UTF-8→BOM→归一→ext→前导）：`load_file` L62–84；BOM/行终止符 `normalize` L98–109。
    2. 双入口 + `Loaded`：`Loaded` L20–26；`load_file` L62；`load_source` L89–95。
    3. `ext(path)` 四步：`ext` L37–44（步骤 1–3）+ `is_lfz` L50–53（步骤 4）。
    4. 前导不产 token + `line = 本地行号 + line_base`：`consume_preamble` L116–126（`line_base=1`）；非 `.lfz` L82 与 `load_source` L93（`line_base=0`）；行号语义文档 L18–19 / L24。
  - 证据：
    - `cargo build`（改 `LastWriteTime` 强制重编）→ `Finished dev profile ... in 0.86s`，**WARNINGS=0**，exit 0。
    - `cargo test` → `test result: ok. 27 passed; 0 failed; 0 ignored`（lib harness；main/doc 各 0），exit 0。
  - 12 个新增测试：`loader::tests::{ext_returns_last_component_extension, is_lfz_is_ascii_case_insensitive, bom_is_skipped_for_file_and_source, line_terminators_normalize_to_lf, missing_or_malformed_preamble_is_cosmos_answer, valid_lfz_preamble_sets_line_base_one, non_lfz_file_skips_preamble_check, empty_non_lfz_file_is_legal_empty_program, load_source_does_not_require_or_consume_preamble, invalid_utf8_is_syntax_error_with_offset, invalid_utf8_beats_missing_preamble, missing_file_is_io_error_without_span}`。
  - 错误落点：读取失败 → `error::io("无法读取：{path}", None)`；非 UTF-8 → `error::syntax(SyntaxMsg::NotUtf8 { off }, Span::START)`（`off = utf8_error().valid_up_to()`，**先编码后前导**）；前导违规 → `error::cosmos_answer()`（`Span::START`）。均走 §10.8 冷路径构造器。

## 阻塞 / 需要支持
- 无硬阻塞。**3 处「契约待确认」**（已按最小合理读法实现，请 team-lead 转 language-architect 裁定；不阻塞 P3.3 开工）：
  1. **`SyntaxMsg::NotUtf8` 的 `span` 取值**：契约 §10.4 仅规定 `Syntax { msg, span }` 有位置，未指明 `NotUtf8` 的具体 line/col。**我的口径**：`Span::START`（line 1 col 1）—— 编码校验发生在任何行/列可稳定计算之前，且精确的字节偏移已由消息「（首个非法字节位于字节偏移 {off}）」承载。
  2. **`#42` 消费后 `text` 的边界**：`syntax.md` §2.2.2 步骤 7d 规定「消费 `#42` 三字符与其后的 `\n`（整个前导行）」。**我的口径**：`text` 自 `#42\n` 之后的第一个字符起（程序体首行），`line_base = 1`。按此实现（无歧义）。
  3. **`ext` 的返回形态**：契约以伪代码 `ext(path) == "lfz"` 表述。**我的口径**：`ext() -> Option<&str>` 返回**原始**扩展名子串（`None` = 无扩展名，`Some("")` = 空扩展名），步骤 4 的折叠独立为 `is_lfz() -> bool`。若期望单一「已折叠判定」API，请告知。
- （沿用 P3.1）`Assert.msg` / `Io.msg` 承载**已组装完成的最终消息**，`message()` 原样返回 —— 待 architect 知悉。

## 下一步计划
- 等待 team-lead 派发 **P3.3 `lexer.rs`**（§2.3 最大匹配表 / §2.8 模式栈 CODE/STR/INTERP / `#` 在 CODE 模式非法），core-dev 链继续。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **强制看 warning**：改 `(Get-Item <file>).LastWriteTime = Get-Date` 再 `cargo build`，否则 cargo 缓存跳过、看不到 warning。
- `ext` 用 `path.rsplit(|c: char| c=='/'||c=='\\').next()`（注意 `'C:'` 的 `:` 不切分）+ `last.rfind('.')`；`eq_ignore_ascii_case("lfz")` 只折叠 ASCII（全角不匹配，已测）。
- 前导校验在**归一化之后**做，故只须检查前 4 字节 `b"#42\n"`（`\r\n`/`\r` 已被归一为 `\n`）；`split_off(4)` 取 body（索引 4 是 ASCII 边界，安全）。
- **先编码、后前导**：`String::from_utf8` 失败即 `SyntaxError`，即使文件是 `.lfz` 且缺前导。
- 测试临时文件放 `std::env::temp_dir()` + `Drop` guard 自动删除，项目目录零残留。
- 不要用 `cargo init`（会覆盖/追加 `.gitignore`/`README.md`/`LICENSE`）。
