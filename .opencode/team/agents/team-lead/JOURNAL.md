# team-lead — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-27 23:50] 阶段收尾：T11-③ 第 1 波完成（skill D1–D7 ‖ 人类文档 ‖ v1.1 七项 ADR）
- 来源: 用户「阶段性结束」（本阶段 = T11-① 审计 → ② 修复 → 发布 v1.2.1 → ③ v1.1 迭代第 1 波）
- 完成:
  - ai-dx-engineer: skill 的 D1–D7 修订（**每条 lfz 实跑验证**）→ SKILL +164 / VERIFICATION +157 行
  - docs-writer: 人类向文档同步（零帧无 Traceback / float `.0` 仅定点 / 3 条新细分消息）→ docs/guide 5 文件
  - language-architect: v1.1 七项 ADR + spec（**零文法改动**）→ 新增 15 内置（54→69）、range 1/2 参、6 条新消息、**错误类仍 12**、兼容性核对表全 ✅
- 关键订正（跨角色 / 实测推翻审计建议）: `FEATURE-AUDIT §7.1` 的「`push`+`join` 替代字符串拼接」**错** —— `push` 遵 A1 每次整体克隆，实测 **也是 O(n²)**（320k=4076ms，比 `+` 的 1582ms 更慢）；真 O(n) = `range(n) |> map |> join`（104ms）或**预分配+下标写**（151ms）
- 产出: 上述文件 + `docs/reports/T11-03-docs-evidence.md`；`DECISIONS.md` [23:40]（ai-dx）+ [22:50]（architect 7 项 ADR）
- 决策: ① **v1.1 #4 文件 IO 需用户追认**（唯一一条标"是"：新外部副作用面 + 环境依赖）→ 追认前不实现；② 可选追认两项：`range(1,4)` 由报错变合法（行为面扩大）、`log(0)`→-Inf / `log(负)`→NaN（IEEE 非报错）
- 下一步（下阶段，待用户发话）: T11-11 提交推送 → T11-12 实现七项 → T11-13/14 黑盒+文档 → T11-15 verifier 复验 + **同题 8 盲测重跑（基线 2/8）** → T11-16 刷新 REQUIREMENTS
- 阻塞: v1.1 #4 文件 IO 待追认；`docs/spec` 领先于实现（正常中间态）
## [2026-09-27 22:40] T11-② 修复批次收口：4 类崩溃 + JSON 缺陷 + span 全修，6 条 ADR
- 来源: 用户批准「全修 + 复验 + 再迭代」（基于 T11-① 条款级审计 157 条款 / 5 FAIL）
- 完成: 4 波派发（架构师 ×3、runtime-dev、tooling-dev、test-engineer、core-dev ×2）
  - 6 条 ADR：`range`/`repeat` 统一 `OverflowError` · §9.2 样例订正 · `float` 显示「最短往返优先」 · 零帧错误无 Traceback · **`PARSE_DEPTH_LIMIT=1000`** · **`AST_DEPTH_LIMIT=10000`（口径正交）**
  - runtime-dev：容量溢出 → `OverflowError`；格式说明符上限；写路径 span 对齐
  - tooling-dev：`;;`×`--json` dump 随 print 转 stderr；`--help` 文案
  - core-dev：解析嵌套上限 + `on_eval_stack` 256 MiB 同栈；AST 深度上限
  - test-engineer：黑盒 +2（溢出负例）
- 产出/证据（team-lead 亲测，非听汇报）: `repeat`/`range` 溢出 → exit 2 + `OverflowError`（原 panic 101）；`(`×1000/1001 → 0/2；`1+1+…` 9999→0 / 10000→2 / **100000→2**（原 -1073741571）；`--json`（含 `;;`）stdout **恰 1 行**；`a[5]=9` 插入符 **col 1**；`cargo test` **445→482/0/0**；`lfz test` **85→87**
- 决策: 新增 2 条用户可见消息模板（`容量溢出` / `表达式嵌套过深`）+ 2 个常量（1000 / 10000）；**错误类仍 12**（无 13）；两上限**口径正交、缺一不可**（架构师量化：≥14× / ≈9× 安全系数，实测帧 18.3/2.93/0.27 KiB）
- 事件: ① core-dev 踩 PowerShell `Set-Content` 无 BOM 毁 UTF-8 中文源码 → 自行 `git checkout` 恢复并改用 `edit`/`write`（已转团队纪律）；② 用户要求临时件迁出 C 盘 → 立 ADR + `.gitignore Temp/`，并首清 C 盘 5.31 MB→1.0 MB、保全盲测证据 28 件进 `docs/reports/blind-test/`
- 下一步: T11-08 黑盒负例 → **T11-09 verifier 全量复验** → **T11-10 release-manager 重建 dist + 刷新 README（445→482）+ 提交批次**
- 阻塞: `bug-20260927-02`（P9 记录 vs C 域实测矛盾）待复验裁定
## [2026-09-27] T11 启动：特性审计 + skill 盲测 + 三路规范符合性审计（P0–P10 已全交付 v1.2.0）
- 来源: 用户指令「完善语言特性，我们现在还有很多特性没有实现，你都检查一下。注意，我们的特性是要服务于agent，所以特性的设计需要满足'便于agent书写'以及性能考虑」→ 我主张「先审计再定」→ 用户复核路线为「**先确定现有计划完全实现无误，然后开始迭代**」
- 完成（本轮）:
  1. **特性缺口审计**（language-architect，`bg_71bb41a2`）交付 `.opencode/team/FEATURE-AUDIT.md`（39238 B / 407 行）：现状盘点 + 12 张候选卡 C1–C12 + **明确 OUT 18 项** + 6 焦点裁定 + v1.1 **IN 7 项** + **P0 文档 7 条**
  2. **skill 盲测**：8 题零上下文 agent（隔离工作区，只给 skill + `lfz.exe`）→ **通过 2/8**（maze_bfs/knapsack_dp）；无偷看证据；报告 9 条 skill 缺口
  3. **三路规范符合性审计**并行派发（verifier）：A 词法文法 `bg_b5cf84dc` / B 语义错误模型 `bg_2e381353` / C 内置 loader runner CLI `bg_593a9150`
  4. **看板/状态对齐**：`TEAM_BOARD.md` 与 `PROJECT_STATE.md` 此前滞后至 09-23（P3 阶段），已按真实进度（v1.2.0，8 项交付物齐备）重写
- 产出: `FEATURE-AUDIT.md`；`TEAM_BOARD.md`（新增 T11 进行中/待办 + 质量基线表）；`PROJECT_STATE.md`（表 3/4 全 ✅ + T11 阶段）；本 STATUS
- 关键裁定（审计）:
  - **盲测 2/8 的根因主要在文档(skill)，不在语言**：7 条暴露项中 5 条是纯 skill 缺口（`range` 无签名、`let` 循环语义、`<` 对齐、`len(string)`、隐性语法未展示），仅 2 条涉语言 → **P0 全在文档侧，语言侧最高 P1**
  - **不加 `s[i]`**：UTF-8 不可变下 `s[i]`≡`chars().nth(i)` 是 O(i)，循环遍历即 **O(n²)**；正解是文档化 `split("", s)`（O(n)）
  - **架构级性能发现 §7.1**：string 不可变 → **循环内 `s = s + c` 拼接是 O(n²)**（盲测 json_mini/expr_eval 即如此）→ doc 引导 `push`+`join`；runtime 可选「`Rc` 强计数为 1 时就地追加」（不改语义/A1）
  - v1.1 **IN 7 项**（全为内置新增/签名扩展 + 1 条 spec 补钉，A1–A7 与 §4.5 全兼容）；**OUT 18 项**（含 `try/catch`、标签 break、`match`、生成器、`..`、`in`、Unicode 折叠、`hint`（永久）、动态宽度、模块、类、类型注解、字典类型、可选链…）
- 事件/发现: **盲测隔离出现 1 处破口** —— 某盲测 agent 把 `examples/test.lfz`（103 B 探针）写进了**项目仓库**（而非隔离工作区），这解释了 `graph_dijkstra` 为何"无产出"；我**未擅自删除**，已记入状态待用户决定
- 决策: 依 **D-010**（重大决策先问用户）—— 审计结论与 IN/OUT 清单**先交用户拍板**，不擅自改冻结 spec；T11 第二步（P0 文档 + v1.1 七项）待审计缺陷单回来后一并请示
- 下一步: 收 3 份符合性审计 → 归并「spec 说了但实现没做/做错」缺陷单 → 交用户拍板 → 进入迭代（文档 ‖ 语言）→ 同题盲测重跑
- 阻塞: 无（等 3 个审计返回）
- 附带: `examples/life.lfz`（128 行 Game of Life，exit 0）为用户点单所写，**尚未提交**

## [2026-09-24 09:05] P3 收官：解释器端到端可用，里程碑 v0.2.0
- 来源: 用户「开工 + 全程版本管理」指令下的连续推进（P3 拆 20 子阶段）
- 完成: 派 core-dev/runtime-dev/tooling-dev/language-architect/verifier/release-manager 完成 P3 全部子阶段（P3.0–P3.10）；P3.11 独立验收三轮
- 产出: `loader`/`lexer`/`ast`/`parser`/`value`/`env`/`evaluator`/`builtins`(§10.7 54/54)/`cli`；`cargo test` **361 passed / 0 failed / 0 ignored**（+9+7=377）；`docs/reports/P3-verification.md`（含 rev.1/2/3）；标签 **`v0.2.0`**（→`931e2b2`）已推送
- 决策: ①parser 因原负责人 2 次未交付 → **改派 Sisyphus-Junior** 并切小批次（含看板记录）；②bug-07 为不篡改既有测试而**有意推迟**，待架构师 A9 裁定后再修；③验收 FAIL 阶段**一律不打标签**（标签只作"通过"的证据）
- 事件: 曾发生「2 修复任务同时超时 + 架构师 `Insufficient Balance`」→ 工作区留下 2 warnings + 3 failed 半成品；**我亲自接手收尾**（保留写对的部分、回退夹带的 bug-07），恢复 0 warning / 全绿
- 结果: **10/10 缺陷闭环**（3×🔴 + 4×🟡 + 2×🟢 + 1 规范侧）、**0 新增回归**；终验 **PASS**
- 下一步: P4 工具链 → P5 黑盒测试集（评分项 2）
- 阻塞: 无

## [2026-09-23 23:59] P0.5 完成：远程 GitHub 仓库上线
- 来源: 承接「开工 + GitHub 远程 + 全程自动版本管理」指令
- 完成: 用户完成 gh 设备码授权（检测到 `AUTH: OK`）→ 派 release-manager 执行 P0.5r：`gh repo create` + `remote add` + `push main` + `push tag v0.1.0` + README 写入仓库链接
- 产出: 远程仓库 **https://github.com/NeitherTourRest/lfz-programing-language**（Public，默认 `main`）；`git ls-remote` 显示 `main`=`35e5f62`、`refs/tags/v0.1.0`（附注→`e060b21`）；本地=远程=35e5f62；提交链 e060b21 → 6873fe7 → 35e5f62
- 决策: 交付物 7（Git 历史）状态更新为「✅ 已建立（本地+远程）」；P0.5 在看板移入已完成
- 关键发现: gh 登录账号 **login = `NeitherTourRest`**（display name = `MakeChase`），故仓库 owner 为 `NeitherTourRest`；gh 委托`登录态在 keyring`，子 agent 调用 gh 仍需注入代理变量
- 下一步: P1 需求基线同步 → P3 核心实现（core-dev + runtime-dev 并行）；release-manager 把本轮团队文档改动提交并推送
- 阻塞: 无

## [2026-09-23 23:45] 开工 · P0.5 版本基线（本地完成 + GitHub 远程待授权）
- 来源: 用户指令「先初始化git版本控制，然后新建github仓库，推送到远程。此后全程自动版本管理 + 实时更新README。协议用MIT」
- 完成: 派 release-manager 完成 P0.5 本地基线；我安装 gh CLI 2.101.0 并启动 GitHub 设备码登录；同步更新 PROJECT_STATE / TEAM_BOARD / 本状态
- 产出: `git init -b main`；初始提交 `e060b21`（67 文件）；附注标签 `v0.1.0`；`LICENSE`（MIT，2026 MakeChase）；`README.md`（MIT + 当前状态）；`.gitignore` 补 Rust；第二个原子提交 `6873fe7`（版本管理纪律 ADR + release-manager STATUS/JOURNAL）；`git status` 干净
- 决策: 仓库名/可见性 = `lfz-programing-language`（Public，用户定）；认证 = gh CLI 浏览器登录（用户定）；版本管理纪律（main / 里程碑原子提交 / conventional commits / 附注标签自 v0.1.0 / 禁 force-push）
- 关键发现: gh 不读 git 的 `http.proxy`，须注入 `HTTPS_PROXY`；直连 github.com:443 超时、api.github.com 可通
- 下一步: 待用户完成设备码授权（code `23AC-7984`）→ `gh repo create --public` + remote + push main/tag
- 阻塞: GitHub 授权（用户操作）；设备码约 15 分钟过期

## [2026-09-23 22:15] P2 语言设计冻结（docs/spec v1 三件套）
- 来源: 用户指令「审核好没有 / 确定可行性吗」+「继续」
- 完成: 收取 core-dev / runtime-dev 对 v0.5 的复审结论 → 判定冻结门禁达标 → 派 language-architect 补 3 处小项并冻结为 `docs/spec/` 三件套
- 产出: `docs/spec/{syntax,semantics,interface-contract}.md`（910/382/298 行，UTF-8 无 BOM，字节 60010/28153/26116）；`DRAFT-LFZ-v0.5.md` 已含 3 处补钉；`DECISIONS.md` 新增 D-016「spec v1 冻结」
- 决策: 门禁判定 = core-dev PASS（文法无回溯可实现）+ runtime-dev CONCERNS 3 项（非架构级、明示不阻塞）→ 达标；同步更新 PROJECT_STATE / TEAM_BOARD（P2 ✅、交付物 1 ✅）
- 下一步: 等待用户「开工」指令 → P0.5（release-manager `git init` + 初始提交）
- 阻塞: 无

## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/team-lead.md`
- 下一步: 等待 team-lead 调度
