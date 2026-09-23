# core-dev — 工作状态
> 最后更新: 2026-09-23 by core-dev

## 当前状态
P3.1（基座类型 `span.rs` + `error.rs`）✅ 完成，待 team-lead 核验 + release-manager 提交 `feat(p3): span and error model`。
**P3 串行门禁已就绪**：`Span` 与 `LzError` 已定死，core-dev 链（P3.2–P3.5）与 runtime-dev 链（P3.6–P3.9）可并行推进。

## 进行中
- （无）

## 最近完成
- **P3.1 基座类型**（2026-09-23）
  - `src/span.rs`（2305B）：`pub struct Span { pub line: u32, pub col: u32 }`（1-based；`col` 按 Unicode 标量），派生 `Clone, Copy, PartialEq, Eq, Hash, Debug`，`Display` = `line {n}, col {m}`；常量 `Span::START = {1,1}`；`const fn new`。
  - `src/error.rs`（32211B，含测试）：
    - `pub enum LzError` **严格照抄 §10.4 的 12 变体与字段顺序**（`Overflow { span, msg }`、`Io { msg, span: Option<Span> }` 等）。
    - 子消息枚举：`SyntaxMsg`（**16 变体** = `semantics.md` §8.1 SyntaxError 细分表 16 行）、`TypeMsg`（**6 变体**）、`OverflowMsg`（**1 变体** = 该行唯一消息）、`ValueMsg`（**2 变体**）。各自实现 `message()`。
    - `class_name()` / `message()` / `span()` 三方法；`Display` = `<类名>: <中文消息>`（不打印 Traceback）。
    - `pub type R<T> = Result<T, Box<LzError>>`（§10.8）；12 个 `#[cold] #[inline(never)]` 构造器 + `boxed()`。
    - `pub struct TraceFrame { pub func_id: u32, pub span: Span }`（§10.3，不存 `Rc<str>`）。**`VmFrame` 本阶段跳过**（P3 树遍历，无需 chunk/ip；契约 §10.3 允许等价结构）。
  - 证据：
    - `cargo build`（强制重编）→ `Finished dev profile ... in 0.63s`，**WARNING_LINES=0**，exit 0。
    - `cargo test` → `test result: ok. 15 passed; 0 failed`（lib harness；另 2 harness 各 0 tests），exit 0。测试名见下。
    - `cargo tree` → 仅 `lfz v0.1.0`（**零第三方依赖**）。
  - 15 个单测：`span::{start_is_line1_col1, display_uses_line_col_form, derives_copy_eq_hash, fields_are_public_and_accessible}` + `error::{class_name_all_twelve_match_spec_exactly, exactly_twelve_concrete_classes_and_base_never_thrown, messages_are_the_spec_chinese_text, syntax_msg_covers_all_sixteen_rows, type_msg_covers_all_six_rows, overflow_msg_is_single_message, span_io_none_others_some, r_is_pointer_sized, display_is_class_colon_message, cold_constructors_build_correct_class, trace_frame_holds_func_id_and_span}`。

## 阻塞 / 需要支持
- （无硬阻塞）**1 处契约歧义已按最小方式落定，请 team-lead 知悉/转 architect 确认**（不阻塞 P3.2 开工）：
  - `LzError::Assert { msg: String, span }` 单字段无法区分 `assert`（消息 `断言失败：{msg}`）与 `fail`（消息 `{msg}`）两种包装（`semantics.md` §8.1 / §4.5.10）。**我的口径**：`msg` 承载**已组装完成的最终消息**（构造点决定包装），`message()` 原样返回。若 architect 另有约定（如再拆分变体），请下 ADR，我再调整。

## 下一步计划
- 等待 team-lead 派发 **P3.2 `loader.rs`**（§10.1：UTF-8 / BOM / 行终止符 / `ext` / `#42` / `line_base`），core-dev 链继续。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- 契约 §10.4 的**字段顺序**要照抄（`Overflow { span, msg }` 是 span 在前）；`class_name()` 用 `_` 通配全部变体，新增变体时编译器会提醒漏配。
- `R<()>` 借助 `Box` 非空 niche，`size_of == size_of::<usize>()`（已用测试锁死）。
- 单变体枚举 `OverflowMsg` 不触发 dead_code（`pub` 项 + `message()` 消费）。
- 强制看 warning 的技巧：先改文件 `LastWriteTime` 再 `cargo build`，否则 cargo 缓存跳过编译、看不到 warning 输出。
- **不要用 `cargo init`**：会覆盖/追加 `.gitignore`、`README.md`、`LICENSE`。
