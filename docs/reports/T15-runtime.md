# T15 attempt 作者实现与验证

日期：2026-10-29。依据三spec §18。实现仅src/evaluator.rs/builtins.rs；73内置名，attempt为Interp特殊内置，不添加通用Builtin/Hof伪实现，不变更Invoke ABI或旧内置的一等值范围。

## 行为

变量遮蔽优先，实参仅求值一次。attempt单参/函数/零形参校验在恢复边界之外；argv可作为既有一等零参数函数，placeholder拒绝。调用前保存完整trace及depth。成功/可捕获失败形成ok/value/error新struct后恢复完整快照；失败记录在恢复前取完整trace，帧只含func/line/col，无虚构attempt帧和file。原成功值及共享拓扑不深复制。

可恢复分类严格按enum匹配业务类和IntegerOutOfRange；Recursion、Capacity、Syntax、Cosmos及宿主panic不捕获，不先清错误帧。func_names/next_func_id、cell/容器/IO/随机等副作用不回滚。没有catch_unwind。错误class是普通字段，但class是保留字，LFZ访问使用r.error["class"]。

## 测试与证据

新增7个unit：成功nil/JSON及恢复；15000连续失败/嵌套/恢复后递归和后续未捕获帧；共享alias/失败前逃逸闭包；外层校验/遮蔽/argv；全部9业务类及JSON数据；非捕获4类和合成Recursion传播；成功/失败完整caller span与depth恢复。

真实缺功能红测red-missing-feature.txt为NameError attempt；先前red.txt是作者误用.class的语法错误，不冒充产品红测。green-initial为第一例绿。green-all中6/7另一个作者语法失误是lambda的=>{}.missing被按块解析，改新增例为=>({}).missing。两份失败日志保留，未改parser或现有语义。

完整cargo test --release：549/549（419+48+28+12+12+15+15），0失败/忽略；cargo build --release成功。target/release/lfz.exe782848B，SHA256 1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c。原始日志及source/Cargo/产物指纹见T15-runtime-evidence/manifest.json。

## 唯一授权旧测试同步

旧lookup_table_names_consistency锁72，已先报告并获得team-lead明确授权：BUILTIN_NAMES.len和dedup由72改73，table_names追加唯一特殊名attempt后仍严格全量比较。原TABLE+HOF总72及HOF7断言保留，未删除任何旧行为检查。非测试生产变更只新特殊路由/helper及名字登记，不含之前被拒性能优化。

## 限制与交接

未改spec/CLI/黑盒/旧v2、Cargo/profile、dist或Git。只用项目工具链--release，Temp项目内，磁盘受限不新建target。生产源和候选已冻结，停止编译；下游独立公开验收/应用/指南/发布复制交team-lead调度。R304仍OPEN；功能通过不代表效率通过或正式发布。本轮未计时、未测RSS。
