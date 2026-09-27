# ppt-presenter — 工作状态
> 最后更新: 2026-09-27 by ppt-presenter

## 当前状态
**P10 答辩材料 — 完成（交付物 8）。** 三件产出全部落地于 `docs/slides/`，关键命令已亲自跑通并留真实转录。

## 进行中
- （无）

## 产出（本次）
| 文件 | 大小 | 说明 |
|---|---|---|
| `docs/slides/LFZ-defense.pptx` | 84304 B | 14 页系统介绍 PPT（python-pptx 生成，16:9） |
| `docs/slides/demo-script.md` | 13860 B | 现场演示脚本（6 步 + 失败预案 + 备份方案） |
| `docs/slides/qa-prep.md` | 20450 B | 问答预案 17 问（含 7 条 ★ 高杀伤力） |

**实测证据（2026-09-27，均亲跑）**：
- PPT 读回：`SLIDE_COUNT=14  SIZE=13.333x7.500  BYTES=84152`；逐页标题正常（P01「LFZ 语言与解释器」… P13「结论与展望」）。
- `cargo run --quiet -- run examples/hello.lfz` → `Hello, LFZ!` exit 0
- `cargo run --quiet -- run tests/fixtures/missing_preamble.lfz` → `CosmosAnswerError: 你忘记了宇宙的答案` exit 2
- `cargo run --quiet -- run --json examples/hello.lfz` → `{"ok":true}` exit 0
- `cargo run --quiet -- test` → `汇总：共 82 个用例，通过 82，失败 0，错误 0` exit 0
- `cargo run --quiet -- run app/sortviz.lfz` → 三阶段 + `5 种算法全部通过正确性校验` exit 0
- `cargo test` → **431 passed / 0 failed / 0 ignored**（361+42+16+12）
- 现场手写小程序（filter/map/sum）→ `evens^2 = [4, 16, 36, 64, 100]` / `sum = 220  n = 5` exit 0

## 阻塞 / 需要支持
- （无）**仅需 team-lead 确认两点**：① 实际答辩时限（决定 PPT 讲稿时长与演示压缩口径）；② 是否需 PPT 同步出 PDF 备份。
- ⚠️ **P9 口径对齐已完成**：verifier `docs/reports/P9-verification.md` 总评 **CONCERNS（无阻塞项；7/8 达标，交付物 8「PPT 已产出待入库」）**。已据此：① 更新 PPT P13 结论页加入 P9 口径与 2 项待闭合（bug-01 + PPT 入库）；② `qa-prep.md` Q13 补充 P9 结论与 bug-01/bug-02。**PPT 入库（git add/commit）非我职责（归 release-manager，我禁 commit）**，已在交付缺口单列出。

## 下一步计划
1. 待 team-lead 给定时限后，微调 `demo-script.md` §4 时间分配。
2. 答辩前排练一遍全流程（按 `demo-script.md` §6 环境核对清单）；预演记录进 STATUS/JOURNAL。
3. 待 release-manager 确认最终 tag（`v0.4-app` 之后是否有 `v1.0`）。

## 关键经验（写给未来的自己）
- **数字要现测**：README 写 377（P3 基线），P5 后 `cargo test` 实为 **431**；PPT 用最新实测并主动标注口径差异（qa-prep Q16 已备答），避免被当场抓住自相矛盾。
- **性能数据必须如实**：LFZ 启动快 3.6×，但大规模纯计算慢 1.2–3.9×，struct/string 两处 O(N²) 到 28.8×/31.2×——照搬 `performance.md`，不美化，并在 qa-prep 里备好根因与优化路线。
- **演示先跑通再写脚本**：本文件所有命令与输出均为亲跑转录，未写"应该不会失败"式空话。
- **PPT 生成环境**：本机 Python 3.13.9 + python-pptx 1.0.2 已就绪；生成脚本走 `%TEMP%`，执行后自删（未污染工作区）。
- **中文编码坑**：PowerShell 控制台 GBK，CJK 会显示乱码但 pptx/md 存的是 UTF-8；验证 CJK 时把标题 dump 到 UTF-8 文件再读，勿信控制台直显。
