# tooling-dev — 工作状态
> 最后更新：2026-10-25 00:57 by tooling-dev

## 当前状态
T12-05 CLI 实现和定向测试完成，待team-lead全量与verifier独立验收。未改runtime/spec/冻结v2或原有测试，未Git。

## 产出与证据
- src/cli.rs：保护-e CODE和--后script参数的选项扫描；保留旧无--额外参数错误。支持lfz [run] -e CODE、lfz [run] -、显式/裸文件--参数。
- 文件/源码统一load→lex→parse→eval与AST析构在同一大栈；调用eval_module_traced_on_thread_with_args，runner eval_case仍默认[]。
- stdin读UTF8到EOF；失败IOError无法读取标准输入，无span；伪路径<command>/<stdin>，line_base0；状态JSON、print/dump/input通道规则不变。
- tests/cli_t12.rs新增12集成测试；旧tests/cli.rs 28/28与bin原48/48均保持原样通过。
- 命令：cargo test --test cli --test cli_t12（40/40，exit0）、cargo test --bin lfz（48/48，exit0）。环境RUSTUP_HOME/CARGO_HOME/TEMP/TMP全指项目Temp/toolchain。
- 永久日志docs/tooling/evidence/T12/{cli-tests,bin-tests}.txt，原始Temp/T12/tooling/同名副本。存在环境路径/增量hard-link警告，不称零warning。
- docs/tooling/runner-contract.md §10同步新CLI，§9旧全局摘除描述更新；src/main.rs说明同步。
- 字节：src/cli.rs 45335，main.rs 779，cli_t12.rs 6066，runner-contract.md 16860。

## 进行中/阻塞
无；等待全量与独立验收。runtime同时开发，计数可能继续增加，以最终执行证据为准。

## 下一步
team-lead协调黑盒/指南/应用/最终构建；需要缺陷修复再唤醒。

## 关键经验
-e紧随token必须无条件源码，不能先全局剥除--json；--只在完整执行入口后转为脚本参数。旧parse形态保留以免改动原测试；test/help/version原解析兼容。