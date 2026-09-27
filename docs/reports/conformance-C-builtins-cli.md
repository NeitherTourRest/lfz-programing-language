# 规范符合性审计 C —— 内置函数表 / loader / test-runner 契约 / CLI

> 审计者: **verifier**（独立验收，只验证不修复） ｜ 日期: 2026-09-27
> 被验对象: 工作树 **git HEAD** 上的 `dist\lfz.exe`（构建产物，`cargo build` 零警告）
> 依据: `docs/spec/interface-contract.md`（§8.1 / §10.1 / §10.2 / §10.5 / §10.7 / §10.8 / §11.2）+ `docs/spec/syntax.md` §2.2
> 方法: **逐条列条款 → 找 witness → 实跑 → PASS/FAIL/N-A + 原文证据**。全部结论来自实际执行（`dist\lfz.exe` 真实进程 + `%TEMP%\opencode\confC\` 临时夹具），**未以只读源码作为结论依据**。
> 临时夹具: `%TEMP%\opencode\confC\`（一次性，审计后清理；未进入任何交付物目录）。

---

## ① 验收清单与结论总表

| 条款 | 范围 | 结论 | 证据引用 |
|---|---|---|---|
| C-1 | §10.7 内置 **54 个存在性** | ✅ PASS | §②C-1；54/54 可调用 |
| C-2 | §10.7 **data-last 签名/参数顺序** | ✅ PASS | §②C-2 |
| C-3 | §10.7 **返回类型** | ✅ PASS | §②C-3 |
| C-4 | `pop([])` → `Index{idx:-1,len:0}` | ✅ PASS | §②C-4 |
| C-5 | `insert` 不支持负索引 | ✅ PASS | §②C-5 |
| C-6 | `min/max/minBy/maxBy` 空 → ValueError 文案 | ✅ PASS | §②C-6 |
| C-7 | `randInt(lo>=hi)` → ValueError 文案 | ✅ PASS | §②C-7 |
| C-8 | `int(NaN/±Inf/超界)` | ✅ PASS | §②C-8 |
| C-9 | `floor/ceil/round` NaN/±Inf | ✅ PASS | §②C-9 |
| C-10 | `round` 银行家舍入 | ✅ PASS | §②C-10 |
| C-11 | `abs` 同型不加宽 | ✅ PASS | §②C-11 |
| C-12 | `floor/ceil/round/sqrt/pow` int→float 加宽 | ✅ PASS | §②C-12 |
| C-13 | `div` 向下取整 / 除零 / 非 int | ✅ PASS | §②C-13 |
| C-14 | `%` 符号随除数 | ✅ PASS | §②C-14 |
| C-15 | `del` 仅数据字段；`del ⟺ has` | ✅ PASS | §②C-15 |
| C-16 | `keys` 字节序升序 | ✅ PASS | §②C-16 |
| C-17 | `has` 方法字段 → false | ✅ PASS | §②C-17 |
| C-18 | `split("",s)` 按字符 | ✅ PASS | §②C-18 |
| C-19 | `upper/lower` 仅 ASCII | ✅ PASS | §②C-19 |
| C-20 | `str/int/float/type` 边界 | ✅ PASS | §②C-20 |
| C-21 | `assert/check/fail` | ✅ PASS | §②C-21 |
| C-22 | `sum` 溢出 / `sort` 混合 / `filter` 非 bool | ✅ PASS | §②C-22 |
| C-23 | `seed` 确定性 / `rand` 区间 / `randInt` 区间 | ✅ PASS | §②C-23 |
| C-24 | `print/eprint/input`（EOF→IOError、prompt） | ✅ PASS | §②C-24 |
| **C-25** | **`repeat` 溢出 → 应 OverflowError** | ❌ **FAIL** | §③ bug-20260927-03（Rust panic） |
| **C-26** | **`range` 超大 n → 应受控** | ⚠️ **FAIL（规范静默）** | §③ bug-20260927-04（Rust panic） |
| C-27 | §10.1 `ext(path)` 四步 | ✅ PASS | §②C-27 |
| C-28 | §10.1 UTF-8 校验失败 → SyntaxError（含字节偏移） | ✅ PASS | §②C-28 |
| C-29 | §10.1 BOM 跳过（单次） | ✅ PASS | §②C-29 |
| C-30 | §10.1 `\r\n`/`\r`/`\n` 归一化 | ✅ PASS | §②C-30 |
| C-31 | §10.1 `#42` 前导 + `line_base` | ✅ PASS | §②C-31 |
| C-32 | §10.1 / syntax §2.2.0 非 `.lfz` 豁免（`dir.lfz/x`、`.bak`、无扩展名） | ✅ PASS | §②C-32 |
| C-33 | §10.2 `Span` line/col（Unicode 标量计数） | ✅ PASS | §②C-33 |
| C-34 | §10.5 `ScopeDebug` / `;;`（内→外、slot 序、遮蔽去重、块、闭包） | ✅ PASS | §②C-34 |
| C-35 | §10.8 错误枚举/类名 / `i64::MIN` 字面量 | ✅ PASS | §②C-35 |
| C-36 | §10.8 `RecursionError` | ✅ PASS | §②C-36 |
| C-37 | §11.2 **T-R1** 缺 `#42` → ERROR/exit 2 | ✅ PASS | §②C-37 |
| C-38 | §11.2 **T-R2** `fixtures/` 排除 + `cases.json` expect | ✅ PASS | §②C-38 |
| C-39 | §11.2 **T-R3** assert→FAIL/exit1；其它类→ERROR/exit2 | ✅ PASS | §②C-39 |
| C-40 | §11.2 **T-R4** 发现规则 | ✅ PASS | §②C-40 |
| C-41 | 退出码：ERROR 支配 FAIL | ✅ PASS | §②C-41 |
| C-42 | CLI `run` / 裸调用 / `test` 分派 | ✅ PASS | §②C-42 |
| C-43 | CLI `--json`（stdout 单 JSON；程序输出转 stderr） | ✅ PASS（print/eprint 路径） | §②C-43 |
| **C-43b** | **CLI `--json` × `;;`（dump 输出通道）** | ❌ **FAIL** | §②C-43b / §③ bug-B-20260927-01 |
| C-44 | CLI `--help/-h`、`--version/-V` | ✅ PASS | §②C-44 |
| C-45 | CLI 退出码映射 0/1/2 | ✅ PASS | §②C-45 |
| C-46 | CLI 错误位置格式（`File "…", line N` + 源行 + 插入符） | ✅ PASS | §②C-46 |
| C-47 | CLI 裸调用非 `.lfz` 拒绝 / 缺文件 IOError | ✅ PASS | §②C-47 |
| C-48 | syntax §2.2.1 前导违规全变体 → CosmosAnswerError | ✅ PASS | §②C-48 |
| C-49 | 观察项（`run --help`、`--json --version`） | ⚪ 观察 | §③ OBS-01/02 |

**统计**：条款 **50**；**PASS 46**；**FAIL 3**（C-25、C-26、C-43b）；**观察 1 组（2 条）**。
内置逐项勾选：**54/54 存在**，**53 项全绿**，**1 项（`repeat`）边界 FAIL**；另 `range` 超大入参 FAIL（规范静默）。

---

## ② 逐条详细证据

### C-1　§10.7 内置表 54 个逐个 —— 存在性 ✅

命令：`dist\lfz.exe run %TEMP%\opencode\confC\bisig.lfz`（夹具内对每个内置至少调用一次；缺失即 NameError 中断）
结果：`EXIT=0`，stderr 仅有一条**刻意制造**的 `check 失败`（来自 `check(check(false)==false,…)` 的内层 `check(false)`），**无真实失败**。

**54 内置逐个勾选表**（“witness”= 实跑时调用的位置）：

| # | 内置 | 存在 | 边界/行为 witness | 结论 |
|---|---|---|---|---|
| 1 | len | ✅ | `len([1,2,3])==3`、`len("中文")==2`、`len({"a":1,"b":2})==2` | PASS |
| 2 | range | ✅ | `range(5)`、`range(0)`、`range(-3)==[]` | PASS（超大入参见 C-26） |
| 3 | push | ✅ | `push(4,[1,2,3])==[1,2,3,4]` | PASS |
| 4 | pop | ✅ | `pop([1,2,3])==[1,2]`；`pop([])`→C-4 | PASS |
| 5 | removeAt | ✅ | `removeAt(1,..)`、`removeAt(-1,..)` | PASS |
| 6 | insert | ✅ | `insert(1,9,[1,2])`、`insert(2,9,[1,2])`；负索引→C-5 | PASS |
| 7 | swap | ✅ | `swap(0,2,..)`、`swap(-1,0,..)` | PASS |
| 8 | slice | ✅ | `slice(0,2,..)`、`slice(1,1,..)==[]`、夹取 | PASS |
| 9 | min | ✅ | `min([3,1,2])==1`；空→C-6 | PASS |
| 10 | max | ✅ | `max([3,1,2])==3`；空→C-6 | PASS |
| 11 | sum | ✅ | `sum([1,2,3])==6`、`sum([])==0`（int）、混合 float；溢出→C-22 | PASS |
| 12 | minBy | ✅ | `minBy((x)=>x,[3,1,2])==1`；空→C-6 | PASS |
| 13 | maxBy | ✅ | `maxBy((x)=>x,[3,1,2])==3`；空→C-6 | PASS |
| 14 | sort | ✅ | `sort([3,1,2])==[1,2,3]`；混合类型→C-22 | PASS |
| 15 | sortBy | ✅ | `sortBy((x)=>-x,[1,2,3])==[3,2,1]` | PASS |
| 16 | map | ✅ | `map((x)=>x*2,[1,2,3])==[2,4,6]` | PASS |
| 17 | filter | ✅ | `filter((x)=>x>1,[1,2,3])==[2,3]`；非 bool→C-22 | PASS |
| 18 | reduce | ✅ | `reduce((a,x)=>a+x,0,[1,2,3])==6`、左折叠 `==123` | PASS |
| 19 | take | ✅ | `take(2,..)`、`take(-1,..)==[]`、夹取高 | PASS |
| 20 | drop | ✅ | `drop(1,..)==[2,3]`、`drop(10,..)==[]` | PASS |
| 21 | each | ✅ | `each((x)=>nil,[1,2,3])==nil` | PASS |
| 22 | keys | ✅ | 字节序→C-16 | PASS |
| 23 | values | ✅ | `values({"b":1,"a":2})==[2,1]` | PASS |
| 24 | entries | ✅ | `entries({"b":1,"a":2})==[["a",2],["b",1]]` | PASS |
| 25 | has | ✅ | `has(...)`；方法字段→C-17 | PASS |
| 26 | del | ✅ | `del("a",{"a":1,"b":2})=={"b":2}`；方法/缺失→C-15 | PASS |
| 27 | split | ✅ | `split(",",..)`；空 sep→C-18 | PASS |
| 28 | join | ✅ | `join("-",["a","b","c"])=="a-b-c"`、`join(",",[])==""` | PASS |
| 29 | trim | ✅ | `trim("  hi  ")=="hi"` | PASS |
| 30 | upper | ✅ | ASCII-only→C-19 | PASS |
| 31 | lower | ✅ | ASCII-only→C-19 | PASS |
| 32 | replace | ✅ | `replace("a","b","banana")=="bbnbnb"` | PASS |
| 33 | repeat | ✅ | `repeat(3,"ab")`、`repeat(0,"ab")==""`；**溢出→FAIL** | **FAIL（C-25）** |
| 34 | startsWith | ✅ | `true`/`false`/空前缀 | PASS |
| 35 | abs | ✅ | 同型→C-11；`abs(i64::MIN)`→C-11 | PASS |
| 36 | floor | ✅ | 加宽→C-12；NaN/Inf→C-9 | PASS |
| 37 | ceil | ✅ | 加宽→C-12；Inf→C-9 | PASS |
| 38 | round | ✅ | 银行家→C-10；NaN/Inf→C-9 | PASS |
| 39 | sqrt | ✅ | `sqrt(4)==2.0`；`sqrt(-1)`=NaN | PASS |
| 40 | pow | ✅ | `pow(2,10)==1024.0`；溢出→±Inf | PASS |
| 41 | div | ✅ | 向下取整→C-13 | PASS |
| 42 | rand | ✅ | `[0,1)`、float→C-23 | PASS |
| 43 | randInt | ✅ | `[lo,hi)`、int；非法区间→C-7 | PASS |
| 44 | seed | ✅ | 确定性→C-23；返回 nil | PASS |
| 45 | str | ✅ | 显示形式→C-20 | PASS |
| 46 | int | ✅ | 边界→C-8/C-20 | PASS |
| 47 | float | ✅ | `float("inf")/("nan")/bad`→C-20 | PASS |
| 48 | type | ✅ | 8 类型名→C-20 | PASS |
| 49 | print | ✅ | 空格连接 + `\n`→C-24 | PASS |
| 50 | eprint | ✅ | stderr 通道→C-24 | PASS |
| 51 | input | ✅ | 读行/prompt/EOF→C-24 | PASS |
| 52 | assert | ✅ | 致命→C-21 | PASS |
| 53 | check | ✅ | 非致命→C-21 | PASS |
| 54 | fail | ✅ | `AssertionError`→C-21 | PASS |

### C-2　data-last 签名 / 参数顺序 ✅
- 正向（数据末参）：`push(4, a0)` 使 `a0==[1,2,3]` 不变、结果 `[1,2,3,4]`；`removeAt/insert/swap/slice/take/drop/has/del/split/join/replace/repeat/startsWith/minBy/maxBy/sortBy/map/filter/reduce/each` 均按 spec 末参秩序实跑通过（`bisig.lfz`、`bisig2.lfz` 全绿）。
- 反向（错误顺序必须报错）：`e_push_badorder.lfz` = `push([1, 2], 3)`
  ```
  TypeError: 运算符 'push' 不支持 int 与 array
  ```
  → 证明第 2 参必须是 array（data-last），PASS。
- 参数数量：`e_push_argcount.lfz`=`push(1)` → `TypeError: 函数 push 期待 2 个参数，得到 1`；`e_type_argcount.lfz`=`type(1, 2)` → `TypeError: 函数 type 期待 1 个参数，得到 2`。

### C-3　返回类型 ✅
`bisig.lfz` 内 `check(type(f(...))=="…")` 全绿：`len/range/pop/…`→`array`、`min`→元素型、`sum`→`int`/`float`、`has/startsWith`→`bool`、`floor/ceil/round/div/int/randInt`→`int`、`sqrt/pow/rand`→`float`、`str/type/join`→`string`、`each/seed/print/eprint/assert`→`nil`、`type((x)=>x)=="function"`。

### C-4　`pop([])` → `Index{idx:-1,len:0}` ✅
`e_pop_empty.lfz` = `pop([])`：
```
IndexError: 下标 -1 越界（长度 0）
```
对照 §10.7 补钉（interface-contract.md L219：`pop([]) → Index { idx: -1, len: 0 }`），逐字一致。

### C-5　`insert` 不支持负索引 ✅
- `e_insert_neg.lfz` = `insert(-1, 9, [1, 2])` → `IndexError: 下标 -1 越界（长度 2）`
- `e_insert_big.lfz` = `insert(3, 9, [1, 2])` → `IndexError: 下标 3 越界（长度 2）`
- 合法域 `[0,len]`：`insert(2,9,[1,2])==[1,2,9]` 通过（L220）。

### C-6　`min/max/minBy/maxBy` 空数组 ✅
| 夹具 | 实际 |
|---|---|
| `min([])` | `ValueError: 空数组没有极值（min）` |
| `max([])` | `ValueError: 空数组没有极值（max）` |
| `minBy((x)=>x,[])` | `ValueError: 空数组没有极值（minBy）` |
| `maxBy((x)=>x,[])` | `ValueError: 空数组没有极值（maxBy）` |

对照 §10.7 补钉 L216 `空数组没有极值（{func}）`，均一致。

### C-7　`randInt(lo>=hi)` ✅
`randInt(7,3)` → `ValueError: 区间非法：7 >= 3`；`randInt(5,5)` → `ValueError: 区间非法：5 >= 5`（L217）。

### C-8　`int` 边界 ✅
| 输入 | 实际 |
|---|---|
| `int(float("nan"))` | `ValueError: 无法把 float 转换为 int（'nan'）` |
| `int(float("inf"))`、`int(float("-inf"))` | `OverflowError: 整数溢出：结果超出 i64 范围` |
| `int(1e30)`（有限浮点截断超界） | `OverflowError: 整数溢出：结果超出 i64 范围` |
| `int("abc")` | `ValueError: 无法把 string 转换为 int（'abc'）` |

对照 L196 + L40 冻结口径，一致。

### C-9　`floor/ceil/round` 的 NaN/±Inf ✅
| 输入 | 实际 |
|---|---|
| `floor(float("nan"))` | `ValueError: 无法把 float 转换为 int（'nan'）` |
| `round(float("nan"))` | `ValueError: 无法把 float 转换为 int（'nan'）` |
| `floor(float("inf"))` / `ceil(float("inf"))` / `round(float("-inf"))` | `OverflowError: 整数溢出：结果超出 i64 范围` |

对照 L218（复用 `int(float)` 口径），一致。

### C-10　`round` 银行家舍入 ✅
`round(0.5)==0`、`round(1.5)==2`、`round(2.5)==2`、`round(3.5)==4`、`round(-2.5)==-2`、`round(2.4)==2`、`round(2.6)==3` 全绿（L188 四舍六入五成双）。

### C-11　`abs` 同型不加宽 ✅
`abs(-3)==3` 且 `type=="int"`；`abs(-3.0)==3.0` 且 `type=="float"`；`abs(-9223372036854775808)` → `OverflowError: 整数溢出：结果超出 i64 范围`（L185/L211）。

### C-12　`floor/ceil/round/sqrt/pow` 加宽 ✅
`floor(3)==3`、`ceil(3)==3`、`round(3)==3`、`sqrt(4)==2.0`、`pow(2,10)==1024.0` 均通过（int 实参先加宽，L210/212）。

### C-13　`div` ✅
`div(7,2)==3`、`div(-7,2)==-4`、`div(6,3)==2`、`div(0,5)==0`；`div(1,0)` → `ZeroDivisionError: 除以零`；`div(7.0, 2)` → `TypeError: 运算符 'div' 不支持 float 与 int`（L191、L64）。

### C-14　`%` 符号随除数 ✅
`-7 % 3 == 2`、`7 % -3 == -2` 通过（semantics §4.2 L64；与 Python 一致）。对照 `7 / 2 == 3.5`（真除法永远 float）。

### C-15　`del` 仅数据字段 ✅
- `del("a", {"a":1,"b":2}) == {"b":2}`（新 struct，不改原）。
- `e_del_method.lfz`（对含方法 `get` 的 struct 调 `del("get", …)`）→ `FieldError: 结构体没有字段 'get'`。
- `e_del_missing.lfz` → `FieldError: 结构体没有字段 'z'`。
- `del ⟺ has`：`has("get",box)==false`、`has("v",box)==true`，与 `del` 一致（L221）。

### C-16　`keys` 字节序 ✅
`keys({"b":1,"a":2})==["a","b"]`；`keys({"z":1,"A":2,"a":3})==["A","a","z"]`（ASCII 字节序）；`values/entries` 与 `keys` 同序（L163–165）。

### C-17　`has` 方法字段 → false ✅
含方法 `get` 的 struct：`has("get", box)==false`、`key(box)==["v"]`、`len(box)==1`（A5 数据面，L166）。

### C-18　`split("",s)` 按字符 ✅
`split("", "abc") == ["a","b","c"]`（L173）。

### C-19　`upper/lower` 仅 ASCII ✅
`upper("abC")=="ABC"`；`upper("é")=="é"`、`lower("Ä")=="Ä"`、`upper("中a")=="中A"` —— 非 ASCII 不被折叠（L176）。

### C-20　`str/int/float/type` 边界 ✅
`str`：`42→"42"`、`3.5→"3.5"`、`2.0→"2.0"`、`true→"true"`、`nil→"nil"`、`[1,2]→"[1, 2]"`、`{"k":1}→"{k: 1}"`、NaN→`"nan"`、Inf→`"inf"`。`int`：`2.9→2`、`-2.9→-2`、`"42"→42`、`true→1`。`float`：`3→3.0`、`"2.5"→2.5`、`"inf"/"nan"` 支持。`type`：8 类型名全对。均通过。

### C-21　`assert/check/fail` ✅
- `assert(true)==nil`（返回 nil）；`assert(false,"nope")` → `AssertionError: 断言失败：nope`；`assert(false)` → `AssertionError: 断言失败`（致命 exit 2）。
- `check(true)==true`、`check(false)==false`（非致命：stderr 一行 `check 失败：…`，程序继续，exit 0）。
- `fail("boom")` → `AssertionError: boom`；`fail()` → `AssertionError: fail()`（L202–204、semantics L252）。

### C-22　`sum` / `sort` / `filter` ✅
`sum([9223372036854775807, 1])` → `OverflowError`；`sort([1, "a"])` → `TypeError: 运算符 'sort' 不支持 string 与 number`；`filter((x)=>1,[1,2])` → `TypeError: 条件必须是 bool，得到 int`（L147/150/153）。

### C-23　随机内置 ✅
`seed(123)` 两次得到相同 `rand()`/`randInt()` 序列（夹具 `misc.lfz` 全绿）；`rand()` 20 次落在 `[0,1)`、类型 float；`randInt` 50 次落在 `[lo,hi)`、类型 int；`seed(1)==nil`。

### C-24　IO 内置 ✅
- `print("a","b","c")` → stdout `a b c\n`（空格连接）。
- `eprint("to-stderr")` → 写 **stderr**（stdout 为空）。
- `input()`（stdin=`hello`）→ 返回 `hello`（去行尾）；`input("P> ")` → 先输出 `P> `（无换行）再读 `world`；**EOF → `IOError: 输入结束（EOF）`**。

### C-25　`repeat` 溢出 —— ❌ FAIL
见 §③ defect **bug-20260927-03**。

### C-26　`range` 超大 n —— ⚠️ FAIL（规范静默）
见 §③ defect **bug-20260927-04**。

### C-27　§10.1 `ext(path)` 四步 ✅
| witness | 路径 | 期望 | 实际 |
|---|---|---|---|
| 最后分量优先 | `…\l_dir.lfz\x`（`l_dir.lfz` 是目录） | 非 `.lfz`（豁免） | `EXIT=0`，输出 `dir-ok` |
| Windows 盘符安全 | （同族 `C:\p\a.lfz`，见 C-31 的 `.lfz` 必检） | `lfz` | 见 C-31 |
| `a.`（空扩展名） | `l_dot.lfz.bak` 类比（扩展名 `bak`） | 非 `.lfz`（豁免） | `EXIT=0`，输出 `bak-ok` |
| 无扩展名 | `l_noext` | 非 `.lfz`（豁免） | `EXIT=0`，输出 `noext-ok` |
| `.LFZ` 大小写 | `l_upper.LFZ`（含 `#42`） | `.lfz`（必检） | `EXIT=0`，输出 `upper-ok` |

> 注：`a.`（尾点文件名）在 Win32 上不可直接创建（系统会剥离尾点），改以 `l_dot.lfz.bak`（最后 `.` 后为 `bak`）+ 无扩展名/`dir.lfz/x` 覆盖步骤 1–3；步骤 4（ASCII 大小写不敏感）由 `.LFZ` 实证。

### C-28　UTF-8 校验失败 → SyntaxError（含字节偏移） ✅
- `l_bad_utf8.txt`（字节 `61 62 63 FF`）→ `SyntaxError: 文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 3）`。
- `l_bad_utf8.lfz`（`FF`，首字节非法，*未有* `#42`）→ `SyntaxError: 文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 0）`。
→ 证明 **“先编码、后前导”**：非 UTF-8 报编码错而非 CosmosAnswerError（L129，syntax §2.2.2 步骤 3）。

### C-29　BOM ✅
- `l_bom.lfz`（`EF BB BF` + `#42\n…`）→ `EXIT=0`，输出 `bom-ok`（跳一个 BOM 后前导合法）。
- `l_bom_before_nl.lfz`（`EF BB BF 0A #42 0A`）→ `CosmosAnswerError`（BOM 后换行 → 前导不在开头）。
- `l_twobom.lfz`（双 BOM + `#42\n`）→ `CosmosAnswerError`（仅跳一个 BOM）。

### C-30　行终止符归一化 ✅
`l_crlf.lfz`、`l_cr.lfz`、`l_crlf_bytes.lfz`（字节级 CRLF）、`l_cr_bytes.lfz`（字节级 CR）均 `EXIT=0` 正常输出（`crlf-ok`/`cr-ok`/`crlf-bytes-ok`/`cr-bytes-ok`），与 LF 等价。

### C-31　`#42` 前导 + `line_base` ✅
- `l_linebase.lfz` = `#42\nprint(1)\n1/0\n` → 错误 `File "…", line 3`（body 本地行 2 + `line_base` 1 = 3），源行 `1/0` + 插入符。
- `l_linebase2.lfz`（body 本地行 3 出错）→ `line 4`。
- `l_plain_err.txt`（非 `.lfz`，body 本地行 2）→ `line 2`（`line_base=0`）。
对照 syntax §2.2.2 L148–150。

### C-32　非 `.lfz` 豁免 ✅
`l_noext`（无扩展名、无 `#42`）、`l_plain.txt`（无 `#42`）、`l_dir.lfz\x`、`l_dot.lfz.bak` 全部 `EXIT=0` 正常执行；`.lfz` 同内容则报 `CosmosAnswerError`（`l_nopre.lfz`）。与 syntax §2.2.0/§2.2.1 一致。

### C-33　§10.2 `Span` line/col ✅
- `span1.lfz`（`let a = "中"` / `a.nope`）→ `File "…", line 3`，插入符指向 `a.nope` 行首（col 1）。
- `span3.lfz`（第 3 个 body 行 `nope_undefined`）→ `NameError`，`line 4`，col 1。
- **Unicode 标量计数**：`span2.lfz` = `print("中", 1 / 0)`，实测 **插入符 `^` 与源行中 `1` 的列号精确相等**（`index of '1' = 15`，`index of '^' = 15`，含渲染缩进）→ 证明 col 按 **Unicode 标量**而非字节计数（若按字节，`中` 占 3 字节会使列号右移 2，实测未右移）。
对照 §10.2 L67 “`col` 1-based, Unicode 标量计数”。

### C-34　§10.5 `ScopeDebug` / `;;` ✅
- `dump1.lfz`（顶层 `a,b,f`；`f` 内 `a,c` 并 `;;`）stdout：
  ```
  a ： 10
  c ： 30
  b ： 2
  f ： <fn f>
  a ： 1
  b ： 2
  f ： <fn f>
  ```
  → 内层→外层；同作用域 slot 升序；**遮蔽去重**（内层 `a=10` 之后外层 `a` 不再重复）。
  **分隔符字节**：`61 20 EF BC 9A 20 31 30` = `a` `U+0020` `U+FF1A`(全角冒号) `U+0020` `1` `0`，逐字符符合 semantics §3.6 L25。
- `dump2.lfz`（块作用域）→ `y ： 2` 然后 `x ： 1`（块内→模块）。
- `dump3.lfz`（闭包捕获）→ `inner ： <fn inner>` / `outer ： 5` / `make ： <fn make>` / `g ： <fn inner>`（闭包可见链含捕获到的 `outer`，§10.5 L119）。

### C-35　§10.8 错误枚举 / `i64::MIN` ✅
- 实跑观察到的类名：`CosmosAnswerError`、`SyntaxError`、`NameError`、`TypeError`、`IndexError`、`FieldError`、`ZeroDivisionError`、`OverflowError`、`ValueError`、`IOError`、`AssertionError`、`RecursionError`（12/12 齐）。
- `#42\nprint(-9223372036854775808)\n` → `EXIT=0`，输出 `-9223372036854775808`（B12 合法）。
- `#42\nprint(9223372036854775808)\n`（无负号）→ `SyntaxError: 整数字面量超出 i64 范围`（L229–232）。

### C-36　`RecursionError` ✅
`rec.lfz`（`fn f(n) => f(n+1)` / `f(0)`）→ `EXIT=2`，末行 `RecursionError: 递归深度超限（超过 10000 层）`；人类可读输出**折叠**（`T>40` → 首 10 帧 + `... 省略 N 帧 ...` + 尾 30 帧，实测 stderr 123 行且含 “省略”）；帧名含 `in <module>` 与 `in f`。`--json` 下 `traceback` **不折叠**（实测 10000 个 `"func":"f"` 帧）。对照 §10.3 L80、§8.4。

### C-37　T-R1 缺 `#42` → ERROR/exit 2 ✅
`lfz test rt\T1`（`a.lfz` 无 `#42`）：
```
ERROR …/a.lfz  (…/a.lfz:1:1)
  File "…/a.lfz", line 1
  CosmosAnswerError: 你忘记了宇宙的答案
汇总：共 1 个用例，通过 0，失败 0，错误 1
```
`EXIT=2`。对照 §11.2 T-R1 L256。

### C-38　T-R2 ✅
- `lfz test rt\T4`（`ok.lfz` + `fixtures/bad.lfz`）→ `EXIT=0`，`通过 1，失败 0，错误 0`（`fixtures/` 未被发现 → T-R4）。
- `lfz test rt\T2`（`fixtures/neg.lfz` + `cases.json` `expect.error=CosmosAnswerError`）→ `PASS`，`EXIT=0`。
- `lfz test rt\T2b`（expect `SyntaxError`，实际 CosmosAnswerError）→ `FAIL`，`EXIT=1`。
- 未知类：`cases.json` 写 `NopeError` → `EXIT=2`，stderr `未知错误类 'NopeError'`。
- 清单非法 JSON → `EXIT=2`，stderr `不是合法 JSON：…`。

### C-39　T-R3 ✅
- `lfz test rt\T3`（`assert(false,"1+1==2")`）→ `FAIL`，`EXIT=1`，输出含 `File "…", line 2, in <module>` + 源行 + 插入符 + `AssertionError: 断言失败：1+1==2`。
- `lfz test rt\T3b`（`1 / 0`）→ `ERROR`，`EXIT=2`，`ZeroDivisionError: 除以零`。
- `check` 失败非致命：`lfz test` 全量套件含 `check(false)` 探针但 `EXIT=0`（stderr 仅警告）。

### C-40　T-R4 ✅
`lfz test`（项目默认）→ `汇总：共 85 个用例，通过 85，失败 0，错误 0`，`EXIT=0`（一个命令跑全部；`tests/fixtures/**` 被排除，负例由 `tests/cases.json` 声明）。

### C-41　ERROR 支配 FAIL ✅
`lfz test rt\Dom`（1 FAIL + 1 ERROR）→ 计数 `通过 0，失败 1，错误 1`，`EXIT=2`（D-008：有 ERROR → 2，优先于 FAIL → 1）。

### C-42　CLI 分派 ✅
`lfz run examples/hello.lfz` → `Hello, LFZ!` exit 0；`lfz examples/hello.lfz`（裸调用）→ 同输出 exit 0；`lfz test` → 85 全绿 exit 0。

### C-43　`--json` ✅
- 成功：`lfz --json run examples/hello.lfz` → **stdout 恰 1 行** `{"ok":true}`，程序输出 `Hello, LFZ!` 转 **stderr**（stdout 无程序文本）→ 满足“stdout 恒为单个 JSON”。
- 失败：`lfz --json run <e_div_zero.lfz>` → stdout **恰 1 行**，字段顺序 `ok/error/message/file/line/col/traceback`，`"error":"ZeroDivisionError"`、`"message":"除以零"`、`"line":2,"col":1`，exit 2；stderr 另有中文诊断块。
- 选项位置不变性：`run --json <f>`、`<f> --json`、`--json <f>` 三者 stdout JSON 完全一致。
- `lfz --json test rt\Json` → stdout **恰 1 行** `{"ok":false,"total":3,"passed":1,"failed":1,"errored":1,"cases":[…]}`，逐用例含 `verdict` 与错误字段。

### C-43b　`--json` × `;;`（dump 输出通道）—— ❌ FAIL
`--json` 的全部实跑中，**只有 `;;`（dump）会污染 stdout**：
```
文件: #42 \n let z = 5 \n ;; \n
命令: dist\lfz.exe --json run <file>
EXIT=0
stdout (2 行):  z ： 5\n{"ok":true}\n     ← 违反 “stdout 恒为单个 JSON”
stderr: (空)
```
对照 §10.6 L127（`--json` 输出 §8.3 字段）+ semantics §8.3“stdout 只写一行 JSON”；`print` 输出已正确转 stderr（C-43 通过），但 `;;` 走的是**未重定向的 stdout 直写**，构成契约缺口。
本项与符合性审计 B 的 `bug-B-20260927-01` **为同一缺陷**（B 已立案，owner=tooling-dev）；C 独立复现，证据一致，**不另编号**。

### C-44　`--help` / `--version` ✅
`--help`、`-h` → exit 0，完整帮助（含裸调用、`--json`、退出码、示例）；`--version`、`-V` → exit 0，stdout `lfz 1.0.0`。

### C-45　退出码映射 ✅
| 场景 | 实测 |
|---|---|
| 成功（run / test 全绿） | 0 |
| 测试有 FAIL（assert） | 1 |
| 测试有 ERROR（非 AssertionError / 缺前导） | 2 |
| run 时 LFZ 运行期错误 | 2 |
| CLI 参数错误（缺子命令 / 未知选项 / `run` 缺参 / 多参） | 2 |
| 裸调用非 `.lfz` / 缺文件 | 2 |
| `--json` 出错 | 仍 2 |

### C-46　错误位置格式 ✅
运行期：`Traceback (most recent call last):` + `File "<path>", line N, in <fn>` + 源行（4 空格缩进）+ 插入符行（`^`）+ `<Class>: <中文消息>`，且 `^` 列号与源行列号精确对齐（见 C-33）。加载/解析期：无 `Traceback` 头；`CosmosAnswerError` 仅 `File "<path>", line 1`（无源行/插入符），`SyntaxError` 有 `File` 帧 + 源行 + 插入符。

### C-47　裸调用拒绝 / 缺文件 ✅
`lfz notes.txt` → `IOError: 只支持 .lfz 脚本文件：'notes.txt'`（exit 2，不读文件）；`lfz nope.lfz` → `IOError: 无法读取：nope.lfz`；`lfz run nope.lfz` 同。

### C-48　前导违规全变体 ✅
`l_pretrail.lfz`（`#42 ` 尾随空格）、`l_prenonl.lfz`（`#42` 无换行）、`l_bom_before_nl.lfz`、`l_twobom.lfz`、`l_nopre.lfz` 全部 → `File "…", line 1` + `CosmosAnswerError: 你忘记了宇宙的答案`（无源码行/插入符）。对照 syntax §2.2.1 L112–121。

---

## ③ 缺陷单汇总

```
【缺陷单】bug-20260927-03（高）—— repeat 溢出触发 Rust panic（进程崩溃）
- 交付物: src/builtins.rs `b_repeat`（经 dist\lfz.exe 复现）
- 最小复现:
    文件 %TEMP%\opencode\confC\e_repeat_min.lfz:
        #42
        repeat(4611686018427387904, "ab")
    命令: dist\lfz.exe run %TEMP%\opencode\confC\e_repeat_min.lfz
    （等价: repeat(9223372036854775807, "ab")）
- 期望（spec 原文）: interface-contract.md L178
    | repeat | repeat(n, s) -> string | string | n <= 0 → 空串；溢出 → OverflowError |
  → 本输入 `n*len` 超过 isize::MAX，属“溢出”，应报 `OverflowError`。
- 实际:
    ↑ 进程 panic（stderr）:
      thread '<unnamed>' panicked at .../alloc/src/raw_vec/mod.rs:28:5:
      capacity overflow
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    退出码 = 101（非 2）；无 `OverflowError`、无 traceback、无中文消息。
- 附加证据（污染 runner）: 把该用例放入 `lfz test` 目录后，整个套件在打印首个 PASS 后崩溃，
    `EXIT=101`、无 `汇总：` 行——一条坏用例**中止整套测试**。
- 根因（定位，非修复）: `b_repeat` 用 `s.len().checked_mul(n as usize)` 只挡 `usize` 溢出；
    `4503599627370496*…` 的 `usize` 乘积可达 `isize::MAX+`，而 `String::repeat` 内部按 `isize` 上限做容量检查 → 触发库 panic。
    （`repeat(9223372036854775807, "abc")` 因 `usize` 乘积溢出被 `checked_mul` 挡住，正确报 OverflowError；故缺陷仅在窄带内显现。）
- 严重度: 高（违反 §10.7 repeat 明文契约；且可导致 `lfz test` 整体崩溃 / 退出码 101）
- 建议 owner: runtime-dev（builtins）
```

```
【缺陷单】bug-20260927-04（中）—— range 超大 n 触发 Rust panic（规范静默）
- 交付物: src/builtins.rs `b_range`（经 dist\lfz.exe 复现）
- 最小复现:
    文件 %TEMP%\opencode\confC\e_range_big.lfz:
        #42
        range(4611686018427387904)
    命令: dist\lfz.exe run %TEMP%\opencode\confC\e_range_big.lfz
- 期望: interface-contract.md L138 `range(n) -> array | [0, 1, …, n-1]；n < 0 → 空数组`。
    规范**未**定义超大 n 的溢出行为（gap）；但作为解释器，任何输入都不应使进程 panic。
- 实际:
      thread '<unnamed>' panicked at .../alloc/src/raw_vec/mod.rs:28:5:
      capacity overflow
    退出码 = 101（非 2）。
- 严重度: 中（规范静默 → 既需实现加护，也建议 spec 补钉明确 `OverflowError`）
- 建议 owner: runtime-dev（builtins）+ language-architect（spec 补钉）
```

```
【缺陷单】bug-B-20260927-01（中）—— `;;` × `--json` 使 stdout 非单个 JSON（C 域独立复现）
- 交付物: CLI `--json` 输出契约（tooling-dev；dump 通道未随 print 一并重定向）
- 最小复现:
    文件 %TEMP%\opencode\confCchk\dumpjson.lfz:
        #42
        let z = 5
        ;;
    命令: dist\lfz.exe --json run <file>
- 期望: interface-contract.md L127 + semantics.md §8.3 —— `--json` 下 **stdout 恒为单个 JSON**。
- 实际: stdout 2 行 = `z ： 5` + `{"ok":true}`；stderr 空。
- 严重度: 中（机器可读输出被污染；与 C-43 的 print 行为不一致）
- 建议 owner: tooling-dev（与符合性审计 B 同案，B 已立案；本报告不重复编号）
```

**观察项**

```
【OBS-01】（低）`lfz run --help` / `lfz test --help`：`--help` 仅在被识别为**首个**参数时生效；
   置于 `run`/`test` 之后会被当作路径（`IOError: 无法读取：--help` / `路径不存在：--help`）。
   非 spec 明文要求，记录备查。
【OBS-02】（低）`lfz --json --version` → stdout 输出 `lfz 1.0.0`（非 JSON）。
   help/version 不包 JSON；`--json` 契约仅针对 `run`/`test`，故非违约，记录备查。
```

---

## ④ 回归记录

| 缺陷 | 状态 | 复验证据 |
|---|---|---|
| bug-20260927-03（repeat overflow panic） | 🔴 **打开**（本次新增） | 本轮首现；待修复后按原命令复现 |
| bug-20260927-04（range huge panic） | 🔴 **打开**（本次新增） | 本轮首现；待修复后按原命令复现 |
| bug-B-20260927-01（`;;` × `--json`，审计 B 立案） | 🔴 **打开**（C 域独立复现） | 本轮 `--json run`（含 `;;`）stdout=2 行，证据见 §②C-43b |
| bug-20260927-01（P9：`s["k"]()` 未绑 self） | ✅ 已闭合（P9 rev.2） | 见 `docs/reports/P9-verification.md` §9 |
| bug-20260927-02（P9：`test --json` stdout 非单行，tests/REPORT 自认） | ⚪ 遗留（本轮未复验） | 本轮 `--json test` **实测 stdout 单行**（C-43），与 P9 记录不一致，建议 team-lead 裁定归属 |

> 说明：本轮未修改任何交付物，仅产出本报告。缺陷由 team-lead 转交 owner；修复后由 verifier 复验并在本表更新。

---

## ⑤ 总体结论

**一句话结论：§10.7 内置表 54/54 存在、53 项语义/边界与契约逐字吻合，§10.1 loader、§10.2 Span、§10.5 ScopeDebug、§10.8、§11.2 T-R1–T-R4 与 CLI 主体契约全部 PASS；三处硬伤为 `repeat` 溢出触发 Rust panic（bug-20260927-03，高）、`range` 超大入参 panic（bug-20260927-04，中）、`;;` × `--json` 使 stdout 非单个 JSON（bug-B-20260927-01，中）——**结论：**有条件符合（C 域）**，修复上述三项并复验通过后方可判“符合”。**

- 符合性: 50 条款中 **46 PASS / 3 FAIL / 1 观察组**。
- 阻塞项: **无**（三处均为例外入参 / 边缘通道，不阻塞 happy path 与现有 85 用例全绿）。
- 风险: `repeat`/`range` 的 panic 会使 `lfz test` 整体崩溃（exit 101）；`;;`×`--json` 破坏机器可读输出——均应在发布前修复。

---

### 附：证据留存

- 临时夹具与原始输出：`%TEMP%\opencode\confC\`（一次性，审计后清理）。
- 关键命令（可复现）：
  - `dist\lfz.exe run %TEMP%\opencode\confC\bisig.lfz`（内置 battery，exit 0）
  - `dist\lfz.exe run %TEMP%\opencode\confC\e_repeat_min.lfz`（panic，exit 101）
  - `dist\lfz.exe run %TEMP%\opencode\confC\e_range_big.lfz`（panic，exit 101）
  - `dist\lfz.exe test`（项目默认，85/85，exit 0）
  - `dist\lfz.exe test %TEMP%\opencode\confC\rt\<T1|T2|T2b|T3|T3b|T4|Dom|Json>`（T-R1–T-R4）
