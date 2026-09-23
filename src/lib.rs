//! LFZ 解释器库。
//!
//! P3.0：仅模块骨架，无任何业务逻辑；各模块类型/函数在后续子阶段填充。
//! 契约依据：`docs/spec/interface-contract.md` §10（模块与类型契约）。

pub mod span; // Span{line,col}（P3.1）
pub mod error; // LfzError 12 变体 + R<T>（P3.1）
pub mod loader; // 加载器（P3.2）
pub mod lexer; // 词法分析（P3.3）
pub mod ast; // AST 节点（P3.4）
pub mod parser; // 语法分析（P3.4 / P3.5）
pub mod value; // 运行时值模型（P3.6）
pub mod env; // 作用域 / 闭包捕获（P3.6）
pub mod evaluator; // 树遍历求值器（P3.7 / P3.8）
pub mod builtins; // 内置函数表（P3.9）
