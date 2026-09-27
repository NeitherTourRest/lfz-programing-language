# LFZ vs Python 性能基准报告

> 项目：LFZ 解释型脚本语言与解释器 ｜ 交付物 4（评分项 3 = 10 分）
> 作者：perf-engineer ｜ 日期：2026-09-27 ｜ 状态：**完成**
> 基准代码：`benchmarks/` ｜ 原始数据：`benchmarks/results/raw.json` ｜ 一键脚本：`benchmarks/run_all.py`

本文用**可复现的测量数据**回答：LFZ 解释器执行典型负载时，与 CPython 相比**慢多少 / 快多少、随规模如何变化、瓶颈在哪、值不值得优化**。全文区分「**数据**」与「**我的分析**」；劣势如实呈现，不美化、不挑样本。

---

## 0. 结论速览（TL;DR）

1. **解释器实现语言是 Rust**（`.opencode/team/DECISIONS.md` **D-011**），**不是** Python。因此 LFZ **并非**"Python 解释器之上再加一层解释"，该项不构成劣势来源；LFZ 的固有开销来自 **树遍历求值 + 运行时按名字查链（无 resolver）**。
2. **进程启动**：LFZ ≈ **11.7 ms**，CPython 3.13 ≈ **42.3 ms** —— LFZ **更快**。故**短负载**下 LFZ 总耗时反而更低（比值 < 1 全部由此产生）。
3. **纯计算负载**（数值循环 / 函数调用 / 递归）在规模足够大后，LFZ 比 CPython **慢 1.2×–3.9×**（同为数量级差距）。CPython 3.13 是带**特化自适应解释器**（PEP 659）的成熟字节码 VM，LFZ 是朴素树遍历器，此差距符合预期。
4. **原生高阶内建**（`map/filter/sort/reduce`）表现最好：最大规模仅 **1.20×**，因为重活由 Rust 原生代码承担。
5. **两处严重退化（超线性）**——这是本报告最重要的发现：
   - `struct_ops`（字符串键写 + `keys()`）：40 000 规模时 **28.80×**；
   - `string_ops`（反复 `push` 建数组）：30 000 规模时 **31.17×**。
   - 实测定位（见 §6）：`push` **每次返回全新数组（整表拷贝）** → O(N²)；struct 字符串键写/读疑似**线性扫描（无哈希索引）** → O(N²)。
6. **优化收益极高**：修复上述两点 + 引入 **resolver/槽位缓存**，有望把 LFZ 在大规模下从"慢一个数量级"拉回"接近甚至优于 CPython"。建议优先做（详见 §6）。
7. **输出一致性**：全部 **18/18** 单元 `LFZ stdout == Python stdout`，无算错（见附录 A）。

---

## 1. 方法与口径

### 1.1 计时口径（关键）

| 项 | 取值 |
|---|---|
| 时钟 | `time.perf_counter()`（高精度单调时钟；**未**使用已废弃的 `time.clock`） |
| 测量对象 | **整进程 wall time**：子进程启动 + 加载 + 词法/语法/求值 + 退出 |
| 启动基线 | 用 `noop` 程序（`fixtures/noop.{lfz,py}`）单独测量；LFZ 11.67 ms / Python 42.34 ms |
| 报告指标 | **total**（含启动）与 **net**（= total − 各自启动基线中位数）两条线；**比值以 total 为主、net 为辅** |
| 预热 | **2 轮**（不计时） |
| 计时轮数 | **5 轮**；报告**中位数**，同时记录最小值 / 最大值（原始逐轮值在 `raw.json`） |
| 构建 | 必须 **`cargo build --release`**（本报告全部数据基于 release 构建） |

> **为何同时报 total 与 net**：LFZ 与 Python 的启动开销**不对称**（LFZ 更快），只看 total 会在小规模处把"启动优势"误读成"吞吐优势"。net 剥离启动后才是纯执行对比。

### 1.2 公平对比约束

- 两侧**同算法、同输入规模、同输出**；规模 `N` 经 **stdin** 传入两侧。
- LFZ 使用原生高阶内建时，Python 侧**同样**使用原生 `map/filter/sorted/functools.reduce`；数值循环 / 函数调用 / 递归基准两侧都用**显式循环/显式递归**，**不使用** `sum()` 等内建（杜绝"LFZ 手写循环 vs Python 内建"的不公平对照）。
- 每个单元**每轮**都断言 `LFZ stdout == Python stdout` 且两侧退出码为 0；不一致即标记 `INVALID` 并排除。
- 全程整数运算，避免浮点格式化差异污染校验值。

### 1.3 复现命令

```powershell
cargo build --release
python benchmarks/run_all.py                 # 默认 warmup=2, runs=5, 每基准 3 个规模
python benchmarks/run_all.py --quick --warmup 1 --runs 3   # 冒烟
```

---

## 2. 环境

| 项 | 值 |
|---|---|
| 机器 | Intel(R) Core(TM) i7-14650HX，16 物理核 / 24 逻辑核，RAM 15.7 GB |
| OS | Windows 11 家庭中文版，build 26200 |
| 构建工具链 | `rustc 1.98.1 (48a229cea 2026-09-01)` / `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| LFZ 解释器 | `lfz 0.1.0`，release 构建（`target\release\lfz.exe`；全量重编 **0 warning / 0 error**，3.08 s） |
| Python | CPython **3.13.9**（Anaconda 发行版，`D:\app\ANACONDA\python.exe`） |
| 测量时间 | 2026-09-27T04:03:53Z |

> 说明：CPython **3.13** 自带**特化自适应解释器**（PEP 659），是当前 CPython 吞吐最高的分支之一；对比不含 PyPy / 其它实现。

---

## 3. 基准清单

| # | 基准 | 类别 | 负载说明 | LFZ 实现 | Python 实现 |
|---|---|---|---|---|---|
| B1 | `numeric_loop` | 数值循环 | `while` 累加 `1..N`，**不用**任何内建聚合 | `benchmarks/lfz/numeric_loop.lfz` | `benchmarks/python/numeric_loop.py` |
| B2 | `function_calls` | 函数调用 | `N` 次调用小纯函数 `f(x)=(31x+7) mod 1000003`，非递归 | `benchmarks/lfz/function_calls.lfz` | `benchmarks/python/function_calls.py` |
| B3 | `recursion_fib` | 递归 | 朴素（指数级）递归 `fib(n)` | `benchmarks/lfz/recursion_fib.lfz` | `benchmarks/python/recursion_fib.py` |
| B4 | `array_builtins` | 数组 + 内建 | `range(N) \|> map(·) \|> filter(·) \|> sort() \|> reduce(+)` | `benchmarks/lfz/array_builtins.lfz` | `benchmarks/python/array_builtins.py` |
| B5 | `struct_ops` | 结构体/字典 | 字符串键写 `s["k<i>"]=i²` `N` 次 + `keys()` 排序遍历求和 | `benchmarks/lfz/struct_ops.lfz` | `benchmarks/python/struct_ops.py` |
| B6 | `string_ops` | 字符串 | 插值构造 → `trim`/`map` → `join` → `replace` → `split` → `repeat` | `benchmarks/lfz/string_ops.lfz` | `benchmarks/python/string_ops.py` |

每个规模均满足"单轮 < 60 s"；`struct_ops`/`string_ops` 因超线性（见 §6）已把最大规模收敛到 40 000 / 30 000（更大规模单轮 > 60 s，**未测**）。

---

## 4. 结果表

比值 = `LFZ total 中位数 / Python total 中位数`；`< 1` 表示 LFZ 总耗时更低（含启动）。单位：秒。

### B1 `numeric_loop`
| N | LFZ 中位数 | LFZ min/max | Python 中位数 | Python min/max | **比值** | LFZ net | Python net |
|---|---|---|---|---|---|---|---|
| 10 000 | 0.01485 | 0.01205 / 0.02157 | 0.04373 | 0.04119 / 0.05387 | **0.34** | 0.00318 | 0.00139 |
| 100 000 | 0.03058 | 0.02804 / 0.03178 | 0.05170 | 0.04693 / 0.05327 | **0.59** | 0.01891 | 0.00937 |
| 1 000 000 | 0.18975 | 0.16497 / 0.19729 | 0.15298 | 0.14511 / 0.19250 | **1.24** | 0.17808 | 0.11064 |

### B2 `function_calls`
| N | LFZ 中位数 | Python 中位数 | **比值** | LFZ net | Python net |
|---|---|---|---|---|---|
| 10 000 | 0.01993 | 0.04653 | **0.43** | 0.00826 | 0.00419 |
| 100 000 | 0.05870 | 0.05941 | **0.99** | 0.04703 | 0.01707 |
| 1 000 000 | 0.42337 | 0.23027 | **1.84** | 0.41170 | 0.18793 |

### B3 `recursion_fib`
| n | LFZ 中位数 | Python 中位数 | **比值** | LFZ net | Python net |
|---|---|---|---|---|---|
| 20 | 0.02304 | 0.04739 | **0.49** | 0.01137 | 0.00505 |
| 24 | 0.07202 | 0.05226 | **1.38** | 0.06035 | 0.00993 |
| 28 | 0.34426 | 0.08896 | **3.87** | 0.33259 | 0.04662 |

### B4 `array_builtins`
| N | LFZ 中位数 | Python 中位数 | **比值** | LFZ net | Python net |
|---|---|---|---|---|---|
| 1 000 | 0.01688 | 0.04908 | **0.34** | 0.00520 | 0.00674 |
| 10 000 | 0.02073 | 0.04910 | **0.42** | 0.00906 | 0.00677 |
| 100 000 | 0.08249 | 0.06881 | **1.20** | 0.07081 | 0.02647 |

### B5 `struct_ops`
| N | LFZ 中位数 | Python 中位数 | **比值** | LFZ net | Python net |
|---|---|---|---|---|---|
| 5 000 | 0.05655 | 0.04933 | **1.15** | 0.04488 | 0.00699 |
| 15 000 | 0.23014 | 0.04671 | **4.93** | 0.21847 | 0.00437 |
| 40 000 | 1.75309 | 0.06087 | **28.80** | 1.74142 | 0.01854 |

### B6 `string_ops`
| N | LFZ 中位数 | Python 中位数 | **比值** | LFZ net | Python net |
|---|---|---|---|---|---|
| 3 000 | 0.06534 | 0.05023 | **1.30** | 0.05366 | 0.00789 |
| 10 000 | 0.26280 | 0.04444 | **5.91** | 0.25113 | 0.00210 |
| 30 000 | 1.68412 | 0.05403 | **31.17** | 1.67244 | 0.01170 |

### 启动基线（noop，7 轮）
| 解释器 | 中位数 | 说明 |
|---|---|---|
| LFZ `lfz run noop.lfz` | **11.67 ms** | 进程启动 + 加载 + 词法/语法/求值空程序 |
| CPython `python noop.py` | **42.34 ms** | 解释器启动 + 加载空脚本 |

---

## 5. 结论

### 5.1 数据事实

- **启动**：LFZ 比 CPython 快约 **3.6×**（11.67 ms vs 42.34 ms）。因此**所有小规模单元的 total 比值 < 1**，本质是"启动优势"，**不是** LFZ 计算更快 —— 见各表 net 列：净执行时间 LFZ 反而更高。
- **随规模趋势**：所有基准的比值**随 N 单调上升**。计算类上升到 **1.2×–3.9×** 后趋于平缓（数量级内）；结构体/字符串类则**发散**（28.8× / 31.2×），呈超线性。
- **公平对照下 LFZ 从未在"大规模纯执行"上快于 CPython**；唯一 LFZ 净执行更快的情形只出现在极小规模（受噪声支配，不足为凭）。

### 5.2 我的分析（与数据分开陈述）

- **差距主因（LFZ 侧）**：P3 求值器是**树遍历 + 运行时按名字查链**（无 resolver）。每次变量读取都要沿作用域链按**名字**查找，每次函数调用都要**新建帧/环境**并绑定参数名。这正是循环/递归/函数调用基准在大规模下落后的原因，且**递归越深、调用越密，固定 per-call 开销占比越高**（`fib` 3.87× 即为实证）。
- **内建为何接近**：`array_builtins` 的排序/折叠重活由 **Rust 原生**完成，树遍历只承担闭包逐元素调用，故最大规模仅 1.20×。
- **为何 LFZ 不是"Python 加一层"**：实现语言是 Rust（D-011），不存在 Python 级解释开销叠加；对照口径是"Rust 树遍历解释器 vs CPython 字节码 VM"。
- **致命的两处超线性**是**语义实现方式**问题，而非"解释器慢"——详见 §6。

### 5.3 总体判断

LFZ 解释器在**同数量级**内可胜任日常脚本负载；真正拖后腿的是**两处容器操作的实现复杂度**。修复后（§6）预期可将 max 规模比值从"数十倍"降到"个位数甚至接近 1"。**结论：值得优化，且优化点明确、收益极高。**

---

## 6. 瓶颈分析与优化建议（给 runtime-dev）

> 本报告**只报告不改代码**；以下为建议，实施归 runtime-dev，经 team-lead 协调。定位证据见附录 C。

### 6.1 🔴 严重：`push` 等"返回新容器"的内建对大数组是 O(N²)

- **现象**：`string_ops` 中反复 `parts = push(x, parts)` 构造 N 元数组，30 000 规模耗时 1.67 s（Python 0.012 s）。
- **定位**：探针显示 `string_ops` 的**全部耗时**来自 push 循环本身（push-only 1.70 s ≈ full 1.63 s，见附录 C）。
- **根因**：`push(v, xs)` 按 A1 语义**返回新数组**；若实现为**整表深拷贝**，则 N 次 push = ΣO(k) = **O(N²)**。
- **建议**：引入 **copy-on-write / 持久化向量**（`Rc<[T]>`+`make_mut`，或 `im`/`rpds` 式结构）：
  - 数组**未被共享**时 push 走 `Vec::push` **原地 O(1) 摊销**；
  - 仅当存在别名（引用共享）时才拷贝，保持 A1 语义不变。
  - 同类检查 `pop/removeAt/insert/swap/slice/del`。**这一项即可消除 string_ops 的 31×。**

### 6.2 🔴 严重：struct 字符串键写/读疑似线性扫描（无哈希索引）

- **现象**：`struct_ops` 40 000 规模 1.74 s（Python 0.019 s）。
- **定位**：build-only（纯写）0.86 s；build + `keys()` + 逐键读 1.73 s（见附录 C）。**写和读都超线性**。
- **根因**：按字符串键 `s[k]` 的写入与读取，若为**对字段表做线性扫描**，n 个唯一键即 O(N²)。
- **建议**：struct 数据面改为 **`IndexMap` 式（HashMap<key→slot> + 顺序数组）**：
  - 键存在 → O(1) 定位后原地写；
  - 新键 → O(1) 摊销追加；
  - `keys()` 需**字节序升序**（A5/B3），可对哈希索引排序或在 `keys()` 时排序（O(n log n) 可接受）。
  - **这一项即可消除 struct_ops 的 28.8×。**

### 6.3 🟠 重要：引入 resolver / 槽位缓存（消除运行时名字查链）

- **现象**：循环/函数/递归基准净执行落后 1.6×–7×，且随调用密度上升。
- **建议（性价比最高）**：
  1. **解析期 resolver**：为每个标识符出现解析出 `(作用域深度, 槽位号)`，写回 AST；求值期以**数组下标**访问，移除按名字查链。
  2. **内联缓存**：全局/闭包变量的解析结果缓存到 AST 节点，命中即跳过查找。
  3. **帧/环境复用**：参数绑定用槽位数组而非名字表；复用调用帧缓冲，避免每次调用新建 `Rc`/`HashMap`。
- **预期**：直接改善所有基准的 net 比值，尤其 B1/B2/B3。

### 6.4 🟡 中长期：字节码 VM

- 树遍历的间接调用与指针追逐是结构性常数开销；将 AST 编译为**栈式/寄存器式字节码**可再降一档。建议在 6.1–6.3 落地、收益见顶后再评估。
- CPython 3.13 的 gap 部分由特化字节码解释器产生；LFZ 若走字节码 + 少量特化即可缩小乃至反超。

### 6.5 🟢 已良好，无需优化

- **进程启动 11.7 ms**：优于 CPython，保持。
- **原生高阶内建**（B4）已接近 CPython，改动风险大于收益，**不建议动**。

### 优先级建议
**6.1 ≈ 6.2（修超线性，收益最大）→ 6.3（普遍提速）→ 6.4（结构性）**。建议 runtime-dev 先做 6.1/6.2，我随即**复测更新本报告数据**。

---

## 7. 局限（诚实声明）

1. **单机单平台**：1 台 Windows 11 / i7-14650HX；未做 CPU 绑核、未严格控制后台负载（重启前请关闭重负载程序）。多轮取中位数以降低波动，仍有若干离群（min/max 见 `raw.json`）。
2. **计时含解释器启动**：已单列 noop 基线与 net，但 total 仍混入启动；小规模结论以 net 为准。
3. **对照实现**：仅对比 CPython 3.13（含特化解释器），**未**对比 PyPy 或其它；"快/慢"结论仅对 CPython 3.13 成立。
4. **样本规模**：`struct_ops`/`string_ops` 最大规模受"单轮 < 60 s"约束收敛在 40 000 / 30 000；**更大规模的趋势为外推**（本报告已标注，不外推具体数字）。
5. **负载代表性**：6 个基准覆盖 5 类典型负载，但**不覆盖** IO、闭包重度、GC 压力等；结论限于所列负载。
6. **优化预期为分析性外推**，非已实现数据；是否达成需 runtime-dev 修复后由本套件复测。

---

## 8. 期望输出与"黑盒 / 回归素材"说明

**6 个基准均为确定性程序**（无 `rand`、无外部输入），输出单行 `RESULT ...`，因此**可直接用作黑盒/回归测试素材**：

```powershell
"10000" | target\release\lfz.exe run benchmarks\lfz\numeric_loop.lfz
# 期望: RESULT total=50005000
```

| 基准 | 期望输出通式 | 已测规模 → 实测输出（LFZ 与 Python 一致） |
|---|---|---|
| `numeric_loop` | `RESULT total=N(N+1)/2` | 10⁴→`50005000`；10⁵→`5000050000`；10⁶→`500000500000` |
| `function_calls` | `RESULT acc=` 迭代 `f(x)=(31x+7) mod 1000003`，初值 1 | 10⁴→`477891`；10⁵→`803074`；10⁶→`27055` |
| `recursion_fib` | `RESULT fib=F(n)` | 20→`6765`；24→`46368`；28→`317811` |
| `array_builtins` | `RESULT total=` 偶数 `(7i mod N)` 之和，i∈[0,N) | 10³→`249500`；10⁴→`24995000`；10⁵→`2499950000` |
| `struct_ops` | `RESULT keys=N total=Σi²`，i∈[0,N) | 5·10³→`keys=5000 total=41654167500`；15·10³→`keys=15000 total=1124887502500`；40·10³→`keys=40000 total=21332533340000` |
| `string_ops` | `RESULT chars=<join 长度> words=N tag=ababab` | 3·10³→`chars=25889`；10⁴→`chars=88889`；30·10³→`chars=288889` |

**交叉验证机制**：harness 每轮断言 LFZ 输出 == Python 输出，本次全部 **18/18 通过**，无算错数据。

---

## 附录 A：一键运行原始输出（完整）

命令：`cargo build --release`（全量重编：`Compiling lfz v0.1.0` → `Finished` 3.08 s，**warning count = 0**，exit 0），随后：

```
python benchmarks\run_all.py --warmup 2 --runs 5
```

```
==============================================================================
LFZ vs Python benchmark harness
==============================================================================
  timestamp_utc : 2026-09-27T04:03:53+00:00
  os            : Windows-11-10.0.26200-SP0
  cpu           : Intel64 Family 6 Model 183 Stepping 1, GenuineIntel
  cpu_count     : 24
  python        : 3.13.9
  python_exe    : D:\app\ANACONDA\python.exe
  lfz           : lfz 0.1.0
  rustc         : rustc 1.98.1 (48a229cea 2026-09-01)
  cargo         : cargo 1.98.1 (797e8a9bc 2026-08-05)
  warmup        : 2
  runs          : 5
  lfz_binary    : target\release\lfz.exe

Measuring startup baseline (noop) ...
  LFZ    noop median = 11.67 ms
  Python noop median = 42.34 ms

benchmark                N  LFZ med(s)   PY med(s)    ratio   LFZ net    PY net   status
----------------------------------------------------------------------------------------
numeric_loop         10000     0.01485     0.04373     0.34   0.00318   0.00139       ok
numeric_loop        100000     0.03058     0.05170     0.59   0.01891   0.00937       ok
numeric_loop       1000000     0.18975     0.15298     1.24   0.17808   0.11064       ok
function_calls       10000     0.01993     0.04653     0.43   0.00826   0.00419       ok
function_calls      100000     0.05870     0.05941     0.99   0.04703   0.01707       ok
function_calls     1000000     0.42337     0.23027     1.84   0.41170   0.18793       ok
recursion_fib           20     0.02304     0.04739     0.49   0.01137   0.00505       ok
recursion_fib           24     0.07202     0.05226     1.38   0.06035   0.00993       ok
recursion_fib           28     0.34426     0.08896     3.87   0.33259   0.04662       ok
array_builtins        1000     0.01688     0.04908     0.34   0.00520   0.00674       ok
array_builtins       10000     0.02073     0.04910     0.42   0.00906   0.00677       ok
array_builtins      100000     0.08249     0.06881     1.20   0.07081   0.02647       ok
struct_ops            5000     0.05655     0.04933     1.15   0.04488   0.00699       ok
struct_ops           15000     0.23014     0.04671     4.93   0.21847   0.00437       ok
struct_ops           40000     1.75309     0.06087    28.80   1.74142   0.01854       ok
string_ops            3000     0.06534     0.05023     1.30   0.05366   0.00789       ok
string_ops           10000     0.26280     0.04444     5.91   0.25113   0.00210       ok
string_ops           30000     1.68412     0.05403    31.17   1.67244   0.01170       ok

All outputs matched: True
Raw data written to ...\benchmarks\results\raw.json
```

> 原始**逐轮**计时值（每单元 5 个）见 `benchmarks/results/raw.json`，助教可复核离群与中位数计算。

## 附录 B：启动基线细节

`noop` 程序 = 仅 `print("RESULT ok=1")`（LFZ 版含 `#42` 首行）。各跑 `max(runs,7)=7` 轮取中位数：

| 解释器 | 中位数 | 用途 |
|---|---|---|
| LFZ | 11.67 ms | total 减此项得 net |
| CPython | 42.34 ms | total 减此项得 net |

## 附录 C：超线性定位探针（临时脚本，测后已删除）

为定位 B5/B6 的复杂度，临时编写两个探针（LFZ，测毕已从 `benchmarks/lfz/` 删除）：

| 探针 | 内容 | N | 结果 |
|---|---|---|---|
| `_probe_struct_build` | 仅写 N 个字符串键，不 `keys()`/不遍历 | 15 000 / 40 000 | **0.137 s / 0.863 s** |
| `struct_ops`（完整） | 写 + `keys()` + 逐键读求和 | 15 000 / 40 000 | **0.211 s / 1.730 s** |
| `_probe_string_push` | 仅 `push` 循环，不 trim/join | 10 000 / 30 000 | **0.280 s / 1.704 s** |
| `string_ops`（完整） | push + trim/join/replace/split | 10 000 / 30 000 | **0.266 s / 1.626 s** |

- **string_ops**：push-only ≈ 完整耗时 → **全部代价在 push**（失败模式 = 整表拷贝的 O(N²)）。
- **struct_ops**：build-only ≈ 一半，写 + 读两段均超线性 → **键写与键读皆为 O(N²) 量级**（失败模式 = 线性扫描）。

数据（3× 规模）与上述探针共同支撑 §6.1 / §6.2 的根因判定。

---

## 附：交付物与证据索引

| 内容 | 路径 |
|---|---|
| 一键 harness（仅标准库） | `benchmarks/run_all.py` |
| LFZ 基准（6） | `benchmarks/lfz/*.lfz` |
| Python 等价实现（6） | `benchmarks/python/*.py` |
| 启动基线 | `benchmarks/fixtures/noop.{lfz,py}` |
| 原始逐轮数据 | `benchmarks/results/raw.json` |
| 基准说明 / 期望输出 | `benchmarks/README.md` |
| 本报告 | `docs/reports/performance.md` |
