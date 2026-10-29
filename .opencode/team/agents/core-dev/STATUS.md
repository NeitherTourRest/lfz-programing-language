# core-dev — 工作状态
> 最后更新：2026-10-29 by core-dev

## 当前状态
T16-A来源元数据基础完成，定向release6/6通过；已通知team-lead runtime可接续编译，本角色停止编译。

## 产出
src/source.rs、src/lib.rs导出、docs/reports/T16-A-core.md及证据目录。SourceId/ModuleId分域、SourceRecord可选Loaded、SourceMap安全索引/checked插入、SourceMetadataError、LocatedFailure按spec§19实现。

## 证据
旧rlib外部消费见证rustc E0432/exit1；新增source::tests定向release6 passed/419 filtered，exit0。原始red/green及manifest保留，不冒称外部见证已在新rlib重新链接。

## 下一步/阻塞
runtime接续，随后全量与独立验收。无core阻塞；本批没有模块加载/文法/CLI功能，R304OPEN。

## 关键经验
保留Span/Loaded/TraceFrame/LzError/Program旧shape，来源归外层记录；加载错误可无源码文本。编号上限使用private helper合成，不真实分配。所有临时项目Temp，只release；不Git/改他人产物/调度。
