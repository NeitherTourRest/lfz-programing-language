# T17 CLI作者阶段与最终交付记录

当前最终结果见末节；以下保留F487阶段及串行交接历史。

日期2026-10-30。tooling-dev。新增两token限额scanner、独立受限执行流水线及13类runner注册。默认unlimited保留原C流水线/runner，不在check/test/help/version隐式加限额。CODE紧邻保护、--后argv原样；ASCII u64/重复/缺值/溢出精确usage，先值错误后适用性。deadline仅在根前端/静态图成功后以checked_add创建，溢出为usage String，不借HostFailure伪造业务错误。

新tests/cli_t17.rs九组：2/4步及0/前导0/u64最大、三来源/两种run拼写/裸入口、任意选项顺序、CODE/argv保护、严格值与重复缺值、非执行拒绝、timeout0及前端错误优先、attempt穿透/已发生输出、图依赖真实来源、按实际平台判断checked_add大值分支。第13类runner注册单测保留原12完整名单。没有新增Ctrl-C handler或本地native内部poll；不声称硬打断阻塞IO。

旧C固定产物不支持两flag的red.json留存。初次默认定向6/8：一个真实接入缺陷是check缺来源优先于限额适用性，已修；另一为已知canonicalize沙箱权限。首次日志保留。修复后完整no-run release重建，再提权运行相同十个harness620/620=467lib+50bin+28cli+12T12+12T14+7B+5C+9T17+15runner+15unit，0failed/ignored，build exit0。该阶段产物942592B/hashf4876fc2850869a2d9d8e511eb7510c4ee59ccde2ea0896cd891220da06bf4c4。

随后architect精确补钉等号形式文案，root要求暂不再编译，等待runtime补实际并发token/HOF/diamond/模板用例后合一次。源码仅改等号族usage为“选项 '{flag}' 不支持等号写法，请使用 '{flag} N'”及对应新断言。完整prefix=族识别不扩大到flagX，CODE/argv保护不变。该最新文案尚未重编译验收；620只属于F487阶段，不能当成最终新源成绩。

docs/tooling/evidence/T17保全原始日志/manifest和f487-cli-snapshot.zip（校验manifest hash后保存cli、runner、新CLI测试和精确exe四成员）。本角色未覆盖dist/Git，runtime/core/spec源未由本角色写。下一步root/runtime交接后本角色唯一最终完整编译；R304仍OPEN。

## 最终编译与全量作者验收（2026-10-30 13:30）

root轻量GO后先确认无其它cargo/rustc，evaluator原始SHA a673cca7cf009ba19ac9897737aa6f75db2dc24941ef31112fe6728c5b2e551d，合并runtime仅追加四项unit及CLI等号精确文案，执行唯一最终完整 `cargo test --release --no-run`。随后在许可canonical模块身份读取环境运行本次新编译十harness：624/624=471lib+50bin+28cli+12T12+12T14+7B+5C+9T17+15runner+15unit，0failed/ignored。runtime新增跨线程mpsc握手取消、七HOF、diamond11/10共享预算、模板默认故障来源均在471lib中通过；此轮没有由tooling越权改runtime源。最终CLI九组包含精确等号文案断言，旧行为测试原样。

`cargo build --release` exit0；最终target/release/lfz.exe943104B，SHA256 `1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45`。evaluator hash编译后仍与交接相同。源/Cargo/cli_t17全部SHA及最终日志SHA在docs/tooling/evidence/T17/final-manifest.json。最终日志名为final2-*.exe.txt，不复用F487的620阶段计数或覆盖其manifest/精确快照。final-compile.txt和final-build.txt记录命令阶段，环境路径canonicalize warning保留。

等号flag族精确报“选项 '{flag}' 不支持等号写法，请使用 '{flag} N'”，不算一次赋值或重复；匹配包含空等号值但不扩大到flagX，CODE和argv仍保护。unchecked平台时间范围没有硬编码猜测，checked_add失败通过Result的usage分支，前端/图失败优先。

本角色最终收工后停止改源/编译/测试。仅写CLI、授权runner注册、新测试和本人文档/证据；未覆盖dist/Git/profile、未改core/runtime/spec/旧测试。公开121与独立验收由root接续；624作者全绿不关闭R304或代替独立性能门禁。
