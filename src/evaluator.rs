//! 树遍历求值器（P3.7 核心）：表达式 / 语句 / 控制流 / 函数与闭包 / 调用。
//!
//! 单一事实源（只读消费，不偏离）：
//! - `docs/spec/semantics.md` §4（求值语义总纲）、§3.6（`;;` 可见链）、§3.7（显示）、
//!   §4.2（类型化运算符 / 无隐式转换）、§4.5.1（求值序 B1）、§4.5.2（A1 引用语义）、
//!   §4.5.3（A2 cell 捕获）、§4.5.4（`for` 快照 / 键序 B2/B3）、§4.5.5（调用帧）、
//!   §4.5.6（IEEE 比较）、§4.5.7（`int`/`float` 精确比较）、§4.5.8（struct 平拷贝）、
//!   §4.5.9（A5/A6）、§4.5.10（`assert`/`check`/`fail`）、§4.5.11（字符串不可变）。
//! - `docs/spec/interface-contract.md` §10.2（`Span`）、§10.3（帧 / 调用点 span）、
//!   §10.5（`ScopeDebug` 可见链）、§10.6（管道脱糖为 `Call`）、§10.7（内置 ABI）、§10.8（`R<T>`）。
//!
//! # 名字解析策略：**运行时查名**（本实现的显式选择与理由）
//!
//! 契约推荐「编译期解析为 `(scope_depth, slot)`」——它契合 P3.6a 的槽位式 [`Env`]，
//! 且热路径零哈希。但**本批的 AST（`ast.rs`）只承载 `String` 名字**，节点上没有任何
//! `(depth, slot)`；且 `parser.rs`（P3.4b）由 core-dev **并行**开发，不能依赖它做一次
//! 名字决议 pass。若自行在求值器里先做一遍决议，等价于在运行时模块里塞进一个编译器，
//! 既越界又脆弱。
//!
//! 故本实现采用**运行时查名**，在 `env.rs` 的 [`Env`] 上实现如下（A2 不被破坏）：
//!
//! - 每个作用域多一张 `名字 → 槽位下标` 表（`define_named` / `local_index` /
//!   `get_local` / `set_local` / `capture_local`）；求值器**沿词法链内→外**按名字查
//!   所属作用域，然后**直读该作用域槽位**（不走「按绝对槽位沿链走」，避免不同作用域
//!   `first_slot` 区间重叠带来的歧义）。
//! - 查名顺序：**调用帧 / 块作用域（不含模块顶层）→ 捕获 cell → 模块顶层（globals）**。
//!   故局部遮蔽捕获、捕获遮蔽全局（§4.5.0 词法作用域）。
//! - **模块顶层不参与 cell 捕获**（§4.5.3）：闭包只捕获「非顶层」的具名绑定；顶层绑定
//!   经共享 `globals` 直接访问。
//! - **A2**：创建闭包时对该链上每个（未捕获）具名局部调用 `Env::capture_local`，
//!   把槽位**原地升级**为共享 `Cell`（`Rc<RefCell<Value>>`），并把 `(名字, cell)` 存入
//!   闭包载荷；闭包体与定义作用域共享同一 cell（外部改对闭包可见、闭包改对外部可见）。
//! - **循环每轮独立 cell**：`for` 每次迭代新建子作用域并绑定迭代变量；块体亦新建子作用域；
//!   故当轮闭包捕获的是**当轮**的 cell，互不影响（§4.5.3）。
//! - **递归**：命名 `fn` 先以 `nil` 预绑定其名（占位）→ 创建闭包（捕获含自身名的 cell）→
//!   再写回闭包值，使函数体内可查到自己（自引用经 cell 成立）。
//!
//! # 本批范围（明确留给 P3.8）
//!
//! 不含：§4.5 确定性八项逐条审计、`check` 的完整接线（`check` 内置已可用，仅不做专项断言）、
//! `RecursionError`（无帧深上限）、`;;`（`Dump`）输出（本批为无操作）、traceback 组装
//! （`TraceFrame` / `func_id` 仅分配不复用）。`==`（A6）与 `< <= > >=` 已接线（复用
//! `Value::deep_eq` 与 `Value::total_cmp`），其边界审计归 P3.8。
//!
//! # 管道
//!
//! `syntax.md` §4.3 / 契约 §10.6：管道在 **parser 阶段脱糖**为普通 `Call`（`ast.rs` 无 `Pipe`
//! 节点）。故求值器不存在管道分支——它只看到 `Call`，data-last 注入已由 parser 完成。

use crate::ast::{
    AssignOp, BinaryOp, Block, Body, ElseBranch, Expr, ExprKind, FnDecl, IfExpr, InterpString,
    LogicalOp, Lvalue, LvalueBase, LvalueSeg, LvalueSegKind, Program, Stmt, StmtKind, StrPart,
    StructDecl, StructLit, StructMember, UnaryOp,
};
use crate::builtins;
use crate::env::{Cell, Env, ScopeChain};
use crate::error::{
    div_zero, field as field_error, index as index_error, name as name_error, overflow, type_error,
    value as value_error, LzError, OverflowMsg, R, TypeMsg, ValueMsg,
};
use crate::span::Span;
use crate::value::{Closure, StructObj, UserFn, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

// ===========================================================================
// 公开入口
// ===========================================================================

/// 求值一个编译单元（`Program`），返回**最后一条语句的值**（无语句 → `nil`）。
///
/// 说明：AST 类型名为 `Program`（非 `Module`）；本函数即任务书所称 `eval_module`。
/// 顶层作用域即模块作用域（globals），不创建额外子作用域。
pub fn eval_module(program: &Program) -> R<Value> {
    let mut interp = Interp::new();
    let top = Scope {
        env: Rc::clone(&interp.globals),
        captured: Rc::new(Vec::new()),
        self_val: None,
    };
    match interp.exec_stmts(&program.stmts, &top)? {
        Flow::Value(v) => Ok(v),
        // 顶层 `return` 由 parser 拒绝（`ReturnOutsideFunction`）；此处防御性接受。
        Flow::Return(v) => Ok(v),
        Flow::Break | Flow::Continue => Ok(Value::Nil),
    }
}

/// 执行一个编译单元，丢弃最终值（供 CLI / tooling 调用）。
pub fn run(program: &Program) -> R<()> {
    eval_module(program).map(|_| ())
}

// ===========================================================================
// 控制流与求值步
// ===========================================================================

/// 求值控制流结果：表达式求值 / 语句落空均产 [`Flow::Value`]。
enum Flow {
    /// 正常值（表达式结果；语句落空为 `nil`）。
    Value(Value),
    /// `break`（由最近循环捕获）。
    Break,
    /// `continue`（由最近循环捕获）。
    Continue,
    /// `return v`（由函数调用帧捕获）。
    Return(Value),
}

/// 多值求值（实参表等）的短路结果：某一步若产生控制流则整体短路。
enum Step<T> {
    /// 全部完成。
    Done(T),
    /// 中途产生控制流，向上冒泡。
    Flow(Flow),
}

// ===========================================================================
// 作用域上下文
// ===========================================================================

/// 一次求值所处的**作用域上下文**：
/// - `env`：当前词法环境（函数帧 / 块 / 顶层）；
/// - `captured`：闭包定义处捕获的 `(名字, cell)`（调用帧内查名在 env 之后回落于此）；
/// - `self_val`：`self` 绑定（仅方法调用帧有值）。
struct Scope {
    env: Rc<RefCell<Env>>,
    captured: Rc<Vec<(Rc<str>, Cell)>>,
    self_val: Option<Value>,
}

impl Scope {
    /// 新建**子块作用域**（继承捕获链与 `self`）。
    fn child(&self) -> Scope {
        Scope {
            env: Env::child_after(&self.env),
            captured: Rc::clone(&self.captured),
            self_val: self.self_val.clone(),
        }
    }
}

// ===========================================================================
// 解释器
// ===========================================================================

/// 树遍历解释器：模块全局环境 + 单调递增的 `func_id` 分配器。
struct Interp {
    /// 模块顶层作用域（globals）；所有函数调用帧直接挂在它之下。
    globals: Rc<RefCell<Env>>,
    /// 函数表下标分配器（traceback 用，P3.8 复用）。
    next_func_id: u32,
}

impl Interp {
    fn new() -> Self {
        Self {
            globals: Env::root(),
            next_func_id: 0,
        }
    }

    // -----------------------------------------------------------------------
    // 名字解析
    // -----------------------------------------------------------------------

    /// 读名字：`env` 链（不含 globals）→ 捕获 cell → globals。均内→外。
    fn lookup(&self, scope: &Scope, name: &str) -> Option<Value> {
        let mut cur = Some(Rc::clone(&scope.env));
        while let Some(e) = cur {
            if Rc::ptr_eq(&e, &self.globals) {
                break;
            }
            if let Some(i) = e.borrow().local_index(name) {
                return e.borrow().get_local(i);
            }
            cur = e.borrow().parent();
        }
        for (n, c) in scope.captured.iter() {
            if n.as_ref() == name {
                return Some(c.borrow().clone());
            }
        }
        let gi = self.globals.borrow().local_index(name);
        gi.and_then(|i| self.globals.borrow().get_local(i))
    }

    /// 写名字（原地；捕获变量写共享 cell）。返回是否找到。
    fn assign_name(&self, scope: &Scope, name: &str, value: Value) -> bool {
        let mut cur = Some(Rc::clone(&scope.env));
        while let Some(e) = cur {
            if Rc::ptr_eq(&e, &self.globals) {
                break;
            }
            let idx = e.borrow().local_index(name);
            if let Some(i) = idx {
                e.borrow_mut().set_local(i, value);
                return true;
            }
            cur = e.borrow().parent();
        }
        for (n, c) in scope.captured.iter() {
            if n.as_ref() == name {
                *c.borrow_mut() = value;
                return true;
            }
        }
        let gi = self.globals.borrow().local_index(name);
        if let Some(i) = gi {
            self.globals.borrow_mut().set_local(i, value);
            return true;
        }
        false
    }

    // -----------------------------------------------------------------------
    // 闭包创建（A2 捕获）
    // -----------------------------------------------------------------------

    /// 创建函数 / 闭包值：沿 env 链（**不含 globals**）捕获具名局部为 cell，再继承父捕获。
    fn make_closure(
        &mut self,
        name: Option<&str>,
        params: Vec<String>,
        body: Body,
        scope: &Scope,
    ) -> Value {
        let mut captured: Vec<(Rc<str>, Cell)> = Vec::new();
        let mut cur = Some(Rc::clone(&scope.env));
        while let Some(e) = cur {
            if Rc::ptr_eq(&e, &self.globals) {
                break;
            }
            let entries = e.borrow().named_indices();
            for (n, idx) in entries {
                if captured.iter().any(|(k, _)| k.as_ref() == n.as_str()) {
                    continue;
                }
                if let Some(cell) = e.borrow_mut().capture_local(idx) {
                    captured.push((Rc::from(n.as_str()), cell));
                }
            }
            cur = e.borrow().parent();
        }
        // 继承父闭包的捕获（内层 env 名字已优先占位）。
        for (n, c) in scope.captured.iter() {
            if captured.iter().any(|(k, _)| k.as_ref() == n.as_ref()) {
                continue;
            }
            captured.push((Rc::clone(n), Rc::clone(c)));
        }
        let func_id = self.next_func_id;
        self.next_func_id += 1;
        let user = UserFn {
            params,
            body,
            captured: Rc::new(captured),
            func_id,
            self_val: scope.self_val.clone(),
        };
        // `def_scope`（`;;` 可见链）留 P3.8；本批用空链占位。
        Value::function(Closure::user(
            name.map(Rc::from),
            Rc::new(ScopeChain::empty()),
            Rc::new(user),
        ))
    }

    // -----------------------------------------------------------------------
    // 语句执行
    // -----------------------------------------------------------------------

    /// 顺序执行语句序列，返回**最后一条**语句的控制流（其 `Value` 即块值）。
    fn exec_stmts(&mut self, stmts: &[Stmt], scope: &Scope) -> R<Flow> {
        let mut last = Flow::Value(Value::Nil);
        for st in stmts {
            last = self.exec_stmt(st, scope)?;
            if !matches!(last, Flow::Value(_)) {
                return Ok(last);
            }
        }
        Ok(last)
    }

    fn exec_stmt(&mut self, stmt: &Stmt, scope: &Scope) -> R<Flow> {
        match &stmt.node {
            StmtKind::Decl {
                mutable,
                name,
                init,
            } => {
                let v = match self.eval_expr(init, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                scope.env.borrow_mut().define_named(name, v, *mutable);
                Ok(Flow::Value(Value::Nil))
            }
            StmtKind::Assign { target, op, value } => {
                self.exec_assign(target, *op, value, scope)
            }
            StmtKind::FnDecl(fd) => {
                self.declare_fn(fd, scope);
                Ok(Flow::Value(Value::Nil))
            }
            StmtKind::StructDecl(sd) => {
                self.declare_struct(sd, scope);
                Ok(Flow::Value(Value::Nil))
            }
            // `if` 是表达式（§1）：作为语句时其值即块值（供函数体以 `if` 收尾）。
            StmtKind::If(e) => self.eval_expr(e, scope),
            StmtKind::While { cond, body } => self.exec_while(cond, body, scope),
            StmtKind::For { var, iter, body } => self.exec_for(var, iter, body, scope),
            StmtKind::Return(opt) => {
                let v = match opt {
                    Some(e) => match self.eval_expr(e, scope)? {
                        Flow::Value(v) => v,
                        f => return Ok(f),
                    },
                    None => Value::Nil,
                };
                Ok(Flow::Return(v))
            }
            StmtKind::Break => Ok(Flow::Break),
            StmtKind::Continue => Ok(Flow::Continue),
            // `;;`（Dump）输出留 P3.8：本批不产生输出、不报错（见模块文档）。
            StmtKind::Dump { .. } => Ok(Flow::Value(Value::Nil)),
            StmtKind::Expr(e) => self.eval_expr(e, scope),
        }
    }

    /// 执行块：新建**子作用域**并顺序执行，返回最后语句的 `Flow`（其 `Value` 即块值）。
    fn exec_block(&mut self, block: &Block, scope: &Scope) -> R<Flow> {
        let child = scope.child();
        self.exec_stmts(&block.stmts, &child)
    }

    /// `while cond body`：每轮求值条件；每轮**新块作用域**（§4.5.3）。
    fn exec_while(&mut self, cond: &Expr, body: &Block, scope: &Scope) -> R<Flow> {
        loop {
            let c = match self.eval_expr(cond, scope)? {
                Flow::Value(v) => v,
                f => return Ok(f),
            };
            if !as_condition(&c, cond.span)? {
                break;
            }
            match self.exec_block(body, scope)? {
                Flow::Value(_) => {}
                Flow::Break => break,
                Flow::Continue => continue,
                Flow::Return(v) => return Ok(Flow::Return(v)),
            }
        }
        Ok(Flow::Value(Value::Nil))
    }

    /// `for var in iter body`：迭代开始**取快照**（§4.5.4 / B2 / B3）。
    ///
    /// array → 元素浅快照；struct → **数据字段键**按 UTF-8 字节序快照。每轮新子作用域并
    /// 绑定 `var`（**各自独立 cell**，§4.5.3）。
    fn exec_for(&mut self, var: &str, iter: &Expr, body: &Block, scope: &Scope) -> R<Flow> {
        let it = match self.eval_expr(iter, scope)? {
            Flow::Value(v) => v,
            f => return Ok(f),
        };
        let items: Vec<Value> = match &it {
            Value::Array(a) => a.borrow().to_vec(),
            Value::Struct(s) => s
                .borrow()
                .data_fields_sorted()
                .iter()
                .map(|&(k, _)| Value::string(k))
                .collect(),
            other => {
                return Err(bad_operands(
                    "for … in",
                    other.type_name(),
                    "array / struct",
                    iter.span,
                ))
            }
        };
        for item in items {
            let child = scope.child();
            child.env.borrow_mut().define_named(var, item, true);
            match self.exec_stmts(&body.stmts, &child)? {
                Flow::Value(_) => {}
                Flow::Break => break,
                Flow::Continue => continue,
                Flow::Return(v) => return Ok(Flow::Return(v)),
            }
        }
        Ok(Flow::Value(Value::Nil))
    }

    /// 声明命名函数：预绑定占位 → 建闭包（捕获自身名）→ 写回闭包（支持递归）。
    fn declare_fn(&mut self, fd: &FnDecl, scope: &Scope) {
        scope
            .env
            .borrow_mut()
            .define_named(&fd.name, Value::Nil, true);
        let cl = self.make_closure(Some(&fd.name), fd.params.clone(), fd.body.clone(), scope);
        let idx = scope.env.borrow().local_index(&fd.name);
        if let Some(i) = idx {
            scope.env.borrow_mut().set_local(i, cl);
        }
    }

    /// 声明 struct 模板：为每个方法建函数值，字段默认值表达式按声明序存档（§4.5.8）。
    fn declare_struct(&mut self, sd: &StructDecl, scope: &Scope) {
        scope
            .env
            .borrow_mut()
            .define_named(&sd.name, Value::Nil, true);
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        for m in &sd.members {
            match m {
                StructMember::Field(fi) => {
                    fields.push((Rc::from(fi.name.as_str()), fi.value.clone()))
                }
                StructMember::Method(fd) => {
                    let cl = self.make_closure(
                        Some(&fd.name),
                        fd.params.clone(),
                        fd.body.clone(),
                        scope,
                    );
                    if let Value::Func(rc) = cl {
                        methods.push((Rc::from(fd.name.as_str()), rc));
                    }
                }
            }
        }
        let def = Value::struct_template(sd.name.as_str(), fields, methods);
        let idx = scope.env.borrow().local_index(&sd.name);
        if let Some(i) = idx {
            scope.env.borrow_mut().set_local(i, def);
        }
    }

    // -----------------------------------------------------------------------
    // 赋值（含复合；A1 原地修改）
    // -----------------------------------------------------------------------

    fn exec_assign(
        &mut self,
        target: &Lvalue,
        op: AssignOp,
        value: &Expr,
        scope: &Scope,
    ) -> R<Flow> {
        // ---- 无后缀：变量重绑定 ----
        if target.path.is_empty() {
            let current = match &target.base {
                LvalueBase::Name(n) => self
                    .lookup(scope, n)
                    .ok_or_else(|| name_error(n.clone(), target.span))?,
                LvalueBase::SelfValue => scope
                    .self_val
                    .clone()
                    .ok_or_else(|| name_error("self".to_string(), target.span))?,
            };
            let newv = match self.eval_rhs(op, Some(current), value, scope, target.span)? {
                Step::Done(v) => v,
                Step::Flow(f) => return Ok(f),
            };
            let ok = match &target.base {
                LvalueBase::Name(n) => self.assign_name(scope, n, newv),
                LvalueBase::SelfValue => false,
            };
            if !ok {
                let nm = match &target.base {
                    LvalueBase::Name(n) => n.clone(),
                    LvalueBase::SelfValue => "self".to_string(),
                };
                return Err(name_error(nm, target.span));
            }
            return Ok(Flow::Value(Value::Nil));
        }

        // ---- 有后缀：容器原地修改（A1） ----
        let mut current = match &target.base {
            LvalueBase::Name(n) => self
                .lookup(scope, n)
                .ok_or_else(|| name_error(n.clone(), target.span))?,
            LvalueBase::SelfValue => scope
                .self_val
                .clone()
                .ok_or_else(|| name_error("self".to_string(), target.span))?,
        };
        let last = target.path.len() - 1;
        // 中间段：逐段解析到「最终容器」。
        for seg in &target.path[..last] {
            current = match self.read_segment(&current, seg, scope)? {
                Step::Done(v) => v,
                Step::Flow(f) => return Ok(f),
            };
        }
        // §4.5.1：先求最终段的键（下标表达式），再求 rhs。
        let seg = &target.path[last];
        let key = match &seg.kind {
            LvalueSegKind::Field(k) => Key::Field(Rc::from(k.as_str())),
            LvalueSegKind::Index(e) => match self.eval_expr(e, scope)? {
                Flow::Value(v) => Key::Index(v),
                f => return Ok(f),
            },
        };
        // 复合赋值需先读当前值（`=` 不必，允许 struct 动态加字段）。
        let current_val = if op == AssignOp::Assign {
            None
        } else {
            Some(self.read_final(&current, &key, seg.span)?)
        };
        let newv = match self.eval_rhs(op, current_val, value, scope, target.span)? {
            Step::Done(v) => v,
            Step::Flow(f) => return Ok(f),
        };
        self.write_final(&current, &key, newv, seg.span)?;
        Ok(Flow::Value(Value::Nil))
    }

    /// 求 RHS 并按 `op` 合成新值（`=` 直接取 RHS；其余读旧值后二元运算）。
    fn eval_rhs(
        &mut self,
        op: AssignOp,
        current: Option<Value>,
        value: &Expr,
        scope: &Scope,
        span: Span,
    ) -> R<Step<Value>> {
        let rv = match self.eval_expr(value, scope)? {
            Flow::Value(v) => v,
            f => return Ok(Step::Flow(f)),
        };
        let out = match op {
            AssignOp::Assign => rv,
            other => {
                let cur = current.expect("复合赋值必有当前值");
                let bop = match other {
                    AssignOp::AddAssign => BinaryOp::Add,
                    AssignOp::SubAssign => BinaryOp::Sub,
                    AssignOp::MulAssign => BinaryOp::Mul,
                    AssignOp::DivAssign => BinaryOp::Div,
                    AssignOp::RemAssign => BinaryOp::Rem,
                    AssignOp::Assign => unreachable!(),
                };
                apply_binary(bop, cur, rv, span)?
            }
        };
        Ok(Step::Done(out))
    }

    /// 解析 lvalue 的**中间段**（字段 / 下标读取），用于走到最终容器。
    fn read_segment(&mut self, cur: &Value, seg: &LvalueSeg, scope: &Scope) -> R<Step<Value>> {
        match &seg.kind {
            LvalueSegKind::Field(k) => Ok(Step::Done(self.get_field(cur, k, seg.span)?)),
            LvalueSegKind::Index(e) => match self.eval_expr(e, scope)? {
                Flow::Value(iv) => Ok(Step::Done(self.index_read(cur, &iv, seg.span)?)),
                f => Ok(Step::Flow(f)),
            },
        }
    }

    // -----------------------------------------------------------------------
    // 表达式求值
    // -----------------------------------------------------------------------

    fn eval_expr(&mut self, expr: &Expr, scope: &Scope) -> R<Flow> {
        let span = expr.span;
        match &expr.node {
            ExprKind::Int(i) => Ok(Flow::Value(Value::Int(*i))),
            ExprKind::Float(x) => Ok(Flow::Value(Value::Float(*x))),
            ExprKind::Str(s) => Ok(Flow::Value(Value::string(s.clone()))),
            ExprKind::Bool(b) => Ok(Flow::Value(Value::Bool(*b))),
            ExprKind::Nil => Ok(Flow::Value(Value::Nil)),
            ExprKind::Interp(is) => self.eval_interp(is, span, scope),
            ExprKind::Ident(name) => match self.lookup(scope, name) {
                Some(v) => Ok(Flow::Value(v)),
                None => Err(name_error(name.clone(), span)),
            },
            ExprKind::SelfRef => match &scope.self_val {
                Some(v) => Ok(Flow::Value(v.clone())),
                None => Err(name_error("self".to_string(), span)),
            },
            ExprKind::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items {
                    match self.eval_expr(it, scope)? {
                        Flow::Value(v) => out.push(v),
                        f => return Ok(f),
                    }
                }
                Ok(Flow::Value(Value::array(out)))
            }
            ExprKind::StructLit(sl) => self.eval_struct_lit(sl, span, scope),
            ExprKind::Field { object, name } => {
                let obj = match self.eval_expr(object, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                Ok(Flow::Value(self.get_field(&obj, name, span)?))
            }
            ExprKind::Index { object, index } => {
                let obj = match self.eval_expr(object, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                let iv = match self.eval_expr(index, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                Ok(Flow::Value(self.index_read(&obj, &iv, span)?))
            }
            ExprKind::Call { callee, args } => self.eval_call(expr, callee, args, scope),
            ExprKind::Unary { op, operand } => {
                let v = match self.eval_expr(operand, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                Ok(Flow::Value(apply_unary(*op, v, span)?))
            }
            ExprKind::Binary { op, left, right } => {
                let l = match self.eval_expr(left, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                let r = match self.eval_expr(right, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                Ok(Flow::Value(apply_binary(*op, l, r, span)?))
            }
            ExprKind::Logical { op, left, right } => {
                self.eval_logical(*op, left, right, span, scope)
            }
            ExprKind::If(ifx) => self.eval_if(ifx, span, scope),
            ExprKind::Lambda(l) => Ok(Flow::Value(self.make_closure(
                None,
                l.params.clone(),
                l.body.clone(),
                scope,
            ))),
        }
    }

    /// `&&` / `||`：两侧与结果均须 `bool`；**短路**（§4.5.1）。
    fn eval_logical(
        &mut self,
        op: LogicalOp,
        left: &Expr,
        right: &Expr,
        span: Span,
        scope: &Scope,
    ) -> R<Flow> {
        let l = match self.eval_expr(left, scope)? {
            Flow::Value(v) => v,
            f => return Ok(f),
        };
        let lb = as_condition(&l, span)?;
        match op {
            LogicalOp::And => {
                if !lb {
                    return Ok(Flow::Value(Value::Bool(false)));
                }
            }
            LogicalOp::Or => {
                if lb {
                    return Ok(Flow::Value(Value::Bool(true)));
                }
            }
        }
        let r = match self.eval_expr(right, scope)? {
            Flow::Value(v) => v,
            f => return Ok(f),
        };
        Ok(Flow::Value(Value::Bool(as_condition(&r, span)?)))
    }

    /// `if` 表达式：只求被选中分支；块值 = 分支块最后一条表达式语句的值（§1 / §4.5.1）。
    fn eval_if(&mut self, ifx: &IfExpr, span: Span, scope: &Scope) -> R<Flow> {
        let c = match self.eval_expr(&ifx.cond, scope)? {
            Flow::Value(v) => v,
            f => return Ok(f),
        };
        if as_condition(&c, span)? {
            return self.exec_block(&ifx.then_block, scope);
        }
        match &ifx.else_branch {
            None => Ok(Flow::Value(Value::Nil)),
            Some(ElseBranch::Block(b)) => self.exec_block(b, scope),
            Some(ElseBranch::If(e)) => self.eval_expr(e, scope),
        }
    }

    /// 富字符串插值：各段从左到右；`${expr[:spec]}` 按格式说明符渲染（§2.8）。
    fn eval_interp(&mut self, is: &InterpString, span: Span, scope: &Scope) -> R<Flow> {
        let mut out = String::new();
        for part in &is.parts {
            match part {
                StrPart::Text(t) => out.push_str(t),
                StrPart::Expr { expr, format_spec } => {
                    let v = match self.eval_expr(expr, scope)? {
                        Flow::Value(v) => v,
                        f => return Ok(f),
                    };
                    out.push_str(&format_value(&v, format_spec.as_deref(), span)?);
                }
            }
        }
        Ok(Flow::Value(Value::string(out)))
    }

    /// struct 字面量 / 模板实例化（§4.5.8 平拷贝）。
    fn eval_struct_lit(&mut self, sl: &StructLit, span: Span, scope: &Scope) -> R<Flow> {
        let mut obj = StructObj::new();
        if let Some(tn) = &sl.type_name {
            let defv = self
                .lookup(scope, tn)
                .ok_or_else(|| name_error(tn.clone(), span))?;
            let def = match defv {
                Value::StructDef(d) => d,
                other => return Err(bad_operands("{}", other.type_name(), "struct 模板", span)),
            };
            // 模板字段默认值：按声明序**重新求值**（可变默认每实例各一份，§4.5.8）。
            for (fname, expr) in &def.fields {
                let v = match self.eval_expr(expr, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                obj.set(fname, v);
            }
            // 方法随实例可用（函数值字段；A5 数据面不含）。
            for (mname, cl) in &def.methods {
                obj.set(mname, Value::Func(Rc::clone(cl)));
            }
        }
        // 字面字段：书写序；覆盖默认值或动态新增（§4.5.8）。
        for fi in &sl.fields {
            let v = match self.eval_expr(&fi.value, scope)? {
                Flow::Value(v) => v,
                f => return Ok(f),
            };
            obj.set(&fi.name, v);
        }
        Ok(Flow::Value(Value::Struct(Rc::new(RefCell::new(obj)))))
    }

    /// 调用表达式（管道已由 parser 脱糖为 `Call`，§10.6）。
    fn eval_call(
        &mut self,
        expr: &Expr,
        callee: &Expr,
        args: &[Expr],
        scope: &Scope,
    ) -> R<Flow> {
        let span = expr.span;
        match &callee.node {
            // `f(...)`：变量优先（可遮蔽同名内置），否则查内置表。
            ExprKind::Ident(name) => {
                if let Some(v) = self.lookup(scope, name) {
                    let argv = match self.eval_args(args, scope)? {
                        Step::Done(v) => v,
                        Step::Flow(f) => return Ok(f),
                    };
                    return self.call_func(v, argv, span, None);
                }
                if builtins::is_builtin(name) {
                    let argv = match self.eval_args(args, scope)? {
                        Step::Done(v) => v,
                        Step::Flow(f) => return Ok(f),
                    };
                    // 位置红线：内置 Span 取**调用点**（`Call` 节点 span）。
                    return Ok(Flow::Value(builtins::call(name, &argv, span)?));
                }
                Err(name_error(name.clone(), callee.span))
            }
            // `recv.m(...)`：方法调用，绑定 `self` 为接收者（§3.7 / A5）。
            ExprKind::Field { object, name } => {
                let recv = match self.eval_expr(object, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                let func = self.get_field(&recv, name, callee.span)?;
                let argv = match self.eval_args(args, scope)? {
                    Step::Done(v) => v,
                    Step::Flow(f) => return Ok(f),
                };
                self.call_func(func, argv, span, Some(recv))
            }
            // 一般被调表达式（如 `arr[0](...)`、`(fn(){...})()`）。
            _ => {
                let cv = match self.eval_expr(callee, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                let argv = match self.eval_args(args, scope)? {
                    Step::Done(v) => v,
                    Step::Flow(f) => return Ok(f),
                };
                self.call_func(cv, argv, span, None)
            }
        }
    }

    /// 求实参（左→右）；任一步产生控制流则短路。
    fn eval_args(&mut self, args: &[Expr], scope: &Scope) -> R<Step<Vec<Value>>> {
        let mut out = Vec::with_capacity(args.len());
        for a in args {
            match self.eval_expr(a, scope)? {
                Flow::Value(v) => out.push(v),
                f => return Ok(Step::Flow(f)),
            }
        }
        Ok(Step::Done(out))
    }

    /// 调用一个值：须为函数值，否则 `TypeError::NotCallable`。
    fn call_func(
        &mut self,
        callee: Value,
        argv: Vec<Value>,
        span: Span,
        self_override: Option<Value>,
    ) -> R<Flow> {
        match callee {
            Value::Func(cl) => Ok(Flow::Value(self.call_user(&cl, argv, span, self_override)?)),
            other => Err(type_error(
                TypeMsg::NotCallable {
                    t: other.type_name().to_string(),
                },
                span,
            )),
        }
    }

    /// 用户函数调用：新调用帧 + 参数绑定 +（A2 捕获链）+ 函数体求值。
    fn call_user(
        &mut self,
        cl: &Rc<Closure>,
        argv: Vec<Value>,
        span: Span,
        self_override: Option<Value>,
    ) -> R<Value> {
        let Some(user) = cl.user.as_ref() else {
            // 占位闭包（无函数体）不可调用。
            return Err(type_error(
                TypeMsg::NotCallable {
                    t: "function".to_string(),
                },
                span,
            ));
        };
        if argv.len() != user.params.len() {
            let name = cl.name.as_deref().unwrap_or("<fn>").to_string();
            return Err(type_error(
                TypeMsg::ArgCount {
                    name,
                    n: user.params.len(),
                    m: argv.len(),
                },
                span,
            ));
        }
        // 调用帧直接挂在 globals 之下；自由局部经捕获 cell 解析（§4.5.3）。
        let frame = Env::child_after(&self.globals);
        for (p, a) in user.params.iter().zip(argv.into_iter()) {
            frame.borrow_mut().define_named(p, a, true);
        }
        let scope = Scope {
            env: Rc::clone(&frame),
            captured: Rc::clone(&user.captured),
            self_val: self_override.or_else(|| user.self_val.clone()),
        };
        match &user.body {
            // 函数体块直接在帧作用域执行（参数与体局部同帧）。
            Body::Block(b) => match self.exec_stmts(&b.stmts, &scope)? {
                Flow::Value(v) => Ok(v),
                Flow::Return(v) => Ok(v),
                Flow::Break | Flow::Continue => Ok(Value::Nil), // parser 已拒绝；防御
            },
            Body::Expr(e) => match self.eval_expr(e, &scope)? {
                Flow::Value(v) => Ok(v),
                Flow::Return(v) => Ok(v),
                Flow::Break | Flow::Continue => Ok(Value::Nil),
            },
        }
    }

    // -----------------------------------------------------------------------
    // 字段 / 下标（读）与写
    // -----------------------------------------------------------------------

    /// `s.k` 读字段（含方法字段）；缺失 → `FieldError`。
    fn get_field(&self, v: &Value, name: &str, span: Span) -> R<Value> {
        match v {
            Value::Struct(s) => s
                .borrow()
                .get(name)
                .cloned()
                .ok_or_else(|| field_error(name.to_string(), span)),
            other => Err(bad_operands(".", other.type_name(), "struct", span)),
        }
    }

    /// `a[i]` / `s[k]` 读（array 支持负索引；struct 键须 string）。
    fn index_read(&self, obj: &Value, idx: &Value, span: Span) -> R<Value> {
        match obj {
            Value::Array(a) => {
                let i = idx
                    .as_int()
                    .ok_or_else(|| bad_operands("[]", idx.type_name(), "int", span))?;
                let len = a.borrow().len();
                let norm = normalize_index(i, len, span)?;
                Ok(a.borrow()[norm].clone())
            }
            Value::Struct(s) => {
                let k = idx
                    .as_str()
                    .ok_or_else(|| bad_operands("[]", idx.type_name(), "string", span))?;
                s.borrow()
                    .get(k)
                    .cloned()
                    .ok_or_else(|| field_error(k.to_string(), span))
            }
            other => Err(bad_operands("[]", other.type_name(), "array / struct", span)),
        }
    }

    /// 复合赋值的旧值读取。
    fn read_final(&self, container: &Value, key: &Key, span: Span) -> R<Value> {
        match key {
            Key::Field(k) => self.get_field(container, k, span),
            Key::Index(iv) => self.index_read(container, iv, span),
        }
    }

    /// 写入最终位置（A1 原地）：`s.k` / `s["k"]` 动态加字段；`a[i]` 越界 → `IndexError`（不扩容）。
    fn write_final(&self, container: &Value, key: &Key, value: Value, span: Span) -> R<()> {
        match (container, key) {
            (Value::Struct(s), Key::Field(k)) => {
                s.borrow_mut().set(k, value);
                Ok(())
            }
            (Value::Struct(s), Key::Index(Value::Str(k))) => {
                s.borrow_mut().set(k, value);
                Ok(())
            }
            (Value::Array(a), Key::Index(v)) => {
                let i = v
                    .as_int()
                    .ok_or_else(|| bad_operands("[]", v.type_name(), "int", span))?;
                let len = a.borrow().len();
                let norm = normalize_index(i, len, span)?;
                a.borrow_mut()[norm] = value;
                Ok(())
            }
            (Value::Struct(_), Key::Index(v)) => {
                Err(bad_operands("[]", v.type_name(), "string", span))
            }
            (Value::Array(_), Key::Field(_)) => {
                Err(bad_operands(".", container.type_name(), "struct", span))
            }
            (other, _) => Err(bad_operands("[]", other.type_name(), "array / struct", span)),
        }
    }
}

/// 最终写入位置：字段名或下标值。
enum Key {
    /// `.name`
    Field(Rc<str>),
    /// `[expr]`
    Index(Value),
}

// ===========================================================================
// 运算符语义（§4.2 / §4.5.6 / §4.5.7）
// ===========================================================================

/// 条件位 / `&&` / `||` / `if` / `while` 的操作数必须为 `bool`（§4.5.0）。
fn as_condition(v: &Value, span: Span) -> R<bool> {
    v.as_bool().ok_or_else(|| {
        type_error(
            TypeMsg::ConditionNotBool {
                t: v.type_name().to_string(),
            },
            span,
        )
    })
}

fn apply_unary(op: UnaryOp, v: Value, span: Span) -> R<Value> {
    match op {
        UnaryOp::Neg => match v {
            Value::Int(i) => i
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange)),
            Value::Float(x) => Ok(Value::Float(-x)),
            other => Err(bad_operands("-", other.type_name(), "number", span)),
        },
        UnaryOp::Not => match v {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            other => Err(type_error(
                TypeMsg::ConditionNotBool {
                    t: other.type_name().to_string(),
                },
                span,
            )),
        },
    }
}

fn apply_binary(op: BinaryOp, l: Value, r: Value, span: Span) -> R<Value> {
    match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
            arith(op, &l, &r, span)
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            compare_values(op, &l, &r, span)
        }
        // A6 深结构相等（`value.rs` 唯一共享实现）。
        BinaryOp::Eq => Ok(Value::Bool(l.deep_eq(&r))),
        BinaryOp::Ne => Ok(Value::Bool(!l.deep_eq(&r))),
    }
}

fn arith(op: BinaryOp, l: &Value, r: &Value, span: Span) -> R<Value> {
    match op {
        BinaryOp::Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_add(*b)
                .map(Value::Int)
                .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::string(format!("{a}{b}"))),
            _ if both_num(l, r) => Ok(Value::Float(unwrap_f64(l) + unwrap_f64(r))),
            _ => Err(bad_operands2("+", l, r, span)),
        },
        BinaryOp::Sub => match (l, r) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_sub(*b)
                .map(Value::Int)
                .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange)),
            _ if both_num(l, r) => Ok(Value::Float(unwrap_f64(l) - unwrap_f64(r))),
            _ => Err(bad_operands2("-", l, r, span)),
        },
        BinaryOp::Mul => match (l, r) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_mul(*b)
                .map(Value::Int)
                .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange)),
            // `string * int`（重复；`n <= 0` → 空串；长度溢出 → OverflowError）。
            (Value::Str(s), Value::Int(n)) => repeat_str(s, *n, span),
            _ if both_num(l, r) => Ok(Value::Float(unwrap_f64(l) * unwrap_f64(r))),
            _ => Err(bad_operands2("*", l, r, span)),
        },
        // `/` 恒返回 `float`（A3）；除数为零（含 float）→ ZeroDivisionError。
        BinaryOp::Div => {
            if let (Value::Int(_), Value::Int(b)) = (l, r) {
                if *b == 0 {
                    return Err(div_zero(false, span));
                }
                return Ok(Value::Float(unwrap_f64(l) / unwrap_f64(r)));
            }
            if both_num(l, r) {
                let (a, b) = (unwrap_f64(l), unwrap_f64(r));
                if b == 0.0 {
                    return Err(div_zero(false, span));
                }
                return Ok(Value::Float(a / b));
            }
            Err(bad_operands2("/", l, r, span))
        }
        // `%` Python 取模（结果符号随除数，A3）；除数为零 → ZeroDivisionError。
        BinaryOp::Rem => {
            if let (Value::Int(a), Value::Int(b)) = (l, r) {
                if *b == 0 {
                    return Err(div_zero(true, span));
                }
                // `i64::MIN % -1` 在数学上为 0（`checked_rem` 报 None）。
                let rem = a.checked_rem(*b).unwrap_or(0);
                let rr = if rem != 0 && ((rem < 0) != (*b < 0)) {
                    rem + *b
                } else {
                    rem
                };
                return Ok(Value::Int(rr));
            }
            if both_num(l, r) {
                let (a, b) = (unwrap_f64(l), unwrap_f64(r));
                if b == 0.0 {
                    return Err(div_zero(true, span));
                }
                let rem = a % b;
                let rr = if rem != 0.0 && ((rem < 0.0) != (b < 0.0)) {
                    rem + b
                } else {
                    rem
                };
                return Ok(Value::Float(rr));
            }
            Err(bad_operands2("%", l, r, span))
        }
        _ => unreachable!("arith 只处理算术运算符"),
    }
}

/// `< <= > >=`：数值组（`int`/`float` 精确比较）或字符串组；涉及 `NaN` → 一律 `false`。
fn compare_values(op: BinaryOp, l: &Value, r: &Value, span: Span) -> R<Value> {
    let nums = both_num(l, r);
    let strs = matches!(l, Value::Str(_)) && matches!(r, Value::Str(_));
    if !nums && !strs {
        return Err(bad_operands2(
            match op {
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                _ => unreachable!(),
            },
            l,
            r,
            span,
        ));
    }
    // IEEE：任何 `< <= > >=` 涉及 NaN → false（§4.5.6）。
    if nums && (is_nan(l) || is_nan(r)) {
        return Ok(Value::Bool(false));
    }
    let ord = l.total_cmp(r).expect("同类可比（§4.5.6）");
    let res = match op {
        BinaryOp::Lt => ord == Ordering::Less,
        BinaryOp::Le => ord != Ordering::Greater,
        BinaryOp::Gt => ord == Ordering::Greater,
        BinaryOp::Ge => ord != Ordering::Less,
        _ => unreachable!(),
    };
    Ok(Value::Bool(res))
}

/// 两侧均为数值（`int` / `float`）。
fn both_num(l: &Value, r: &Value) -> bool {
    l.as_f64().is_some() && r.as_f64().is_some()
}

/// 唯一加宽入口 `Value::as_f64` 的取值（调用点已保证 `Some`）。
fn unwrap_f64(v: &Value) -> f64 {
    v.as_f64().expect("both_num 已保证数值")
}

fn is_nan(v: &Value) -> bool {
    matches!(v, Value::Float(x) if x.is_nan())
}

/// `string * int` 重复（`n <= 0` → 空串）。
fn repeat_str(s: &str, n: i64, span: Span) -> R<Value> {
    if n <= 0 {
        return Ok(Value::string(""));
    }
    s.len()
        .checked_mul(n as usize)
        .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange))?;
    Ok(Value::string(s.repeat(n as usize)))
}

/// 负索引规范化；越界 → `IndexError`（`idx` = 用户原值，`len` = 容器长度）。
fn normalize_index(i: i64, len: usize, span: Span) -> R<usize> {
    let n = len as i64;
    let norm = if i < 0 { i + n } else { i };
    if norm < 0 || norm >= n {
        Err(index_error(i, len, span))
    } else {
        Ok(norm as usize)
    }
}

/// `TypeError::BadOperands`（`op` 符号 + 两侧类型名），位置取运算符节点 `span`。
fn bad_operands2(op: &str, l: &Value, r: &Value, span: Span) -> Box<LzError> {
    type_error(
        TypeMsg::BadOperands {
            op: op.to_string(),
            lt: l.type_name().to_string(),
            rt: r.type_name().to_string(),
        },
        span,
    )
}

/// `TypeError::BadOperands`（一个操作数类型名 + 期望类型描述）。
fn bad_operands(op: &str, lt: &str, rt: &str, span: Span) -> Box<LzError> {
    type_error(
        TypeMsg::BadOperands {
            op: op.to_string(),
            lt: lt.to_string(),
            rt: rt.to_string(),
        },
        span,
    )
}

// ===========================================================================
// 格式说明符（§2.8：`[ [fill] align ][sign][width][.precision][type]`）
// ===========================================================================

/// 单个富字符串段的格式说明符（Python 风格子集）。
struct FmtSpec {
    fill: char,
    align: Option<char>,
    sign: Option<char>,
    zero: bool,
    width: Option<usize>,
    precision: Option<usize>,
    ty: Option<char>,
}

impl FmtSpec {
    fn parse(raw: &str, span: Span) -> R<FmtSpec> {
        // 防御：若 parser 把前导 `:` 一并放入，剥掉（M6 约定为 `:` 之后的文本）。
        let s = raw.strip_prefix(':').unwrap_or(raw);
        let mut fs = FmtSpec {
            fill: ' ',
            align: None,
            sign: None,
            zero: false,
            width: None,
            precision: None,
            ty: None,
        };
        let bad = || value_error(ValueMsg::BadFormatSpec { spec: s.to_string() }, span);
        let cs: Vec<char> = s.chars().collect();
        let mut i = 0;
        // [ [fill] align ]
        if i < cs.len() && is_align(cs[i]) {
            fs.align = Some(cs[i]);
            i += 1;
        } else if i + 1 < cs.len() && is_align(cs[i + 1]) {
            fs.fill = cs[i];
            fs.align = Some(cs[i + 1]);
            i += 2;
        }
        // [sign]
        if i < cs.len() && matches!(cs[i], '+' | '-' | ' ') {
            fs.sign = Some(cs[i]);
            i += 1;
        }
        // [0]
        if i < cs.len() && cs[i] == '0' {
            fs.zero = true;
            i += 1;
        }
        // [width]
        let ws = i;
        while i < cs.len() && cs[i].is_ascii_digit() {
            i += 1;
        }
        if i > ws {
            let w: String = cs[ws..i].iter().collect();
            fs.width = Some(w.parse::<usize>().map_err(|_| bad())?);
        }
        // [.precision]
        if i < cs.len() && cs[i] == '.' {
            i += 1;
            let ps = i;
            while i < cs.len() && cs[i].is_ascii_digit() {
                i += 1;
            }
            if i == ps {
                return Err(bad());
            }
            let p: String = cs[ps..i].iter().collect();
            fs.precision = Some(p.parse::<usize>().map_err(|_| bad())?);
        }
        // [type]
        if i < cs.len() {
            let t = cs[i];
            if !matches!(t, 'd' | 'x' | 'X' | 'o' | 'b' | 'f' | 'e' | 's') {
                return Err(bad());
            }
            fs.ty = Some(t);
            i += 1;
        }
        if i != cs.len() {
            return Err(bad());
        }
        Ok(fs)
    }

    /// 按 `width` / `align` / `zero` 填充（`numeric` 决定默认对齐与零填充语义）。
    fn pad(&self, body: String, numeric: bool) -> String {
        let Some(width) = self.width else {
            return body;
        };
        let len = body.chars().count();
        if len >= width {
            return body;
        }
        let fill_n = width - len;
        if self.zero && numeric && self.align.is_none() {
            let (sign, rest) = split_sign(&body);
            return format!("{sign}{}{rest}", "0".repeat(fill_n));
        }
        let align = self.align.unwrap_or(if numeric { '>' } else { '<' });
        let fill = |n: usize| self.fill.to_string().repeat(n);
        match align {
            '>' => format!("{}{body}", fill(fill_n)),
            '^' => {
                let left = fill_n / 2;
                format!("{}{body}{}", fill(left), fill(fill_n - left))
            }
            _ => format!("{body}{}", fill(fill_n)),
        }
    }
}

fn is_align(c: char) -> bool {
    matches!(c, '<' | '>' | '^')
}

/// 拆出前导符号（`-` / `+` / 空格）。
fn split_sign(s: &str) -> (String, String) {
    let mut it = s.chars();
    match it.next() {
        Some(c) if matches!(c, '-' | '+' | ' ') => (c.to_string(), it.as_str().to_string()),
        _ => (String::new(), s.to_string()),
    }
}

/// 渲染一个值（`spec == None` 时用 §3.7 显示形式；否则套格式说明符）。
fn format_value(v: &Value, spec: Option<&str>, span: Span) -> R<String> {
    let Some(raw) = spec else {
        return Ok(v.to_string());
    };
    let fs = FmtSpec::parse(raw, span)?;
    match fs.ty {
        None => {
            let body = v.to_string();
            Ok(fs.pad(body, is_numeric(v)))
        }
        Some('s') => match v {
            Value::Str(s) => Ok(fs.pad(s.to_string(), false)),
            other => Err(format_mismatch(raw, other, span)),
        },
        Some('d') => match v {
            Value::Int(i) => Ok(fs.pad(int_digits(*i, 10, false, &fs), true)),
            other => Err(format_mismatch(raw, other, span)),
        },
        Some(t @ ('x' | 'X' | 'o' | 'b')) => match v {
            Value::Int(i) => {
                let base = match t {
                    'x' | 'X' => 16,
                    'o' => 8,
                    _ => 2,
                };
                Ok(fs.pad(int_digits(*i, base, t == 'X', &fs), true))
            }
            other => Err(format_mismatch(raw, other, span)),
        },
        Some('f') => match v.as_f64() {
            Some(x) => {
                let body = apply_sign(format!("{:.*}", fs.precision.unwrap_or(6), x), x, &fs);
                Ok(fs.pad(body, true))
            }
            None => Err(format_mismatch(raw, v, span)),
        },
        Some('e') => match v.as_f64() {
            Some(x) => {
                let body = apply_sign(format!("{:.*e}", fs.precision.unwrap_or(6), x), x, &fs);
                Ok(fs.pad(body, true))
            }
            None => Err(format_mismatch(raw, v, span)),
        },
        _ => Err(value_error(
            ValueMsg::BadFormatSpec {
                spec: raw.to_string(),
            },
            span,
        )),
    }
}

fn is_numeric(v: &Value) -> bool {
    matches!(v, Value::Int(_) | Value::Float(_))
}

/// 整数按进制渲染（含符号，依据 `fs.sign`）。
fn int_digits(i: i64, base: u32, upper: bool, fs: &FmtSpec) -> String {
    let mag = i.unsigned_abs();
    let digits = match base {
        2 => format!("{mag:b}"),
        8 => format!("{mag:o}"),
        16 if upper => format!("{mag:X}"),
        16 => format!("{mag:x}"),
        _ => format!("{mag}"),
    };
    let sign = if i < 0 {
        "-".to_string()
    } else {
        match fs.sign {
            Some('+') => "+".to_string(),
            Some(' ') => " ".to_string(),
            _ => String::new(),
        }
    };
    format!("{sign}{digits}")
}

/// 浮点按 `fs.sign` 追加正号 / 空格（负号已由 `format!` 产出）。
fn apply_sign(body: String, x: f64, fs: &FmtSpec) -> String {
    if x.is_sign_negative() {
        return body;
    }
    match fs.sign {
        Some('+') => format!("+{body}"),
        Some(' ') => format!(" {body}"),
        _ => body,
    }
}

fn format_mismatch(spec: &str, v: &Value, span: Span) -> Box<LzError> {
    type_error(
        TypeMsg::FormatSpecMismatch {
            spec: spec.to_string(),
            t: v.type_name().to_string(),
        },
        span,
    )
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FieldInit, Lambda, Spanned};

    // ---- AST 构造辅助（不依赖 parser；parser 由 core-dev 并行开发） ----

    fn sp() -> Span {
        Span::START
    }
    fn e(node: ExprKind) -> Expr {
        Spanned::new(node, sp())
    }
    fn st(node: StmtKind) -> Stmt {
        Spanned::new(node, sp())
    }
    fn int(n: i64) -> Expr {
        e(ExprKind::Int(n))
    }
    fn flt(x: f64) -> Expr {
        e(ExprKind::Float(x))
    }
    fn bool_(b: bool) -> Expr {
        e(ExprKind::Bool(b))
    }
    fn sstr(s: &str) -> Expr {
        e(ExprKind::Str(s.to_string()))
    }
    fn ident(n: &str) -> Expr {
        e(ExprKind::Ident(n.to_string()))
    }
    fn self_ref() -> Expr {
        e(ExprKind::SelfRef)
    }
    fn bin(op: BinaryOp, l: Expr, r: Expr) -> Expr {
        e(ExprKind::Binary {
            op,
            left: Box::new(l),
            right: Box::new(r),
        })
    }
    fn logical(op: LogicalOp, l: Expr, r: Expr) -> Expr {
        e(ExprKind::Logical {
            op,
            left: Box::new(l),
            right: Box::new(r),
        })
    }
    fn neg(x: Expr) -> Expr {
        e(ExprKind::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(x),
        })
    }
    fn call(name: &str, args: Vec<Expr>) -> Expr {
        e(ExprKind::Call {
            callee: Box::new(ident(name)),
            args,
        })
    }
    fn call_expr(callee: Expr, args: Vec<Expr>) -> Expr {
        e(ExprKind::Call {
            callee: Box::new(callee),
            args,
        })
    }
    fn arr(items: Vec<Expr>) -> Expr {
        e(ExprKind::Array(items))
    }
    fn index(obj: Expr, idx: Expr) -> Expr {
        e(ExprKind::Index {
            object: Box::new(obj),
            index: Box::new(idx),
        })
    }
    fn field(obj: Expr, name: &str) -> Expr {
        e(ExprKind::Field {
            object: Box::new(obj),
            name: name.to_string(),
        })
    }
    fn lambda(params: &[&str], body: Body) -> Expr {
        e(ExprKind::Lambda(Box::new(Lambda {
            params: params.iter().map(|s| s.to_string()).collect(),
            body,
        })))
    }
    fn block(stmts: Vec<Stmt>) -> Block {
        Block { span: sp(), stmts }
    }
    fn if_expr(c: Expr, then: Vec<Stmt>, els: Option<Block>) -> Expr {
        e(ExprKind::If(IfExpr {
            cond: Box::new(c),
            then_block: block(then),
            else_branch: els.map(ElseBranch::Block),
        }))
    }
    fn prog(stmts: Vec<Stmt>) -> Program {
        Program { span: sp(), stmts }
    }
    fn decl(name: &str, init: Expr) -> Stmt {
        st(StmtKind::Decl {
            mutable: true,
            name: name.to_string(),
            init,
        })
    }
    fn ldecl(name: &str, init: Expr) -> Stmt {
        st(StmtKind::Decl {
            mutable: false,
            name: name.to_string(),
            init,
        })
    }
    fn expr_stmt(x: Expr) -> Stmt {
        st(StmtKind::Expr(x))
    }
    fn if_stmt(c: Expr, then: Vec<Stmt>) -> Stmt {
        st(StmtKind::If(if_expr(c, then, None)))
    }
    fn assign_var(name: &str, v: Expr) -> Stmt {
        assign_op(name, AssignOp::Assign, v)
    }
    fn assign_op(name: &str, op: AssignOp, v: Expr) -> Stmt {
        st(StmtKind::Assign {
            target: Lvalue {
                span: sp(),
                base: LvalueBase::Name(name.to_string()),
                path: vec![],
            },
            op,
            value: v,
        })
    }
    fn assign_seg(base: &str, path: Vec<LvalueSegKind>, v: Expr) -> Stmt {
        st(StmtKind::Assign {
            target: Lvalue {
                span: sp(),
                base: LvalueBase::Name(base.to_string()),
                path: path
                    .into_iter()
                    .map(|kind| LvalueSeg { span: sp(), kind })
                    .collect(),
            },
            op: AssignOp::Assign,
            value: v,
        })
    }
    fn fn_decl(name: &str, params: &[&str], body: Body) -> Stmt {
        st(StmtKind::FnDecl(FnDecl {
            span: sp(),
            name: name.to_string(),
            params: params.iter().map(|s| s.to_string()).collect(),
            body,
        }))
    }
    fn field_init(name: &str, value: Expr) -> FieldInit {
        FieldInit {
            span: sp(),
            name: name.to_string(),
            value,
        }
    }
    fn struct_decl(name: &str, members: Vec<StructMember>) -> Stmt {
        st(StmtKind::StructDecl(StructDecl {
            span: sp(),
            name: name.to_string(),
            members,
        }))
    }
    fn lit_struct(name: &str, fields: Vec<FieldInit>) -> Expr {
        e(ExprKind::StructLit(StructLit {
            type_name: Some(name.to_string()),
            fields,
        }))
    }
    fn interp(parts: Vec<StrPart>) -> Expr {
        e(ExprKind::Interp(InterpString { parts }))
    }
    fn text(t: &str) -> StrPart {
        StrPart::Text(t.to_string())
    }
    fn iseg(x: Expr, spec: Option<&str>) -> StrPart {
        StrPart::Expr {
            expr: x,
            format_spec: spec.map(|s| s.to_string()),
        }
    }

    // ---- 运行辅助 ----

    fn run_val(stmts: Vec<Stmt>) -> Value {
        eval_module(&prog(stmts)).expect("求值应成功")
    }
    fn run_str(stmts: Vec<Stmt>) -> String {
        run_val(stmts).to_string()
    }
    fn err_class(stmts: Vec<Stmt>) -> &'static str {
        eval_module(&prog(stmts)).unwrap_err().class_name()
    }

    // ---- 1. 算术 / 比较 / 逻辑 ----

    #[test]
    fn arithmetic_precedence_and_string_concat() {
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Add, int(1), bin(BinaryOp::Mul, int(2), int(3))))]),
            "7"
        );
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Mul, bin(BinaryOp::Add, int(1), int(2)), int(3)))]),
            "9"
        );
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Add, sstr("a"), sstr("b")))]),
            "ab"
        );
        // `/` 恒 float（A3）。
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Div, int(10), int(4)))]),
            "2.5"
        );
        // int/float 混合算术 → 加宽为 float（§4.5.7）。
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Add, int(1), flt(0.5)))]),
            "1.5"
        );
        assert_eq!(run_str(vec![expr_stmt(neg(int(3)))]), "-3");
    }

    #[test]
    fn int_overflow_and_division_by_zero() {
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Add, e(ExprKind::Int(i64::MAX)), int(1)))]),
            "OverflowError"
        );
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Div, int(1), int(0)))]),
            "ZeroDivisionError"
        );
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Rem, int(1), int(0)))]),
            "ZeroDivisionError"
        );
        // 除零含 float（§4.5.6）。
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Div, flt(1.0), flt(0.0)))]),
            "ZeroDivisionError"
        );
    }

    #[test]
    fn modulo_python_semantics() {
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Rem, int(7), int(3)))]), "1");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Rem, neg(int(7)), int(3)))]), "2");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Rem, int(7), neg(int(3))))]), "-2");
    }

    #[test]
    fn comparisons_and_nan_ieee() {
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Lt, int(1), int(2)))]), "true");
        // int/float 精确比较（§4.5.7）。
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Eq, int(1), flt(1.0)))]), "true");
        assert_eq!(
            run_str(vec![expr_stmt(bin(
                BinaryOp::Eq,
                e(ExprKind::Int(9_007_199_254_740_993)),
                flt(9_007_199_254_740_992.0)
            ))]),
            "false"
        );
        // IEEE：涉及 NaN 的 `< <= > >=` 均 false；`NaN == NaN` false（§4.5.6）。
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Lt, flt(f64::NAN), int(1)))]), "false");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Ge, flt(f64::NAN), int(1)))]), "false");
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Eq, flt(f64::NAN), flt(f64::NAN)))]),
            "false"
        );
        // 字符串按 UTF-8 字节序。
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Lt, sstr("Z"), sstr("a")))]), "true");
        // 不可比较 → TypeError。
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Lt, int(1), sstr("a")))]),
            "TypeError"
        );
    }

    #[test]
    fn equality_is_deep_and_cycle_safe() {
        let a = arr(vec![int(1), arr(vec![int(2)])]);
        let b = arr(vec![int(1), arr(vec![int(2)])]);
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Eq, a, b))]), "true");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Ne, int(1), sstr("1")))]), "true");
    }

    #[test]
    fn logical_short_circuits() {
        // `false && (1/0 == 0)` 不触发除零（短路）。
        let deref = bin(BinaryOp::Eq, bin(BinaryOp::Div, int(1), int(0)), int(0));
        assert_eq!(
            run_str(vec![expr_stmt(logical(LogicalOp::And, bool_(false), deref.clone()))]),
            "false"
        );
        assert_eq!(
            run_str(vec![expr_stmt(logical(LogicalOp::Or, bool_(true), deref))]),
            "true"
        );
        // 非 bool 操作数 → TypeError。
        assert_eq!(
            err_class(vec![expr_stmt(logical(LogicalOp::And, int(1), bool_(true)))]),
            "TypeError"
        );
    }

    // ---- 2. if / while / for ----

    #[test]
    fn if_expression_yields_branch_value() {
        let stmts = vec![
            ldecl(
                "star",
                if_expr(
                    bool_(true),
                    vec![expr_stmt(sstr("yes"))],
                    Some(block(vec![expr_stmt(sstr("no"))])),
                ),
            ),
            expr_stmt(ident("star")),
        ];
        assert_eq!(run_str(stmts), "yes");

        // 条件非 bool → TypeError。
        assert_eq!(
            err_class(vec![expr_stmt(if_expr(int(1), vec![expr_stmt(int(0))], None))]),
            "TypeError"
        );
    }

    #[test]
    fn while_loop_accumulates() {
        let stmts = vec![
            decl("i", int(0)),
            decl("t", int(0)),
            st(StmtKind::While {
                cond: bin(BinaryOp::Lt, ident("i"), int(5)),
                body: block(vec![
                    assign_op("t", AssignOp::AddAssign, ident("i")),
                    assign_op("i", AssignOp::AddAssign, int(1)),
                ]),
            }),
            expr_stmt(ident("t")),
        ];
        assert_eq!(run_str(stmts), "10");
    }

    #[test]
    fn for_loop_with_break_and_continue() {
        // for i in [1..5] { if i == 3 { continue }; if i == 5 { break }; s += i } → 1+2+4 = 7
        let stmts = vec![
            decl("s", int(0)),
            st(StmtKind::For {
                var: "i".to_string(),
                iter: arr(vec![int(1), int(2), int(3), int(4), int(5)]),
                body: block(vec![
                    if_stmt(bin(BinaryOp::Eq, ident("i"), int(3)), vec![st(StmtKind::Continue)]),
                    if_stmt(bin(BinaryOp::Eq, ident("i"), int(5)), vec![st(StmtKind::Break)]),
                    assign_op("s", AssignOp::AddAssign, ident("i")),
                ]),
            }),
            expr_stmt(ident("s")),
        ];
        assert_eq!(run_str(stmts), "7");
    }

    #[test]
    fn for_over_struct_iterates_keys_sorted() {
        // 匿名 struct 的键按 UTF-8 字节序：b, a → 迭代序 a, b。
        let obj = e(ExprKind::StructLit(StructLit {
            type_name: None,
            fields: vec![field_init("b", int(2)), field_init("a", int(1))],
        }));
        let stmts = vec![
            decl("m", obj),
            decl("acc", sstr("")),
            st(StmtKind::For {
                var: "k".to_string(),
                iter: ident("m"),
                body: block(vec![assign_op("acc", AssignOp::AddAssign, ident("k"))]),
            }),
            expr_stmt(ident("acc")),
        ];
        assert_eq!(run_str(stmts), "ab");
        // 不可迭代类型 → TypeError（§4.5.4）。
        assert_eq!(
            err_class(vec![st(StmtKind::For {
                var: "x".to_string(),
                iter: int(1),
                body: block(vec![]),
            })]),
            "TypeError"
        );
    }

    // ---- 3. 函数 / 递归 / 闭包 ----

    #[test]
    fn factorial_recursion() {
        let body = Body::Expr(if_expr(
            bin(BinaryOp::Le, ident("n"), int(1)),
            vec![expr_stmt(int(1))],
            Some(block(vec![expr_stmt(bin(
                BinaryOp::Mul,
                ident("n"),
                call("fact", vec![bin(BinaryOp::Sub, ident("n"), int(1))]),
            ))])),
        ));
        let stmts = vec![
            fn_decl("fact", &["n"], body),
            expr_stmt(call("fact", vec![int(5)])),
        ];
        assert_eq!(run_str(stmts), "120");
    }

    #[test]
    fn closure_counter_shares_cell() {
        let inc = fn_decl(
            "inc",
            &[],
            Body::Block(block(vec![
                assign_op("n", AssignOp::AddAssign, int(1)),
                expr_stmt(ident("n")),
            ])),
        );
        let make = fn_decl(
            "makeCounter",
            &[],
            Body::Block(block(vec![decl("n", int(0)), inc, expr_stmt(ident("inc"))])),
        );
        let stmts = vec![
            make,
            ldecl("c", call("makeCounter", vec![])),
            expr_stmt(call("c", vec![])),
            expr_stmt(call("c", vec![])),
            expr_stmt(call("c", vec![])),
        ];
        // 三次调用共享同一 cell（A2）→ 3。
        assert_eq!(run_str(stmts), "3");
    }

    #[test]
    fn loop_iterations_get_independent_cells() {
        // var fns = []; for i in range(3) { fns = push(fn() => i, fns) }
        // 调用每个闭包应得各自当轮的 i（0,1,2）而非共享的 3（§4.5.3）。
        let mk = lambda(&[], Body::Expr(ident("i")));
        let stmts = vec![
            decl("fns", arr(vec![])),
            st(StmtKind::For {
                var: "i".to_string(),
                iter: call("range", vec![int(3)]),
                body: block(vec![assign_var("fns", call("push", vec![mk, ident("fns")]))]),
            }),
            decl("a", call_expr(index(ident("fns"), int(0)), vec![])),
            decl("b", call_expr(index(ident("fns"), int(1)), vec![])),
            decl("c", call_expr(index(ident("fns"), int(2)), vec![])),
            // a + b*10 + c*100 = 0 + 10 + 200 = 210
            expr_stmt(bin(
                BinaryOp::Add,
                ident("a"),
                bin(
                    BinaryOp::Add,
                    bin(BinaryOp::Mul, ident("b"), int(10)),
                    bin(BinaryOp::Mul, ident("c"), int(100)),
                ),
            )),
        ];
        assert_eq!(run_str(stmts), "210");
    }

    #[test]
    fn user_fn_arity_mismatch_is_type_error() {
        let f = fn_decl(
            "f",
            &["a", "b"],
            Body::Expr(bin(BinaryOp::Add, ident("a"), ident("b"))),
        );
        assert_eq!(
            err_class(vec![f, expr_stmt(call("f", vec![int(1)]))]),
            "TypeError"
        );
    }

    // ---- 4. 数组 / 结构体 / 字段 / 下标 ----

    #[test]
    fn array_index_read_and_negative() {
        assert_eq!(
            run_str(vec![expr_stmt(index(arr(vec![int(1), int(2), int(3)]), int(0)))]),
            "1"
        );
        assert_eq!(
            run_str(vec![expr_stmt(index(arr(vec![int(1), int(2), int(3)]), neg(int(1))))]),
            "3"
        );
        // 越界 → IndexError。
        assert_eq!(
            err_class(vec![expr_stmt(index(arr(vec![int(1)]), int(5)))]),
            "IndexError"
        );
    }

    #[test]
    fn a1_array_write_is_visible_through_alias() {
        let stmts = vec![
            ldecl("a", arr(vec![int(1), int(2), int(3)])),
            ldecl("b", ident("a")),
            assign_seg("b", vec![LvalueSegKind::Index(int(0))], int(99)),
            expr_stmt(bin(
                BinaryOp::Add,
                index(ident("a"), int(0)),
                index(ident("b"), int(0)),
            )),
        ];
        // a 与 b 指向同一 array（A1）：99 + 99 = 198。
        assert_eq!(run_str(stmts), "198");
    }

    #[test]
    fn a1_struct_field_write_through_alias_and_dynamic_add() {
        let stmts = vec![
            ldecl(
                "s",
                e(ExprKind::StructLit(StructLit {
                    type_name: None,
                    fields: vec![field_init("k", int(1))],
                })),
            ),
            ldecl("t", ident("s")),
            assign_seg("t", vec![LvalueSegKind::Field("k".to_string())], int(42)),
            // 动态新增字段（§4.5.8）。
            assign_seg("s", vec![LvalueSegKind::Field("z".to_string())], int(7)),
            expr_stmt(bin(
                BinaryOp::Add,
                field(ident("s"), "k"),
                field(ident("t"), "z"),
            )),
        ];
        assert_eq!(run_str(stmts), "49");
    }

    #[test]
    fn nested_index_write_through_path() {
        let stmts = vec![
            ldecl("m", arr(vec![arr(vec![int(1), int(2)]), arr(vec![int(3), int(4)])])),
            assign_seg(
                "m",
                vec![LvalueSegKind::Index(int(1)), LvalueSegKind::Index(int(0))],
                int(9),
            ),
            expr_stmt(index(index(ident("m"), int(1)), int(0))),
        ];
        assert_eq!(run_str(stmts), "9");
    }

    #[test]
    fn struct_template_method_binds_self() {
        let norm = StructMember::Method(FnDecl {
            span: sp(),
            name: "norm".to_string(),
            params: vec![],
            body: Body::Expr(bin(
                BinaryOp::Add,
                field(self_ref(), "x"),
                field(self_ref(), "y"),
            )),
        });
        let stmts = vec![
            struct_decl(
                "Point",
                vec![
                    StructMember::Field(field_init("x", int(0))),
                    StructMember::Field(field_init("y", int(0))),
                    norm,
                ],
            ),
            ldecl(
                "p",
                lit_struct("Point", vec![field_init("x", int(3)), field_init("y", int(4))]),
            ),
            expr_stmt(call_expr(field(ident("p"), "norm"), vec![])),
        ];
        assert_eq!(run_str(stmts), "7");
    }

    #[test]
    fn struct_template_defaults_are_reevaluated_per_instance() {
        // 默认值 `[]` 每实例各一份、不共享（§4.5.8）。
        let stmts = vec![
            struct_decl(
                "Bag",
                vec![
                    StructMember::Field(field_init("xs", arr(vec![]))),
                    StructMember::Method(FnDecl {
                        span: sp(),
                        name: "put".to_string(),
                        params: vec!["v".to_string()],
                        body: Body::Expr(call("push", vec![ident("v"), field(self_ref(), "xs")])),
                    }),
                ],
            ),
            ldecl("a", lit_struct("Bag", vec![])),
            ldecl("b", lit_struct("Bag", vec![])),
            assign_seg(
                "a",
                vec![LvalueSegKind::Field("xs".to_string())],
                call("push", vec![int(1), field(ident("a"), "xs")]),
            ),
            expr_stmt(call("len", vec![field(ident("b"), "xs")])),
        ];
        // 改 a.xs 不影响 b.xs（各一份默认数组）。
        assert_eq!(run_str(stmts), "0");
    }

    // ---- 5. 内置调用（≥3 个：len / push / str / range） ----

    #[test]
    fn builtins_len_push_str_range() {
        let stmts = vec![
            ldecl("xs", arr(vec![int(1), int(2), int(3)])),
            // push 返回新数组、不改原容器（A1）。
            ldecl("ys", call("push", vec![int(4), ident("xs")])),
            expr_stmt(bin(
                BinaryOp::Add,
                call("len", vec![ident("xs")]),
                bin(BinaryOp::Mul, call("len", vec![ident("ys")]), int(100)),
            )),
        ];
        // len(xs)=3 + len(ys)=4 * 100 = 403
        assert_eq!(run_str(stmts), "403");
        assert_eq!(run_str(vec![expr_stmt(call("str", vec![int(42)]))]), "42");
        assert_eq!(
            run_str(vec![expr_stmt(call("range", vec![int(4)]))]),
            "[0, 1, 2, 3]"
        );
    }

    #[test]
    fn builtin_call_span_is_call_site_and_argcount_checks() {
        // 内置参数个数不符 → TypeError（ArgCount），位置取调用点。
        assert_eq!(err_class(vec![expr_stmt(call("len", vec![]))]), "TypeError");
        // 未定义名字（非内置）→ NameError。
        assert_eq!(err_class(vec![expr_stmt(ident("nope"))]), "NameError");
        // 调用非函数 → TypeError。
        assert_eq!(
            err_class(vec![expr_stmt(call_expr(int(1), vec![]))]),
            "TypeError"
        );
    }

    // ---- 6. 插值 / 格式说明符 ----

    #[test]
    fn interpolation_plain_and_format_spec() {
        let stmts = vec![ldecl("n", int(42)), expr_stmt(interp(vec![iseg(ident("n"), None)]))];
        assert_eq!(run_str(stmts), "42");

        assert_eq!(
            run_str(vec![
                ldecl("n", int(42)),
                expr_stmt(interp(vec![iseg(ident("n"), Some("05d"))]))
            ]),
            "00042"
        );
        assert_eq!(
            run_str(vec![expr_stmt(interp(vec![iseg(flt(3.14159), Some(".2f"))]))]),
            "3.14"
        );
        assert_eq!(
            run_str(vec![
                ldecl("n", int(42)),
                expr_stmt(interp(vec![
                    text("a"),
                    iseg(ident("n"), None),
                    text("b")
                ]))
            ]),
            "a42b"
        );
        // 格式说明符语法非法 → ValueError。
        assert_eq!(
            err_class(vec![expr_stmt(interp(vec![iseg(int(1), Some("zz"))]))]),
            "ValueError"
        );
    }

    // ---- 7. 控制流表达式的块值 ----

    #[test]
    fn function_body_block_value_is_last_expr() {
        // fn f() { 1; 2 } → 2（块值 = 最后一条表达式语句）。
        let f = fn_decl(
            "f",
            &[],
            Body::Block(block(vec![expr_stmt(int(1)), expr_stmt(int(2))])),
        );
        assert_eq!(run_str(vec![f, expr_stmt(call("f", vec![]))]), "2");
        // fn g() { if true { 7 } else { 8 } } → 7（`if` 收尾为块值）。
        let g = fn_decl(
            "g",
            &[],
            Body::Block(block(vec![st(StmtKind::If(if_expr(
                bool_(true),
                vec![expr_stmt(int(7))],
                Some(block(vec![expr_stmt(int(8))])),
            )))])),
        );
        assert_eq!(run_str(vec![g, expr_stmt(call("g", vec![]))]), "7");
    }

    #[test]
    fn return_propagates_out_of_loops_and_branches() {
        let f = fn_decl(
            "find",
            &[],
            Body::Block(block(vec![
                st(StmtKind::For {
                    var: "i".to_string(),
                    iter: arr(vec![int(1), int(2), int(3)]),
                    body: block(vec![if_stmt(
                        bin(BinaryOp::Eq, ident("i"), int(2)),
                        vec![st(StmtKind::Return(Some(ident("i"))))],
                    )]),
                }),
                expr_stmt(int(0)),
            ])),
        );
        assert_eq!(run_str(vec![f, expr_stmt(call("find", vec![]))]), "2");
    }

    // ---- 8. 位置红线：运行时错误必须带节点 span（非 `Span::START`） ----

    #[test]
    fn runtime_errors_carry_node_span() {
        let at = Span::new(5, 3);
        let bad = Spanned::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(int(1)),
                right: Box::new(sstr("a")),
            },
            at,
        );
        let err = eval_module(&prog(vec![Spanned::new(StmtKind::Expr(bad), at)])).unwrap_err();
        assert_eq!(err.class_name(), "TypeError");
        assert_eq!(err.span(), Some(at));
    }
}
