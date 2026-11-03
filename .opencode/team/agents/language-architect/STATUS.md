# language-architect — 工作状态
> 最后更新：2026-11-03 by language-architect

## 当前状态
正式冻结三spec §25及T19-contract.md9931B，ADR追加，root可GO三owner。T19-contract-proposal为历史。未源改/编译/Git。

## 精确接口
forEachLine(callback,maxLineBytes,path)，callback(line,lineNo)严格bool，返回lines/stopped，76内置。data-last、无新能力/文法/Value/CLI；原readFile不改。
core唯一error.rs五ValueMsg：LineLimitRange/LinePathNul/LineTooLong{line:i64,limit:usize}/LineUtf8{line:i64}/LineNumberOverflow，固定消息§25.3；Io调用Span Some且固定“无法逐行读取：{path}”。
tooling唯一line_reader/lib注册：LineControl deadline/token，ReadLine number/text，LineReadError纯owned；open(path,max,&control)/next_line(&mut self,&control)/finish(self)->()。
runtime唯一evaluator/builtins：完整类型/arity预检再open，mapping及call_value原home/source，共用session预算，native只cancel/deadline不扣步。

## 行/关闭边界
1..16MiB LF前rawbytes含CR；CRLF仅去一CR，EOF末CR/BOM保留、空/尾LF口径已冻。有界8KiB块，callbackfalse不处理后坏行，UTF8/超长回调前ValueError，JSONL显式callback内attempt。
普通file预metadata+opened核验不是TOCTOU/阻塞沙箱。RAII所有路径关闭，finish只是消费drop不报告close错误，不sync_all/FFI/filediag。root已批准拒runtime额外close诊断建议，ADR清楚。
关闭证据要求Windows share_mode0排他重开+handle plateau/Drop spy；三档规模峰值/吞吐独立oracle必须实测。

## 证据/下一步
文档diff检查exit0，仅换行提示。无产品/性能完成声明，原用户28/1/1保护，R304OPEN。root给GO实现后独立验收，仅当前轮交付收口，不T20/自动化。
