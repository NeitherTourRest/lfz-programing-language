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
//! # P3.8 语义定稿（本节已落地）
//!
//! - §4.5 确定性八项：B1 求值序、B2 `for` 快照、B3 键字节序、B4 递归/深结构上限、B6 IEEE、
//!   B7 平拷贝、B13 `int→float` 加宽（唯一入口 [`Value::as_f64`]）。
//! - A4：`check` 非致命（stderr 一行 + 返回 `false` + 继续）；`assert` / `fail` 致命。
//! - A5 / A6：struct 数据面排除方法字段；`==` / `!=` 复用 [`Value::deep_eq_bounded`]（环安全 + 精确比较）。
//! - `RecursionError`（[`RECURSION_LIMIT`] = 10000；调用帧与深结构两条路径）。
//! - `;;`（[`StmtKind::Dump`]）：沿可见链内→外 / 同层 slot 升序 / 遮蔽去重，渲染为纯函数
//!   [`render_dump`]，写 **stdout**。
//! - traceback：[`eval_module_traced`] 按 §10.3 N1 组装帧栈（自最外层→最内层）。
//!
//! # 求值线程（栈深）
//!
//! 树遍历器实现 10000 层递归所需栈远超主线程默认值，故 [`eval_module_traced`] 在
//! **专用大栈线程**（[`EVAL_STACK_SIZE`]）上求值，结果经 [`Transfer`] 在 `join` 边界移交。
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
    div_zero, field as field_error, index as index_error, io as io_error, name as name_error,
    overflow, recursion, type_error, value as value_error, LzError, OverflowMsg, R, TraceFrame,
    TypeMsg, ValueMsg,
};
use crate::span::Span;
use crate::value::{Closure, StructObj, UserFn, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::io::Write;
use std::rc::Rc;

// ===========================================================================
// 公开常量 / 结果
// ===========================================================================

/// 求值帧与深结构处理的**递归深度上限**（§4.5.5：默认 10000；超过 → `RecursionError`）。
pub const RECURSION_LIMIT: u32 = 10_000;

/// 模块顶层帧的 `func_id`（traceback 显示名恒为 `<module>`，§10.3 / §8.4）。
const MODULE_FUNC_ID: u32 = 0;

/// 求值专用线程的栈大小（§4.5.5）。
///
/// 树遍历器实现 10000 层递归需要远超主线程默认栈（Windows 常见 1–8 MiB）的空间，
/// 故求值在**专用大栈线程**上进行（见 [`eval_module_traced`]）。
const EVAL_STACK_SIZE: usize = 256 * 1024 * 1024;

/// 跨线程移交载荷：`join` 提供 happens-before，移交前后**同一时刻只有一个线程**访问其内容，
/// 故不存在数据竞争（`Rc` 的非原子计数不会被并发触碰）。
struct Transfer<T>(T);

// SAFETY: `Transfer` 仅在 `std::thread::scope` 的 `join` 边界处移交：生产线程在返回前不再
// 访问该值，消费线程在 `join` 返回后才访问；`join` 建立 happens-before，二者不并发。
unsafe impl<T> Send for Transfer<T> {}

/// 带 traceback 的求值结果（§10.3 / §8.2）。
///
/// 错误模型 `LzError`（`error.rs`，只读）**不携带**帧栈字段，故帧栈由本结构在**错误发生时**
/// 自最外层→最内层带出，供 CLI / tooling 组装 `Traceback (most recent call last):` 输出。
pub struct TracedRun {
    /// 求值结果（最后一条语句的值；出错则为 `Err`）。
    pub result: R<Value>,
    /// 帧栈，**最外层 → 最内层**（§10.3）：首帧恒为模块帧 `<module>`，末帧为出错点所在帧。
    pub frames: Vec<TraceFrame>,
    /// `func_id → 函数名`：索引 0 = 模块；`Some(名)` = 命名函数；`None` = 匿名 `<fn>`。
    func_names: Vec<Option<Rc<str>>>,
}

impl TracedRun {
    /// 该帧在 traceback 中的**显示名**（§8.4）：`<module>` / 函数名 / `<fn>`。
    #[must_use]
    pub fn frame_name(&self, frame: &TraceFrame) -> Rc<str> {
        if frame.func_id == MODULE_FUNC_ID {
            return Rc::from("<module>");
        }
        match self.func_names.get(frame.func_id as usize) {
            Some(Some(n)) => Rc::clone(n),
            _ => Rc::from("<fn>"),
        }
    }
}

/// 渲染 `;;` 的可见变量（**纯函数**，便于单测；§3.6）。
///
/// - `entries` 须**已按 §3.6 排序**（作用域内→外、同层 slot 升序、遮蔽去重）；
/// - 行格式（逐字符精确）：`<name>` + `U+0020` + `U+FF1A`（全角冒号）+ `U+0020` + `<value>` + `\n`；
/// - 值渲染复用 §3.7 显示形式（`Value::to_string()`）；
/// - 空链 ⇒ **空串**（零行，不报错）。
#[must_use]
pub fn render_dump(entries: &[(Rc<str>, Value)]) -> String {
    let mut out = String::new();
    for (name, value) in entries {
        out.push_str(name);
        out.push_str(" ： ");
        out.push_str(&value.to_string());
        out.push('\n');
    }
    out
}

/// 函数体「首节点」`Span`（§10.3：压入帧时的初始 span）。
fn body_first_span(body: &Body) -> Span {
    match body {
        Body::Block(b) => b.stmts.first().map_or(b.span, |s| s.span),
        Body::Expr(e) => e.span,
    }
}

// ===========================================================================
// 公开入口
// ===========================================================================

/// 求值一个编译单元（`Program`），返回**最后一条语句的值**（无语句 → `nil`）。
///
/// 说明：AST 类型名为 `Program`（非 `Module`）；本函数即任务书所称 `eval_module`。
/// 顶层作用域即模块作用域（globals），不创建额外子作用域。
pub fn eval_module(program: &Program) -> R<Value> {
    eval_module_traced(program).result
}

/// 执行一个编译单元并**带出 traceback 帧栈**（§10.3 / §8.2）。
///
/// 帧栈在求值过程中按 §10.3 N1 维护：**进入一次 `Call` 时先把当前帧 span 更新为调用点，
/// 再压入新帧**；出错时帧栈即自**最外层→最内层**的 traceback。`TracedRun::frame_name`
/// 按 `func_id` 反查显示名（`<module>` / 函数名 / `<fn>`，§8.4）。
///
/// 求值在**专用大栈线程**（[`EVAL_STACK_SIZE`]）上进行，以保证 §4.5.5 的 10000 层递归上限
/// 可达且不会因耗尽调用方栈而崩溃；`TracedRun` 经 [`Transfer`] 在 `join` 边界移交。
///
/// `result` 为最后一条语句的值（顶层 `return` 防御性接受；无语句 → `nil`）。
pub fn eval_module_traced(program: &Program) -> TracedRun {
    std::thread::scope(|scope| {
        let handle = match std::thread::Builder::new()
            .stack_size(EVAL_STACK_SIZE)
            .spawn_scoped(scope, || Transfer(eval_module_on_thread(program)))
        {
            Ok(h) => h,
            // 线程创建失败（OS 资源不足）时退化为当前栈求值（仍受逻辑深度上限保护）。
            Err(_) => return eval_module_on_thread(program),
        };
        match handle.join() {
            Ok(transfer) => transfer.0,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

/// [`eval_module_traced`] 的实际求值体（在求值线程上运行）。
fn eval_module_on_thread(program: &Program) -> TracedRun {
    let mut interp = Interp::new();
    let top = Scope {
        env: Rc::clone(&interp.globals),
        captured: Rc::new(Vec::new()),
        self_val: None,
    };
    // 模块帧：初始 span = 首条语句 span（空程序回退到 `Program.span`）。
    let init_span = program.stmts.first().map_or(program.span, |s| s.span);
    interp.trace.push(TraceFrame {
        func_id: MODULE_FUNC_ID,
        span: init_span,
    });
    let result = match interp.exec_stmts(&program.stmts, &top) {
        Ok(Flow::Value(v)) | Ok(Flow::Return(v)) => Ok(v),
        // 顶层 `break` / `continue` 由 parser 拒绝；此处防御性接受。
        Ok(Flow::Break | Flow::Continue) => Ok(Value::Nil),
        Err(e) => Err(e),
    };
    TracedRun {
        result,
        frames: std::mem::take(&mut interp.trace),
        func_names: std::mem::take(&mut interp.func_names),
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

/// 树遍历解释器：模块全局环境 + 单调递增的 `func_id` 分配器 + traceback 帧栈。
struct Interp {
    /// 模块顶层作用域（globals）；所有函数调用帧直接挂在它之下。
    globals: Rc<RefCell<Env>>,
    /// 函数表下标分配器（traceback 用）；`1` 起，`0` 保留给模块帧（§10.3）。
    next_func_id: u32,
    /// 当前求值帧深度（用户函数调用层数，§4.5.5）。
    depth: u32,
    /// traceback 帧栈：首元素为模块帧，末元素为当前帧（§10.3）。
    trace: Vec<TraceFrame>,
    /// `func_id → 函数名`；索引 0 = 模块（`None` 占位）。
    func_names: Vec<Option<Rc<str>>>,
}

impl Interp {
    fn new() -> Self {
        Self {
            globals: Env::root(),
            next_func_id: 1,
            depth: 0,
            trace: Vec::new(),
            func_names: vec![None], // [0] = 模块帧（`<module>`）
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
    // `;;` 可见链（§3.6）
    // -----------------------------------------------------------------------

    /// 收集当前**可见名字链**（§3.6）：作用域链**内→外**、同层按**声明序 = slot 升序**、
    /// **遮蔽去重**（同名只取最内层，每名字恰好一行）。
    ///
    /// 顺序 = 名字解析顺序：块 / 函数帧链（不含 globals，内→外）→ 闭包捕获 cell → 模块顶层。
    /// 与 [`crate::env::ScopeDebugTable::visible_slots`]（契约 §10.5 的 `ScopeDebug` 链遍历）
    /// 语义一致：运行时 `Env` 的 per-scope 名字表以**声明序**组织（即 slot 升序），本方法沿
    /// 词法链内→外行走并去重，等价于沿 `ScopeDebug` 的 `parent` 链内→外遍历。
    fn visible_entries(&self, scope: &Scope) -> Vec<(Rc<str>, Value)> {
        let mut seen: Vec<String> = Vec::new();
        let mut out: Vec<(Rc<str>, Value)> = Vec::new();

        // 1) 块 / 函数帧链（内→外），不含 globals。
        let mut cur = Some(Rc::clone(&scope.env));
        while let Some(e) = cur {
            if Rc::ptr_eq(&e, &self.globals) {
                break;
            }
            let bindings = e.borrow().bindings(); // 声明序（slot 升序）
            for (pos, (n, idx)) in bindings.iter().enumerate() {
                // 同作用域同名：只取**最后**定义（有效遮蔽，与 `Env::local_index` 一致）。
                let is_last = !bindings[pos + 1..].iter().any(|(m, _)| m == n);
                if is_last && !seen.iter().any(|s| s == n) {
                    if let Some(v) = e.borrow().get_local(*idx) {
                        seen.push(n.clone());
                        out.push((Rc::from(n.as_str()), v));
                    }
                }
            }
            cur = e.borrow().parent();
        }

        // 2) 闭包捕获 cell（定义作用域的外层自由变量）。
        for (n, c) in scope.captured.iter() {
            if !seen.iter().any(|s| s == n.as_ref()) {
                seen.push(n.to_string());
                out.push((Rc::clone(n), c.borrow().clone()));
            }
        }

        // 3) 模块顶层（最外层）。
        let gbind = self.globals.borrow().bindings();
        for (pos, (n, idx)) in gbind.iter().enumerate() {
            let is_last = !gbind[pos + 1..].iter().any(|(m, _)| m == n);
            if is_last && !seen.iter().any(|s| s == n) {
                if let Some(v) = self.globals.borrow().get_local(*idx) {
                    seen.push(n.clone());
                    out.push((Rc::from(n.as_str()), v));
                }
            }
        }
        out
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
        // 函数表：`func_id → 名字`（`None` = 匿名 `<fn>`），报错时反查（§10.3）。
        self.func_names.push(name.map(Rc::from));
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
            // `;;`：输出当前**可见名字链**（§3.6：内→外、同层 slot 升序、遮蔽去重）。
            // 渲染逻辑为纯函数 [`render_dump`]，本处只负责收集 + 打印到 stdout。
            StmtKind::Dump { .. } => {
                let entries = self.visible_entries(scope);
                let text = render_dump(&entries);
                if !text.is_empty() {
                    let mut out = std::io::stdout();
                    out.write_all(text.as_bytes())
                        .map_err(|e| io_error(format!("无法写入：{e}"), Some(stmt.span)))?;
                }
                Ok(Flow::Value(Value::Nil))
            }
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
                    //
                    // P3.9b：高阶内置（map/filter/reduce/sortBy/minBy/maxBy/each）需要「调用用户函数」
                    // 的能力——注入一个经 [`Interp::call_value`] 调用回调的闭包（窄 ABI，见 builtins
                    // 的 `Invoke`）。
                    let mut invoke = |f: &Value, a: &[Value], s: Span| -> R<Value> {
                        self.call_value(f, a, s)
                    };
                    return Ok(Flow::Value(builtins::call_with(
                        name, &argv, span, &mut invoke,
                    )?));
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

    /// 调用一个**函数值**并解包结果（P3.9b 高阶内置回调的注入点，见 `builtins::Invoke`）。
    ///
    /// 与 [`Interp::call_func`] 同口径：非函数 → `TypeError::NotCallable`；函数值 → [`Interp::call_user`]
    /// （其 `Return` / 函数体末值 / `Break`/`Continue` 已在 `call_user` 内收口为 `R<Value>`）。
    /// 位置取**回调调用点**（内置的 `Call` 节点 span），满足「运行时错误必须带位置」。
    fn call_value(&mut self, callee: &Value, args: &[Value], span: Span) -> R<Value> {
        match callee {
            Value::Func(cl) => self.call_user(cl, args.to_vec(), span, None),
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
        // §4.5.5：求值帧深度上限（默认 10000；含顶层帧，超过 → RecursionError）。
        if self.depth >= RECURSION_LIMIT {
            return Err(recursion(self.depth + 1, RECURSION_LIMIT, span));
        }
        self.depth += 1;
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
        // §10.3 N1：进入 Call **先把当前帧 span 更新为调用点**，再压入新帧；
        // 新帧初始 span = 函数体首节点 span。
        if let Some(top) = self.trace.last_mut() {
            top.span = span;
        }
        self.trace.push(TraceFrame {
            func_id: user.func_id,
            span: body_first_span(&user.body),
        });
        let outcome = match &user.body {
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
        };
        // 仅**成功**返回时弹帧 / 退深度；出错时经 `?` 提前返回，帧栈保留供 traceback（§8.2）。
        self.trace.pop();
        self.depth -= 1;
        outcome
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
        // A6 深结构相等（`value.rs` 唯一共享实现；深度受 §4.5.5 上限约束）。
        BinaryOp::Eq => eq_values(&l, &r, false, span),
        BinaryOp::Ne => eq_values(&l, &r, true, span),
    }
}

/// `==` / `!=`：复用 [`Value::deep_eq_bounded`]（A6 环安全 + 数据面 + 精确比较）；
/// 深结构超限 → `RecursionError`（§4.5.5）。
fn eq_values(l: &Value, r: &Value, negate: bool, span: Span) -> R<Value> {
    match l.deep_eq_bounded(r, RECURSION_LIMIT) {
        Ok(eq) => Ok(Value::Bool(eq ^ negate)),
        Err(depth) => Err(recursion(depth, RECURSION_LIMIT, span)),
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
    use crate::env::ScopeId;

    /// 端到端：`lex` → `parse` → `eval_module`（parser 当前最小子集）。
    fn eval_src(src: &str) -> R<Value> {
        let tokens = crate::lexer::lex(src, 1)?;
        let program = crate::parser::parse(&tokens)?;
        eval_module(&program)
    }

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

    // =======================================================================
    // P3.8 — §4.5 确定性八项 / A4 / A5 / A6 / Recursion / `;;` / traceback
    // =======================================================================

    /// §4.5.1（B1）：二元运算先左后右；赋值先求下标表达式、再求 rhs。用带副作用的 `mark` 观测。
    #[test]
    fn b1_evaluation_order_is_left_to_right() {
        // `fn mark(n) { log = push(n, log); n }`
        let mark = || {
            fn_decl(
                "mark",
                &["n"],
                Body::Block(block(vec![
                    assign_var("log", call("push", vec![ident("n"), ident("log")])),
                    expr_stmt(ident("n")),
                ])),
            )
        };
        // 二元 `mark(1) + mark(2)`：先左后右 → log = [1, 2]。
        let stmts = vec![
            decl("log", arr(vec![])),
            mark(),
            expr_stmt(bin(
                BinaryOp::Add,
                call("mark", vec![int(1)]),
                call("mark", vec![int(2)]),
            )),
            expr_stmt(ident("log")),
        ];
        assert_eq!(run_str(stmts), "[1, 2]");

        // 赋值 `a[mark(0)] = mark(9)`：先求 `a` 与下标 `mark(0)`，再求 rhs `mark(9)` → log = [0, 9]。
        let stmts = vec![
            decl("log", arr(vec![])),
            mark(),
            ldecl("a", arr(vec![int(0), int(0)])),
            assign_seg(
                "a",
                vec![LvalueSegKind::Index(call("mark", vec![int(0)]))],
                call("mark", vec![int(9)]),
            ),
            expr_stmt(ident("log")),
        ];
        assert_eq!(run_str(stmts), "[0, 9]");
    }

    /// §4.5.4（B2）：`for` 迭代开始时取快照，迭代中改长度不影响次数。
    #[test]
    fn b2_for_iterates_over_snapshot() {
        let stmts = vec![
            decl("xs", arr(vec![int(1), int(2), int(3)])),
            decl("s", int(0)),
            st(StmtKind::For {
                var: "v".to_string(),
                iter: ident("xs"),
                body: block(vec![
                    // 首轮改 `xs`（push 返回新数组 → 重绑定），不应影响本次迭代。
                    if_stmt(
                        bin(BinaryOp::Eq, ident("v"), int(1)),
                        vec![assign_var("xs", call("push", vec![int(4), ident("xs")]))],
                    ),
                    assign_op("s", AssignOp::AddAssign, ident("v")),
                ]),
            }),
            // s = 1+2+3 = 6（快照）；迭代后 len(xs) = 4 → 6*10 + 4 = 64
            expr_stmt(bin(
                BinaryOp::Add,
                bin(BinaryOp::Mul, ident("s"), int(10)),
                call("len", vec![ident("xs")]),
            )),
        ];
        assert_eq!(run_str(stmts), "64");
    }

    /// §4.5.4（B3）：struct 键序 = UTF-8 字节序升序（`keys` 与 `for` 一致）。
    #[test]
    fn b3_struct_keys_are_byte_order_sorted() {
        let obj = || {
            e(ExprKind::StructLit(StructLit {
                type_name: None,
                fields: vec![
                    field_init("b", int(2)),
                    field_init("a", int(1)),
                    field_init("A", int(0)),
                ],
            }))
        };
        let stmts = vec![ldecl("m", obj()), expr_stmt(call("keys", vec![ident("m")]))];
        // 'A'(0x41) < 'a'(0x61) < 'b'(0x62)。
        assert_eq!(run_str(stmts), "[\"A\", \"a\", \"b\"]");

        let stmts = vec![
            ldecl("m", obj()),
            decl("acc", sstr("")),
            st(StmtKind::For {
                var: "k".to_string(),
                iter: ident("m"),
                body: block(vec![assign_op("acc", AssignOp::AddAssign, ident("k"))]),
            }),
            expr_stmt(ident("acc")),
        ];
        assert_eq!(run_str(stmts), "Aab");
    }

    /// §4.5.5（B4）：深递归超过 10000 层 → `RecursionError`（不是栈溢出 / `OverflowError`）。
    #[test]
    fn b4_deep_recursion_yields_recursion_error() {
        let f = fn_decl(
            "loop",
            &["n"],
            Body::Block(block(vec![expr_stmt(call("loop", vec![ident("n")]))])),
        );
        let run = eval_module_traced(&prog(vec![
            f,
            expr_stmt(call("loop", vec![int(1)])),
        ]));
        match run.result {
            Ok(v) => panic!("应递归超限，得到 {v}"),
            Err(e) => {
                assert_eq!(e.class_name(), "RecursionError");
                assert_eq!(e.message(), "递归深度超限（超过 10000 层）");
                match e.as_ref() {
                    LzError::Recursion { depth, limit, .. } => {
                        assert_eq!(*limit, RECURSION_LIMIT);
                        assert_eq!(*depth, RECURSION_LIMIT + 1);
                    }
                    other => panic!("应为 Recursion，得到 {}", other.class_name()),
                }
            }
        }
    }

    /// §4.5.6（B6）：IEEE 规则——`NaN` 比较、`±Inf`、除零不产 `Inf`、显示形式。
    #[test]
    fn b6_float_ieee_rules() {
        let nan = || call("float", vec![sstr("nan")]);
        let inf = || call("float", vec![sstr("inf")]);
        // NaN != NaN；涉及 NaN 的 `< <= > >=` 均 false。
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Eq, nan(), nan()))]), "false");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Ne, nan(), nan()))]), "true");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Lt, nan(), flt(1.0)))]), "false");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Ge, nan(), flt(1.0)))]), "false");
        // Inf 比较：+Inf 大于任何有限值；Inf == Inf。
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Gt, inf(), flt(1.0)))]), "true");
        assert_eq!(run_str(vec![expr_stmt(bin(BinaryOp::Eq, inf(), inf()))]), "true");
        // 除零（含 float）→ ZeroDivisionError，不产 Inf。
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Div, flt(1.0), flt(0.0)))]),
            "ZeroDivisionError"
        );
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Rem, flt(1.0), flt(0.0)))]),
            "ZeroDivisionError"
        );
        // 显示：nan / inf / -inf（§3.7）。
        assert_eq!(run_str(vec![expr_stmt(nan())]), "nan");
        assert_eq!(run_str(vec![expr_stmt(inf())]), "inf");
        assert_eq!(
            run_str(vec![expr_stmt(call("float", vec![sstr("-inf")]))]),
            "-inf"
        );
    }

    /// §4.5.8（B7）：struct 实例化 = 平拷贝（可变默认值每实例各一份；覆盖 / 动态新增）。
    #[test]
    fn b7_struct_instantiation_is_flat_copy() {
        let stmts = vec![
            struct_decl(
                "Bag",
                vec![
                    StructMember::Field(field_init("xs", arr(vec![]))),
                    StructMember::Field(field_init("n", int(0))),
                ],
            ),
            // a 覆盖 n=5、动态新增 z=9；b 用默认。
            ldecl(
                "a",
                lit_struct(
                    "Bag",
                    vec![field_init("n", int(5)), field_init("z", int(9))],
                ),
            ),
            ldecl("b", lit_struct("Bag", vec![])),
            // 改 a.xs（push 返回新数组写回）→ b.xs 仍空（默认各一份，不共享）。
            assign_seg(
                "a",
                vec![LvalueSegKind::Field("xs".to_string())],
                call("push", vec![int(1), field(ident("a"), "xs")]),
            ),
            // a.n + a.z + len(b.xs) = 5 + 9 + 0 = 14
            expr_stmt(bin(
                BinaryOp::Add,
                bin(
                    BinaryOp::Add,
                    field(ident("a"), "n"),
                    field(ident("a"), "z"),
                ),
                call("len", vec![field(ident("b"), "xs")]),
            )),
        ];
        assert_eq!(run_str(stmts), "14");
    }

    /// §4.5.7（B13）：`int → float` 加宽是唯一隐式转换；混合算术为 float；精确比较不误判。
    #[test]
    fn b13_int_float_widening_is_the_only_implicit_conversion() {
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Add, int(1), flt(0.5)))]),
            "1.5"
        );
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Mul, int(2), flt(0.25)))]),
            "0.5"
        );
        // 精确比较（§4.5.7）：大整数不因加宽丢精度而误判相等。
        assert_eq!(
            run_str(vec![expr_stmt(bin(
                BinaryOp::Eq,
                e(ExprKind::Int(9_007_199_254_740_993)),
                flt(9_007_199_254_740_992.0)
            ))]),
            "false"
        );
        // 无其它隐式转换：`int + string` → TypeError。
        assert_eq!(
            err_class(vec![expr_stmt(bin(BinaryOp::Add, int(1), sstr("a")))]),
            "TypeError"
        );
    }

    /// §4.5.10（A4）：`check` 非致命（返回 false 且**继续执行**）；`assert` / `fail` 致命。
    #[test]
    fn a4_check_nonfatal_assert_and_fail_fatal() {
        // check(true) → true。
        assert_eq!(
            run_str(vec![expr_stmt(call("check", vec![bool_(true)]))]),
            "true"
        );
        // check(false) → false；后续语句仍执行（继续）。
        assert_eq!(
            run_str(vec![
                ldecl("r", call("check", vec![bool_(false), sstr("soft")])),
                decl("after", int(7)),
                expr_stmt(ident("after")),
            ]),
            "7"
        );
        assert_eq!(
            run_str(vec![
                ldecl("r", call("check", vec![bool_(false)])),
                expr_stmt(ident("r")),
            ]),
            "false"
        );
        // check 非 bool → TypeError。
        assert_eq!(
            err_class(vec![expr_stmt(call("check", vec![int(1)]))]),
            "TypeError"
        );

        // assert(true) → nil（继续）；assert(false) → AssertionError（致命）。
        assert_eq!(
            run_str(vec![expr_stmt(call("assert", vec![bool_(true)]))]),
            "nil"
        );
        let e = eval_module(&prog(vec![expr_stmt(call(
            "assert",
            vec![bool_(false)],
        ))]))
        .unwrap_err();
        assert_eq!(e.class_name(), "AssertionError");
        assert_eq!(e.message(), "断言失败");
        let e = eval_module(&prog(vec![expr_stmt(call(
            "assert",
            vec![bool_(false), sstr("boom")],
        ))]))
        .unwrap_err();
        assert_eq!(e.class_name(), "AssertionError");
        assert_eq!(e.message(), "断言失败：boom");

        // fail() → AssertionError，消息 `fail()`；fail(msg) → `{msg}`。
        let e = eval_module(&prog(vec![expr_stmt(call("fail", vec![]))])).unwrap_err();
        assert_eq!(e.class_name(), "AssertionError");
        assert_eq!(e.message(), "fail()");
        let e = eval_module(&prog(vec![expr_stmt(call("fail", vec![sstr("boom")]))])).unwrap_err();
        assert_eq!(e.message(), "boom");
    }

    /// §4.5.9（A5）：struct 数据面（`len` / `keys` / `has` / display / `==`）**不含**方法字段。
    #[test]
    fn a5_struct_data_plane_excludes_method_fields() {
        let def = || {
            struct_decl(
                "P",
                vec![
                    StructMember::Field(field_init("x", int(0))),
                    StructMember::Method(FnDecl {
                        span: sp(),
                        name: "m".to_string(),
                        params: vec![],
                        body: Body::Expr(int(1)),
                    }),
                ],
            )
        };
        let with_m = || lit_struct("P", vec![field_init("x", int(1))]);
        let anon = || {
            e(ExprKind::StructLit(StructLit {
                type_name: None,
                fields: vec![field_init("x", int(1))],
            }))
        };
        // `==` 忽略方法字段：带方法实例 == 无方法匿名 struct（数据面相同）。
        assert_eq!(
            run_str(vec![
                def(),
                ldecl("a", with_m()),
                ldecl("b", anon()),
                expr_stmt(bin(BinaryOp::Eq, ident("a"), ident("b"))),
            ]),
            "true"
        );
        // len / keys 仅数据字段。
        assert_eq!(
            run_str(vec![
                def(),
                ldecl("a", with_m()),
                expr_stmt(call("len", vec![ident("a")])),
            ]),
            "1"
        );
        assert_eq!(
            run_str(vec![
                def(),
                ldecl("a", with_m()),
                expr_stmt(call("keys", vec![ident("a")])),
            ]),
            "[\"x\"]"
        );
        // display 仅数据字段。
        assert_eq!(
            run_str(vec![def(), ldecl("a", with_m()), expr_stmt(ident("a"))]),
            "{x: 1}"
        );
        // has：方法名 → false；数据字段 → true。
        assert_eq!(
            run_str(vec![
                def(),
                ldecl("a", with_m()),
                expr_stmt(call("has", vec![sstr("m"), ident("a")])),
            ]),
            "false"
        );
        assert_eq!(
            run_str(vec![
                def(),
                ldecl("a", with_m()),
                expr_stmt(call("has", vec![sstr("x"), ident("a")])),
            ]),
            "true"
        );
    }

    /// §4.5.9（A6）：`==` 环安全（重访即相等）+ 身份优先；`!=` 取反；不报错、不死循环。
    #[test]
    fn a6_equality_is_cycle_safe_and_identity_first() {
        // `a = [0]; a[0] = a` → a = [a]（自引用环）；b 同构。
        let mk = |name: &str| -> Vec<Stmt> {
            vec![
                ldecl(name, arr(vec![int(0)])),
                assign_seg(name, vec![LvalueSegKind::Index(int(0))], ident(name)),
            ]
        };
        let mut stmts = mk("a");
        stmts.extend(mk("b"));
        stmts.push(expr_stmt(bin(BinaryOp::Eq, ident("a"), ident("b"))));
        assert_eq!(run_str(stmts), "true");

        // 身份优先：a == a → true；a != a → false。
        assert_eq!(
            run_str(vec![
                ldecl("a", arr(vec![int(1)])),
                expr_stmt(bin(BinaryOp::Eq, ident("a"), ident("a"))),
            ]),
            "true"
        );
        assert_eq!(
            run_str(vec![
                ldecl("a", arr(vec![int(1)])),
                expr_stmt(bin(BinaryOp::Ne, ident("a"), ident("a"))),
            ]),
            "false"
        );
    }

    /// §3.6：`render_dump` 为纯函数；行格式 `名字 ： 值`（全角冒号）；空链 → 空串。
    #[test]
    fn dump_render_is_pure_and_formats_lines() {
        assert_eq!(render_dump(&[]), "");
        let entries = vec![
            (Rc::from("x"), Value::Int(2)),
            (Rc::from("s"), Value::string("hi")),
            (Rc::from("b"), Value::Bool(true)),
        ];
        assert_eq!(render_dump(&entries), "x ： 2\ns ： hi\nb ： true\n");
        // 分隔串 = U+0020 + U+FF1A（全角冒号）+ U+0020，逐字符断言。
        let one = render_dump(&[(Rc::from("k"), Value::Int(1))]);
        let cs: Vec<char> = one.chars().collect();
        assert_eq!(
            cs,
            vec!['k', ' ', '\u{ff1a}', ' ', '1', '\n'],
            "行格式必须是 `<name> ： <value>`"
        );
    }

    /// §3.6：可见链**内→外**、同层**声明序（slot 升序）**、**遮蔽去重**；含捕获与全局。
    #[test]
    fn dump_visible_entries_inner_to_outer_shadow_dedup() {
        let interp = Interp::new();
        interp
            .globals
            .borrow_mut()
            .define_named("x", Value::Int(1), false);
        interp
            .globals
            .borrow_mut()
            .define_named("g", Value::Int(9), false);
        let child = Env::child_after(&interp.globals);
        child.borrow_mut().define_named("x", Value::Int(2), false);
        child.borrow_mut().define_named("y", Value::Int(3), false);
        let cell: Cell = Rc::new(RefCell::new(Value::Int(7)));
        let scope = Scope {
            env: Rc::clone(&child),
            captured: Rc::new(vec![(Rc::from("z"), cell)]),
            self_val: None,
        };
        let entries = interp.visible_entries(&scope);
        let got: Vec<(String, String)> = entries
            .iter()
            .map(|(n, v)| (n.to_string(), v.to_string()))
            .collect();
        // 内层 x（遮蔽全局 x）、内层 y、捕获 z、全局 g；全局 x 被去重。
        assert_eq!(
            got,
            vec![
                ("x".to_string(), "2".to_string()),
                ("y".to_string(), "3".to_string()),
                ("z".to_string(), "7".to_string()),
                ("g".to_string(), "9".to_string()),
            ]
        );
        assert_eq!(render_dump(&entries), "x ： 2\ny ： 3\nz ： 7\ng ： 9\n");
    }

    /// `;;`（Dump）语句可执行、返回 `nil`（输出到 stdout；内容由纯函数测试覆盖）。
    #[test]
    fn dump_statement_runs_and_returns_nil() {
        let stmts = vec![
            decl("x", int(1)),
            st(StmtKind::Dump { scope: ScopeId(0) }),
        ];
        assert_eq!(run_str(stmts), "nil");
    }

    /// §10.3（N1）：帧栈自最外层→最内层；进入 Call 先把当前帧 span 更新为调用点，再压新帧。
    #[test]
    fn traceback_frames_follow_call_site_rule() {
        let call_at = |name: &str, at: Span| {
            Spanned::new(
                ExprKind::Call {
                    callee: Box::new(Spanned::new(ExprKind::Ident(name.to_string()), at)),
                    args: vec![],
                },
                at,
            )
        };
        // inner 体内错误节点 `1 / 0` 位于 (30,5)；首节点 span = 该语句 span。
        let inner_body = Body::Block(block(vec![Spanned::new(
            StmtKind::Expr(Spanned::new(
                ExprKind::Binary {
                    op: BinaryOp::Div,
                    left: Box::new(int(1)),
                    right: Box::new(int(0)),
                },
                Span::new(30, 5),
            )),
            Span::new(30, 5),
        )]));
        let outer_body = Body::Block(block(vec![Spanned::new(
            StmtKind::Expr(call_at("inner", Span::new(20, 3))),
            Span::new(20, 3),
        )]));
        let program = prog(vec![
            fn_decl("inner", &[], inner_body),
            fn_decl("outer", &[], outer_body),
            Spanned::new(
                StmtKind::Expr(call_at("outer", Span::new(10, 1))),
                Span::new(10, 1),
            ),
        ]);
        let run = eval_module_traced(&program);
        assert_eq!(
            run.result.as_ref().unwrap_err().class_name(),
            "ZeroDivisionError"
        );
        // 三帧：模块（外层调用点）→ outer（内层调用点）→ inner（错误节点）。
        assert_eq!(run.frames.len(), 3);
        assert_eq!(run.frames[0].span, Span::new(10, 1));
        assert_eq!(run.frames[1].span, Span::new(20, 3));
        assert_eq!(run.frames[2].span, Span::new(30, 5));
        // 帧名（§8.4）。
        assert_eq!(run.frame_name(&run.frames[0]).as_ref(), "<module>");
        assert_eq!(run.frame_name(&run.frames[1]).as_ref(), "outer");
        assert_eq!(run.frame_name(&run.frames[2]).as_ref(), "inner");
        // 模块帧 func_id = 0；函数帧 id 互不相同。
        assert_eq!(run.frames[0].func_id, 0);
        assert_ne!(run.frames[1].func_id, run.frames[2].func_id);
    }

    /// 端到端：`lex` + `parse` + `eval_module`（parser 当前最小子集）。
    #[test]
    fn end_to_end_lex_parse_eval_smoke() {
        assert_eq!(
            eval_src("let x = 1\nx").map(|v| v.to_string()).unwrap(),
            "1"
        );
        assert_eq!(
            eval_src("let s = \"hi\"\ns").map(|v| v.to_string()).unwrap(),
            "hi"
        );
        // 语法错误经 `R` 冒泡（`SyntaxError`）。
        assert_eq!(eval_src("1 2").unwrap_err().class_name(), "SyntaxError");
    }

    // ---- P3.9b 高阶内置：端到端 / 求值器集成 ---------------------------------

    /// P3.9b 端到端（`lex` + `parse` + `eval_module`）：`map` 经求值器**路由到高阶表**；
    /// 回调非函数 → `TypeError`（而非 `NameError`，证明 `is_builtin` / `call_with` 已接通）。
    #[test]
    fn e2e_hof_map_routes_and_rejects_non_function_callback() {
        let e = eval_src("map(1, [1, 2])").unwrap_err();
        assert_eq!(e.class_name(), "TypeError");
        assert_eq!(e.message(), "不可调用：int 不是函数");
    }

    /// P3.9b 端到端（`lex` + `parse` + `eval_module`）：高阶内置的 `data-last` 容器类型与
    /// 参数个数校验（`TypeError`）。
    #[test]
    fn e2e_hof_data_last_and_arg_count_checks() {
        assert_eq!(eval_src("sortBy(nil, [3, 1])").unwrap_err().class_name(), "TypeError");
        assert_eq!(eval_src("each(1)").unwrap_err().class_name(), "TypeError");
        assert_eq!(eval_src("reduce(nil, 0)").unwrap_err().class_name(), "TypeError");
    }

    /// P3.9b：真实用户**闭包**经求值器（`Invoke` 注入）驱动 `map` / `reduce` / 闭包捕获 / `filter`。
    ///
    /// 说明：本机 parser 仍在并行实现 lambda 语法（`core-dev` P3.4/P3.5），故用**程序化 AST**
    /// 构造闭包（`eval_src` 的 lex+parse 路径暂无法产生 lambda）；HOF 的**路由**已由上面两条
    /// `lex+parse+eval` 用例覆盖。本用例端到端验证「闭包捕获 + 调用帧 + 回调注入」全链路。
    #[test]
    fn hof_drives_user_closures_through_evaluator() {
        // map(fn(x) x * 2, [1, 2, 3]) → [2, 4, 6]
        let map_prog = prog(vec![
            ldecl(
                "double",
                lambda(&["x"], Body::Expr(bin(BinaryOp::Mul, ident("x"), int(2)))),
            ),
            expr_stmt(call(
                "map",
                vec![ident("double"), arr(vec![int(1), int(2), int(3)])],
            )),
        ]);
        assert_eq!(eval_module(&map_prog).unwrap().to_string(), "[2, 4, 6]");

        // reduce(fn(a, b) a + b, 0, [1, 2, 3]) → 6
        let reduce_prog = prog(vec![
            ldecl(
                "add",
                lambda(&["a", "b"], Body::Expr(bin(BinaryOp::Add, ident("a"), ident("b")))),
            ),
            expr_stmt(call(
                "reduce",
                vec![ident("add"), int(0), arr(vec![int(1), int(2), int(3)])],
            )),
        ]);
        assert_eq!(eval_module(&reduce_prog).unwrap().to_string(), "6");

        // 闭包捕获外层变量（A2）：k = 10; map(fn(x) x + k, [1, 2]) → [11, 12]
        let cap_prog = prog(vec![
            ldecl("k", int(10)),
            ldecl(
                "addk",
                lambda(&["x"], Body::Expr(bin(BinaryOp::Add, ident("x"), ident("k")))),
            ),
            expr_stmt(call("map", vec![ident("addk"), arr(vec![int(1), int(2)])])),
        ]);
        assert_eq!(eval_module(&cap_prog).unwrap().to_string(), "[11, 12]");

        // filter 谓词经求值器返回非 bool → TypeError（ConditionNotBool）。
        let bad_prog = prog(vec![
            ldecl("bad", lambda(&["x"], Body::Expr(ident("x")))),
            expr_stmt(call("filter", vec![ident("bad"), arr(vec![int(1)])])),
        ]);
        let e = eval_module(&bad_prog).unwrap_err();
        assert_eq!(e.class_name(), "TypeError");
        assert_eq!(e.message(), "条件必须是 bool，得到 int");
    }
}
