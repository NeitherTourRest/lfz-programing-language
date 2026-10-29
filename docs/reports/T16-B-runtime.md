# T16-B 旧求值入口模块执行预检（源码交接）

日期：2026-10-29。责任人runtime-dev。状态：源码准备完成，尚未编译；不是产品验收结果。

依据三spec §20，旧eval_module*与run都汇合eval_module_on_thread_with_args。本批在Interp::new及任何exec之前调用module_declaration_span，根有ImportDecl或ExportDecl便返回SyntaxError“模块执行尚未启用”，错误与唯一module帧均用首次直接声明span，func_names=[None]、origins=None。未调用图构建、文件加载或初始化，不走到声明才报错。exec_stmt仍对两新叶变体防御性拒绝；parser负责禁止源程序嵌套声明，宿主非法手工AST不扩大旧保证。

仅修改src/evaluator.rs，不改value、globals/session、core、CLI、规范或旧测试。新增3项unit：六个旧eval/run入口精准消息与单module帧；late export之前的writeFile哨兵原字节不变以及早前未知名不求值；首次import声明定位且不读取缺依赖文件，旧export标识符表达式仍求值。测试临时文件在Temp/T16-B-runtime。

已执行git diff --check，通过。未编译/运行新增测试，结果必须等core在所有owner适配后统一release执行；不能把源码检查当作行为通过。无dist/profile/Git变更。C的模块Session及运行语义尚未实施，等待独立冻结；R304OPEN，无性能或正式发布声明。
