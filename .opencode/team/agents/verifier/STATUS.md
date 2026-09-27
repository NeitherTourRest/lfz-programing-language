# verifier — 工作状态
> 最后更新: 2026-09-27 by verifier

## 当前状态
**T11-09 修复批次全量复验（7 类修复）已完成 → 结论：通过（7/7 + 回归全绿 + 0 新缺陷）。**
- 产出：`docs/reports/T11-reverification.md`（18313 B）；一手原始证据：`docs/reports/T11-reverification-evidence/`（12 文件，83 KB）。
- 被验产物：**`target\release\lfz.exe`**（**710144 B**，mtime `2026-09-27 22:26:59`，SHA256 `619AFD2E…`），由 `cargo build --release` 从当前工作树源码构建（二跑 0.03s 无重编译 = 与源码同源）。**未用 `dist\lfz.exe`（704000 B，修复前）**。
- 统计：**39 复验项 → PASS 39 / FAIL 0 / 新缺陷单 0**；观察项 2（非阻塞）。

## 逐项结论（证据见报告 §②）
1. `repeat(2^62,"ab")` → **exit 2** + `OverflowError: 容量溢出：所需容量超出可分配上限`（原 101）。✅
2. `range(2^62)` → **exit 2** + 同上。✅
3. 解析嵌套 `PARSE_DEPTH_LIMIT=1000`：`(`×1000→**0**；`(`×1001（平衡/前缀）、`[`×1001（前缀/平衡）、`{"a":`×1001、`fn(){`×1001 → **exit 2** + `SyntaxError: 嵌套深度超限（超过 1000 层）`（`--json` col=1001）；**无 `-1073741571`**。✅ (8/8)
4. `AST_DEPTH_LIMIT=10000`：`1+1+…` 9999→**0**、10000/10001→**2**、**100000→2**；`a[0][0]…` 10001/100000→**2**（`--json` col=1）。✅ (7/7)
5. `;;`×`--json`：stdout **恰 1 行** `{"ok":true}`(12B)，`;;`→stderr；**非 json 回归** `;;` 仍在 stdout(8B)；交织顺序 `A/z/B`→stderr。✅ (3/3)
6. 越界写 span：`a[5]=9` 插入符 **col 1**（基座）、`--json` col=1；读 `print(a[5])` col=7（同基座）；口径一致。✅ (4/4)
7. `syntax.md` §9.2 样例 B 逐字实跑 → exit 0，输出 `the/fox/quick`，与订正后期望块 **逐字节一致**。✅
8. 回归：`cargo test` **482/0/0**（lib 379 + main 48 + cli 28 + test_runner 12 + unit 15）；`lfz test` **90/90 exit 0**；`cargo build --all-targets`（clean 后重编）**0 warning**。✅

## 顺带裁定
- **`bug-20260927-02` → 不成立（不可复现）→ 关闭**：`lfz test --json` 与 `lfz --json test` 的 stdout 均**单个 JSON**（1 行 9098 B 可解析），程序输出落 stderr。P9“25 行”记录过时，C 域实测正确。

## 观察项（非阻塞，待 team-lead 收口）
- **OBS-T11-R1（低）**：`lfz test` 实际 **90** 用例（任务书期望 89 为陈旧；test-engineer STATUS 明载 87→90 = +2 负例 +1 正例文件）。`TEAM_BOARD/PROJECT_STATE/README` 仍记 85/87 → 由 **T11-10** 刷新。
- **OBS-T11-R2（低，性能）**：`str()` 遍历 1,000,000 层嵌套数组耗时 ~167 s（300000 层 ~11 s），exit 0 不崩溃；疑似显示/环检测路径超线性。仅建议，非本次范围。

## 进行中 / 阻塞
- （无）。本波无阻塞。

## 下一步计划
- 若团队启动 **T11-05 v1.1 语言补强**，verifier 可在实现后做定向复验。
- 等 **T11-10** 重建 `dist/lfz.exe` 后，可复跑本报告 §②（对 `dist` 产物做一次等价性抽查）。
- `Temp/t11v/`（**非本会话创建**）仍在；本会话仅自清了 `Temp/T11/`。

## 关键经验（写给未来的自己）
- **产物同源是复验前提**：先声明 `target/release/lfz.exe` 的字节数/mtime/SHA256 + “二跑 0 编译”证明，再下结论；绝不复用 `dist\lfz.exe`（修复前）。
- **`lfz test` 需在 repo 根运行**：在 `Temp/` 下运行会因找不到 `tests/` 而报 `未发现测试目录 'tests'`（exit 2）。
- **`--json` 位置不限**：`test --json` 与 `--json test` 等价（help 明示）；裁定 bug-02 时必须两种都跑，避免位置差异误判。
- **深左链用大 N 验证收口**：`1+1+…`/`a[0][0]…` 必须测到 100000，才能确认从 `-1073741571` 变为受控 exit 2；边界（9999/10000）单测。
- **PowerShell 5.1 控制台打印中文乱码**：用 `Start-Process -RedirectStandardOutput` 落盘 + `ReadAllText(UTF8)` 读，绕开控制台码页；**不要**用自定义函数包装含 `-` 的参数。
- **超大重复行会撑爆证据文件**：落盘前用正则折叠 `(.)\1{79,}`，1.7 MB → 13 KB。
