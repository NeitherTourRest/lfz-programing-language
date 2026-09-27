# 规范符合性审计 A —— 词法 / 文法 / 解析 / 5 特色

> 审计对象：`docs/spec/syntax.md`（910 行，v1 FROZEN，D-016）+ `docs/spec/interface-contract.md` §10.6
> 审计类型：**独立验收（只验证、不修复）** ｜ 审计者：verifier
> 日期：2026-09-27 ｜ 结论档位：**有条件通过（见缺陷单；1 项缺陷在 spec 侧）**

---

## 0. 审计基线（被审计物与环境）

| 项 | 值 | 证据 |
|---|---|---|
| 被审计可执行物 | `dist\lfz.exe`（704,000 bytes） | `Get-Item dist\lfz.exe` → `Length=704000`，`LastWriteTime=2026-09-27 13:01:12` |
| 源码基线 | git HEAD = **`34e445f`**（`docs(adr): record lfz-programming skill package upgrade`） | `git rev-parse --short HEAD` |
| 工作树（src） | **clean**（`git status --porcelain src` 空输出） | 与 HEAD 一致 |
| 新鲜度 | 最新 `src` 文件 mtime = `2026-09-27 12:59:52`（`src/cli.rs`）**早于** exe mtime `13:01:12` | 产物晚于源码，判定为新鲜 |
| 行为等价性 | `dist\lfz.exe` 与 `cargo run -q -- run` 对 3 个样例的 stdout **逐字节一致** | `s91_grades` 334/334、`s92_words` 377/377、`s94_refs` 52/52，`identical=True` |
| 构建 | `cargo build` → `Finished dev ... in 0.04s`（已最新，无 warning） | 上文命令输出 |
| 平台 | Windows / PowerShell 5.1；`dist\lfz.exe <file>`（等价 `run`） | — |

> **方法**：逐条把 `syntax.md` 的规范性条款映射为可执行 witness（临时 `.lfz` 夹具放 `%TEMP%\opencode\conformance-a\`），**实跑**取退出码 + stdout + stderr，再与条文比对。夹具为一次性验证脚本，未进入任何交付物目录。所有结论均附命令与实测输出原文。

---

## 1. 条款清单与结论总表

> `§` = `syntax.md` 行号；`IC §10.6` = `interface-contract.md` §10.6 行号。
> 结论三值：**PASS / FAIL / N-A**。

### 1.1 词法（Lexical）

| ID | 出处（§:行） | 条款（规范性） | witness | 结论 |
|---|---|---|---|---|
| A-1 | §2.2.1:114 / B2:407-410 | `#42` 恰 3 字符 + 行终止符；`#42`+EOF → CosmosAnswerError；空程序体合法 | `ok.lfz` `h_nonl.lfz` `h_wsbody.lfz` `h_commentbody.lfz` | PASS |
| A-2 | §2.2.1.4:120 / B5:422-425 | 前导无变体：`# 42`/`#42 `/`#43`/`#42abc`/`#42\t`/`##42`/`#42;`/`#42//x` 全拒 | 8 个 `h_*` 夹具 | PASS |
| A-3 | §6-B3:412-415 | 前导必须第 1 行第 1 列，前置换行/空格/Tab → CosmosAnswerError | `h_leadnl.lfz` `h_leadsp.lfz` | PASS |
| A-4 | §6-B1:402-405 | 空 `.lfz` → CosmosAnswerError；空非 `.lfz`（0 字节 `.txt`）→ 合法空程序 | `h_empty.lfz` `empty.txt` | PASS |
| A-5 | §2.1:84 / B4:417-420 | 开头 BOM 静默跳过（仅一次）；BOM 后换行/双 BOM/非开头 BOM 均拒 | `bom_ok/bom_min/bom_nl/bom_dbl/bom_mid` | PASS |
| A-6 | §2.1:85 / B7:432-435 | 接受 LF/CRLF/CR 并归一化；`U+2028` 非行终止符 → Cosmos | `h_crlf` `h_cr` `h_2028` | PASS |
| A-7 | §2.1:86 / B8:437-440 | 非 UTF-8 → `SyntaxError`（先编码后前导） | `h_bad.lfz`（`#42\n\xFF\xFE`） | PASS |
| A-8 | §2.2.0:93-110 / B9,B11 | 仅扩展名 `lfz`（ASCII 大小写不敏感）要求前导；`foo.txt/foo/foo.lfz.bak/foo.lfz.txt/.gitignore` 豁免；`Foo.LFZ`/`a.b.Lfz` 要求 | `h_note.txt` `h_noext` `h.bak` `h.lfz.txt` `.gitignore` `foo.lfz.bak` `ALPHA.lfz` `h_CASE.LFZ` `a.b.Lfz` | PASS |
| A-9 | §6-B12:476 | `//#42` 仅第 1 行触发 Cosmos；其它行为合法注释 | `h_c1.lfz` `h_c2.lfz` | PASS |
| A-10 | §2.4:174-176 / A22:387 | `//` 行注释；`/* */` 不可嵌套、等价一个空格、不产生 NEWLINE、跨行粘语句 → SyntaxError | `cm_line` `cm_block` `cm_block_ml` `cm_glue` | PASS |
| A-11 | §2.4:177 / IC §10.6:125 | 未闭合块注释 `/*`→EOF → `SyntaxError`（span 指向 `/`，消息「块注释在此处未闭合」） | `cm_unterm.lfz` | PASS |
| A-12 | §2.3:170 / A18:383 | `//` 恒为行注释，绝不作除法；`/` 为真除法 | `cm_divcomment.lfz` `cm_div.lfz` | PASS |
| A-13 | §2.6:186-192 | 标识符仅 ASCII；`_x`/`x_` 合法；非 ASCII（`é`）→ SyntaxError | `id_ok.lfz` `id_nonascii.lfz` | PASS |
| A-14 | §2.6:193-196 | 16 关键字 + 未来保留字不可作标识符 | `kw_let.lfz` `kw_future.lfz` | PASS |
| A-15 | §2.6:197 | 关键字不可作 `.字段`/`member`/`field_init` 裸名；字符串键不受限 | `kw_field_dot` `kw_member` `kw_fieldinit` `kw_field_str` | PASS |
| A-16 | §2.6:192 / A24:389 | 单独 `_` 为 `PLACEHOLDER`；绑定/主位使用 → SyntaxError | `id_ph_ok` `id_ph_bind` `ph_outside` | PASS |
| A-17 | §2.7:202-207 | `INT = dec / 0x / 0b / 0o`；`1_000` 合法 | `num_under/hex/bin/oct/funder` | PASS |
| A-18 | §2.7:203-207 | 浮点 `3.14`/`2.5e-3`/`1e10`（指数式无小数点） | `num_float.lfz` | PASS |
| A-19 | §2.7:208 / A23:388 | `1.` = `INT(1)`+`.`（字段位）；`.5` 非法；无 `..` 记号 | `num_dot` `num_leaddot` `mm_range` | PASS |
| A-20 | §B12:49 / IC §10.8:229-232 | `-9223372036854775808` 合法；越界（含 `0x…` > u64）→ SyntaxError「整数字面量超出 i64 范围」；非紧邻一元 `-` 的 `2^63` → SyntaxError | `num_i64min/max/ovf/hexovf/binmin` | PASS |
| A-21 | §2.8:222 | 转义全集 `\n \t \r \\ \" \e \$` 正确解码（`\e`=ESC） | `s_esc.lfz` | PASS |
| A-22 | §2.8:222 / M5:54 | 未列举转义（`\q`）→ SyntaxError | `s_unlisted.lfz` | PASS |
| A-23 | §2.8:222,227 / A14:379 | STR 内 `{`/`}`/`#` 均为普通字面字符 | `s_brace.lfz` `ic_hashstr.lfz` | PASS |
| A-24 | §2.8:225-226 | 字符串未闭合 / 裸换行 / EOF → SyntaxError | `s_unclosed` `s_bare_nl` `s_eof` | PASS |
| A-25 | §2.8:243 / A15:380 | `$` 后非 `{` 为字面 `$`；`\${` → 字面 `${` | `s_dollar.lfz` `esc_dollar.lfz` | PASS |
| A-26 | §2.8:222-231 | 插值多段 `${a}-${b}-${a+b}`；嵌套字符串 `"${len("abc")}"`；嵌套插值 `"${ "${n}" }"` | `ip_basic` `ip_multi` `ip_nstr` `ip_ninterp` | PASS |
| A-27 | §2.8:231 / M1:50 | INTERP/STR 内裸换行 → SyntaxError「插值表达式不能跨行」 | `ip_barenl.lfz` | PASS |
| A-28 | §A15:380 | `"${x"` 未闭合 → SyntaxError | `ip_unclosed.lfz` | PASS |
| A-29 | §2.8:236-238 | format_spec `[fill][align][sign][width][.prec][type]` | `fmt_right/fill/zero/prec/hex/bin/oct/sign/hash/stype` | PASS |
| A-30 | §2.8:240 | 非法说明符 → ValueError；类型不符 → TypeError | `fmt_bad` `fmt_mis2` `fmt_mis3` | PASS |
| A-31 | §2.8:232 | 仅在 `depth==1` 切第一个 `:`；嵌套结构体字面量的 `:` 不切 | `ip_colon.lfz` | PASS |
| A-32 | §2.3:156-169 | 最大匹配；禁止合成 `>> << ++ -- -> .. ?? :: \| &` | `mm_gt2/pipe1/amp/arrow/range/inc/dcolon` `a16_subneg` | PASS |
| A-33 | §2.3:161-165 / A10-12:375-377 | `;`/`;;`/`;;;`/`; ;`/`;;;;` 最长匹配行为 | `mm_semi/dump3/semi2/dump4/dump` | PASS |
| A-34 | §2.2.2:153-154 / IC §10.6:125 | CODE 模式 `#` 非法（`let x = 1 # 2` → SyntaxError） | `ic_hash.lfz` | PASS |

### 1.2 文法（§3–§7）

| ID | 出处（§:行） | 条款 | witness | 结论 |
|---|---|---|---|---|
| A-35 | §7:524 | `let`/`var` 声明；`let` 不可重绑定、`var` 可 | `st_letvar.lfz` | PASS |
| A-36 | §7:525-526 | 赋值含复合 `+= -= *= /= %=` | `st_add/sub/mul/div/mod` | PASS |
| A-37 | §7:541 / §3.3:283 | 表达式语句；语句首 `{` 走 struct 字面量 | `st_exprstmt.lfz` | PASS |
| A-38 | §7:573-574 | `if`/`else if`/`else` | `st_if` `st_elif` | PASS |
| A-39 | §7:535 | `while <expr> block` | `st_while.lfz` | PASS |
| A-40 | §7:536 | `for <id> in <expr> block` | `st_for.lfz` | PASS |
| A-41 | §7:538-539 | `break`/`continue` | `st_break` `st_continue` | PASS |
| A-42 | §7:537 / A19:384 | `return [expr]`；行尾即返回 `nil` | `st_return` `nl_return` | PASS |
| A-43 | §7:529-530 | `fn` 声明三种体：块 / `=> expr` / `=> {block}` | `st_fn` `st_arrow` `st_arrow_block` | PASS |
| A-44 | §7:531-533,569 | `struct` 模板 + 实例化（缺省字段继承） | `st_struct` `struct_ml` | PASS |
| A-45 | §7:515 | 块 `{}`（可嵌于 `()`/`[]`） | `nl_blockcall` `nested_block` | PASS |
| A-46 | §7:540 / A10:375 | `;;` 独立语句、独占逻辑行（允许行首/尾空白与尾随 `//`） | `st_dump` `dump_comment` `dump_ws` `dumpa10` | PASS |
| A-47 | §7:555-566 | 字面量/标识符/数组/结构体/字段/下标/调用（含链式 `p.a[1]`） | `ex_arr` `ex_idx` `chain_field/index/mix` | PASS |
| A-48 | §7:554 / A16:381 | 一元 `-`/`!`；`5--3`=`8`（无 `--` 记号） | `ex_unary` `pr_uneg` `a16_subneg` | PASS |
| A-49 | §7:547-553 | 二元算术/比较/相等/逻辑 | `pr_muladd` `pr_left` `pr_cmpeq` `pr_andor` `op_*` | PASS |
| A-50 | §7:573-574 | `if` 表达式（有/无 else；作实参） | `ex_ifexpr` `ex_ifnoelse` `ifexpr_arg` | PASS |
| A-51 | §7:575-576 / A20:385 | 函数字面量 `fn(...)` 与 `(...) => ...`；`(a+b)=>c` → SyntaxError | `ex_lamfn/lamarrow/lamablock` `empty_lam` `a20_lambda` | PASS |
| A-52 | §4.3:337-338 / §4.4:349-350 | 管道脱糖：data-last；`_` 注入；链式；lambda RHS | `f1_datalast/rhs/lambda/chain` `ex_pipe` | PASS |
| A-53 | §4.3 规则6:342 / A24:389 | `_` 不穿 λ（λ 内非法）；≥2 个 `_` 非法；深层非 λ 嵌套合法 | `ph_lambda` `ph_two` `ph_deep` `id_ph_ok` | PASS |
| A-54 | §4.1:315-329 / §4.4 | 优先级与结合性（`\|>` 比 `+` 松、比比较紧；全左结合；`1<2<3`=(1<2)<3→TypeError） | `pr_muladd/pipeadd/pipecmp/pipeaddbad/left` `cmp_chain` | PASS |
| A-55 | §3.2:261-273 | 换行模式栈 SIG/IGN（实参/数组/成员表内 NEWLINE 忽略；块内有效） | `nl_ign` `nl_sig` `struct_ml` `arr_ml` `nested_block` | PASS |
| A-56 | §3.1:254-255 / A2:367 / A17:382 | 括号是唯一续行；行尾运算符、行首 `\|>` 不续行 | `nl_paren` `nl_badcont` `nl_leadpipe` `a17_pipeparen` | PASS |
| A-57 | §3.5:303-305 / A3,A4:368-369 | 块前换行跳过；`else` 前换行回退；无悬挂 else，绑定最近未闭合 `if` | `nl_ifbrace` `nl_else` `nl_else2` `dangle` `nl_trap` | PASS |
| A-58 | §3.3 规则2:283-286 / A9:374 / IC §10.6:126 | 无裸块语句；语句首 `{` = 匿名 struct；`{ let x=1 }`/`{ ;; }` → SyntaxError；`{}`/`{ "k":1 }` 合法 | `ic_anon` `ic_stmtlet` `ic_stmtdump` `st_exprstmt` | PASS |
| A-59 | §3.4:295-299 / A6:371 | NO_BRACE_LITERAL：`if`/`while` 条件位与 `for` 可迭代位禁裸 struct；括号暂清 | `nbl_ok` `nbl_bad` `nbl_paren` `nbl_while` `nbl_for` | PASS |
| A-60 | §3.3:289 / A27:392 / M3:52 | `=>` 后紧邻 `{` 恒为块；返回 struct 须 `=> ({…})` | `st_arrow_block` `m3_bad` `m3_ok` | PASS |
| A-61 | §A7:372 / A8:373 | 成员/元素/实参逗号必填、尾逗号可选 | `st_struct` `ex_arr` `a8_argcomma` | PASS |
| A-62 | §A11:376 | 单 `;` 文法从不接受 → SyntaxError | `st_semi_block` `mm_semi` | PASS |
| A-63 | §A21:386 | 赋值左侧须为 lvalue（`f()=1`、`a+b=1` → SyntaxError） | `a21_call` `a21_expr` | PASS |
| A-64 | §A26:391 | `if_expr`/`lambda` 不作 postfix 基（须加括号） | `pr_a26.lfz` | PASS |
| A-65 | §semantics §3.6 / A13:378 | `;;` 语义：完整可见链、内→外、slot 升序、遮蔽去重、stdout 交织、闭包捕获可见 | `dumpshadow` `dump_closure` `dump_while` `ic_dumporder` | PASS |

### 1.3 五大特色（D-007）

| ID | 出处 | 条款 | witness | 结论 |
|---|---|---|---|---|
| A-66 | §4.3:331 / §12:831 | 特色1 管道（parser 脱糖、运行时零开销） | `f1_*` `ex_pipe*` | PASS |
| A-67 | §1:71 / §12:831 | 特色2 合一 struct（对象面孔 `.k` 与字典面孔 `["k"]` 同源存储；方法不入数据面） | `f2_same` `f2_dictset` `f2_method` `f2_mcall` | PASS |
| A-68 | §2.8:210 / §12:831 | 特色3 富字符串插值（多段/嵌套/format_spec） | `f3_fmt` + A-26..A-31 | PASS |
| A-69 | §8 / D-007；IC §10.4 | 特色4 结构化错误 + `assert`/`check`：12 错误类 + 位置 + 插入符 + traceback + `--json` | `e_name/type/index/field/div/mod/divbi/ovf/value/io/assert/check/recur/trace` `--json` | PASS |
| A-70 | §1:72 / §2.3:170 / §4.2 | 特色5 确定性语义：溢出即错、无 truthiness、唯一隐式 `int→float`、`/`真除/`div`下取整/`%`随除数、`seed` 可复现 | `e_ovf` `f5_truthy/widen/truediv/div/divneg/mod/mod2/seed/float/absmin` | PASS |

### 1.4 spec 样例程序（§9）

| ID | 出处（§:行） | 条款 | witness | 结论 |
|---|---|---|---|---|
| A-71 | §9.4:815-823 | 样例 D `refs.lfz` 输出与 spec 逐行一致 | `s94_refs.lfz` | PASS |
| A-72 | §9.1:673-680 | 样例 A `grades.lfz` 的 `;;` dump 输出与 spec 逐行一致 | `s91_grades.lfz` | PASS |
| A-73 | §9.2:733-740 | 样例 B `words.lfz` 的 `;;` dump 输出与 spec 逐行一致 | `s92_words.lfz` | PASS |
| A-74 | §9.2:742-750 | 样例 B `words.lfz` 程序主体输出与 spec 一致 | `s92_words.lfz` | **FAIL** |
| A-75 | §9.3:752-778 | 样例 C 片段作为 `.lfz` 文件可运行（exit 0） | `s93_bars.lfz` | PASS |

### 1.5 接口契约 §10.6

| ID | 出处（IC:行） | 条款 | witness | 结论 |
|---|---|---|---|---|
| A-76 | IC §10.6:125 | lexer：模式栈 CODE/STR/INTERP、最大匹配表、CODE 模式 `#` 非法、未闭合块注释 → SyntaxError | A-11/A-23/A-26/A-31/A-34 | PASS |
| A-77 | IC §10.6:126 | parser：SIG/IGN 栈、NO_BRACE_LITERAL、`;;`→Dump、管道→Call、字段名须 IDENT、语句首 `{` 非裸块 | A-46/A-52/A-55/A-58/A-59 + A-15 | PASS |

### 1.6 N-A（规范延后 / 环境不可构造）

| ID | 出处 | 说明 | witness | 结论 |
|---|---|---|---|---|
| A-78 | §6-B9:443 / B11:469 | REPL / stdin / `-e` 入口前导豁免。spec 状态列为 `✅/⏸`（延后）。实测 `lfz run -e 'print(1+2)'` → 参数错误 exit 2；`lfz run -` → `IOError`（未实现 stdin） | 见 §3.3 | N-A |
| A-79 | §2.2.0:107 | `foo.`（扩展名为空串）豁免判定。Windows 文件系统不支持尾点文件名，无法构造等价 witness；仅静态核对 `ext()` 定义 | — | N-A |

---

## 2. 逐条详细证据（原始命令与输出）

> 通用命令形态：`dist\lfz.exe <file.lfz>`；输出经 `Start-Process -RedirectStandardOutput/Error` 取原始字节后按 UTF-8 解码。下文 `⇥` 表示换行。

### 2.1 词法证据（精选）

**(A-1/A-2/A-3/A-4) 前导严格性**
```
[ok.lfz]               exit=0 out=<OK>
[h_empty.lfz]          exit=2 err=<File "...", line 1⇥CosmosAnswerError: 你忘记了宇宙的答案>
[h_nonl.lfz] (#42 无换行) exit=2 err=<... line 1⇥CosmosAnswerError: 你忘记了宇宙的答案>
[h_space.lfz] (# 42)   exit=2 err=<... CosmosAnswerError>
[h_trailsp.lfz] (#42␠) exit=2 err=<... CosmosAnswerError>
[h_43.lfz] [#43]       exit=2 err=<... CosmosAnswerError>
[h_abc.lfz] [#42abc]   exit=2 err=<... CosmosAnswerError>
[h_tab.lfz] [#42\t]    exit=2 err=<... CosmosAnswerError>
[h_hash2.lfz] [##42]   exit=2 err=<... CosmosAnswerError>
[h_leadnl.lfz]         exit=2 err=<... CosmosAnswerError>
[h_leadsp.lfz] [␠#42]  exit=2 err=<... CosmosAnswerError>
[h_semi.lfz] [#42;]    exit=2 err=<... CosmosAnswerError>
[h_comment.lfz][#42//x] exit=2 err=<... CosmosAnswerError>
[plain_min.lfz] [#42\n] exit=0 out=<>
[empty.txt] (0 byte)   exit=0 out=<>
```
> 注：`CosmosAnswerError` 输出**不含**源码行/插入符/编号/hint（B10 符合），消息恒为「你忘记了宇宙的答案」。

**(A-5) BOM**
```
[bom_ok.lfz] 前16字节 = EF BB BF 23 34 32 0A 70 72 69 6E 74 28 22 42 4F   → exit=0 out=<BOMOK>
[bom_min.lfz] (BOM+#42\n)                                                    → exit=0 out=<>
[bom_nl.lfz] (BOM+\n#42)        → exit=2 CosmosAnswerError
[bom_dbl.lfz] (BOM+BOM+#42)     → exit=2 CosmosAnswerError
[bom_mid.lfz] (#42\n [BOM] print) → exit=2 SyntaxError: 非法字符 '<BOM>'  (line 2)
[nobom.txt] (BOM + print, 非.lfz) → exit=0 out=<TXT-BOM-OK>
```

**(A-6) 行终止符**
```
[h_crlf.lfz] exit=0 out=<CRLF>
[h_cr.lfz]   exit=0 out=<CR>
[h_2028.lfz] exit=2 CosmosAnswerError
```

**(A-7) 非 UTF-8**
```
[h_bad.lfz] exit=2 err=<File "...", line 1⇥SyntaxError: 文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 4）>
```

**(A-8/A-9) 扩展名与 N2**
```
[h_note.txt]  exit=0 out=<TXT-NO-HEADER>   (# 执行)
[h_noext]     exit=0 out=<NOEXT>
[h_dot.]      exit=0 out=<DOT>
[h.bak]       exit=0 out=<BAK>
[h.lfz.txt]   exit=0 out=<LFZTXT>
[.gitignore]  exit=0 out=<GITIGN>
[foo.lfz.bak] exit=0 out=<BAK2>
[ALPHA.lfz]   exit=0 out=<ALPHA>
[h_CASE.LFZ]  exit=0 out=<CASE>
[a.b.Lfz]     exit=0 out=<MULTIDOT>
[h_c1.lfz] (#1行=//#42) exit=2 CosmosAnswerError
[h_c2.lfz] (#2行=//#42) exit=0 out=<ok>
```

**(A-10/A-11/A-12) 注释**
```
[cm_line.lfz]   exit=0 out=<1⇥2>
[cm_block.lfz]  exit=0 out=<1⇥2>
[cm_block_ml.lfz] (块注释跨行粘成一行) exit=0 out=<1>
[cm_glue.lfz] (两语句被跨行块注释粘接) exit=2 err=<... line 3⇥SyntaxError: 语句之间必须有换行>
[cm_unterm.lfz] exit=2 err=<... line 2⇥/* unterminated⇥    ^⇥SyntaxError: 块注释在此处未闭合（缺少 '*/'）>
[cm_div.lfz] (7 / 2) exit=0 out=<3.5>
[cm_divcomment.lfz] (4 // 2) exit=2 err=<SyntaxError: 这里期待 ')'，但得到 '文件末尾'>
```

**(A-13/A-14/A-15/A-16) 标识符、关键字、字段名、占位符**
```
[id_ok.lfz]        exit=0 out=<3>            (_x / x_ 合法)
[id_nonascii.lfz]  exit=2 err=<SyntaxError: 非法字符 'é'>
[id_ph_ok.lfz]     exit=0 out=<3>
[id_ph_bind.lfz]   exit=2 err=<SyntaxError: 占位符 '_' 只能出现在管道右侧的调用实参中>
[kw_let.lfz]       exit=2 err=<SyntaxError: 这里期待 变量名，但得到 'let'>
[kw_future.lfz]    exit=2 err=<SyntaxError: 这里期待 变量名，但得到 'match'>
[kw_field_dot.lfz] exit=2 err=<SyntaxError: 这里期待 字段名，但得到 'self'>
[kw_field_str.lfz] exit=0 out=<1>            (r["self"] 合法)
[kw_member.lfz]    exit=2 err=<SyntaxError: 这里期待 字段名或方法，但得到 'self'>
[kw_fieldinit.lfz] exit=2 err=<SyntaxError: 这里期待 字段名，但得到 'self'>
```

**(A-17/A-18/A-19/A-20) 数值**
```
[num_under.lfz] exit=0 out=<2000>          (1000 + 1_000)
[num_hex.lfz]   exit=0 out=<255>           (0xFF)
[num_bin.lfz]   exit=0 out=<10>            (0b1010)
[num_oct.lfz]   exit=0 out=<493>           (0o755)
[num_float.lfz] exit=0 out=<3.14⇥0.0025⇥10000000000.0>
[num_funder.lfz]exit=0 out=<1000.5>        (1_000.5)
[num_dot.lfz]   exit=2 err=<SyntaxError: 这里期待 字段名，但得到 ')'>   (1.)
[num_leaddot.lfz]exit=2 err=<SyntaxError: 这里期待 表达式，但得到 '.'>  (.5)
[num_i64min.lfz]exit=0 out=<-9223372036854775808>
[num_i64max.lfz]exit=0 out=<9223372036854775807>
[num_ovf.lfz]   exit=2 err=<SyntaxError: 整数字面量超出 i64 范围>
[num_hexovf.lfz]exit=2 err=<SyntaxError: 整数字面量超出 i64 范围>
[num_binmin.lfz]exit=2 err=<SyntaxError: 整数字面量超出 i64 范围>   (0 - 2^63)
```

**(A-21…A-31) 字符串 / 插值 / format_spec**
```
[s_esc.lfz]    exit=0 out=<nl:⇥|tab:<TAB>|cr:<CR>|bs:\|q:"|esc:<ESC>|dol:$>
[s_unlisted.lfz] exit=2 err=<SyntaxError: 字符串中不支持的转义 '\q'>
[s_brace.lfz]  exit=0 out=<{}>
[s_unclosed.lfz] exit=2 err=<SyntaxError: 字符串字面量在此处未闭合>
[s_bare_nl.lfz]  exit=2 err=<SyntaxError: 字符串字面量在此处未闭合>
[s_eof.lfz]      exit=2 err=<SyntaxError: 字符串字面量在此处未闭合>
[s_dollar.lfz]   exit=0 out=<a$b>
[esc_dollar.lfz] exit=0 out=<${x}>           (\${)
[ip_basic.lfz]   exit=0 out=<x=5>
[ip_multi.lfz]   exit=0 out=<1-2-3>
[ip_nstr.lfz]    exit=0 out=<3>              ("${len("abc")}")
[ip_ninterp.lfz] exit=0 out=<3>              ("${ "${n}" }")
[ip_barenl.lfz]  exit=2 err=<SyntaxError: 插值表达式不能跨行；请把表达式写在一行内>
[ip_unclosed.lfz]exit=2 err=<SyntaxError: 字符串字面量在此处未闭合>
[ip_colon.lfz]   exit=0 out=<5>              ${ { "a": 5 }["a"] } 的 ':' 不切
[fmt_right.lfz]  exit=0 out=<   42>
[fmt_fill.lfz]   exit=0 out=<**42***>        (*^7)
[fmt_zero.lfz]   exit=0 out=<00042>          (05d)
[fmt_prec.lfz]   exit=0 out=<3.14>           (.2f)
[fmt_hex.lfz]    exit=0 out=<ff>             (:x)
[fmt_bin.lfz]    exit=0 out=<101>            (:b)
[fmt_oct.lfz]    exit=0 out=<10>             (:o)
[fmt_sign.lfz]   exit=0 out=<+42>            (:+d)
[fmt_hash.lfz]   exit=0 out=<###42>          (:#>5, '#' 字面)
[fmt_bad.lfz]    exit=2 err=<ValueError: 格式说明符非法：'zzz'>
[fmt_mis2.lfz]   exit=2 err=<TypeError: 格式说明符 'd' 不适用于 string>
[fmt_mis3.lfz]   exit=2 err=<TypeError: 格式说明符 'd' 不适用于 float>
```

**(A-32/A-33/A-34) 最大匹配**
```
[mm_gt2.lfz] (1 >> 2)   exit=2 err=<SyntaxError: 这里期待 表达式，但得到 '>'>
[mm_pipe1.lfz] (true | false) exit=2 err=<SyntaxError: 非法字符 '|'>
[mm_amp.lfz]  (true & false)  exit=2 err=<SyntaxError: 非法字符 '&'>
[mm_arrow.lfz] (1 -> 2)  exit=2 err=<SyntaxError: 这里期待 表达式，但得到 '>'>
[mm_range.lfz] (1..2)    exit=2 err=<SyntaxError: 这里期待 字段名，但得到 '.'>
[mm_inc.lfz]   (x++)     exit=2 err=<SyntaxError: 这里期待 表达式，但得到 '+'>
[mm_dcolon.lfz](1 :: 2)  exit=2 err=<SyntaxError: 这里期待 ')'，但得到 ':'>
[mm_semi.lfz]  (print(1);)   exit=2 err=<SyntaxError: 语句之间必须有换行>
[mm_dump3.lfz] (;;;)     exit=2 err=<SyntaxError: 语句之间必须有换行>
[mm_semi2.lfz] (; ;)     exit=2 err=<SyntaxError: 单独的 ';' 非法；打印变量请用 ';;'>
[mm_dump4.lfz] (;;;;)    exit=2 err=<SyntaxError: 语句之间必须有换行>
[mm_dump.lfz]  (let x=1 \n ;;) exit=0 out=<x ： 1>
[ic_hash.lfz]  (let x = 1 # 2) exit=2 err=<SyntaxError: '#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)>
```

### 2.2 文法证据（精选）

```
[st_letvar.lfz]  exit=0 out=<4>
[st_add.lfz] exit=0 out=<3>   [st_sub.lfz] exit=0 out=<3>   [st_mul.lfz] exit=0 out=<15>
[st_div.lfz] exit=0 out=<2.5> [st_mod.lfz] exit=0 out=<1>
[st_if.lfz]   exit=0 out=<gt>
[st_elif.lfz] exit=0 out=<b>
[st_while.lfz]exit=0 out=<0⇥1⇥2>
[st_for.lfz]  exit=0 out=<0⇥1⇥2>
[st_break.lfz]exit=0 out=<0⇥1>
[st_continue.lfz] exit=0 out=<0⇥2⇥3>
[st_return.lfz] exit=0 out=<pos⇥neg>
[st_fn.lfz]   exit=0 out=<5>
[st_arrow.lfz]exit=0 out=<5>
[st_arrow_block.lfz] exit=0 out=<1>
[st_struct.lfz] exit=0 out=<10⇥2>            (P{x:10}：x=10, 缺省 y=2)
[st_method.lfz] exit=0 out=<6>               (self 方法写回)
[st_semi_block.lfz] exit=2 err=<SyntaxError: 语句之间必须有换行>   (块内单 ';')
[ex_arr.lfz]  exit=0 out=<3>   [ex_idx.lfz] exit=0 out=<20⇥30>   (负索引)
[ex_ifexpr.lfz] exit=0 out=<a> [ex_ifnoelse.lfz] exit=0 out=<nil>
[ex_lamfn/lamarrow/lamablock] 均 exit=0 out=<42>
[ex_pipe.lfz] exit=0 out=<[1, 2, 3]>         ([3,1,2] |> sort())
[ex_pipeph.lfz] exit=0 out=<7>               (10 |> sub(_, 3))
[ex_pipechain.lfz] exit=0 out=<[20, 40]>
[ex_pipenlam.lfz] exit=0 out=<6>
[pr_muladd.lfz] exit=0 out=<14>   [pr_pipeadd.lfz] exit=0 out=<3>
[pr_pipecmp.lfz] exit=0 out=<true> [pr_pipeaddbad.lfz] exit=2 err=<SyntaxError: 这里期待 ')'，但得到 '+'>
[pr_left.lfz] exit=0 out=<5>       (10-3-2 左结合)
[cmp_chain.lfz] exit=2 err=<TypeError: 运算符 '<' 不支持 bool 与 int>  (1<2<3 左结合)
[pr_a26.lfz] exit=2 err=<SyntaxError: 这里期待 ')'，但得到 '('>
[a20_lambda.lfz] exit=2 err=<SyntaxError: ...>  ((a+b)=>c)
[a21_call.lfz]  exit=2 err=<SyntaxError: 赋值左侧必须是变量、字段或下标>
[a21_expr.lfz]  exit=2 err=<SyntaxError: 赋值左侧必须是变量、字段或下标>
[nl_ign.lfz]    exit=0 out=<1 2>             (实参内换行忽略)
[nl_sig.lfz]    exit=2 err=<SyntaxError: 语句之间必须有换行>
[nl_blockcall.lfz] exit=0 out=<11>           (块嵌在实参内，块内换行有效)
[nl_paren.lfz]  exit=0 out=<3>               (括号续行)
[nl_badcont.lfz]exit=2 err=<SyntaxError: 表达式未结束：行尾不能终止表达式；请用括号跨行>
[nl_leadpipe.lfz]exit=2 err=<SyntaxError: 这里期待 表达式，但得到 '|>'>
[nl_ifbrace.lfz]exit=0 out=<brace-nl>        (块前换行)
[nl_else.lfz]   exit=0 out=<b>  [nl_else2.lfz] exit=2 err=<SyntaxError: 这里期待 表达式，但得到 'else'>
[dangle.lfz]    exit=0 out=<o>               (else 绑最近未闭合 if)
[nl_trap.lfz]   exit=0 out=<ran>             (A3 陷阱：两句不报错)
[nbl_ok.lfz] exit=0 out=<pos>  [nbl_paren.lfz] exit=0 out=<pos>  [nbl_while.lfz] exit=0 out=<1>
[nbl_bad.lfz] exit=2 err=<SyntaxError: 语句之间必须有换行>
[nbl_for.lfz] exit=2 err=<SyntaxError: 语句之间必须有换行>
[m3_bad.lfz] exit=2 err=<SyntaxError: 语句之间必须有换行>
[m3_ok.lfz]  exit=0 out=<1>
[ph_lambda.lfz] exit=2 err=<SyntaxError: 占位符 '_' 只能出现在管道右侧的调用实参中>
[ph_two.lfz] exit=2 err=<SyntaxError: 管道右侧调用最多只能有一个 '_'>
[ph_deep.lfz] exit=0 out=<1>
[ic_anon.lfz] exit=0 out=<struct>  [ic_stmtlet.lfz] exit=2 [ic_stmtdump.lfz] exit=2
[dumpshadow.lfz] exit=0 out=<x ： 2⇥f ： <fn f>>      (遮蔽去重，内层胜)
[dump_closure.lfz] exit=0 out=<inc ： <fn inc>⇥n ： 1⇥makeCounter ： <fn makeCounter>⇥c ： <fn inc>>  (闭包捕获可见)
[dump_while.lfz] exit=0 out=<t ： 0⇥i ： 0⇥t ： 10⇥i ： 1>  (每次迭代 dump)
[ic_dumporder.lfz] exit=0 out=<1⇥x ： 1⇥end>          (stdout 交织)
```

### 2.3 五大特色证据

```
[f2_same.lfz]     exit=0 out=<1⇥1⇥2>       (s.a / s["a"] 同源；s.a=2 后 s["a"]=2)
[f2_dictset.lfz]  exit=0 out=<9>           (s["b"]=9 后 s.b=9)
[f2_method.lfz]   exit=0 out=<["a"]⇥1⇥false>  (keys 仅数据字段；len=1；has("m")=false)
[f2_mcall.lfz]    exit=0 out=<1>
[f2_del.lfz]      exit=0 out=<{}>
[f2_delmethod.lfz]exit=2 err=<FieldError: 结构体没有字段 'm'>
[f1_datalast.lfz] exit=0 out=<a,b>
[f1_rhs.lfz]      exit=0 out=<3>
[f1_lambda.lfz]   exit=0 out=<20>
[f1_place.lfz]    exit=0 out=<0>           (7 |> div(2, _) = div(2,7) = 0)
[f3_fmt.lfz]      exit=0 out=<LFZ-00042-2a-      42>
[e_name.lfz]  exit=2 err=<NameError: 未定义的名字 'undefinedVar'>
[e_type.lfz]  exit=2 err=<TypeError: 运算符 '+' 不支持 int 与 string>
[e_index.lfz] exit=2 err=<IndexError: 下标 5 越界（长度 2）>
[e_field.lfz] exit=2 err=<FieldError: 结构体没有字段 'b'>
[e_div.lfz]   exit=2 err=<ZeroDivisionError: 除以零>
[e_mod.lfz]   exit=2 err=<ZeroDivisionError: 对零取模>
[e_divbi.lfz] exit=2 err=<ZeroDivisionError: 除以零>
[e_ovf.lfz]   exit=2 err=<OverflowError: 整数溢出：结果超出 i64 范围>
[e_value.lfz] exit=2 err=<ValueError: 无法从 string 转换到 int：'abc'>
[e_io.lfz]    exit=2 err=<IOError: 输入结束（EOF）>
[e_assert.lfz]exit=2 err=<AssertionError: 断言失败：boom>   (致命，未打印 after)
[e_check.lfz] exit=0 out=<continued> err=<check 失败：warn>  (非致命，继续)
[e_recur.lfz] exit=2 err=<RecursionError: 递归深度超限（超过 10000 层）> + traceback 折叠「... 省略 9961 帧 ...」（头 10 + 尾 30）
[e_trace.lfz] exit=2 err=<Traceback ...>
   File "...", line 4, in <module>   main()
   File "...", line 3, in main       fn main() { half("x") }
   File "...", line 2, in half       fn half(n) => n / 2
   TypeError: 运算符 '/' 不支持 string 与 int
[e_assert_json --json] exit=2
   {"ok":false,"error":"AssertionError","message":"断言失败：boom","file":"...","line":2,"col":1,
    "traceback":[{"file":"...","line":2,"func":"<module>"}]}
[f5_truediv.lfz] exit=0 out=<3.5>   [f5_div.lfz] exit=0 out=<3>   [f5_divneg.lfz] exit=0 out=<-4>
[f5_mod.lfz] exit=0 out=<2>  (-7 % 3)   [f5_mod2.lfz] exit=0 out=<-2>  (7 % -3)
[f5_truthy.lfz] exit=2 err=<TypeError: 条件必须是 bool，得到 int>
[f5_widen.lfz] exit=0 out=<2.5>      (唯一隐式 int→float)
[f5_absmin.lfz] exit=2 err=<OverflowError: 整数溢出：结果超出 i64 范围>
[f5_seed.lfz] exit=0 out=<44⇥44>     (同 seed 同序列)
[f5_float.lfz] exit=0 out=<0.30000000000000004>  (IEEE 754 浮点)
```

### 2.4 样例程序证据（§9）

**(A-71) §9.4 `refs.lfz`** —— spec 期望（L817-823）vs 实测：
```
spec:                          实测:
1 2 3                          1 2 3
99 99                          99 99
[3, 1, 2]  [1, 2, 3]           [3, 1, 2]  [1, 2, 3]
{me: <cycle>}                  {me: <cycle>}
true                           true
```
→ **逐行一致 PASS**（exit 0）。

**(A-72) §9.1 `grades.lfz` 的 `;;` dump** —— spec 期望（L676-680）vs 实测：
```
spec:                                                    实测:
Student ： <struct Student>                               Student ： <struct Student>
sortDesc ： <fn sortDesc>                                 sortDesc ： <fn sortDesc>
average ： <fn average>                                   average ： <fn average>
roster ： [{name: "Alice", score: 93}, {name: "Bob",      roster ： [{name: "Alice", score: 93}, {name: "Bob",
score: 67}, {name: "Cara", score: 88}]                    score: 67}, {name: "Cara", score: 88}]
```
→ **逐行一致 PASS**（程序体其余输出：`平均分 = 82.67`、`最高分 = Alice (93)`、★/· 标记均合理）。

**(A-73) §9.2 `words.lfz` 的 `;;` dump** —— spec 期望（L735-739）vs 实测：
```
spec:  Word ： <struct Word>
       countWords ： <fn countWords>
       text ： the quick the fox the dog a quick fox
       freq ： {a: {count: 1, text: "a"}, dog: {count: 1, text: "dog"}, fox: {count: 2, text: "fox"}, quick: {count: 2, text: "quick"}, the: {count: 3, text: "the"}}
实测:  与上逐字一致 → PASS
```

**(A-74) §9.2 `words.lfz` 程序主体** —— spec 期望（L744-750）vs 实测：
```
spec 期望:                         实测:
词频 Top 3                         词频 Top 3
         the │   3                           the │   3
       quick │   2                           fox │   2
         fox │   2                         quick │   2
pi=3.142  n=00042  hex=2a  right=    42   pi=3.142  n=00042  hex=2a  right=    42
```
→ **顺序不一致 FAIL**（tie 组 `quick`/`fox` 位置相反）。详见缺陷单 BUG-A-01。

**(A-75) §9.3 `bars.lfz` 片段** —— `exit=0`，`stdoutBytes=5438367`，stderr 空，tail 字节 `8E 92 E5 BA 8F E5 AE 8C E6 88 90 0A` = `排序完成\n` → **可运行 PASS**。

### 2.5 接口契约 §10.6 证据

- lexer（A-76）：模式栈 CODE/STR/INTERP 见 A-23/A-26/A-31；最大匹配见 A-32/A-33；CODE `#` 非法见 A-34；未闭合块注释见 A-11。→ 全覆盖 PASS。
- parser（A-77）：SIG/IGN 见 A-55；NO_BRACE_LITERAL 见 A-59；`;;`→Dump 见 A-46/A-65；管道→Call 见 A-52；字段名须 IDENT 见 A-15；语句首 `{` 非裸块见 A-58。→ 全覆盖 PASS。

### 2.6 与「源」（cargo run）的等价性

```
对 s91_grades / s92_words / s94_refs 分别取 dist\lfz.exe 与 cargo run -q -- run 的 stdout 原始字节：
  s91_grades.lfz: distExit=0 cargoExit=0 bytes=334/334 identical=True
  s92_words.lfz : distExit=0 cargoExit=0 bytes=377/377 identical=True
  s94_refs.lfz  : distExit=0 cargoExit=0 bytes=52/52  identical=True
```
→ 被审计物 `dist\lfz.exe` 与 HEAD 源码行为一致。

---

## 3. 缺陷单

```
【缺陷单】BUG-A-01（spec 侧）
- 交付物: docs/spec/syntax.md §9.2「样例 B —— 词频统计」期望输出（L742-750）
- 分类: 规范内部不一致（样例期望值 与 §10.7 规范性排序/键序规则 冲突）；**非解释器缺陷**
- 最小复现:
    1) 把 §9.2 words.lfz（L685-730）原样保存为 s92_words.lfz（首行 #42）
    2) 运行: dist\lfz.exe s92_words.lfz
- 期望（spec L744-750）:
    词频 Top 3
             the │   3
           quick │   2
             fox │   2
- 实际（exit 0）:
    词频 Top 3
             the │   3
             fox │   2
           quick │   2
- 判据（实测）:
    §10.7 规定 keys(s)=数据字段键、字节序升序；values(s) 与 keys 同序；sortBy 稳定。
    实测 keys({b,a,c})=["a","b","c"]、values= [2,1,3]（a,b,c 序）、sortBy 对相同键保持输入序（稳定）。
    §9.2 中 keys 排序为 a<dog<fox<quick<the（'f'=0x66 < 'q'=0x71），
    values 同序，sortBy((w)=>-w.count) 稳定升序 → the(3), fox(2), quick(2)。
    → 解释器输出「the/fox/quick」**符合全部规范性条款**；spec §9.2 样例期望「the/quick/fox」自相矛盾。
- 严重度: 低（文档/样例错误；不影响解释器行为，但会误导黑盒测试期望与评分）
- 建议 owner: language-architect（仅订正 §9.2 样例期望输出，或显式声明 tie 处理；**不得改实现**）
```

> 说明：本审计**未发现解释器侧 FAIL**。唯一 FAIL 属规范文本自洽性问题。

---

## 4. 观察项（非 FAIL，记录备查）

```
【观察 obs-A-01】入口模式 `-e` / stdin 未实现（spec 标注 ⏸ 延后）
- 出处: syntax.md §6-B9:443（正例含 `lfz run -e ...`、`echo ... | lfz run -`）；B11:469 状态列 `✅/⏸`
- 实测: `lfz run -e 'print(1+2)'` → exit 2 err=<lfz: 'run' 只接受一个 <file> 参数>；
        `lfz run -` → exit 2 err=<IOError: 无法读取->
- 判定: N-A（规范允许延后）；不属词法/文法违反。若课程要求 `-e`/stdin，需 tooling-dev 另行实现。

【观察 obs-A-02】CLI `--help` 文案与实现/规范不一致
- 出处: `dist\lfz.exe --help` 输出含「<file> 须以 .lfz 结尾；否则报错并退出码 2」
- 实测: 非 `.lfz` 文件（`h_note.txt`/`h_noext`/`.gitignore`/`foo.lfz.bak`）**均可正常运行**，符合 §2.2.0 扩展名豁免
- 判定: help 文案陈旧/误导（属 tooling-dev 文档瑕疵，低）。不影响规范符合性。

【观察 obs-A-03】个别 SyntaxError 消息措辞不精确
- 例: `(a + b) => c`、`fn f() => { k: 1 }`、`if P { x:1 }.x > 0 {}` 的报错均落为「语句之间必须有换行」，
  类名正确（SyntaxError）、位置正确，但消息未点明真实原因。低，属实现体验问题。
```

---

## 5. 统计

| 项 | 数量 |
|---|---|
| 条款总数（checked） | 79 |
| **PASS** | **76** |
| **FAIL** | **1**（BUG-A-01，spec 侧） |
| **N-A** | **2**（A-78 `-e`/stdin 延后；A-79 `foo.` 空扩展名 Windows 不可构造） |

细分：词法 34 条（全 PASS）；文法 31 条（全 PASS）；5 特色 5 条（全 PASS）；样例 5 条（4 PASS / 1 FAIL）；IC §10.6 2 条（全 PASS）；N-A 2 条。

**解释器侧 FAIL = 0**；唯一的 FAIL 是**规范样例文本自洽性**问题（§9.2 期望值 vs §10.7 规则）。

覆盖说明：**全验**（每条规范性条款均有独立 witness 实跑）；同一类（如每转义/每关键字/每数值进制）采用「全量清单核对 + 逐项实跑」；未做仅抽样。

---

## 6. 总体结论（一句话）

> **LFZ 解释器对 `syntax.md`（v1 冻结）词法 / 文法 / 解析 / 5 特色的规范性条款实现完整且行为正确（79 条中 76 PASS、0 条解释器缺陷、2 条 N-A 属规范延后/环境不可构造）；唯一 FAIL 为 `syntax.md` §9.2 样例期望输出与 §10.7 排序稳定性/键序规则自相矛盾（BUG-A-01，spec 侧，owner language-architect，严重度低）——结论档位：有条件通过。**

---

*本报告仅为验证结论，未改动任何交付物；夹具与临时脚本存于 `%TEMP%\opencode\conformance-a\`，未进入仓库。*
