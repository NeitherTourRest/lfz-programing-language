# 规范符合性审计 B —— 求值语义 / 错误模型

> 报告类型：独立验收（模拟助教）· 只验证不修复
> 审计人：verifier ｜ 日期：2026-09-27
> 被验提交：git HEAD `34e445f`（工作树相对被验二进制无 src 改动）
> 被验二进制：`target/release/lfz.exe`（704000 bytes，2026-09-27 13:01:12）**=`dist/lfz.exe`**；`cargo build --release` 返回 `Finished`（0.04s，未重编）→ 二进制与当前源码一致。
> 审计依据：`docs/spec/semantics.md`（唯一语义事实源，§3.6/§3.7/§4.5/§8）、`docs/spec/interface-contract.md`（§8.1/§10.3/§10.7）。
> 环境：Windows / PowerShell 5.1；LFZ 输出为 UTF-8（raw-byte 采集，规避 GBK 控制台转码）。
> 方法：逐条列条款 → 建 witness（`.lfz` 夹具）→ **实跑** → 比对 spec 原文 → PASS/FAIL + 证据。全部结论来自真实运行，未只读源码。
> 说明：本轮为新增审计域 B（语义/错误模型），与先前 P3/P9 报告无重叠缺陷编号，故本报告自带缺陷编号 `bug-B-20260927-NN`。

---

## ① 验收清单与结论总表

| ID | 条款（spec 出处） | Witness（夹具） | 结论 |
|---|---|---|---|
| B-01 | §4.5.0 值模型：标量按值 / 引用共享 / 条件必须 bool / 类型名 | `P_valuemodel` | **PASS** |
| B-02 | §4.5.1 求值序 B1：二元 / 调用（callee→args）/ 数组字面量 / 索引 a→i / 赋值 lvalue→rhs / 复合赋值单次求值 / `&&`·`\|\|` 短路 / 插值 / `if` 选中分支 / struct 字段序 | `P_evalorder`、`P_structorder2` | **PASS** |
| B-03 | §4.5.2 A1 引用语义：别名 / 传参按引用 / 赋值语法原地改 / 索引写入不扩容越界 `IndexError` / `let` 只锁重绑定 / 13 个内置"返回新值不改原容器" | `P_a1_ref`、`P_builtins2`、`N_index_write_oob`、`N_type_letrebind`、`N_let_compound` | **PASS** |
| B-04 | §4.5.3 A2 cell 捕获：共享 cell / 外部后改对闭包可见 / 全局不走 cell / 循环每轮独立 cell | `P_closures` | **PASS** |
| B-05 | §4.5.4 B2 `for` 快照 + B3 键序：array 长度快照 / 元素内部改动可见 / struct 键快照 / 键字节序升序 / 可迭代表限制 | `P_for`、`N_for_string` | **PASS** |
| B-06 | §4.5.5 B4 递归上限 10000 → `RecursionError`（消息逐字符） | `N_recursion`、`N_deep` | **PASS** |
| B-07 | §4.5.6 B5 float IEEE：NaN/Inf 产地、比较、显示、除零不产 Inf、排序全序 | `P_float`、`N_divzero_float/int/mod`、`N_divzero_div`、`N_modzero_float` | **PASS** |
| B-08 | §4.5.7 B13：int→float 加宽、精确比较、`int(x)` 极窄边界、数值内置形参加宽、`floor/ceil/round` 边界 | `P_intfloat`、`N_value_nan/inf/int_big`、`N_floor_nan`、`N_ceil_inf`、`N_round_big` | **PASS** |
| B-09 | §4.5.8 B7 struct 实例化平拷贝：默认值每实例重求值不共享 / 动态字段 / 覆盖未知字段新增 / 方法可用不属数据面 | `P_flatcopy`、`P_structorder2` | **PASS** |
| B-10 | §4.5.9 A5/A6：`==` 环安全、类型不同 false、struct 忽略函数字段、display `<cycle>`、`del` 数据面不变量 | `P_eqdisplay` | **PASS** |
| B-11 | §4.5.10 A4：`assert` 致命 / `check` 非致命（stderr 一行 + 返回 false + 不改退出码）/ 条件非 bool `TypeError` / `fail` | `P_check`、`N_assert`、`N_fail`、`N_assert_nonbool`、`N_check_nonbool` | **PASS** |
| B-12 | §4.5.11 字符串不可变（`+`/`upper`/`replace`/`trim` 均返回新串） | `P_string` | **PASS** |
| B-13 | §3.6 `;;` 可见链：内→外 / 同作用域槽升序 / 遮蔽去重 / 闭包可见链 / 含 `fn`·模板名·形参 / 行格式（U+0020 U+FF1A U+0020）/ 空链零行 / 通道 stdout·执行序交织 / 可重入 | `P_dump`、`P_dump2`、`P_dump3`、`P_interleave` | **PASS** |
| B-14 | §3.6 #7 「`;;` 通道 = stdout（**与 print 同通道**）」在 `--json` 模式下的一致性 | `P_interleave`（json） | **FAIL**（见缺陷 `bug-B-20260927-01`） |
| B-15 | §3.7 显示形式：各类型顶层 / 嵌套 string 加引号转义 / struct 键字节序且不含函数字段 / `<cycle>` / `<fn 名>`·`<fn>`·`<struct 名>` | `P_display`、`P_display2`、`P_eqdisplay` | **PASS** |
| B-16 | §3.7 float「最短往返；整值浮点显示 `.0`」 | `P_floatfmt` | **PASS**（含观察 `obs-B-02`） |
| B-17 | §8.1 **12 个错误类逐个触发** + 中文消息逐字符 | 19 个 `N_*` 夹具（见 §2） | **PASS** |
| B-18 | §8.1 `SyntaxError` 细分消息 | `N_syntax_char/semi/unclosed/overflow_lit/cjkid/cjkstr` | **PASS** |
| B-19 | §8.1 `TypeError` 细分消息 | `N_type_op/cond/call/pipe/argc/letrebind`、`N_assert_nonbool`、`N_check_nonbool`、`N_filter_nonbool`、`N_sort_mixed` | **PASS** |
| B-20 | §8.1 `ValueError` 子场景 + `IndexError` 的 `idx`/`len` 取值（含 `pop([])` → `-1`/`0`） | `N_value_convert/extremum/range/nan`、`N_index_array/pop/insert/write_oob` | **PASS** |
| B-21 | §8.2 位置 `line N, col M`：**列号按 Unicode 标量计数** | `N_syntax_char`（ASCII col 11）、`N_syntax_cjkstr`（CJK 串 col 14，非 18） | **PASS** |
| B-22 | §8.2 加载/解析 2 类**不带** `Traceback` 头 | `N_cosmos`、`N_overflow_lit` | **PASS** |
| B-23 | §8.2 运行期 10 类**带** `Traceback` 头 | 全部运行期 `N_*` | **PASS**（含观察 `obs-B-01`） |
| B-24 | §8.2/§10.3 traceback 帧序（最外层→最内层）+ 帧位置 = 调用点 / 引发节点 + `func` 名（`<module>`/函数名/匿名 `<fn>`/闭包保留定义名） | `N_traceback`、`N_tb_anon`、`N_tb_closure`、`N_deep` | **PASS** |
| B-25 | §8.2 traceback 折叠（T>40：首 10 → 省略行 → 尾 30） | `N_deep`（T=47 → 省略 7）、`N_recursion`（T=10001 → 省略 9961） | **PASS** |
| B-26 | §8.2 退出码 `0`/`1`/`2` | `tv_ok`/`tv_fail`、全部 `N_*`（含 `--json`） | **PASS** |
| B-27 | §8.3 示例 1–5 逐字符（含 `--json` 示例 4 两例） | `N_syntax_char`、`N_traceback`、`N_cosmos`、`N_cosmos`(json)、`P_check` | **PASS** |
| B-28 | §8.4 `--json` 字段 + `traceback` **不折叠** | `N_cosmos`(json)、`N_traceback`(json)、deep/rec json（帧数 47 / 10001） | **PASS** |

**统计：条款 28 条 → PASS 27 · FAIL 1 · N/A 0；另附 3 项低危观察（`obs-B-01/02/03`）。**

---

## ② 逐条详细证据

> 采集方式：`target\release\lfz.exe run <file>`（stdin ← NUL），stdout/stderr 分别重定向到文件，按 UTF-8 读取；`--json` 用 `lfz --json <file>`。夹具路径前缀省略为 `<W>\`。

### B-01 §4.5.0 值模型 —— PASS
`P_valuemodel`（exit 0）断言：`var x=1; let y=x; x=2; y==1`（标量按值）、`let a=[1]; let b=a; b[0]=2; a[0]==2`（引用共享）、`var s="a"; let t=s; s="b"; t=="a"`（字符串值语义）、`type()` 八种名称、`type(TT)=="struct"`。全部通过，无输出。**结论：PASS。**

### B-02 §4.5.1 求值序 B1 —— PASS
`P_evalorder`（exit 0）用 `tap(n)` 记录副作用序列，断言：
- 二元 `tap(1)+tap(2)` → log `[1,2]`；
- 调用 callee 先于实参 `pick()(tap(1),tap(2))` → `[0,1,2]`；
- 数组字面量 `[tap(1),tap(2),tap(3)]` → `[1,2,3]`；
- 索引 `arr[mkidx()]`（i 求值）→ log `[2]`；
- 赋值 `arr2[lhsI()] = rhsV()` → `[2,3]`（lvalue 下标先于 rhs）；
- 复合赋值 `arr2[sideIdx()] += 1` → `calls==1`（基/下标只求值一次）；
- `false && yes()` / `true || no()` → `sc==0`（短路）；`true && yes()` → `sc==1`；
- 插值 `"${tap(5)}-${tap(6)}"` → `[5,6]`；
- `if false … else …` 选中分支。

`P_structorder2`（exit 0）：`struct S{a:t(1), b:t(2)}` + `S{ b:t(100), a:t(200) }` → stdout `[1, 2, 100, 200]`，即**默认值按声明序（a=1,b=2）先求值，再按字面书写序（b=100,a=200）求值覆盖**，与 §4.5.1/§4.5.8 完全一致。**结论：PASS。**

### B-03 §4.5.2 A1 引用语义 —— PASS
`P_a1_ref`（exit 0）断言别名（`b=a; b[0]=99 ⇒ a[0]==99`；`t=s; t.k=42 ⇒ s.k==42`；`t["k"]=43 ⇒ s.k==43`）、传参按引用（`setFirst(arr)` 原地）、并**逐个**验证内置返回新值不改原：`push/pop/removeAt/insert/swap/slice/sort/sortBy/map/filter/take/drop/del` 全部 `新值==期望 && 原容器==原值`；`let locked=[0]; locked[0]=5` 合法。`P_builtins2` 追加 `min/max/minBy/maxBy/reduce/each` 不改原容器，且 `a[i]=v`/`s.k=v`/`s["k"]=v` 原地修改合法。
`N_index_write_oob`（exit 2）：`let a=[1,2,3]; a[5]=9` → `IndexError: 下标 5 越界（长度 3）`（**不自动扩容**）。
`N_type_letrebind`（exit 2）：`let a=1; a=2` → `TypeError: 不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容`，插入符指向 `a` 首字符。
`N_let_compound`（exit 2）：`let a=1; a+=1` → 同类 `TypeError`。**结论：PASS。**

### B-04 §4.5.3 A2 cell 捕获 —— PASS
`P_closures`（exit 0）：`mk()` 返回 `[inc,get,set]`，`get()` 初值 0 → `inc()` 后 1（共享 cell）→ 外部 `set(100)` 后 `get()==100`（闭包可见外部修改）→ `inc()` 后 101；`adders()` 每轮 `let j=i` 被闭包各自捕获 → `A[0](10)=10, A[1](10)=11, A[2](10)=12`；全局 `var g=1; fn readg()=>g; g=5; readg()==5`（全局不走 cell）。**结论：PASS。**

### B-05 §4.5.4 B2/B3 —— PASS
`P_for`（exit 0）：迭代中 `grow=push(e,grow)` 不改变次数（`seen==3`）；元素为容器时 `r[0]=9` 可见（`rows[0][0]==9`）；`{"b","a","c"}` 键序 `["a","b","c"]`；`{"a","Z"}` → `["Z","a"]`（ASCII 字节序）；struct 迭代中新增 `s2["z"]` 不改变本次次数（`cnt==2`）。
`N_for_string`（exit 2）：`for ch in "abc"` → `TypeError: 运算符 'for … in' 不支持 string 与 array / struct`（`for` 对 string 报 TypeError，符合 §4.5.4「仅 array/struct 可迭代」）。**结论：PASS。**

### B-06 §4.5.5 B4 递归上限 —— PASS
`N_recursion`（exit 2）：`fn rec(n){ rec(n+1) }; rec(0)` → stderr 末行 **`RecursionError: 递归深度超限（超过 10000 层）`**，带 `Traceback` 头，折叠为「首 10 帧 → `  ... 省略 9961 帧 ...` → 尾 30 帧」（T=10001）。JSON 复核 `traceback` 长度 = **10001**。**结论：PASS。**

### B-07 §4.5.6 B5 float IEEE —— PASS
`P_float`（exit 0）：`float("nan")/("inf")/("-inf")` 支持；`nan==nan` false、`nan!=nan` true、四种序比较全 false；`inf==inf` true、`inf>1.0`、`ninf<-1.0`；`str()` → `nan`/`inf`/`-inf`；`sqrt(-1.0)` 为 NaN；`pow(10.0,400.0)==inf`；全序 `sort([3.0,nan,1.0,inf,ninf])` → `[ninf,1.0,3.0,inf,nan]`，`min(xs)==ninf`，`max(xs)` 为 NaN（NaN 排最后）。
除零均**不产 Inf**：`N_divzero_int`（`1/0`）→ `除以零`；`N_divzero_float`（`1.0/0.0`）→ `除以零`；`N_divzero_mod`（`1%0`）→ `对零取模`；`N_modzero_float`（`1.0%0.0`）→ `对零取模`；`N_divzero_div`（`div(5,0)`）→ `除以零`。**结论：PASS。**

### B-08 §4.5.7 B13 —— PASS
`P_intfloat`（exit 0）：`type(1+2.0)=="float"`；`7/2==3.5`、`-7/2==-3.5`；Python 取模 `-7%3==2`、`7%-3==-2`；`div(-7,2)==-4`；**精确比较** `!(9007199254740993 == 9007199254740992.0)` 且 `9007199254740993 > 9007199254740992.0`；`int(2.9)==2`、`int(-2.9)==-2`（向零）；`int(true)==1`；`int("42")==42`；数值内置加宽 `sqrt(4)==2.0`、`floor/ceil/round(3)`、`pow(2,3)==8.0`；`abs` 同型（`type(abs(-3))=="int"`、`type(abs(-3.0))=="float"`）；banker's rounding `round(2.5)==2`、`round(3.5)==4`。
`int(x)` 边界：`N_value_nan`（`int(float("nan"))`）→ `ValueError: 无法把 float 转换为 int（'nan'）`；`N_value_inf`（`int(float("inf"))`）→ `OverflowError: 整数溢出：结果超出 i64 范围`；`N_value_int_big`（`int(1e30)`）→ `OverflowError`。
`floor/ceil/round` 边界：`N_floor_nan` → `ValueError: 无法把 float 转换为 int（'nan'）`；`N_ceil_inf` → `OverflowError`；`N_round_big`（`round(1e30)`）→ `OverflowError`。**结论：PASS。**

### B-09 §4.5.8 B7 平拷贝 —— PASS
`P_flatcopy`（exit 0）：`struct Bag{items:[]}`，`b1.items=push(1,b1.items)` 后 `b1` 长度 1、`b2` 长度 0（可变默认值不共享）；覆盖未知字段 `Bag{extra:5}` 新增；动态字段 `b4.z=9`；`has("z", Bag{})==false`；方法 `p.get()==7` 且 `!has("get",p)`、`keys(p)==["x"]`；匿名 `{ "k":1 }`、空 `{}`。`P_structorder2` 证明默认值**每实例重新求值**。**结论：PASS。**

### B-10 §4.5.9 A5/A6 —— PASS
`P_eqdisplay`（exit 0）：`[1]=={"a":1}` false、`1=="1"` false、`true==1` false；含动态函数字段的 struct `s1==s2`（**忽略函数值字段**）且 `!has("f",s1)`、`keys(s1)==["x"]`；自引用 `c1.me=c1; c2.me=c2` → `c1==c1`、`c1==c2`（重访视为相等）、`c1==c3`(n 不同) false；`str(d)=="{me: <cycle>, n: 1}"`（**键字节序 + `<cycle>`**）；自引用 array `xs==ys`；`del` 不变量：`has("a",dd)` true → `!has("a",del("a",dd))`。**结论：PASS。**

### B-11 §4.5.10 A4 —— PASS
`P_check`（exit 0）：stdout `继续运行`，stderr 两行 `check 失败：软断言示例` / `check 失败`，`check(1==2,...)` 返回 `false`，退出码 **0**；`--json` 下 P_check 输出 `{"ok":true}`（**check 不产生 error**）。
`N_assert`（exit 2）→ `AssertionError: 断言失败：boom`（致命，带 Traceback）；`N_fail`（exit 2）→ `AssertionError: 自定义失败`；`N_assert_nonbool`（exit 2）→ `TypeError: 条件必须是 bool，得到 int`；`N_check_nonbool`（exit 2）→ 同类 `TypeError`。**结论：PASS。**

### B-12 §4.5.11 字符串不可变 —— PASS
`P_string`（exit 0）：`upper(s)`/`replace(...)`/`s+"d"`/`trim(...)` 均返回新串且 `s` 不变。**结论：PASS。**

### B-13 §3.6 `;;` 可见链 —— PASS
`P_dump`（exit 0）stdout（**原始字节** `61 6C 70 68 61 20 EF BC 9A 20 31 0A` …）：
```
alpha ： 1
beta ： 2
gamma ： <fn gamma>
Delta ： <struct Delta>
```
证明：含 `let`/`var`/`fn` 名/`struct` 模板名；同作用域**槽升序**；分隔串 = `U+0020 U+FF1A U+0020`（`20 EFBC9A 20`，全角冒号）。
`P_dump2`（exit 0）证明**内→外** + **遮蔽去重** + 闭包/块作用域可见链：
```
inner ： 2        // 函数局部
outer ： 1
f ： <fn f>
blockVar ： 3     // 块作用域
outer ： 1
f ： <fn f>
rr ： 2
name ： inner     // 遮蔽：只打印最内层 name
outer ： 1
f ： <fn f>
rr ： 2
g ： <fn g>
```
`P_dump3`（exit 0）：函数内 `;;` 打印形参 `p ： 9` 与局部 `local ： 9`（槽升序），string 值 `s ： hi`（顶层原样），array `arr ： [1, 2]`。
`P_interleave`（exit 0，**常规模式**）stdout：
```
A
B
z ： 5
C
```
证明 `;;` 在 **stdout**、与 `print` **按执行序交织**、空可见链输出**零行**。**结论：PASS。**（→ `--json` 交互见 B-14）

### B-14 §3.6 #7「`;;` 与 print 同通道」在 `--json` 下 —— **FAIL**
见 §3 缺陷 `bug-B-20260927-01`。证据：`P_interleave` 以 `--json` 运行，**stdout 混入非 JSON**、`;;` 与 `print` 分属不同通道。**结论：FAIL。**

### B-15 §3.7 显示形式 —— PASS
`P_display`（exit 0）stdout：
```
nil
true
false
42
1.0
1.5
top string
["a", "b"]
{k: "v"}
[1, "s", true, nil]
{a: 3}
<struct T>
<fn named>
<fn>
q"x
```
顶层 string 原样（`top string`）；嵌套 string 加引号（`["a", "b"]`、`{k: "v"}`）；struct 只列非函数字段（`{a: 3}`）；模板 `<struct T>`；命名函数 `<fn named>`；匿名 `<fn>`；`\"` 转义。
`P_display2`（exit 0）：`{ "b":1, "a":2 }` → `{a: 2, b: 1}`（**键字节序**）；`["x\"y", "a\nb"]` → `["x\"y", "a\nb"]`（嵌套转义）；`0.1+0.2` → `0.30000000000000004`；`[1,[2,3]]`；`{ "z": {"y":1} }` → `{z: {y: 1}}`；`-7` → `-7`。**结论：PASS。**

### B-16 §3.7 float 最短往返 / 整值 `.0` —— PASS（含观察）
`P_floatfmt`（exit 0）stdout：
```
100.0
1000000000000000.0
1e16
1e20
1e21
0.0001
1e-5
1.2345678901234568e17
-0.0
2.5
```
`1.0→1.0`、`100.0→100.0`、`-0.0→-0.0`、最短往返（`1.2345678901234568e17`）均正确。**≥1e16 的整值浮点以指数形式输出、未带 `.0`**（`1e16`/`1e20`），与「整值浮点显示 `.0`」的字面读法存在张力 → 记为 `obs-B-02`（规范歧义）。**结论：PASS（附观察）。**

### B-17 §8.1 12 类错误逐个触发 + 消息 —— PASS
| 类 | 夹具 | 退出码 | 末行消息（实测） |
|---|---|---|---|
| `CosmosAnswerError` | `N_cosmos` | 2 | `你忘记了宇宙的答案`（无 Traceback/源码行/插入符） |
| `SyntaxError` | `N_syntax_char/semi/unclosed/overflow_lit/cjkid/cjkstr` | 2 | 见 B-18 |
| `NameError` | `N_name` | 2 | `未定义的名字 'undefined_var'` |
| `TypeError` | `N_type_op/cond/call/pipe/argc/letrebind/assert_nonbool/check_nonbool/filter_nonbool/sort_mixed` | 2 | 见 B-19 |
| `IndexError` | `N_index_array/pop/insert/write_oob` | 2 | 见 B-20 |
| `FieldError` | `N_field_dot`、`N_field_del_method` | 2 | `结构体没有字段 'b'` / `结构体没有字段 'm'` |
| `ZeroDivisionError` | `N_divzero_int/float/mod/div`、`N_modzero_float` | 2 | `除以零` / `对零取模` |
| `OverflowError` | `N_overflow_arith`、`N_value_inf/int_big`、`N_ceil_inf`、`N_round_big` | 2 | `整数溢出：结果超出 i64 范围` |
| `ValueError` | `N_value_convert/extremum/range/nan`、`N_floor_nan` | 2 | 见 B-20 |
| `IOError` | `N_io_eof`、文件不存在 | 2 | `输入结束（EOF）` / `无法读取：<path>` |
| `AssertionError` | `N_assert`、`N_fail` | 2 | `断言失败：boom` / `自定义失败` |
| `RecursionError` | `N_recursion` | 2 | `递归深度超限（超过 10000 层）` |

**12 类全部成功触发**，消息与 spec 模板逐字符一致（引号/全角括号/全角冒号均核对）。**结论：PASS。**

### B-18 §8.1 `SyntaxError` 细分消息 —— PASS
- 非法字符：`N_syntax_char` → `非法字符 '$'`；`N_syntax_cjkid`（ASCII-only 标识符 §2.6）→ `非法字符 '变'`；
- 单 `;`：`N_syntax_semi` → `单独的 ';' 非法；打印变量请用 ';;'`；
- 字符串未闭合：`N_syntax_unclosed` → `字符串字面量在此处未闭合`；
- 整数字面量越界：`N_overflow_lit` → `整数字面量超出 i64 范围`（**无 Traceback 头**，解析期）。

### B-19 §8.1 `TypeError` 细分消息 —— PASS
`运算符 '+' 不支持 int 与 string`（`N_type_op`）；`条件必须是 bool，得到 int`（`N_type_cond`、`N_assert_nonbool`、`N_check_nonbool`、`N_filter_nonbool`）；`不可调用：int 不是函数`（`N_type_call`）；`管道右侧必须是函数，得到 int`（`N_type_pipe`）；`函数 f 期待 2 个参数，得到 1`（`N_type_argc`）；`不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容`（`N_type_letrebind`、`N_let_compound`）。另 `sort([1,"a"])` → `运算符 'sort' 不支持 string 与 number`（§10.7 元素类型不一致，属 TypeError）。

### B-20 §8.1 `ValueError` / `IndexError` 子场景 —— PASS
`无法把 string 转换为 int（'abc'）`（`N_value_convert`）；`无法把 float 转换为 int（'nan'）`（`N_value_nan`、`N_floor_nan`）；`空数组没有极值（min）`（`N_value_extremum`）；`区间非法：5 >= 5`（`N_value_range`）。
`IndexError` 的 `idx`/`len`：`pop([])` → `下标 -1 越界（长度 0）`（钉死 `idx=-1,len=0`）；`insert(-1,9,[1,2])` → `下标 -1 越界（长度 2）`；`a[5]` 读 → `下标 5 越界（长度 3）`。

### B-21 §8.2 位置 / Unicode 标量列号 —— PASS
`N_syntax_char`（`let x = 1 $ 2`）：`$` 在第 2 行第 **11** 列 → 插入符行 = 4 空格 + 10 空格 + `^`；JSON `col:11`。
`N_syntax_cjkstr`（`let s = "中文" $ 2`）：`$` 的**标量**列 = **14**（JSON `col:14`）。若按字节计数应为 18（`中`/`文` 各 3 字节）→ 证明**按 Unicode 标量计数**。**结论：PASS。**

### B-22 §8.2 加载/解析无 Traceback —— PASS
`N_cosmos`：仅
```
File "<path>", line 1
CosmosAnswerError: 你忘记了宇宙的答案
```
（无 `Traceback` 头、无源码行、无插入符）。`N_overflow_lit` 同属解析期、无 `Traceback` 头。**结论：PASS。**

### B-23 §8.2 运行期带 Traceback —— PASS（附观察）
10 个运行期类均出现 `Traceback (most recent call last):` 头（逐一核对）。**观察**：文件不存在时（`N_io_eof` 之外的 `lfz run nope.lfz`）输出仅 `IOError: 无法读取：<path>`（**无 Traceback 头**，`--json` 的 `traceback:[]`）——因错误发生在执行前、帧栈为空。记为 `obs-B-01`。**结论：PASS（附观察）。**

### B-24 §8.2/§10.3 traceback 帧序与 func 名 —— PASS
`N_traceback`（对齐 §8.3 示例 2）：外层帧 `line 5, in <module>` 指向 `half(10)`（col 9），内层帧 `line 3, in half` 指向 `n`（col 5）。
`N_tb_anon`（`let g=(x)=>x/0; g(1)`）：帧名 **`<fn>`**。
`N_tb_closure`（`make()` 内定义 `inner`）：帧名 **`inner`**（闭包保留定义名）。
帧序 = 最外层→最内层。**结论：PASS。**

### B-25 §8.2 traceback 折叠 —— PASS
`N_deep`（T=47）：首 10 帧 → 省略行 **`  ... 省略 7 帧 ...`**（前缀 2 空格）→ 尾 30 帧 → 末行。
`N_recursion`（T=10001）：省略行 `  ... 省略 9961 帧 ...`。JSON `traceback` 长度分别为 47 / 10001（**不折叠**，见 B-28）。**结论：PASS。**

### B-26 §8.2 退出码 —— PASS
`lfz test tv_ok` → exit **0**（`汇总：共 1 个用例，通过 1，失败 0，错误 0`）；`lfz test tv_fail` → exit **1**（`失败 1`，含 `AssertionError: 断言失败：intentional failure`）；所有错误（含 `RecursionError`、`--json`）→ exit **2**；正常成功 → exit **0**。**结论：PASS。**

### B-27 §8.3 示例 1–5 —— PASS
- 示例 1（`SyntaxError` 逐字符）与实测一致（`$` col 11）；
- 示例 2（`ZeroDivisionError` traceback 帧/列）一致；
- 示例 3（`CosmosAnswerError` 无源码行/插入符）一致；
- 示例 4（`--json` 两例）逐字段一致（见 B-28）；
- 示例 5（`check` 非致命）一致：stdout `继续运行`、stderr `check 失败：软断言示例`、退出码 `0`。

### B-28 §8.4 `--json` 字段 + 不折叠 —— PASS
`N_cosmos`（json）：`{"ok":false,"error":"CosmosAnswerError","message":"你忘记了宇宙的答案","file":"…","line":1,"col":1,"traceback":[{"file":"…","line":1,"func":"<module>"}]}` — 与 §8.3 示例 4 第 1 例逐字段一致（仅路径不同）。
`N_traceback`（json）：`…"line":3,"col":5,"traceback":[{"…","line":5,"func":"<module>"},{"…","line":3,"func":"half"}]` — 与示例 4 第 2 例一致。
成功：`{"ok":true}`（`P_display2`、`P_interleave` 等）。`N_deep`/`N_recursion` 的 `traceback` 帧数 47 / 10001（**完整不折叠**）。`error` 取值均为 12 类名之一；`check` 失败不产生 `error`。**结论：PASS。**

---

## ③ 缺陷单汇总

### 【缺陷单】bug-B-20260927-01（`;;` × `--json` 污染 stdout）
- **交付物**：CLI 的 `--json` 输出契约 / `;;`（`Dump`）输出通道。
- **最小复现**：
  1. 建 `d.lfz`（temp）：
     ```
     #42
     let z = 5
     ;;
     ```
  2. 运行 `lfz run --json d.lfz`（或 `lfz --json d.lfz`）。
- **期望**（引 spec 原文）：
  - `docs/spec/semantics.md` L27「**通道**：**stdout**（与 `print` 同通道，按程序执行顺序交织）。」
  - `docs/guide/errors.md` L247「开启后 **stdout 只写一行 JSON**，人类可读诊断（含程序 `print` 输出）改到 stderr。」
  → 期望：`;;` 与 `print` 同通道（JSON 模式下应随 print 一起转 stderr），stdout **仅**一行 JSON。
- **实际**（贴输出，`P_interleave` 以 `--json` 运行）：
  - **stdout**：
    ```
    z ： 5
    {"ok":true}
    ```
  - **stderr**：
    ```
    A
    B
    C
    ```
  → `print` 输出（A/B/C）被转入 stderr，但 `;;` 输出（`z ： 5`）仍写入 **stdout**，致 **stdout 两行、非纯 JSON**；`;;` 与 `print` 通道不一致。
- **严重度**：**中**。仅当程序使用 `;;` 且启用 `--json` 时触发；但破坏「stdout 只写一行 JSON」的机器可读契约（评分项 4 特色 4 的一部分），且违反 §3.6 #7 的"同通道"断言。
- **建议 owner**：**tooling-dev**（CLI `render`/输出重定向落点，见 interface-contract §10.6）；如 `;;` 由 runtime 直接写 stdout，则需 tooling-dev 与 runtime-dev 协同。

### 观察项（低危 / 规范歧义，非阻塞）
- **obs-B-01（低）**：文件不存在时 `IOError: 无法读取：<path>` **不带 `Traceback` 头**（`--json` 的 `traceback:[]`）。§8.2 将 `IOError` 列入"运行期 10 类（带 Traceback 头）"，但此错发生在执行前、帧栈为空——属 §8.2 未覆盖的零帧情形。建议 language-architect 明确"零帧运行期错误"的渲染。owner：language-architect / tooling-dev。
- **obs-B-02（低，规范歧义）**：`P_floatfmt` 显示 `1e16`/`1e20`/`1e21` 等**大整值浮点以指数形式输出、无 `.0`**，与 `docs/spec/semantics.md` L42「整值浮点显示 `.0`（`1.0`）」的字面读法存在张力；"最短往返"与"整值 `.0`"两条规则在 `|x|≥1e16` 时冲突。需裁定。owner：language-architect（规则）/ runtime-dev（实现）。
- **obs-B-03（低）**：`a[5] = 9`（索引写入越界）插入符指向 **col 2**（`[`），而 `print(a[5])`（索引读取越界）插入符指向 **col 7**（基 `a`）。§8.2 L299 要求插入符指向"引发错误的最小 AST 节点的首字符"；同一索引节点读写两种形态位置不一致，疑为下标写入 span 偏差 1。owner：runtime-dev。

> 无「高」严重度缺陷；核心语义（A1/A2/A4/A5/A6、B1–B7/B13）与 12 类错误模型**未发现阻断性偏差**。

---

## ④ 回归记录

| 缺陷 | 首次发现 | 状态 | 复验证据 |
|---|---|---|---|
| （本轮为新增审计域 B，无历史缺陷复验项） | — | — | — |

> 说明：本报告不复验 P3/P9 的既有缺陷（`bug-20260927-01`「`s["k"]()` 未绑定 self」等已由 P9 rev.2 判 PASS，见 `docs/reports/P9-verification.md`）；本轮独立构造的夹具未触发该旧缺陷回归。若 team-lead 要求，可另开回归轮。

---

## ⑤ 总体结论

**有条件交付（1 项中危缺陷 + 3 项低危观察/规范歧义，见缺陷单）。**
- 求值语义 §4.5 全节（§4.5.0–§4.5.11）与 §3.6/§3.7、§8 错误模型与 `--json` 契约的**主体完全符合规范**：28 条中 27 条 PASS，证据均为实跑命令与原始输出。
- 唯一 **FAIL**：`;;` 在 `--json` 下未与 `print` 同通道，使 stdout 非纯 JSON（`bug-B-20260927-01`，中）。该缺陷不阻断普通 `lfz run`/`lfz test`，但影响机器可读输出契约，建议在最终发布前修复并复验。
- 另有 3 项低危观察（零帧 `IOError` 无 Traceback、大整值浮点指数形式无 `.0`、索引写入插入符列号），均需规范/实现裁定，不阻断交付。

---

## 附录 A：审计方法与可复现说明

- **被验二进制一致性**：`cargo build --release` 返回 `Finished … in 0.04s`（未重编）→ `target\release\lfz.exe`（704000 B）= `dist\lfz.exe`，与工作树源码一致。审计期间未修改任何 `src/**`、`docs/spec/**`、`tests/**`。
- **夹具位置**：`%TEMP%\opencode\confB\{w,w2,w3}\*.lfz`（一次性验证脚本/夹具，不入交付物）；运行器 `runner.ps1` 将每个夹具的 `exit / stdout / stderr` 分别重定向到文件后**按 UTF-8 解码**汇总（规避 Windows 控制台 GBK 转码）。
- **`;;` 行格式字节级证据**：`P_dump` stdout 前 11 字节 = `61 6C 70 68 61 20 EF BC 9A 20 31`（`alpha` + `20` + `EFBC9A`(U+FF1A) + `20` + `1`），与 §3.6 #5 逐字节一致。
- **`--json` 帧数证据**：`ConvertFrom-Json` 解析 `N_deep`/`N_recursion` 输出，`traceback.Count` = 47 / 10001。
- **退出码证据**：`lfz test <dir>`（ok→0，fail→1）；`run`/`--json`（成功→0，错误→2）。

## 附录 B：夹具清单（witness → 条款）
- 正向（exit 0）：`P_valuemodel`(B-01)、`P_evalorder`(B-02)、`P_structorder2`(B-02/B-08/B-09)、`P_a1_ref`(B-03)、`P_builtins2`(B-03)、`P_closures`(B-04)、`P_for`(B-05)、`P_float`(B-07)、`P_intfloat`(B-08)、`P_flatcopy`(B-09)、`P_eqdisplay`(B-10)、`P_check`(B-11)、`P_string`(B-12)、`P_dump/P_dump2/P_dump3/P_interleave`(B-13)、`P_display/P_display2`(B-15)、`P_floatfmt`(B-16)。
- 负向（exit 2）：`N_cosmos`、`N_syntax_char/unicode/cjkid/cjkstr/unclosed/semi`、`N_name`、`N_type_op/cond/call/pipe/argc/letrebind`、`N_assert/assert_nonbool`、`N_check_nonbool`、`N_fail`、`N_index_array/pop/insert/write_oob`、`N_field_dot/del_method`、`N_divzero_int/float/mod/div`、`N_modzero_float`、`N_overflow_arith/lit`、`N_value_convert/nan/inf/int_big/extremum/range`、`N_floor_nan/ceil_inf/round_big`、`N_io_eof`、`N_recursion`、`N_traceback/tb_anon/tb_closure/deep`、`N_for_string`、`N_sort_mixed`、`N_filter_nonbool`、文件不存在。
- 作废夹具（**自身构造错误，非实现缺陷，已撤回**）：`P_evalorder`(v1，对 `呼叫()[下标]=` 赋值——spec 只允许变量/字段/下标为 lvalue，`SyntaxError` 属正确行为)、`P_valuemodel`(v1，对 `let` 重绑定——`TypeError` 属正确行为)、`P_structorder`(v1，覆盖值为字面量未加 `t()`，期望写错)。三者在 `w2/w3` 已修正。

*（本报告为独立验证产物，只用于记录验证事实与证据；不承诺修复时限，不修改任何交付物。）*
