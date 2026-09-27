//! `tests/unit/` 单元测试入口（core-dev 专属）。
//!
//! Cargo 将 `tests/unit/main.rs` 视为名为 `unit` 的集成测试目标；同目录其余文件为子模块。
//! 本目录测试解释器**前端**（lexer / parser / ast / error）的单元行为；
//! `tests/lfz/**` + `cases.json` + `coverage-matrix.md` + `REPORT.md` 为 test-engineer 的
//! **黑盒**产物，归其所有，本目录不触碰。

mod ast_depth;
mod nesting_depth;
