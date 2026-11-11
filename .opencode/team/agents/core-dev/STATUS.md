# core-dev — 工作状态
> 最后更新：2026-11-04 by core-dev

## 当前状态
T20 core实现与只读review完成并停源，等待tooling统一release。未编译、未运行新unit，不声称验收PASS。

## 产出
src/lexer.rs内部typed EOF+前缀token；src/parser.rs公开ReplInputStatus/probe_repl_input沿原loader/parser，私有EOF expectation；src/error.rs新增ReplModuleUnsupported与ReplCapacity。保旧接口/消息/13类。docs/reports/T20-core-review.md记风险与范围。

## 证据
四组probe及一组error新unit已落盘；真实LF缺操作数和裸字符串仍Error，先前坏token不能被尾未闭注释遮蔽。diff --check通过（未编译）。SHA lexer429FA13206E6FE85956ABE494681DF42E0468698CF8CBC5FCCFCCE6468E92FA2；parserB3AD0318F0175514007CA73B33887A2CF3123B2E036C713DAB282254D416E2B8；error6081094B0C8E23E2F362D07E7401CE04ECEBCBC359F6526B7FEB16D01271C44F。

## 下一步/边界
tooling等runtime停源后唯一统一编译；发现本域问题由root唤醒修复。无其它owner源/Git写入/外装/网络；仅T20不automation，R304OPEN。
