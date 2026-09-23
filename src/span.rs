//! 源码位置 `Span { line, col }`（1-based）—— 全项目唯一定义。
//!
//! 契约：`docs/spec/interface-contract.md` §10.2。
//! - `line` / `col` 均 **1-based**；`col` 按 **Unicode 标量值**计数（非字节偏移）。
//! - 这是 AST / 错误 / traceback / `--json` 统一使用的**唯一位置类型**（B10）：
//!   本项目**不使用字节偏移**表示位置。
//! - `line` 已是「本地行号 + `loader.line_base`」后的绝对行号（§10.1 / §10.2）。

use std::fmt;

/// 源码位置（1-based 行 / 列；列按 Unicode 标量计数）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Span {
    /// 1-based 行号（= 本地行号 + `loader.line_base`）。
    pub line: u32,
    /// 1-based 列号（按 Unicode 标量值计数）。
    pub col: u32,
}

impl Span {
    /// 第 1 行第 1 列 —— `CosmosAnswer` 变体的固定位置（§10.8 / B11）。
    pub const START: Span = Span { line: 1, col: 1 };

    /// 构造一个位置。
    #[must_use]
    pub const fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    /// 形如 `line 3, col 5`（与 `semantics.md` §8.2 的行列记法一致）。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, col {}", self.line, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn start_is_line1_col1() {
        assert_eq!(Span::START, Span { line: 1, col: 1 });
        assert_eq!(Span::new(1, 1), Span::START);
    }

    #[test]
    fn display_uses_line_col_form() {
        assert_eq!(Span::new(3, 5).to_string(), "line 3, col 5");
        assert_eq!(Span::new(1, 1).to_string(), "line 1, col 1");
    }

    #[test]
    fn derives_copy_eq_hash() {
        let a = Span::new(2, 7);
        let b = a; // Copy：a 仍可用
        assert_eq!(a, b); // PartialEq
        assert_ne!(a, Span::new(2, 8));

        let mut set: HashSet<Span> = HashSet::new(); // Hash + Eq
        assert!(set.insert(a));
        assert!(!set.insert(b));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn fields_are_public_and_accessible() {
        let s = Span::new(12, 34);
        assert_eq!((s.line, s.col), (12, 34));
    }
}
