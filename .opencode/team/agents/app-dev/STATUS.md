# app-dev — 工作状态
> 最后更新：2026-10-30 by app-dev

## 当前状态
T16四模块jsonbatch作者验收33/33通过，锁C最终cdedc9/902656B；独立验收由verifier负责。T15应用38/38及全部旧应用保持不变。

## 产出
app/jsonbatch_modules/{common,record,batch,main}.lfz、verify_jsonbatch_modules.py、T16-JSONBATCH-MODULES.md、t16-evidence/.gitattributes。
Python AST PASS。菱形共享计数、caller同名全局不串、两次执行cache隔离、坏JSON恢复/行号/错误数据及输入输出保留都有oracle。

## 下一步/阻塞
默认sandbox首个check因模块访问权限IOError失败已保全；同一产物获许可提权原33进程全过，结果/manifest/报告已落盘。没有编译/Git/改产品，所有Temp项目内，D约105MiB只写小文件；无应用阻塞，可交独立verifier。

## 经验
模块路径与IO路径基准不同；attempt记录仍无file；共享state.calls须等于总记录数。应用非流式、非原子，保持字面路径检测限制。
