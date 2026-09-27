# perf-engineer — 工作状态
> 最后更新: 2026-09-27 12:06 by perf-engineer

## 当前状态
**P6 完成**：LFZ vs Python 性能基准套件 + 正式报告已交付（评分项 3 = 10 分，交付物 4）。
- 基准：`benchmarks/`（6 基准 × 3 规模，LFZ + 功能等价 Python，一键 `run_all.py`）。
- 报告：`docs/reports/performance.md`（方法/环境/数据表/结论/瓶颈/局限/期望输出）。
- 一键复现：`cargo build --release; python benchmarks/run_all.py` → 18/18 输出一致。

## 进行中
- （无）

## 阻塞 / 需要支持
- （无）

## 下一步计划
- 等 runtime-dev 按报告 §6 实施优化（push COW / struct 哈希索引 / resolver+槽位缓存）后，**我复测并更新报告数据**。
- 本套件可供 verifier 独立复跑核验。

## 关键经验（写给未来的自己）
- **实现语言是 Rust（D-011）**，不是 Python → LFZ **不是**"Python 加一层"；差距来自**树遍历 + 运行时名字查链**，不是 Python 级开销。
- **口径**：整进程 wall time；LFZ 启动 ~11.7 ms 快于 CPython ~42.3 ms ⇒ **小规模 total 比值 < 1 是启动优势**，必须看 **net** 才有意义。
- **已定位 2 处超线性（只报告不改）**：① `push` 返回新数组→整表拷贝 O(N²）（string_ops 30k=31.17×）；② struct 字符串键疑似线性扫描 O(N²）（struct_ops 40k=28.80×）。
- 基准**确定性**、可作黑盒/回归素材；期望输出表在 `benchmarks/README.md`。
- 环境：i7-14650HX / Win11 build 26200 / rustc 1.98.1 / CPython 3.13.9（Anaconda）。
- 禁区遵守：未改 `src/**`、`docs/spec/**`、`README`、`tests/**`；未 commit/tag/push。
