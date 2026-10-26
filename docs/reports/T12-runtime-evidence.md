# T12 运行时交付证据

> 2026-10-25 00:57，runtime-dev；依据interface-contract §16冻结契约。独立验收另由verifier执行。

## 实现
- `src/json_codec.rs`：独立严格JSON解析/编码，字节游标单遍解析、精确i64/有限f64、Unicode代理对与固定字节偏移错误；BTreeSet检查解码重复键，一次构造array/struct；128容器深度、路径环检测、UTF-8键排序及方法数据面过滤。
- `src/evaluator.rs`：两条with_args入口，旧入口默认空参数；Interp持有参数快照与单一argv函数值。直接/别名/闭包/高阶调用共享派发，变量优先遮蔽，内置不新增用户帧。
- `src/value.rs`：Closure增加可选builtin标签，仅用于argv；所有现有构造器补None，外部没有结构体字面构造需迁移。StructObj字段索引保留raw_fields/first-match/原位更新/插入序/clone与Debug，不暴露随机HashMap迭代。
- `src/builtins.rs`：三内置与72项一致性断言；无上下文直接argv调用返回新空数组。
- `src/error.rs`：经team-lead专项唯一写者授权，仅新增JSON两ValueMsg变体与精确消息。`src/lib.rs`导出codec。未动bin历史json.rs、CLI、spec、原黑盒、冻结测评或指南。

## 可复现命令与结果
项目内Rust 1.90工具链，RUSTUP_HOME=`Temp/toolchain/rustup`、CARGO_HOME=`Temp/toolchain/cargo`、TEMP/TMP=`Temp/toolchain`。

- `Temp/toolchain/cargo/bin/cargo.exe test --lib`：初版388既有测试通过，日志[first-build.txt](T12-runtime-evidence/first-build.txt)。
- 新增8组JSON边界单测首次8/8通过，[json-tests.txt](T12-runtime-evidence/json-tests.txt)。
- argv/索引首次2/3通过，失败为Rust期望数组字符串漏引号，产品实现未因此修改；保留[first-argv-index-tests.txt](T12-runtime-evidence/first-argv-index-tests.txt)。订正新测试的显示期望后全量lib **399 passed / 0 failed / 0 ignored**，[lib-tests.txt](T12-runtime-evidence/lib-tests.txt)。
- 新增11项测试覆盖strict语法/i64边界/浮点负零及次正规数往返/非法Unicode代理对/NUL与任意字符串键/解码重复键/128与129容器/共享和环/方法过滤与失败优先级/arity与span/argv一等身份与遮蔽隔离/索引clone与2000动态键。

工具链有路径canonicalize与增量缓存hard-link失败后复制警告，不能声称零警告。以上是lib定向验证，不替代CLI/黑盒/性能/独立验收；未观察到abort，不执行故意崩溃压测。
