//! 作用域 / cell 捕获（A2）+ 逐作用域 `ScopeDebug` 可见链（§10.5）。
//!
//! 依据（只读消费）：
//! - `docs/spec/semantics.md` §3.6（`;;` 可见链：内→外、同层 slot 升序、遮蔽去重）、
//!   §4.5.2（A1 引用语义）、§4.5.3（A2 闭包**按 cell 引用**捕获）。
//! - `docs/spec/interface-contract.md` §10.5（per-scope `ScopeDebug` / `ScopeId` / `Dump { scope }`）。
//!
//! 本模块分两块，职责分离、互不干扰：
//!
//! 1. **运行时** [`Env`]：作用域链 + 变量槽位 + A2 cell 升级。
//!    热路径（`get` / `set` / `capture`）**只**在槽位上操作，**不**触碰任何调试信息。
//! 2. **调试** [`ScopeDebugTable`] / [`ScopeDebug`] / [`ScopeChain`] / [`NameInterner`]：
//!    编译期一次性构造、构造后只读、`Rc` 共享；**仅在执行 `;;`（DUMP）时被读取**
//!    （零热路径开销），供内→外遍历 / 遮蔽去重。

use crate::value::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ===========================================================================
// 运行时：作用域链 + 槽位 + A2 cell
// ===========================================================================

/// A2 **共享可变单元**：被闭包捕获的局部变量升级为它；闭包与定义作用域共享同一 `Rc`。
///
/// 外部后续修改对闭包可见，闭包内修改对外部亦可见（§4.5.3）。
pub type Cell = Rc<RefCell<Value>>;

/// 变量槽位。
///
/// - 未被捕获的局部直接存 [`Value`]（热路径零额外分配）；
/// - 一旦被闭包捕获，**原地升级**为 [`Cell`]（A2），此后读写都经该共享单元。
#[derive(Clone, Debug)]
enum Slot {
    Direct(Value),
    Shared(Cell),
}

/// 运行时词法作用域：作用域链 + 变量槽位。
///
/// 槽位下标为**绝对下标** = `first_slot + 本作用域内偏移`，与 [`ScopeDebug::first_slot`]
/// 同一编号空间，便于 `;;` 由槽位读回值。读写沿作用域链**内→外**查找（内层遮蔽外层）。
#[derive(Debug)]
pub struct Env {
    first_slot: u32,
    slots: Vec<Slot>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    /// 新建**顶层**作用域（无父）。
    #[must_use]
    pub fn root() -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            first_slot: 0,
            slots: Vec::new(),
            parent: None,
        }))
    }

    /// 新建子作用域，`first_slot` 显式给定（块 / 循环体 / 函数调用帧）。
    #[must_use]
    pub fn child(parent: &Rc<RefCell<Env>>, first_slot: u32) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            first_slot,
            slots: Vec::new(),
            parent: Some(Rc::clone(parent)),
        }))
    }

    /// 在 `parent` 之后新建子作用域（`first_slot` = 父作用域末槽 + 1）。
    ///
    /// 循环体可**复用**同一 `first_slot` 反复新建子作用域：每次迭代是**新绑定**、**各自新的 cell**
    /// （§4.5.3），已捕获的 cell 由闭包独立持有，互不影响。
    #[must_use]
    pub fn child_after(parent: &Rc<RefCell<Env>>) -> Rc<RefCell<Env>> {
        let first_slot = {
            let p = parent.borrow();
            p.first_slot + p.slots.len() as u32
        };
        Env::child(parent, first_slot)
    }

    /// 在**当前作用域**分配一个新槽位，返回其绝对下标。
    pub fn define(&mut self, value: Value) -> u32 {
        let slot = self.first_slot + self.slots.len() as u32;
        self.slots.push(Slot::Direct(value));
        slot
    }

    /// 读取槽位（沿链内→外；已升级为 cell 的槽位读其当前值）。
    #[must_use]
    pub fn get(&self, slot: u32) -> Option<Value> {
        if slot >= self.first_slot {
            let idx = (slot - self.first_slot) as usize;
            if let Some(s) = self.slots.get(idx) {
                return Some(match s {
                    Slot::Direct(v) => v.clone(),
                    Slot::Shared(c) => c.borrow().clone(),
                });
            }
        }
        self.parent.as_ref().and_then(|p| p.borrow().get(slot))
    }

    /// 写入槽位（**原地**；沿链内→外查找）。返回是否找到。
    pub fn set(&mut self, slot: u32, value: Value) -> bool {
        if slot >= self.first_slot {
            let idx = (slot - self.first_slot) as usize;
            if idx < self.slots.len() {
                match &mut self.slots[idx] {
                    Slot::Direct(v) => *v = value,
                    Slot::Shared(c) => *c.borrow_mut() = value,
                }
                return true;
            }
        }
        match &self.parent {
            Some(p) => p.borrow_mut().set(slot, value),
            None => false,
        }
    }

    /// 把槽位**原地升级**为共享 [`Cell`] 并返回（A2）；再次调用返回**同一** cell。
    ///
    /// 沿链内→外查找（闭包创建时捕获外层自由变量用）。未被捕获的局部在首次捕获时才装箱。
    pub fn capture(&mut self, slot: u32) -> Option<Cell> {
        if slot >= self.first_slot {
            let idx = (slot - self.first_slot) as usize;
            if idx < self.slots.len() {
                let value = match &self.slots[idx] {
                    Slot::Shared(c) => return Some(Rc::clone(c)),
                    Slot::Direct(v) => v.clone(),
                };
                let cell: Cell = Rc::new(RefCell::new(value));
                self.slots[idx] = Slot::Shared(Rc::clone(&cell));
                return Some(cell);
            }
        }
        self.parent.as_ref().and_then(|p| p.borrow_mut().capture(slot))
    }

    /// 本作用域首个槽位的绝对下标。
    #[must_use]
    pub fn first_slot(&self) -> u32 {
        self.first_slot
    }

    /// 本作用域槽位数。
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// 父作用域（顶层为 `None`）。
    #[must_use]
    pub fn parent(&self) -> Option<Rc<RefCell<Env>>> {
        self.parent.clone()
    }
}

// ===========================================================================
// 调试：per-scope ScopeDebug 可见链（§10.5）
// ===========================================================================

/// 名字在 interned 表中的下标（§10.5：名字 interned，**仅在** `;;` / 报错时解析）。
pub type FuncNameId = u32;

/// 作用域在 [`ScopeDebugTable`] 中的下标（§10.5 `Dump { scope: ScopeId }`）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ScopeId(pub u32);

/// 单个作用域的调试信息（§10.5，**字段与语义照契约**）。
///
/// - `first_slot`：本作用域首个局部槽位的**绝对下标**（与 [`Env`] 槽位同编号空间）。
/// - `names`：`slot → 名字`（按**声明序**；名字 interned）。
/// - `parent`：词法可见链的上一层（最外层为 `None`）。**不是**动态调用链。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ScopeDebug {
    /// 本作用域首个局部槽位的绝对下标。
    pub first_slot: u32,
    /// `slot → 名字`（按声明序）。
    pub names: Vec<FuncNameId>,
    /// 词法可见链的上一层（最外层为 `None`）。
    pub parent: Option<ScopeId>,
}

impl ScopeDebug {
    /// 构造一个作用域调试条目。
    #[must_use]
    pub fn new(first_slot: u32, names: Vec<FuncNameId>, parent: Option<ScopeId>) -> Self {
        Self {
            first_slot,
            names,
            parent,
        }
    }

    /// 本作用域声明的名字数量（= 槽位数）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// 本作用域是否无绑定。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// 名字 interner（§10.5）：标识符只在解析期哈希一次，`;;` / 报错时反查。
#[derive(Clone, Debug, Default)]
pub struct NameInterner {
    names: Vec<Rc<str>>,
    index: HashMap<Rc<str>, FuncNameId>,
}

impl NameInterner {
    /// 新建空表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 取名字的 [`FuncNameId`]；首次出现时登记。
    pub fn intern(&mut self, name: &str) -> FuncNameId {
        if let Some(&id) = self.index.get(name) {
            return id;
        }
        let id = self.names.len() as FuncNameId;
        let rc: Rc<str> = Rc::from(name);
        self.names.push(Rc::clone(&rc));
        self.index.insert(rc, id);
        id
    }

    /// 反查名字（**仅** `;;` / 报错时调用）。
    #[must_use]
    pub fn resolve(&self, id: FuncNameId) -> Option<Rc<str>> {
        self.names.get(id as usize).map(Rc::clone)
    }

    /// 已登记的名字数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// 逐作用域调试表（arena）：编译期一次性构造，之后**只读**，可由多个闭包 `Rc` 共享（§10.5）。
#[derive(Clone, Debug, Default)]
pub struct ScopeDebugTable {
    scopes: Vec<ScopeDebug>,
}

impl ScopeDebugTable {
    /// 新建空表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加一个作用域，返回其 [`ScopeId`]。
    pub fn push(&mut self, scope: ScopeDebug) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(scope);
        id
    }

    /// 按下标取作用域。
    #[must_use]
    pub fn get(&self, id: ScopeId) -> Option<&ScopeDebug> {
        self.scopes.get(id.0 as usize)
    }

    /// 作用域数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    /// 可见链枚举器（供 `;;`，§3.6 / §10.5）。
    ///
    /// 从 `from` 沿 `parent` 链**内→外**遍历；各作用域内按 `first_slot` + 声明序（slot 升序）；
    /// **遮蔽去重**（同名只取最内层，每名字恰好一项）。返回 `(绝对槽位, 名字)`——调用方据槽位
    /// 从 [`Env`] 读回值，按 §3.6 行格式 `<name> ： <value>` 输出。
    #[must_use]
    pub fn visible_slots(&self, from: ScopeId, names: &NameInterner) -> Vec<(u32, Rc<str>)> {
        let mut seen: HashSet<Rc<str>> = HashSet::new();
        let mut out: Vec<(u32, Rc<str>)> = Vec::new();
        let mut cur = Some(from);
        while let Some(id) = cur {
            let Some(scope) = self.get(id) else { break };
            for (i, &name_id) in scope.names.iter().enumerate() {
                let Some(name) = names.resolve(name_id) else {
                    continue;
                };
                if seen.insert(Rc::clone(&name)) {
                    out.push((scope.first_slot + i as u32, name));
                }
            }
            cur = scope.parent;
        }
        out
    }
}

/// 闭包携带的**词法定义处可见链**（§10.5）：`Rc` 共享的只读 [`ScopeDebugTable`] + 定义处的 [`ScopeId`]。
///
/// 使函数体内 `;;` 能看到闭包捕获到的外层名字（§3.6 第 2 条）。
#[derive(Clone, Debug)]
pub struct ScopeChain {
    /// 只读调试表（`Rc` 共享）。
    pub table: Rc<ScopeDebugTable>,
    /// 定义处的最内层作用域。
    pub scope: ScopeId,
}

impl ScopeChain {
    /// 空链（无可见变量）——顶层函数 / 测试用。
    #[must_use]
    pub fn empty() -> Self {
        let mut table = ScopeDebugTable::new();
        let scope = table.push(ScopeDebug::new(0, Vec::new(), None));
        Self {
            table: Rc::new(table),
            scope,
        }
    }

    /// 从定义处内→外枚举可见槽位。
    #[must_use]
    pub fn visible_slots(&self, names: &NameInterner) -> Vec<(u32, Rc<str>)> {
        self.table.visible_slots(self.scope, names)
    }
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Closure;

    #[test]
    fn a2_captured_local_upgrades_to_shared_cell() {
        let outer = Env::root();
        let n = outer.borrow_mut().define(Value::Int(0));
        // 闭包创建时捕获 n → 原地升级为共享 cell（A2）。
        let cell = outer.borrow_mut().capture(n).expect("slot exists");

        // 模拟 `fn inc() { n += 1; n }` 被反复调用：写回同一共享单元。
        for _ in 0..3 {
            let cur = match &*cell.borrow() {
                Value::Int(i) => *i,
                other => panic!("expected int, got {}", other.type_name()),
            };
            *cell.borrow_mut() = Value::Int(cur + 1);
        }

        // 定义作用域立即看到闭包的累加结果（共享同一单元）。
        assert_eq!(outer.borrow().get(n).map(|v| v.to_string()), Some("3".to_string()));
        // 再次捕获返回同一 cell（不再新分配）。
        let again = outer.borrow_mut().capture(n).expect("slot exists");
        assert!(Rc::ptr_eq(&cell, &again));
    }

    #[test]
    fn a2_loop_iterations_get_independent_cells() {
        let root = Env::root();
        let mut cells: Vec<Cell> = Vec::new();
        for i in 0..3 {
            // 每次迭代 = 新的块作用域 + 新的槽位（各自新的 cell，§4.5.3）。
            let body = Env::child_after(&root);
            let slot = body.borrow_mut().define(Value::Int(i));
            cells.push(body.borrow_mut().capture(slot).expect("slot exists"));
        }
        // 三个 cell 互不相同（互不共享）。
        for i in 0..cells.len() {
            for j in (i + 1)..cells.len() {
                assert!(!Rc::ptr_eq(&cells[i], &cells[j]), "cell {i} 与 {j} 不应共享");
            }
        }
        // 修改一个不影响其它。
        *cells[0].borrow_mut() = Value::Int(100);
        assert_eq!(cells[1].borrow().to_string(), "1");
        assert_eq!(cells[2].borrow().to_string(), "2");
    }

    #[test]
    fn env_get_set_walks_scope_chain() {
        let root = Env::root();
        let a = root.borrow_mut().define(Value::Int(1));
        let child = Env::child_after(&root);
        let b = child.borrow_mut().define(Value::Int(2));
        assert_eq!(a, 0);
        assert_eq!(b, 1);

        // 子可读 / 写父槽位（沿链）。
        assert_eq!(child.borrow().get(a).map(|v| v.to_string()), Some("1".to_string()));
        assert!(child.borrow_mut().set(a, Value::Int(9)));
        assert_eq!(root.borrow().get(a).map(|v| v.to_string()), Some("9".to_string()));
        // 本作用域槽位。
        assert_eq!(child.borrow().get(b).map(|v| v.to_string()), Some("2".to_string()));
        // 未定义槽位。
        assert!(root.borrow().get(99).is_none());
        assert!(!root.borrow_mut().set(99, Value::Nil));
    }

    #[test]
    fn scope_debug_inner_shadows_outer() {
        let mut names = NameInterner::new();
        let x = names.intern("x");
        let y = names.intern("y");
        let z = names.intern("z");
        let mut table = ScopeDebugTable::new();
        let module = table.push(ScopeDebug::new(0, vec![x, y], None));
        let inner = table.push(ScopeDebug::new(2, vec![x, z], Some(module)));

        let got: Vec<(u32, String)> = table
            .visible_slots(inner, &names)
            .into_iter()
            .map(|(s, n)| (s, n.to_string()))
            .collect();
        // 内→外；同层 slot 升序；遮蔽去重（x 取内层 slot 2）。
        assert_eq!(
            got,
            vec![
                (2, "x".to_string()),
                (3, "z".to_string()),
                (1, "y".to_string()),
            ]
        );
    }

    #[test]
    fn scope_debug_empty_chain_yields_nothing() {
        let chain = ScopeChain::empty();
        let names = NameInterner::new();
        assert!(chain.visible_slots(&names).is_empty());
    }

    #[test]
    fn name_interner_is_stable_and_resolvable() {
        let mut names = NameInterner::new();
        let x1 = names.intern("x");
        let y = names.intern("y");
        let x2 = names.intern("x");
        assert_eq!(x1, x2);
        assert_ne!(x1, y);
        assert_eq!(names.len(), 2);
        assert_eq!(names.resolve(x1).as_deref(), Some("x"));
        assert_eq!(names.resolve(y).as_deref(), Some("y"));
        assert_eq!(names.resolve(99), None);
    }

    #[test]
    fn closure_carries_def_scope_chain() {
        let mut names = NameInterner::new();
        let captured = names.intern("captured");
        let mut table = ScopeDebugTable::new();
        let outer = table.push(ScopeDebug::new(0, vec![captured], None));
        let inner = table.push(ScopeDebug::new(1, Vec::new(), Some(outer)));
        let chain = Rc::new(ScopeChain {
            table: Rc::new(table),
            scope: inner,
        });

        let f = Closure::named("inc", Rc::clone(&chain));
        // 闭包定义处的可见链能看到捕获到的外层名字。
        let got: Vec<(u32, String)> = f
            .def_scope
            .visible_slots(&names)
            .into_iter()
            .map(|(s, n)| (s, n.to_string()))
            .collect();
        assert_eq!(got, vec![(0u32, "captured".to_string())]);
    }
}
