# language-architect — 工作状态
> 最后更新：2026-10-27 19:35 by language-architect

## 当前状态
T13两项内部分配优化语义边界审查完成；T14非执行check命令已按批准草案定案到三spec §17。T13不改语言行为；T14契约已冻结但未实施新命令，不声称性能或产品通过。

## 产出与证据
- docs/reports/T13-design.md（6017B）：依据semantics §4.5.1–4.5.5/4.5.11、contract §10.3/10.7/16.6，审查call_user/call_value/eval_args/Env/高阶源快照/b_join。
- 许可私有call_user用ExactSizeIterator<Item=Value>，直接Vec::into_iter、高阶slice.iter().cloned，消除高阶临时参数Vec；仍保留fresh Env/形参cell/mutable/顺序/self/trace/原arity与recursion先后。
- 许可无回调join持数组只读Ref验证并拼接，省Value快照；检查顺序/首错/span/新字符串保持；不泛化有用户回调的源数组快照。
- DECISIONS仅追加2026-10-27 19:26 ADR；用户初始事故记录及既有条目未改。T13报告8类独立差分矩阵是待执行输入，不冒称实测。
- docs/reports/T14-check-design.md（冻结后9682B）：三来源、loader前导/UTF8/行偏移、CLI CODE选项保护、专属mode/check与executed/false JSON定案、大栈内AST析构、明确只语法非运行保证；12类验收矩阵及未纳入范围。
- T14三spec §17和19:33 ADR已定案：check --help/CODE外--拒绝；usage保持文本，合法check JSON三个成功键及既有失败字段。12项文档结构自查通过，未运行产品或性能测试、不Git、不改源码/已有测试/团队看板。

## 下一步 / 阻塞
- T13由runtime实现、verifier独立语义差分、perf按原19线实测；本设计许可不能代替证据。
- T14可由tooling按三spec §17实施，须等T13性能计时完全结束；当前候选9cda0c6不混新CLI改动。本角色完成后静默。
- 无设计阻塞。旧v2封存输入/manifest、T12历史结果及三处用户修改保留。

## 关键经验
临时载体消除不同于语言帧消除；Rc克隆可以延后，但用户求值顺序和逃逸cell不能改。无用户调用的只读区段可避免容器快照，有回调则不能持源Ref。check只跑前端仍须大栈与深AST同栈析构，成功不能被宣传成运行有效/零IO/安全沙箱。
