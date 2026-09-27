# LFZ 性能基准套件（P6 / 评分项 3）

本目录是**交付物 4**（LFZ vs Python 性能基准与报告）的基准程序与一键运行器。
正式报告见 [`../docs/reports/performance.md`](../docs/reports/performance.md)。

## 目录结构

```
benchmarks/
├── run_all.py            # 一键基准 harness（仅标准库，无第三方依赖）
├── lfz/                  # LFZ 基准实现（每个 .lfz 首行必须 #42）
│   ├── numeric_loop.lfz  # 数值循环：while 累加 1..N（不用任何内建聚合）
│   ├── function_calls.lfz# 高频函数调用：N 次调用小纯函数
│   ├── recursion_fib.lfz # 递归：朴素递归 fib(n)
│   ├── array_builtins.lfz# 数组+内建：range|>map|>filter|>sort|>reduce
│   ├── struct_ops.lfz    # 结构体/字典：N 次字符串键写 + keys() + 求和
│   └── string_ops.lfz    # 字符串：插值构造/trim/join/replace/split/repeat
├── python/               # 与 lfz/ 一一对应的功能等价 Python 实现
├── fixtures/             # noop 启动基线（noop.lfz / noop.py）
└── results/raw.json      # harness 写出的原始逐轮计时数据（可复核）
```

每个基准的输入规模 `N` 通过 **stdin**（一行整数）传入，LFZ 与 Python 两侧一致。

## 运行（可复现）

```powershell
# 1) 必须使用 release 构建
cargo build --release

# 2) 一键运行（默认 warmup=2、runs=5，覆盖每个基准 3 个规模）
python benchmarks/run_all.py

# 可选：快速冒烟（每个基准只跑第一个规模）
python benchmarks/run_all.py --quick --warmup 1 --runs 3
```

harness 会在**每个基准每一轮**都断言 `LFZ stdout == Python stdout`；不一致的单元标记
`INVALID` 并排除，从而防止"计时成功但算错"的无效数据混入报告。原始逐轮数据写入
`benchmarks/results/raw.json`。

## 作为黑盒 / 回归素材

这 6 个基准都是**确定性**程序（无 `rand`、无 `input` 之外的外部输入），输出为单行
`RESULT ...`，因此可直接作为**回归测试**素材：

```powershell
# 单基准回归：规模经 stdin 传入
"10000" | target\release\lfz.exe run benchmarks\lfz\numeric_loop.lfz
```

## 期望输出（交叉验证用）

设输入规模为 `N`：

| 基准 | 期望输出 | 已测规模 → 实际输出 |
|---|---|---|
| `numeric_loop` | `RESULT total=<N(N+1)/2>` | 10⁴→`total=50005000`；10⁵→`total=5000050000`；10⁶→`total=500000500000` |
| `function_calls` | `RESULT acc=<迭代 f(x)=(31x+7) mod 1000003 N 次，初值 1>` | 10⁴→`acc=477891`；10⁵→`acc=803074`；10⁶→`acc=27055` |
| `recursion_fib` | `RESULT fib=<斐波那契>` | 20→`fib=6765`；24→`fib=46368`；28→`fib=317811` |
| `array_builtins` | `RESULT total=<偶数的 (7i mod N) 之和，i∈[0,N)>` | 10³→`total=249500`；10⁴→`total=24995000`；10⁵→`total=2499950000` |
| `struct_ops` | `RESULT keys=N total=<Σi²，i∈[0,N)>` | 5·10³→`keys=5000 total=41654167500`；15·10³→`keys=15000 total=1124887502500`；40·10³→`keys=40000 total=21332533340000` |
| `string_ops` | `RESULT chars=<join 后长度> words=N tag=ababab` | 3·10³→`chars=25889`；10⁴→`chars=88889`；30·10³→`chars=288889` |

> `chars` 的通式：每个元素 `"  item<i>  "` 去空白后为 `item<i>`，以 `,` 连接；
> `chars = Σ_{i=0}^{N-1} len("item"+str(i)) + (N-1)`。

## 公平对比原则

- 两侧**同算法、同规模、同输出**；LFZ 用原生高阶内建时，Python 也用原生 `map/filter/sorted/reduce`。
- 数值循环 / 函数调用 / 递归基准：两侧都用**显式循环/显式递归**，不使用 `sum()` 之类内建（避免"LFZ 手写循环 vs Python 内建"的不公平对照）。
- 计时口径见报告：whole-process wall time（含进程启动），并单列 noop 启动基线与 net 值。
