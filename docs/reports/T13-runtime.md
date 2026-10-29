# T13 运行时分配优化与作者功能证据

日期：2026-10-27。基于已推送候选 `cbc974b`。本轮保留72内置、现有语义和原19项性能门槛；此报告只描述源码机制及作者功能结果，性能判断交由 perf-engineer 的冻结候选测量与 verifier 独立审查。

## 两处改动

1. 高阶调用之前每次 `call_value` 都把参数切片 `to_vec()`，再把该临时向量交给 `call_user`。现在 `call_user` 接收精确长度的标准值迭代器：普通调用移交已求值参数的 `Vec::into_iter()`；高阶调用使用切片的 `iter().cloned()`，绑定每个参数时才克隆 `Value`。非空回调不再需要该临时参数向量。普通调用不增加参数克隆。此结论由源码路径可直接确认，不冒称测得峰值内存变化。
2. `join` 之前克隆整个输入数组的 `Vec<Value>`，随后构造 `Vec<&str>` 并拼接。现在在无回调、无求值、无写入的同步函数内持有输入数组的只读借用，直接左到右验证并构造 `Vec<&str>`；仍保留最终拼接分配。省去的仅是整个 `Value` 向量快照及其逐元素引用计数增减。

`call_user` 仍在消费迭代器之前按原顺序检查 argv特殊内置、用户体、参数数量、递归深度。每次仍创建新 Env、按声明序绑定可变参数、保留独立可捕获cell；self覆盖/捕获、trace调用点、错误时保帧及成功退深度均未改。只调用 `into_iter()` 或 `iter().cloned()`，不引入有副作用的自定义迭代器。

`join` 仍先验证sep、再验证array、再报告最左非string元素，span和错误文案沿用原路径。`bad_operands` 只读 `type_name`，即使元素是容器自身也不重借数组。输出仍是新不可变字符串。高阶内置的源数组快照完全保留，回调修改输入容器时仍遍历入口快照。

## 新增测试及验证

先新增七个语义回归测试，在旧实现上实跑 **7/7**，随后进行优化；另外新增join自环错误路径用例，候选共 **8/8**。这些语义本应旧新一致，不把作者测试编译失误或本来通过的回归用例称为产品红测。旧有测试未删除或修改。

新增覆盖：每次回调参数重新赋值及闭包独立cell；数组/结构体引用语义和map入口快照、左到右顺序；captured self、参数遮蔽及直接调用；回调参数数量精确span且未加帧；回调错误保留trace；join Unicode/空项/源alias；join校验顺序与精确span；join非string自环不重借。

| 命令 | 作者结果 | 原始证据 |
| -- | -- | -- |
| `cargo test --release t13_ --lib`（旧实现，七用例） | 7 passed，0 failed | `T13-runtime-evidence/baseline-regressions.txt` |
| `cargo test --release t13_ --lib`（新实现，八用例） | 8 passed，0 failed | `T13-runtime-evidence/candidate-regressions.txt` |
| `cargo test --release` | **530/530**，0 failed/ignored；412+48+28+12+15+15 | `T13-runtime-evidence/cargo-test-release.txt` |
| `cargo build --release` | exit0，最终no-op确认正式二进制已是当前源码 | `T13-runtime-evidence/build-release.txt` |

构建沿用项目内 Rust1.90、零第三方依赖、现有 Cargo release opt-level=2/LTO/codegen1/strip；没有新增target目录或更改profile。toolchain在项目Temp，TEMP/TMP也定向项目Temp。构建日志包含环境路径canonicalize警告，不是产品编译错误。

最终作者候选 `target/release/lfz.exe`：**761344B**，SHA256 **9cda0c6fd5d85178335e8b758a843fc5f66574826bab4a80f2418a318e171cf9**。旧 `dist/lfz.exe` 47ED/759808B保持，交由调度者在最终验收后决定交付替换。证据manifest记录源码/Cargo/二进制及日志hash；证据目录按原字节入库。

## 限制与交接

- 本轮未变更任何语言规范、AST接口、旧测试、旧基线、Cargo版本/profile、dist或Git历史。
- `string_ops` 仍包含按A1语义返回新数组的反复push，其累计复制成本未在此轮消除；不能未经证明改成原地突变。
- 泛型实例化和二进制布局变化可能影响启动及其它路径；包体相对旧候选增加1536B。源码上少分配不等于所有负载实测更快，正式性能结果待双基线冻结测量。
- 作者完成后不再并发构建/测试，不干扰perf计时。R-304是否关闭需完整原19线及独立结论，不能用此处530功能通过替代。

## [2026-10-27 19:51] 实测拒收与生产实现撤回

上述9CDA候选及530功能通过是完整保留的历史作者阶段结果。正式同期性能结果未形成验收改善证据：map三档相对T12慢约6.5%–16.2%，small4约30.4%，string8k约13.7%；这些是同期测量观察，不声称已定位LLVM等微观原因。完整测量、原19线及限制以 `T13-performance.md` 为准。源码少分配没有转化为本轮可接收的性能结果，team-lead决定拒收本候选，不继续择窗重测同一候选。

在verifier核对冻结源码测前/测后/发布指纹一致、team-lead明确GO之后，已将 `src/evaluator.rs` 和 `src/builtins.rs` 的生产实现准确恢复至 `cbc974b631d225e34a746ad96410f02b629b3c37`。每个文件的整个原始前缀与该Git对象逐字节相同，仅保留末尾新增测试模块（7+1共8项），没有删改任何旧测试。

`T13-runtime-evidence/retraction-source-check.json` 记录两文件生产前缀原/新SHA相同，以及保留8项新增测试的静态核对。原manifest、候选日志与所有性能失败证据未回写；9CDA冻结源码/二进制档案由发布经理保留。因此旧manifest仍绑定被拒候选，不能用来证明撤回后源码或后续T14产物。

本角色未在撤回后构建或测试；按调度安排由tooling在T14统一release构建/全量验证（应包含保留8项测试），避免并发与重复编译。当前target旧候选exe不被当作已撤回源码的产物，dist47ED仍未改；R-304保持开放，正式发布未批准。后续T14基于恢复后的T12运行时推进，不把这次失败候选冒充已应用优化。
