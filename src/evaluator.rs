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
use crate::env::{Env, ScopeChain};
use crate::error::{
    div_zero, field as field_error, index as index_error, name as name_error,
    overflow, recursion, type_error, value as value_error, LzError, OverflowMsg, R, TraceFrame,
    TypeMsg, ValueMsg,
};
use crate::span::Span;
use crate::value::{CapturedVar, Closure, StructObj, UserFn, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
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
struct RunOrigins {
    sources: Rc<crate::source::SourceMap>,
    frame_sources: Vec<crate::source::SourceId>,
    error_source: Option<crate::source::SourceId>,
}

pub struct TracedRun {
    origins: Option<RunOrigins>,
    host_diagnostics: Vec<String>,
    http_diagnostics: Vec<String>,
    /// 求值结果（最后一条语句的值；出错则为 `Err`）。
    pub result: R<Value>,
    /// 帧栈，**最外层 → 最内层**（§10.3）：首帧恒为模块帧 `<module>`，末帧为出错点所在帧。
    pub frames: Vec<TraceFrame>,
    /// `func_id → 函数名`：索引 0 = 模块；`Some(名)` = 命名函数；`None` = 匿名 `<fn>`。
    func_names: Vec<Option<Rc<str>>>,
}

impl TracedRun {
    pub fn host_diagnostics(&self)->&[String]{&self.host_diagnostics}
    pub fn http_diagnostics(&self)->&[String]{&self.http_diagnostics}
    /// 按§19完整校验后原子替换来源sidecar；旧求值入口不自动附加。
    pub fn attach_origins(
        &mut self,
        sources: Rc<crate::source::SourceMap>,
        frame_sources: Vec<crate::source::SourceId>,
        error_source: Option<crate::source::SourceId>,
    ) -> Result<(), crate::source::SourceMetadataError> {
        use crate::source::SourceMetadataError as E;
        if frame_sources.len() != self.frames.len() {
            return Err(E::FrameCount { expected: self.frames.len(), actual: frame_sources.len() });
        }
        for &id in &frame_sources {
            if sources.get(id).is_none() { return Err(E::UnknownSource { id }); }
        }
        match (&self.result, error_source) {
            (Ok(_), Some(_)) => return Err(E::UnexpectedErrorSource),
            (Err(_), None) => return Err(E::MissingErrorSource),
            _ => {}
        }
        if let Some(id) = error_source {
            if sources.get(id).is_none() { return Err(E::UnknownSource { id }); }
        }
        self.origins = Some(RunOrigins { sources, frame_sources, error_source });
        Ok(())
    }

    /// 实际帧下标的来源；宿主修改帧数后不猜测或回退来源。
    pub fn frame_source(&self, index: usize) -> Option<&crate::source::SourceRecord> {
        let origins = self.origins.as_ref()?;
        if self.frames.len() != origins.frame_sources.len() { return None; }
        self.frames.get(index)?;
        origins.sources.get(*origins.frame_sources.get(index)?)
    }

    pub fn error_source(&self) -> Option<&crate::source::SourceRecord> {
        if self.result.is_ok() { return None; }
        let origins = self.origins.as_ref()?;
        origins.sources.get(origins.error_source?)
    }

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

#[derive(Debug)]
pub enum GraphExecutionError { InvalidGraph { reason: String }, Origins(crate::source::SourceMetadataError) }
impl std::fmt::Display for GraphExecutionError {
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {match self{Self::InvalidGraph{reason}=>write!(f,"{reason}"),Self::Origins(e)=>write!(f,"来源元数据不一致：{e:?}")}}
}
impl std::error::Error for GraphExecutionError {}
fn invalid_graph(reason:&str)->GraphExecutionError{GraphExecutionError::InvalidGraph{reason:reason.into()}}
fn validate_execution_graph(g:&crate::modules::ModuleGraph)->Result<(),GraphExecutionError>{
 use std::collections::HashSet;
 let n=g.units.len();if n==0||g.entry.0 as usize>=n||g.initialization_order.len()!=n{return Err(invalid_graph("模块集合或初始化顺序无效"));}
 let mut position=vec![usize::MAX;n];
 for (p,id) in g.initialization_order.iter().enumerate(){let i=id.0 as usize;if i>=n||position[i]!=usize::MAX{return Err(invalid_graph("初始化顺序不是排列"));}position[i]=p;}
 if g.initialization_order.last()!=Some(&g.entry){return Err(invalid_graph("入口必须最后初始化"));}
 let mut sources=HashSet::new();
 for (index,u) in g.units.iter().enumerate(){
  if u.id.0 as usize!=index||!sources.insert(u.source_id)||g.sources.get(u.source_id).and_then(|s|s.loaded.as_ref()).is_none(){return Err(invalid_graph("模块身份或来源无效"));}
  let mut bindings=HashSet::new();let mut imports=vec![];let mut exports=vec![];let mut other=false;
  for s in &u.program.stmts {match &s.node{
   StmtKind::ImportDecl(d)=>{if other{return Err(invalid_graph("导入顺序无效"));}imports.push((d,s.span));bindings.insert(d.alias.as_str());},
   StmtKind::ExportDecl(d)=>{other=true;for name in &d.names{exports.push(name);}},
   StmtKind::Decl{name,..}=>{other=true;bindings.insert(name.as_str());},
   StmtKind::FnDecl(d)=>{other=true;bindings.insert(d.name.as_str());},
   StmtKind::StructDecl(d)=>{other=true;bindings.insert(d.name.as_str());},_=>{other=true;}
  }}
  if imports.len()!=u.imports.len()||exports.len()!=u.exports.len(){return Err(invalid_graph("声明元数据数量不一致"));}
  let mut aliases=HashSet::new();
  for ((d,span),r) in imports.iter().zip(&u.imports){
   let target=r.target.0 as usize;
   if target>=n||position[target]>=position[index]||d.alias!=r.alias||d.alias_span!=r.alias_span||d.path_span!=r.path_span||*span!=r.span||!aliases.insert(d.alias.as_str()){return Err(invalid_graph("导入元数据无效"));}
  }
  for s in &u.program.stmts{let name=match &s.node{StmtKind::Decl{name,..}=>Some(name.as_str()),StmtKind::FnDecl(d)=>Some(d.name.as_str()),StmtKind::StructDecl(d)=>Some(d.name.as_str()),_=>None};if name.is_some_and(|n|aliases.contains(n)){return Err(invalid_graph("导入绑定冲突"));}}
  let mut seen=HashSet::new();for (d,r) in exports.iter().zip(&u.exports){if d.name!=r.name||d.span!=r.span||!bindings.contains(d.name.as_str())||!seen.insert(d.name.as_str()){return Err(invalid_graph("导出元数据无效"));}}
 }
 Ok(())
}
#[derive(Clone,Copy,PartialEq)]
enum ModuleState{Uninitialized,Initializing,Ready,Failed}
struct ModuleInstance{env:Rc<RefCell<Env>>,namespace:Option<Value>,state:ModuleState,source:crate::source::SourceId}
/// 调用者必须在大栈内持有并析构graph；不重新读取或解析。
#[derive(Clone, Debug, Default)]
pub struct EvalLimits {
 pub max_steps: Option<u64>,
 pub deadline: Option<std::time::Instant>,
 pub cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}
#[derive(Clone,Debug,Default)]
pub struct EvalCapabilities { pub subprocess:bool, pub http:bool }
#[derive(Clone,Debug,Default)]
pub struct EvalContext { pub limits:EvalLimits, pub capabilities:EvalCapabilities }
pub fn eval_graph_traced_on_thread_with_args(g:&crate::modules::ModuleGraph,args:&[String])->Result<TracedRun,GraphExecutionError>{
 return eval_graph_traced_on_thread_with_args_and_limits(g,args,EvalLimits::default());
}
pub fn eval_graph_traced_on_thread_with_args_and_limits(g:&crate::modules::ModuleGraph,args:&[String],limits:EvalLimits)->Result<TracedRun,GraphExecutionError>{
 eval_graph_traced_on_thread_with_args_and_context(g,args,EvalContext{limits,..Default::default()})
}
pub fn eval_graph_traced_on_thread_with_args_and_context(g:&crate::modules::ModuleGraph,args:&[String],context:EvalContext)->Result<TracedRun,GraphExecutionError> {
 validate_execution_graph(g)?;
 let mut i=Interp::new();i.limits=context.limits;i.capabilities=context.capabilities;i.script_args=args.iter().map(|s|Value::string(s.clone())).collect();
 i.modules=Some(g.units.iter().map(|u|ModuleInstance{env:Env::root(),namespace:None,state:ModuleState::Uninitialized,source:u.source_id}).collect());
 let mut result=Ok(Value::Nil);
 for id in &g.initialization_order{
  let unit=&g.units[id.0 as usize];
  i.active_module=*id;i.globals=Rc::clone(&i.modules.as_ref().unwrap()[id.0 as usize].env);
  i.modules.as_mut().unwrap()[id.0 as usize].state=ModuleState::Initializing;
  for import in &unit.imports{
   let target=&i.modules.as_ref().unwrap()[import.target.0 as usize];
   if target.state!=ModuleState::Ready{return Err(invalid_graph("依赖尚未初始化"));}
   let ns=target.namespace.as_ref().ok_or_else(||invalid_graph("缺少模块命名空间"))?.clone();
   i.globals.borrow_mut().define_named(&import.alias,ns,false);
  }
  let scope=Scope{env:Rc::clone(&i.globals),captured:Rc::new(vec![]),self_val:None};
  let at=unit.program.stmts.first().map_or(unit.program.span,|s|s.span);
  i.trace.push(TraceFrame{func_id:MODULE_FUNC_ID,span:at});i.trace_sources.push(unit.source_id);
  result=i.exec_stmts(&unit.program.stmts,&scope).map(|f|match f{Flow::Value(v)|Flow::Return(v)=>v,_=>Value::Nil});
  if result.is_err(){i.modules.as_mut().unwrap()[id.0 as usize].state=ModuleState::Failed;break;}
  let mut entries=vec![];
  for export in &unit.exports{let value={let env=i.globals.borrow();env.local_index(&export.name).and_then(|n|env.get_local(n))}.ok_or_else(||invalid_graph("导出绑定在初始化后缺失"))?;entries.push((export.name.clone(),value));}
  let m=&mut i.modules.as_mut().unwrap()[id.0 as usize];m.namespace=Some(Value::object(entries));m.state=ModuleState::Ready;
  i.trace.pop();i.trace_sources.pop();
 }
 let error_source=if result.is_err(){i.fault_source.or_else(||i.trace_sources.last().copied())}else{None};
 let mut out=TracedRun{result,frames:i.trace,func_names:i.func_names,http_diagnostics:i.http_diagnostics,host_diagnostics:i.host_diagnostics,origins:None};
 out.attach_origins(Rc::clone(&g.sources),i.trace_sources,error_source).map_err(GraphExecutionError::Origins)?;
 Ok(out)
}

fn validate_http(args:&[Value],span:Span)->R<crate::http_control::HttpRequest>{
 use crate::error::ValueMsg as V;
 use crate::http_control::{valid_method,valid_url,valid_headers,HttpRequest};
 if args.len()!=6{return Err(type_error(TypeMsg::ArgCount{name:"httpRequest".into(),n:6,m:args.len()},span));}
 let bad=|v:&Value,rt:&str|type_error(TypeMsg::BadOperands{op:"httpRequest".into(),lt:v.type_name().into(),rt:rt.into()},span);
 let invalid=|v|crate::error::value(v,span);
 let method=args[0].as_str().ok_or_else(||bad(&args[0],"string"))?;
 let url=args[1].as_str().ok_or_else(||bad(&args[1],"string"))?;
 let h=args[2].as_array().ok_or_else(||bad(&args[2],"array"))?;
 let body=args[3].as_str().ok_or_else(||bad(&args[3],"string"))?;
 let timeout=args[4].as_int().ok_or_else(||bad(&args[4],"int"))?;
 let max=args[5].as_int().ok_or_else(||bad(&args[5],"int"))?;
 let mut headers=vec![];
 for pair in h.borrow().iter(){
  let p=pair.as_array().ok_or_else(||bad(pair,"array"))?;let p=p.borrow();
  if p.len()!=2{return Err(invalid(V::HttpHeaders));}
  let k=p[0].as_str().ok_or_else(||bad(&p[0],"string"))?;
  let v=p[1].as_str().ok_or_else(||bad(&p[1],"string"))?;
  headers.push((k.to_string(),v.to_string()));
 }
 if !valid_method(method){return Err(invalid(V::HttpMethod));}
 if !valid_url(url){return Err(invalid(V::HttpUrl));}
 if !valid_headers(&headers){return Err(invalid(V::HttpHeaders));}
 if body.len()>16777216||(matches!(method,"GET"|"HEAD")&&!body.is_empty()){return Err(invalid(V::HttpBody));}
 if !(1..=600000).contains(&timeout){return Err(invalid(V::HttpTimeoutRange));}
 if !(0..=16777216).contains(&max){return Err(invalid(V::HttpResponseRange));}
 for (_,v) in &mut headers{*v=v.trim_matches([' ','\t']).to_string();}
 Ok(HttpRequest{method:method.to_string(),url:url.to_string(),headers,body:body.as_bytes().to_vec(),timeout_ms:timeout as u64,max_response_bytes:max as usize})
}

fn validate_exec(args:&[Value],span:Span)->R<crate::process_control::ProcessRequest>{
 use crate::error::ValueMsg as V;
 if args.len()!=3{return Err(type_error(TypeMsg::ArgCount{name:"exec".into(),n:3,m:args.len()},span));}
 let bad=|v:&Value,rt:&str|type_error(TypeMsg::BadOperands{op:"exec".into(),lt:v.type_name().into(),rt:rt.into()},span);
 let array=args[0].as_array().ok_or_else(||bad(&args[0],"array"))?;
 let items=array.borrow();let mut argv=Vec::new();
 for v in items.iter(){argv.push(v.as_str().ok_or_else(||bad(v,"string"))?.to_string());}
 let invalid=|v|crate::error::value(v,span);
 if argv.is_empty(){return Err(invalid(V::ProcessArgvEmpty));}
 for arg in &argv{if arg.contains('\0'){return Err(invalid(V::ProcessArgNul));}}
 if !crate::process_control::executable_valid(&argv[0]){return Err(invalid(V::ProcessExecutable));}
 let timeout=args[1].as_int().ok_or_else(||bad(&args[1],"int"))?;
 let max=args[2].as_int().ok_or_else(||bad(&args[2],"int"))?;
 if !(1..=600000).contains(&timeout){return Err(invalid(V::ProcessTimeoutRange));}
 if !(0..=16777216).contains(&max){return Err(invalid(V::ProcessOutputRange));}
 Ok(crate::process_control::ProcessRequest{argv,timeout_ms:timeout as u64,max_output_bytes:max as usize})
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

/// 判定一个 `Call` 是否由**管道脱糖**产生（`|>`；AST 无 `Pipe` 节点，§10.6）。
///
/// 依据 `parser.rs` 的 `Span` 不变量（`parse_postfix`：「每个后缀节点的 `span` 一律取基座
/// （primary）的起始位置」）：
/// - **普通调用**：`Call.span == callee.span`（同一基座起始位置）；
/// - **管道脱糖**：`desugar_pipe` 把 `Call.span` 置为**左操作数**的 `span`、被调者为**右侧**，
///   而 `|>` 左侧在右侧之前，故 `callee.span != expr.span`（严格在后）。
///
/// 该不变量用于把「管道右侧非函数」（`TypeMsg::PipeRhsNotFunction`，§8.1）与
/// 「调用非函数」（`TypeMsg::NotCallable`）区分开；其它调用保持 `NotCallable` 不变。
fn is_pipe_desugared(expr_span: Span, callee: &Expr) -> bool {
    callee.span != expr_span
}

/// 被调值非函数时的 `TypeError` 细分（§8.1）：管道脱糖的 `Call` → `PipeRhsNotFunction`
/// （`管道右侧必须是函数，得到 {t}`）；其余调用 → `NotCallable`（`不可调用：{t} 不是函数`）。
fn non_function_call_msg(v: &Value, piped: bool) -> TypeMsg {
    let t = v.type_name().to_string();
    if piped {
        TypeMsg::PipeRhsNotFunction { t }
    } else {
        TypeMsg::NotCallable { t }
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
    eval_module_traced_with_args(program, &[])
}

/// 独立脚本参数快照；沿用大栈保证。
pub fn eval_module_traced_with_args(program: &Program, args: &[String]) -> TracedRun {
    eval_module_traced_with_args_and_limits(program,args,EvalLimits::default())
}

pub fn eval_module_traced_with_args_and_limits(program:&Program,args:&[String],limits:EvalLimits)->TracedRun {
 eval_module_traced_with_args_and_context(program,args,EvalContext{limits,..Default::default()})
}
pub fn eval_module_traced_with_args_and_context(program:&Program,args:&[String],context:EvalContext)->TracedRun {
 on_eval_stack(move || eval_module_traced_on_thread_with_args_and_context(program,args,context))
}

/// 在**专用大栈线程**（[`EVAL_STACK_SIZE`] = 256 MiB ≥ 64 MiB）上运行 `f` 并返回其结果
/// （`interface-contract.md` §10.9 R-S2）。
///
/// `T` 可含 `Rc`（非 `Send`）：结果经 [`Transfer`] 在 `join` 边界移交，故 `T` **无** `Send`
/// 约束（闭包 `F` 须 `Send`，以便在子线程上运行）。线程创建失败（OS 资源不足）时**退化**为
/// 当前栈执行（逻辑深度上限仍生效）。
///
/// **用途**：CLI 把 `load → lex → parse → eval`（含 `Program` 的**析构**）整条流水线放到**同一**
/// 大栈，使 `PARSE_DEPTH_LIMIT` 层可达、深左偏 AST 的递归析构不溢出。
pub fn on_eval_stack<T, F>(f: F) -> T
where
    F: FnOnce() -> T + Send,
{
    use std::sync::{Arc, Mutex};
    // 闭包放入共享槽：线程创建失败时可在当前栈上取回并执行（保底退化）。
    let slot = Arc::new(Mutex::new(Some(f)));
    std::thread::scope(|scope| {
        let worker = Arc::clone(&slot);
        let spawned = std::thread::Builder::new()
            .stack_size(EVAL_STACK_SIZE)
            .spawn_scoped(scope, move || {
                let f = worker
                    .lock()
                    .expect("栈执行槽锁未中毒")
                    .take()
                    .expect("栈执行闭包只被调用一次");
                Transfer(f())
            });
        match spawned {
            Ok(handle) => match handle.join() {
                Ok(transfer) => transfer.0,
                Err(payload) => std::panic::resume_unwind(payload),
            },
            // 线程创建失败（OS 资源不足）：在当前栈上执行（逻辑深度上限仍生效）。
            Err(_) => {
                let f = slot
                    .lock()
                    .expect("栈执行槽锁未中毒")
                    .take()
                    .expect("栈执行闭包只被调用一次");
                f()
            }
        }
    })
}

/// 在**当前线程**上求值（不新开线程）。
///
/// 供已位于大栈线程的调用方（如 CLI 经 [`on_eval_stack`]）复用，使 `parse` 与 `eval`
/// 处于**同一**大栈（§10.9 R-S2「整条流水线同栈」），避免嵌套线程。
pub fn eval_module_traced_on_thread(program: &Program) -> TracedRun {
    eval_module_traced_on_thread_with_args(program, &[])
}

/// 调用方已在大栈；不重复创建线程。
pub fn eval_module_traced_on_thread_with_args(program: &Program, args: &[String]) -> TracedRun {
    eval_module_on_thread_with_args(program, args)
}

/// [`eval_module_traced`] 的实际求值体（在求值线程上运行）。
fn eval_module_on_thread_with_args(program: &Program, args: &[String]) -> TracedRun {
    eval_module_traced_on_thread_with_args_and_limits(program,args,EvalLimits::default())
}
pub fn eval_module_traced_on_thread_with_args_and_limits(program:&Program,args:&[String],limits:EvalLimits)->TracedRun {
 eval_module_traced_on_thread_with_args_and_context(program,args,EvalContext{limits,..Default::default()})
}
pub fn eval_module_traced_on_thread_with_args_and_context(program:&Program,args:&[String],context:EvalContext)->TracedRun {
    // B过渡：扫描整个根Program，在任何LFZ语句及副作用之前拒绝模块执行。
    if let Some(first_decl_span) = crate::modules::module_declaration_span(program) {
        return TracedRun {
            http_diagnostics: vec![],host_diagnostics: vec![], origins: None,
            result: Err(Box::new(LzError::Syntax {
                msg: crate::error::SyntaxMsg::ModuleExecutionUnavailable, span: first_decl_span,
            })),
            frames: vec![TraceFrame { func_id: MODULE_FUNC_ID, span: first_decl_span }],
            func_names: vec![None],
        };
    }
    let mut interp = Interp::new();
    interp.limits=context.limits;interp.capabilities=context.capabilities;
    interp.script_args = args.iter().map(|s| Value::string(s.clone())).collect();
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
        http_diagnostics: std::mem::take(&mut interp.http_diagnostics),host_diagnostics: std::mem::take(&mut interp.host_diagnostics), origins: None,
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
/// - `captured`：闭包定义处捕获的 `(名字, cell, 是否可变)`（调用帧内查名在 env 之后回落于此）；
/// - `self_val`：`self` 绑定（仅方法调用帧有值）。
struct Scope {
    env: Rc<RefCell<Env>>,
    captured: Rc<Vec<CapturedVar>>,
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

/// 变量重绑定（`exec_assign` 无后缀路径）的三态结果 —— [`Interp::assign_name`] 返回值
/// （ADR P3.11 裁定 1 #2）。
enum AssignOutcome {
    /// 命中**可变**绑定并写入成功（`var` / 形参 / `for` 变量 / `fn` / `struct` 名等）。
    Assigned,
    /// 命中**不可变** `let` 绑定，拒绝重绑定 → 报 `TypeError::ImmutableRebind`（§4.5.2）。
    Immutable,
    /// 沿词法可见链**未找到**该名字。
    NotFound,
}

// ===========================================================================
// 解释器
// ===========================================================================

/// 树遍历解释器：模块全局环境 + 单调递增的 `func_id` 分配器 + traceback 帧栈。
struct Interp {
    capabilities: EvalCapabilities,
    host_diagnostics: Vec<String>,
    http_diagnostics: Vec<String>,
    limits: EvalLimits,
    modules: Option<Vec<ModuleInstance>>,
    trace_sources: Vec<crate::source::SourceId>,
    fault_source: Option<crate::source::SourceId>,
    active_module: crate::source::ModuleId,
    /// 模块顶层作用域（globals）；所有函数调用帧直接挂在它之下。
    globals: Rc<RefCell<Env>>,
    script_args: Vec<Value>,
    argv_function: Value,
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
        let mut argv_closure = Closure::named("argv", Rc::new(ScopeChain::empty()));
        argv_closure.builtin = Some("argv");
        Self {
            capabilities: EvalCapabilities::default(), http_diagnostics:vec![],host_diagnostics:vec![],
            limits: EvalLimits::default(),
            script_args: Vec::new(),
            argv_function: Value::function(argv_closure),
            modules: None, trace_sources: vec![], fault_source: None,
            active_module: crate::source::ModuleId::ENTRY,
            globals: Env::root(),
            next_func_id: 1,
            depth: 0,
            trace: Vec::new(),
            func_names: vec![None], // [0] = 模块帧（`<module>`）
        }
    }

    /// 把当前帧 `span` 覆盖为错误的 `span`（§10.3 N1：「该帧当前正在求值的最小 AST 节点」）。
    ///
    /// 在 [`Interp::eval_expr`] / [`Interp::exec_stmt`] 的**错误冒泡处**调用：非 `Call` 节点
    /// （字段 / 下标 / 一元 / 二元 / 语句级错误等）不经过 [`Interp::call_user`] 的「进入 Call
    /// 先把当前帧 `span` 更新为调用点」路径，若不覆盖则会停留在帧初始 `span`，使 traceback
    /// 位置过期（P3 验收 bug-20260924-04）。`Io` 等 `span = None` 的错误不改变帧 `span`。
    ///
    /// 错误最深处的 `span` 即「引发错误的最小 AST 节点」；错误沿调用链冒泡时该 `span` 不变，
    /// 故重复覆盖到同一值是无害的（`call_user` 出错时不弹帧，当前帧恒为最内层帧）。
    fn poll(&mut self,span:Span)->R<()> {
        use crate::error::ExecutionLimitReason as Reason;
        let reason = if self.limits.cancel.as_ref().is_some_and(|c|c.load(std::sync::atomic::Ordering::Acquire)) { Some(Reason::Cancelled) }
        else if self.limits.deadline.is_some_and(|d|std::time::Instant::now()>=d) { Some(Reason::Deadline) }
        else if self.limits.max_steps==Some(0) { Some(Reason::StepBudget) } else { None };
        if let Some(reason)=reason {
            let error=crate::error::execution_limit(reason,span);
            self.note_error_span(&error);return Err(error);
        }
        if let Some(remaining)=self.limits.max_steps.as_mut(){*remaining-=1;}
        Ok(())
    }

    fn note_error_span(&mut self, e: &LzError) {
        if self.modules.is_some() && self.fault_source.is_none() { self.fault_source = self.trace_sources.last().copied(); }
        if let Some(span) = e.span() {
            if let Some(top) = self.trace.last_mut() {
                top.span = span;
            }
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
        for (n, c, _) in scope.captured.iter() {
            if n.as_ref() == name {
                return Some(c.borrow().clone());
            }
        }
        let gi = self.globals.borrow().local_index(name);
        gi.and_then(|i| self.globals.borrow().get_local(i))
    }

    /// 写名字（原地；捕获变量写共享 cell）。返回三态结果（[`AssignOutcome`]）：
    ///
    /// - 沿 env 链 / 捕获 cell / globals **内→外**解析名字；
    /// - 命中**不可变 `let`** 绑定 → [`AssignOutcome::Immutable`]（**不写入**）；
    /// - 命中可变绑定（`var` / 形参 / `for` 变量 / `fn` / `struct` 名）→ 写入并返回
    ///   [`AssignOutcome::Assigned`]；
    /// - 全部未命中 → [`AssignOutcome::NotFound`]。
    ///
    /// `let` 只锁重绑定、不锁内容（`semantics.md` §4.5.2）：`a = v` 非法，但 `a[i] = v` /
    /// `s.k = v` 走容器原地修改路径（A1），不经本方法。
    fn assign_name(&self, scope: &Scope, name: &str, value: Value) -> AssignOutcome {
        let mut cur = Some(Rc::clone(&scope.env));
        while let Some(e) = cur {
            if Rc::ptr_eq(&e, &self.globals) {
                break;
            }
            // RefCell 陷阱：先取 `idx` 结束借用，再做可变性查询 / 写入。
            let idx = e.borrow().local_index(name);
            if let Some(i) = idx {
                if e.borrow().local_mutable(i) == Some(false) {
                    return AssignOutcome::Immutable;
                }
                e.borrow_mut().set_local(i, value);
                return AssignOutcome::Assigned;
            }
            cur = e.borrow().parent();
        }
        for (n, c, mutable) in scope.captured.iter() {
            if n.as_ref() == name {
                if !*mutable {
                    return AssignOutcome::Immutable;
                }
                *c.borrow_mut() = value;
                return AssignOutcome::Assigned;
            }
        }
        let gi = self.globals.borrow().local_index(name);
        if let Some(i) = gi {
            if self.globals.borrow().local_mutable(i) == Some(false) {
                return AssignOutcome::Immutable;
            }
            self.globals.borrow_mut().set_local(i, value);
            return AssignOutcome::Assigned;
        }
        AssignOutcome::NotFound
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
        for (n, c, _) in scope.captured.iter() {
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
        let mut captured: Vec<CapturedVar> = Vec::new();
        let mut cur = Some(Rc::clone(&scope.env));
        while let Some(e) = cur {
            if Rc::ptr_eq(&e, &self.globals) {
                break;
            }
            let entries = e.borrow().named_indices();
            for (n, idx) in entries {
                if captured.iter().any(|(k, _, _)| k.as_ref() == n.as_str()) {
                    continue;
                }
                // 捕获时同步记录该绑定的可变性（`let` → `false`），供闭包内重绑定判定。
                let mutable = e.borrow().local_mutable(idx).unwrap_or(true);
                if let Some(cell) = e.borrow_mut().capture_local(idx) {
                    captured.push((Rc::from(n.as_str()), cell, mutable));
                }
            }
            cur = e.borrow().parent();
        }
        // 继承父闭包的捕获（内层 env 名字已优先占位）。
        for (n, c, mutable) in scope.captured.iter() {
            if captured.iter().any(|(k, _, _)| k.as_ref() == n.as_ref()) {
                continue;
            }
            captured.push((Rc::clone(n), Rc::clone(c), *mutable));
        }
        let func_id = self.next_func_id;
        self.next_func_id += 1;
        // 函数表：`func_id → 名字`（`None` = 匿名 `<fn>`），报错时反查（§10.3）。
        self.func_names.push(name.map(Rc::from));
        let user = UserFn {
            home_module: self.active_module,
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
        self.poll(stmt.span)?;
        match self.exec_stmt_inner(stmt, scope) {
            Ok(flow) => Ok(flow),
            Err(e) => {
                self.note_error_span(&e);
                Err(e)
            }
        }
    }

    /// [`Interp::exec_stmt`] 的执行体（不含帧 `span` 覆盖）。
    fn exec_stmt_inner(&mut self, stmt: &Stmt, scope: &Scope) -> R<Flow> {
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
            // 渲染逻辑为纯函数 [`render_dump`]；写通道由 [`builtins::write_dump`] 决定，
            // **与 `print` 同通道**（`--json` 下二者一起转 stderr，§3.6 #7）。
            StmtKind::Dump { .. } => {
                let entries = self.visible_entries(scope);
                let text = render_dump(&entries);
                if !text.is_empty() {
                    builtins::write_dump(&text, stmt.span)?;
                }
                Ok(Flow::Value(Value::Nil))
            }
            StmtKind::ImportDecl(_) | StmtKind::ExportDecl(_) => if self.modules.is_some() { Ok(Flow::Value(Value::Nil)) } else { Err(Box::new(LzError::Syntax {
                msg: crate::error::SyntaxMsg::ModuleExecutionUnavailable, span: stmt.span,
            })) },
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
        // 仅直接声明在此块创建绑定；嵌套块自行管理其作用域。
        let binds_locals = body.stmts.iter().any(|s| matches!(s.node,
            StmtKind::Decl { .. } | StmtKind::FnDecl(_) | StmtKind::StructDecl(_)));
        loop {
            let c = match self.eval_expr(cond, scope)? {
                Flow::Value(v) => v,
                f => return Ok(f),
            };
            if !as_condition(&c, cond.span)? {
                break;
            }
            let flow = if binds_locals { self.exec_block(body, scope)? }
                else { self.exec_stmts(&body.stmts, scope)? };
            match flow {
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
        let def = if self.modules.is_some() {
            let dummy=self.make_closure(Some(&format!("{}::<defaults>",sd.name)),vec![],Body::Block(Block{span:Span::START,stmts:vec![]}),scope);
            let Value::Func(cl)=dummy else {unreachable!()};let u=cl.user.as_ref().unwrap();
            Value::struct_template_in_context(sd.name.as_str(),fields,methods,crate::value::TemplateContext{home_module:u.home_module,captured:Rc::clone(&u.captured),self_val:u.self_val.clone(),defaults_func_id:u.func_id})
        } else { Value::struct_template(sd.name.as_str(), fields, methods) };
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
            let outcome = match &target.base {
                LvalueBase::Name(n) => self.assign_name(scope, n, newv),
                // `self` 是只读绑定，不属可重绑定变量（保持报 `NameError`）。
                LvalueBase::SelfValue => AssignOutcome::NotFound,
            };
            match outcome {
                AssignOutcome::Assigned => {}
                AssignOutcome::Immutable => {
                    let nm = match &target.base {
                        LvalueBase::Name(n) => n.clone(),
                        LvalueBase::SelfValue => "self".to_string(),
                    };
                    return Err(type_error(
                        TypeMsg::ImmutableRebind { name: nm },
                        target.span,
                    ));
                }
                AssignOutcome::NotFound => {
                    let nm = match &target.base {
                        LvalueBase::Name(n) => n.clone(),
                        LvalueBase::SelfValue => "self".to_string(),
                    };
                    return Err(name_error(nm, target.span));
                }
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
        // §8.2：插入符指向「引发错误的最小 AST 节点的首字符」。对下标 / 字段访问，**读取**
        // 路径（`ExprKind::Index` / `Field` 节点）的 span 取**基座起始**（parser
        // `parse_postfix` 不变量），故**写入**路径也须用基座 span（`target.span` = 基座 token
        // 起始），而非下标节的 `[` / `.` 位置（obs-B-03：同一访问读写两形态 column 须一致）。
        let err_span = target.span;
        // 中间段：逐段解析到「最终容器」。
        for seg in &target.path[..last] {
            current = match self.read_segment(&current, seg, err_span, scope)? {
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
            Some(self.read_final(&current, &key, err_span)?)
        };
        let newv = match self.eval_rhs(op, current_val, value, scope, target.span)? {
            Step::Done(v) => v,
            Step::Flow(f) => return Ok(f),
        };
        self.write_final(&current, &key, newv, err_span)?;
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
    ///
    /// `err_span` 为**基座 span**（`target.span`）：字段 / 下标错误位置须指向基座首字符，
    /// 与读取路径一致（obs-B-03 / §8.2）。
    fn read_segment(
        &mut self,
        cur: &Value,
        seg: &LvalueSeg,
        err_span: Span,
        scope: &Scope,
    ) -> R<Step<Value>> {
        match &seg.kind {
            LvalueSegKind::Field(k) => Ok(Step::Done(self.get_field(cur, k, err_span)?)),
            LvalueSegKind::Index(e) => match self.eval_expr(e, scope)? {
                Flow::Value(iv) => Ok(Step::Done(self.index_read(cur, &iv, err_span)?)),
                f => Ok(Step::Flow(f)),
            },
        }
    }

    // -----------------------------------------------------------------------
    // 表达式求值
    // -----------------------------------------------------------------------

    /// 求值一个表达式（**错误冒泡时**把当前帧 `span` 覆盖为该错误的 `span`，§10.3 N1）。
    ///
    /// 见 [`Interp::note_error_span`]：保证运行期错误（字段 / 下标 / 一元 / 二元 / 作实参求值等
    /// **非 `Call`** 节点）的 traceback 指向引发错误的**最小 AST 节点**，而非停留在帧初始
    /// `span`（P3 验收 bug-20260924-04）。
    fn eval_expr(&mut self, expr: &Expr, scope: &Scope) -> R<Flow> {
        self.poll(expr.span)?;
        match self.eval_expr_inner(expr, scope) {
            Ok(flow) => Ok(flow),
            Err(e) => {
                self.note_error_span(&e);
                Err(e)
            }
        }
    }

    /// [`Interp::eval_expr`] 的求值体（不含帧 `span` 覆盖）。
    fn eval_expr_inner(&mut self, expr: &Expr, scope: &Scope) -> R<Flow> {
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
                None if name == "argv" => Ok(Flow::Value(self.argv_function.clone())),
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
            let context=def.default_context.as_ref().filter(|_|!def.fields.is_empty());
            let saved=(self.active_module,Rc::clone(&self.globals));
            let defaults_scope;
            let use_scope=if let Some(c)=context {
                if self.depth>=RECURSION_LIMIT{return Err(recursion(self.depth+1,RECURSION_LIMIT,span));}
                if let Some(top)=self.trace.last_mut(){top.span=span;}
                self.depth+=1;self.active_module=c.home_module;
                self.globals=Rc::clone(&self.modules.as_ref().unwrap()[c.home_module.0 as usize].env);
                defaults_scope=Scope{env:Rc::clone(&self.globals),captured:Rc::clone(&c.captured),self_val:c.self_val.clone()};
                self.trace.push(TraceFrame{func_id:c.defaults_func_id,span:def.fields[0].1.span});
                self.trace_sources.push(self.modules.as_ref().unwrap()[c.home_module.0 as usize].source);
                &defaults_scope
            } else {scope};
            let outcome=(|| -> R<Flow> {
                for (fname,expr) in &def.fields {
                    if context.is_some(){if let Some(top)=self.trace.last_mut(){top.span=expr.span;}}
                    let v=match self.eval_expr(expr,use_scope)?{Flow::Value(v)=>v,f=>return Ok(f)};
                    obj.set(fname,v);
                }
                Ok(Flow::Value(Value::Nil))
            })();
            if context.is_some(){self.active_module=saved.0;self.globals=saved.1;}
            match outcome {Err(e)=>return Err(e),Ok(f)=>{
                if context.is_some(){self.trace.pop();self.trace_sources.pop();self.depth-=1;}
                if !matches!(f,Flow::Value(_)){return Ok(f);}
            }}
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
        // P3 验收 bug-20260924-05：管道脱糖产生的 `Call`（`|>`）在右侧非函数时
        // 应报 `TypeMsg::PipeRhsNotFunction`（`管道右侧必须是函数，得到 {t}`），
        // 而非通用的 `NotCallable`。判据见 [`is_pipe_desugared`]。
        let piped = is_pipe_desugared(expr.span, callee);
        match &callee.node {
            // `f(...)`：变量优先（可遮蔽同名内置），否则查内置表。
            ExprKind::Ident(name) => {
                if let Some(v) = self.lookup(scope, name) {
                    let argv = match self.eval_args(args, scope)? {
                        Step::Done(v) => v,
                        Step::Flow(f) => return Ok(f),
                    };
                    return self.call_func(v, argv, span, None, piped);
                }
                if name == "argv" {
                    let args = match self.eval_args(args, scope)? {
                        Step::Done(v) => v,
                        Step::Flow(f) => return Ok(f),
                    };
                    return self.call_func(self.argv_function.clone(), args, span, None, piped);
                }
                if name == "httpRequest" {
                    let argv=match self.eval_args(args,scope)?{Step::Done(v)=>v,Step::Flow(f)=>return Ok(f)};
                    return Ok(Flow::Value(self.http_request(&argv,span)?));
                }
                if name == "exec" {
                    let argv=match self.eval_args(args,scope)?{Step::Done(v)=>v,Step::Flow(f)=>return Ok(f)};
                    return Ok(Flow::Value(self.exec_process(&argv,span)?));
                }
                if name == "attempt" {
                    let argv = match self.eval_args(args, scope)? {
                        Step::Done(v) => v,
                        Step::Flow(f) => return Ok(f),
                    };
                    return Ok(Flow::Value(self.attempt_call(&argv, span)?));
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
                self.call_func(func, argv, span, Some(recv), false)
            }
            // `recv["m"](...)`：下标取到 struct 的字符串键字段（含方法字段）后调用，
            // 与 `recv.m(...)` 等价（§4.5：`s.k ≡ s["k"]`）——同样绑定 `self` 为接收者。
            // 仅 struct + string 键触发绑定；数组下标等其余被调表达式沿用通用路径。
            ExprKind::Index { object, index } => {
                let recv = match self.eval_expr(object, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                let iv = match self.eval_expr(index, scope)? {
                    Flow::Value(v) => v,
                    f => return Ok(f),
                };
                if matches!(recv, Value::Struct(_)) && iv.as_str().is_some() {
                    let func = self.index_read(&recv, &iv, callee.span)?;
                    let argv = match self.eval_args(args, scope)? {
                        Step::Done(v) => v,
                        Step::Flow(f) => return Ok(f),
                    };
                    self.call_func(func, argv, span, Some(recv), false)
                } else {
                    let cv = self.index_read(&recv, &iv, callee.span)?;
                    let argv = match self.eval_args(args, scope)? {
                        Step::Done(v) => v,
                        Step::Flow(f) => return Ok(f),
                    };
                    self.call_func(cv, argv, span, None, piped)
                }
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
                self.call_func(cv, argv, span, None, piped)
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
        piped: bool,
    ) -> R<Flow> {
        match callee {
            Value::Func(cl) => Ok(Flow::Value(self.call_user(&cl, argv, span, self_override)?)),
            other => Err(type_error(non_function_call_msg(&other, piped), span)),
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

    /// §18：仅此边界恢复调用状态；用户副作用及函数ID表不回滚。
    fn http_request(&mut self,args:&[Value],span:Span)->R<Value>{
        use crate::http_control::*;
        let request=validate_http(args,span)?;
        if !self.capabilities.http{return Err(crate::error::value(crate::error::ValueMsg::HttpDisabled,span));}
        let report=run(request,HttpControl{deadline:self.limits.deadline,cancel:self.limits.cancel.clone()});
        self.consume_http_report(report,span)
    }
    fn consume_http_report(&mut self,report:crate::http_control::HttpReport,span:Span)->R<Value>{
        use crate::http_control::HttpFailure as F;
        use crate::error::{ValueMsg as V,ExecutionLimitReason as E};
        self.http_diagnostics.extend(report.diagnostics);
        let response=report.result.map_err(|f|match f {
            F::Cancelled=>crate::error::execution_limit(E::Cancelled,span),F::Deadline=>crate::error::execution_limit(E::Deadline,span),
            F::Timeout=>crate::error::value(V::HttpTimeout,span),F::ResponseLimit=>crate::error::value(V::HttpResponseLimit,span),
            F::Unsupported=>crate::error::value(V::HttpUnsupported,span),
            F::Io{operation,message}=>Box::new(LzError::Io{msg:format!("HTTP {operation}失败：{message}"),span:Some(span)})
        })?;
        let body=String::from_utf8(response.body).map_err(|_|crate::error::value(V::HttpUtf8,span))?;
        let headers=response.headers.into_iter().map(|(k,v)|Value::array(vec![Value::string(k),Value::string(v)])).collect();
        Ok(Value::object(vec![("status".into(),Value::Int(response.status as i64)),("headers".into(),Value::array(headers)),("body".into(),Value::string(body))]))
    }

    fn exec_process(&mut self,args:&[Value],span:Span)->R<Value>{
        use crate::process_control::*;
        let request=validate_exec(args,span)?;
        if !self.capabilities.subprocess{return Err(crate::error::value(crate::error::ValueMsg::ProcessDisabled,span));}
        let report=run(request,ProcessControl{deadline:self.limits.deadline,cancel:self.limits.cancel.clone()});
        self.consume_process_report(report,span)
    }
    fn consume_process_report(&mut self,report:crate::process_control::ProcessReport,span:Span)->R<Value>{
        use crate::process_control::ProcessFailure as F;
        use crate::error::{ValueMsg as V,ExecutionLimitReason as E};
        self.host_diagnostics.extend(report.diagnostics);
        let output=report.result.map_err(|f|match f {
            F::Cancelled=>crate::error::execution_limit(E::Cancelled,span),F::Deadline=>crate::error::execution_limit(E::Deadline,span),
            F::Timeout=>crate::error::value(V::ProcessTimeout,span),F::OutputLimit=>crate::error::value(V::ProcessOutputLimit,span),
            F::Unsupported=>crate::error::value(V::ProcessUnsupported,span),
            F::Io{operation,message}=>Box::new(LzError::Io{msg:format!("子进程{operation}失败：{message}"),span:Some(span)})
        })?;
        let stdout=String::from_utf8(output.stdout).map_err(|_|crate::error::value(V::ProcessUtf8,span))?;
        let stderr=String::from_utf8(output.stderr).map_err(|_|crate::error::value(V::ProcessUtf8,span))?;
        Ok(Value::object(vec![("stdout".into(),Value::string(stdout)),("stderr".into(),Value::string(stderr)),("exitCode".into(),output.exit_code.map_or(Value::Nil,Value::Int)),("signal".into(),output.signal.map_or(Value::Nil,Value::Int))]))
    }

    fn attempt_call(&mut self, args: &[Value], span: Span) -> R<Value> {
        if args.len() != 1 {
            return Err(type_error(TypeMsg::ArgCount { name: "attempt".into(), n: 1, m: args.len() }, span));
        }
        let Value::Func(cl) = &args[0] else {
            return Err(type_error(TypeMsg::NotCallable { t: args[0].type_name().into() }, span));
        };
        if cl.builtin != Some("argv") {
            let Some(user) = &cl.user else {
                return Err(type_error(TypeMsg::NotCallable { t: "function".into() }, span));
            };
            if !user.params.is_empty() {
                return Err(type_error(TypeMsg::ArgCount { name: cl.name.as_deref().unwrap_or("<fn>").into(), n: user.params.len(), m: 0 }, span));
            }
        }
        let saved_sources=self.trace_sources.clone(); let saved_fault=self.fault_source;
        let saved_module=self.active_module;let saved_globals=Rc::clone(&self.globals);
        let saved_trace = self.trace.clone();
        let saved_depth = self.depth;
        let result = match self.call_value(&args[0], &[], span) {
            Ok(value) => Value::object(vec![
                ("ok".into(), Value::Bool(true)), ("value".into(), value), ("error".into(), Value::Nil),
            ]),
            Err(error) if Self::attempt_recoverable(&error) => {
                let location = |n: Option<u32>| n.map(|v| Value::Int(i64::from(v))).unwrap_or(Value::Nil);
                let frames = self.trace.iter().map(|frame| {
                    let name = if frame.func_id == MODULE_FUNC_ID { "<module>" }
                        else { self.func_names.get(frame.func_id as usize).and_then(|n| n.as_deref()).unwrap_or("<fn>") };
                    Value::object(vec![("func".into(), Value::string(name)),
                        ("line".into(), Value::Int(i64::from(frame.span.line))),
                        ("col".into(), Value::Int(i64::from(frame.span.col)))])
                }).collect();
                let data = Value::object(vec![
                    ("class".into(), Value::string(error.class_name())),
                    ("message".into(), Value::string(error.message())),
                    ("line".into(), location(error.span().map(|s| s.line))),
                    ("col".into(), location(error.span().map(|s| s.col))),
                    ("traceback".into(), Value::array(frames)),
                ]);
                Value::object(vec![("ok".into(), Value::Bool(false)), ("value".into(), Value::Nil), ("error".into(), data)])
            }
            Err(error) => return Err(error), // 原样保留故障帧，不吞终止类错误。
        };
        self.trace_sources=saved_sources;self.fault_source=saved_fault;
        self.active_module=saved_module;self.globals=saved_globals;
        self.trace = saved_trace;
        self.depth = saved_depth;
        Ok(result)
    }

    fn attempt_recoverable(error: &LzError) -> bool {
        matches!(error, LzError::Name { .. } | LzError::Type { .. } | LzError::Index { .. }
            | LzError::Field { .. } | LzError::DivZero { .. } | LzError::Value { .. }
            | LzError::Io { .. } | LzError::Assert { .. }
            | LzError::Overflow { msg: OverflowMsg::IntegerOutOfRange, .. })
    }

    /// 用户函数调用：新调用帧 + 参数绑定 +（A2 捕获链）+ 函数体求值。
    fn call_user(
        &mut self,
        cl: &Rc<Closure>,
        argv: Vec<Value>,
        span: Span,
        self_override: Option<Value>,
    ) -> R<Value> {
        if cl.builtin == Some("argv") {
            if !argv.is_empty() {
                return Err(type_error(TypeMsg::ArgCount { name: "argv".into(), n: 0, m: argv.len() }, span));
            }
            return Ok(Value::array(self.script_args.clone()));
        }
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
        let caller_module=self.active_module;let caller_globals=Rc::clone(&self.globals);
        if let Some(modules)=&self.modules { self.active_module=user.home_module;self.globals=Rc::clone(&modules[user.home_module.0 as usize].env); }
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
        if let Some(modules)=&self.modules { self.trace_sources.push(modules[self.active_module.0 as usize].source); }
        let body_result = match &user.body { Body::Block(b)=>self.exec_stmts(&b.stmts,&scope),Body::Expr(e)=>self.eval_expr(e,&scope) };
        self.active_module=caller_module;self.globals=caller_globals;
        let outcome = match body_result {
            Ok(Flow::Value(v))|Ok(Flow::Return(v))=>Ok(v),Ok(_)=>Ok(Value::Nil),Err(e)=>return Err(e),
        };
        if self.modules.is_some(){ self.trace_sources.pop(); }
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
///
/// 与内置 `repeat(n, s)` 同一口径（§10.7 / ADR bug-20260927-04）：`len * n` 超过
/// `isize::MAX`（`String` 最大可分配字节数）时，`str::repeat` 会以 `capacity overflow`
/// **panic**，故提前拦截为受控 `OverflowError`（`容量溢出：所需容量超出可分配上限`）。
fn repeat_str(s: &str, n: i64, span: Span) -> R<Value> {
    if n <= 0 {
        return Ok(Value::string(""));
    }
    let total = s
        .len()
        .checked_mul(n as usize)
        .ok_or_else(|| overflow(span, OverflowMsg::Capacity))?;
    if total > isize::MAX as usize {
        return Err(overflow(span, OverflowMsg::Capacity));
    }
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

/// 格式说明符 `width` 允许的最大值（防御性上限）。
///
/// `width` 仅用于填充（`"0".repeat` / `format!` 拼接），本身不受 `core::fmt` 精度限制；但
/// 超大 `width`（如 `"{:99999999999d}"`）会请求不可行的分配，`String::repeat` / 分配器随之
/// **abort**（`memory allocation of N bytes failed`，非受控错误）。此处设一个远大于正常排版
/// 需要的上限；超出即按 `syntax.md` §2.8「非法说明符 → `ValueError`」拒绝。
const MAX_FMT_WIDTH: usize = 1_000_000;

/// 格式说明符 `precision` 的**硬上限**（`core::fmt` 实现限制）。
///
/// 动态精度 `format!("{:.*}", p, x)` 在 `p >= 65536` 时直接以
/// `Formatting argument out of range` **panic**（实测 `p = 65535` 正常、`65536` 崩溃）。
/// 故 `precision > 65535` 一律按非法说明符拒绝（`syntax.md` §2.8）。
const MAX_FMT_PRECISION: usize = 65_535;

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
            let width = w.parse::<usize>().map_err(|_| bad())?;
            if width > MAX_FMT_WIDTH {
                return Err(bad()); // 超大 width → 受控 ValueError（非分配 abort）
            }
            fs.width = Some(width);
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
            let precision = p.parse::<usize>().map_err(|_| bad())?;
            if precision > MAX_FMT_PRECISION {
                return Err(bad()); // 超大 precision → 受控 ValueError（非 core::fmt panic）
            }
            fs.precision = Some(precision);
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
    use crate::env::Cell;
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

    /// bug-20260924-06 回归（**可运行**部分）：`var` 绑定非 `let`，重绑定**合法**
    /// （§4.5.2 可变性边界：`var` / 形参 / `for` 变量 / `fn` / `struct` 名一律可变）。
    #[test]
    fn var_rebind_is_still_allowed() {
        let stmts = vec![
            decl("b", int(1)),
            assign_var("b", int(2)),
            expr_stmt(ident("b")),
        ];
        assert_eq!(run_str(stmts), "2");
    }

    /// bug-20260924-06 回归：`let` 重绑定 → `TypeError` + 逐字符消息（ADR 2026-09-24 00:30 裁定 1 /
    /// `semantics.md` §4.5.2「`let` 只锁重绑定，不锁内容」）。
    ///
    /// 覆盖两条解析路径：① 顶层 `let`（globals 路径）；② 闭包**捕获**的 `let`（captured cell
    /// 路径，验证捕获时携带的 `mutable = false`）。
    #[test]
    fn let_rebind_is_type_error() {
        // ① 顶层：let a = 1; a = 2 → TypeError
        let top = vec![ldecl("a", int(1)), assign_var("a", int(2))];
        let e = eval_module(&prog(top)).expect_err("let 重绑定应报错");
        assert_eq!(e.class_name(), "TypeError");
        assert_eq!(
            e.message(),
            "不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容"
        );

        // ② 闭包捕获：fn outer() { let x = 1; fn inner() { x = 2; } inner(); } outer()
        let cap = vec![
            fn_decl(
                "outer",
                &[],
                Body::Block(block(vec![
                    ldecl("x", int(1)),
                    fn_decl("inner", &[], Body::Block(block(vec![assign_var("x", int(2))]))),
                    expr_stmt(call("inner", vec![])),
                ])),
            ),
            expr_stmt(call("outer", vec![])),
        ];
        let e = eval_module(&prog(cap)).expect_err("闭包内重绑定捕获的 let 应报错");
        assert_eq!(e.class_name(), "TypeError");
        assert_eq!(
            e.message(),
            "不能重新赋值 let 变量 'x'；let 只锁重绑定，不锁内容"
        );
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
    fn struct_method_via_bracket_index_call_binds_self() {
        // bug-20260927-01：`s["k"]()` 与 `s.k()` 必须等价（semantics.md §4.5 L51 / §3.7）——
        // 经下标取到的方法字段被调用时同样绑定 `self`。
        let norm2 = StructMember::Method(FnDecl {
            span: sp(),
            name: "norm2".to_string(),
            params: vec![],
            body: Body::Expr(bin(
                BinaryOp::Add,
                bin(BinaryOp::Mul, field(self_ref(), "x"), field(self_ref(), "x")),
                bin(BinaryOp::Mul, field(self_ref(), "y"), field(self_ref(), "y")),
            )),
        });
        // 写自身字段的方法：每次 `bump()` 令 `self.n += 1` 并返回新值，
        // 证明方括号取到的方法在调用时 `self` 确为原接收者。
        let bump = StructMember::Method(FnDecl {
            span: sp(),
            name: "bump".to_string(),
            params: vec![],
            body: Body::Block(block(vec![
                st(StmtKind::Assign {
                    target: Lvalue {
                        span: sp(),
                        base: LvalueBase::SelfValue,
                        path: vec![LvalueSeg {
                            span: sp(),
                            kind: LvalueSegKind::Field("n".to_string()),
                        }],
                    },
                    op: AssignOp::Assign,
                    value: bin(BinaryOp::Add, field(self_ref(), "n"), int(1)),
                }),
                expr_stmt(field(self_ref(), "n")),
            ])),
        });
        let stmts = vec![
            struct_decl(
                "Point",
                vec![
                    StructMember::Field(field_init("x", int(0))),
                    StructMember::Field(field_init("y", int(0))),
                    StructMember::Field(field_init("n", int(0))),
                    norm2,
                    bump,
                ],
            ),
            ldecl(
                "p",
                lit_struct("Point", vec![field_init("x", int(3)), field_init("y", int(4))]),
            ),
            // 两种取法结果一致（同一 `self` 接收者）。
            ldecl("a", call_expr(field(ident("p"), "norm2"), vec![])),
            ldecl("b", call_expr(index(ident("p"), sstr("norm2")), vec![])),
            // 经方括号与点号各调一次 `bump`：`self.n` 由 0 累计到 2（写自身字段生效）。
            expr_stmt(call_expr(index(ident("p"), sstr("bump")), vec![])),
            expr_stmt(call_expr(field(ident("p"), "bump"), vec![])),
            // 断言：a == b（等价）且 a == 25 且 p["n"] == 2（self 生效）。
            expr_stmt(logical(
                LogicalOp::And,
                bin(BinaryOp::Eq, ident("a"), ident("b")),
                logical(
                    LogicalOp::And,
                    bin(BinaryOp::Eq, ident("a"), int(25)),
                    bin(BinaryOp::Eq, index(ident("p"), sstr("n")), int(2)),
                ),
            )),
        ];
        assert_eq!(run_str(stmts), "true");
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

    /// bug-20260927-03 同源路径：`string * int` 重复溢出 → 受控
    /// `OverflowError`（`容量溢出：所需容量超出可分配上限`），而非 panic。
    #[test]
    fn string_mul_overflow_is_controlled_overflow_error() {
        let n_bad = 4_611_686_018_427_387_904i64; // 2 * n_bad = 2^63 > isize::MAX
        let err = eval_module(&prog(vec![expr_stmt(bin(
            BinaryOp::Mul,
            sstr("ab"),
            e(ExprKind::Int(n_bad)),
        ))]))
        .unwrap_err();
        assert_eq!(err.class_name(), "OverflowError");
        assert_eq!(err.message(), "容量溢出：所需容量超出可分配上限");
        // 正常 / 非正重复数不受影响。
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Mul, sstr("ab"), int(3)))]),
            "ababab"
        );
        assert_eq!(
            run_str(vec![expr_stmt(bin(BinaryOp::Mul, sstr("ab"), int(0)))]),
            ""
        );
    }

    /// obs-B-03 回归：索引越界写入的插入符位置须与读取一致，指向「最小 AST 节点」的
    /// 首字符 = 基座（`a`），而非下标节 `[`（§8.2）。
    #[test]
    fn index_write_oob_span_points_to_base_like_read() {
        // 写入：Lvalue.span = (5,3)（基座 `a`）；下标节 span = (5,7)（`[`）。
        let at_base = Span::new(5, 3);
        let at_bracket = Span::new(5, 7);
        let write = vec![
            ldecl("a", arr(vec![int(1), int(2), int(3)])),
            st(StmtKind::Assign {
                target: Lvalue {
                    span: at_base,
                    base: LvalueBase::Name("a".to_string()),
                    path: vec![LvalueSeg {
                        span: at_bracket,
                        kind: LvalueSegKind::Index(Spanned::new(ExprKind::Int(5), at_bracket)),
                    }],
                },
                op: AssignOp::Assign,
                value: int(9),
            }),
        ];
        let err = eval_module(&prog(write)).unwrap_err();
        assert_eq!(err.class_name(), "IndexError");
        assert_eq!(err.span(), Some(at_base), "写入越界插入符应指向基座首字符（非 `[`）");

        // 读取：Index 表达式节点 span = 基座起始（parser `parse_postfix` 不变量）。
        let read_base = Span::new(6, 3);
        let read = vec![
            ldecl("a", arr(vec![int(1), int(2), int(3)])),
            expr_stmt(Spanned::new(
                ExprKind::Index {
                    object: Box::new(Spanned::new(ExprKind::Ident("a".to_string()), read_base)),
                    index: Box::new(int(5)),
                },
                read_base,
            )),
        ];
        let err = eval_module(&prog(read)).unwrap_err();
        assert_eq!(err.class_name(), "IndexError");
        assert_eq!(err.span(), Some(read_base), "读取越界插入符指向基座首字符");

        // 中间段字段缺失（read_segment 路径）：错误 span 亦取基座（非 `.`）。
        let at_s = Span::new(8, 2);
        let mid = vec![
            ldecl(
                "s",
                e(ExprKind::StructLit(StructLit {
                    type_name: None,
                    fields: vec![],
                })),
            ),
            st(StmtKind::Assign {
                target: Lvalue {
                    span: at_s,
                    base: LvalueBase::Name("s".to_string()),
                    path: vec![
                        LvalueSeg {
                            span: Span::new(8, 4),
                            kind: LvalueSegKind::Field("b".to_string()),
                        },
                        LvalueSeg {
                            span: Span::new(8, 6),
                            kind: LvalueSegKind::Field("c".to_string()),
                        },
                    ],
                },
                op: AssignOp::Assign,
                value: int(1),
            }),
        ];
        let err = eval_module(&prog(mid)).unwrap_err();
        assert_eq!(err.class_name(), "FieldError");
        assert_eq!(err.span(), Some(at_s), "中间段字段缺失插入符亦应指向基座");
    }

    /// 硬化回归：超大 `width` / `precision` 不得触发分配 abort / `core::fmt` panic，
    /// 须报受控 `ValueError`（`syntax.md` §2.8「非法说明符 → `ValueError`」）。
    #[test]
    fn format_spec_extent_is_bounded_without_panic() {
        // width 超上限（此前会请求 ~100GB 分配并 abort）。
        assert_eq!(
            err_class(vec![expr_stmt(interp(vec![iseg(int(1), Some("99999999999d"))]))]),
            "ValueError"
        );
        // precision 超 `core::fmt` 硬上限 65535（此前 `format!` panic）。
        assert_eq!(
            err_class(vec![expr_stmt(interp(vec![iseg(flt(1.5), Some(".999999999f"))]))]),
            "ValueError"
        );
        assert_eq!(
            err_class(vec![expr_stmt(interp(vec![iseg(flt(1.5), Some(".65536f"))]))]),
            "ValueError"
        );
        // 无类型说明符亦受 width 上界约束。
        assert_eq!(
            err_class(vec![expr_stmt(interp(vec![iseg(int(1), Some("99999999999"))]))]),
            "ValueError"
        );
        // 正常宽度 / 边界内 precision 仍工作。
        assert_eq!(
            run_str(vec![expr_stmt(interp(vec![iseg(int(42), Some("05d"))]))]),
            "00042"
        );
        assert_eq!(
            run_str(vec![expr_stmt(interp(vec![iseg(flt(1.5), Some(".65535f"))]))])
                .chars()
                .count(),
            65537
        );
        // 上限内的较大 width 可用（1000 宽）。
        assert_eq!(
            run_str(vec![expr_stmt(interp(vec![iseg(int(1), Some("1000d"))]))])
                .chars()
                .count(),
            1000
        );
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
            captured: Rc::new(vec![(Rc::from("z"), cell, true)]),
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

#[cfg(test)]
mod t12_tests {
    use super::*;
    fn run(src: &str, args: &[&str]) -> R<Value> {
        let tokens = crate::lexer::lex(src, 0)?;
        let program = crate::parser::parse(&tokens)?;
        eval_module_traced_with_args(&program, &args.iter().map(|s| (*s).to_string()).collect::<Vec<_>>()).result
    }
    #[test]
    fn argv_snapshot_identity_alias_closure_hof_and_shadowing() {
        let src = r#"
assert(argv == argv)
assert(type(argv) == "function")
assert(str(argv) == "<fn argv>")
let f = argv
let saved = f()
saved[0] = "changed"
assert(f()[0] == "中")
let store = [f]
assert(store[0]() == f())
let object = {"call": f}
assert(object.call() == f())
fn capture() => f()
assert(capture() == f())
let out = map((x) => f(), [1,2])
assert(out[0] == f())
fn apply(z) => z()
assert(apply(f) == f())
let argv = () => ["shadow"]
assert(argv()[0] == "shadow")
f()
"#;
        assert_eq!(run(src, &["中", "", "--json"]).unwrap().to_string(), r#"["中", "", "--json"]"#);
        assert_eq!(run("argv()", &[]).unwrap().to_string(), "[]");
        assert_eq!(run("argv()", &["fresh"]).unwrap().to_string(), r#"["fresh"]"#);
        assert_eq!(run("argv()", &[]).unwrap().to_string(), "[]");
    }
    #[test]
    fn argv_bad_arity_has_call_site_and_no_added_user_frame() {
        let tokens = crate::lexer::lex("let f = argv\nf(1)\n", 0).unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        let result = eval_module_traced_with_args(&program, &[]);
        assert_eq!(result.frames.len(), 1);
        let e = result.result.unwrap_err();
        assert_eq!(e.message(), "函数 argv 期待 0 个参数，得到 1");
        assert_eq!(e.span(), Some(Span::new(2, 1)));
    }
}


#[cfg(test)]
mod t12_scope_elision_tests {
    use super::*;
    fn eval(src: &str) -> Value {
        let tokens = crate::lexer::lex(src, 0).unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        eval_module(&program).unwrap()
    }
    #[test]
    fn declaration_free_loop_closures_share_outer_cell() {
        eval(r#"
fn make() {
    var x = 0
    var i = 0
    let fs = [nil, nil, nil]
    while i < 3 {
        fs[i] = () => x
        x += 1
        i += 1
    }
    fs
}
let fs = make()
assert(fs[0]() == 3)
assert(fs[1]() == 3)
assert(fs[2]() == 3)
"#);
    }
    #[test]
    fn loops_with_decl_fn_or_struct_keep_fresh_bindings() {
        eval(r#"
var i = 0
let fs = [nil, nil, nil]
while i < 3 {
    let current = i
    fs[i] = () => current
    i += 1
}
assert(fs[0]() == 0)
assert(fs[1]() == 1)
assert(fs[2]() == 2)
i = 0
while i < 3 {
    fn f() => f
    fs[i] = f
    i += 1
}
assert(fs[0]() == fs[0])
assert(fs[1]() == fs[1])
assert(fs[0] != fs[1])
i = 0
while i < 3 {
    struct S { n: i }
    fs[i] = S {}
    i += 1
}
assert(fs[0].n == 0)
assert(fs[1].n == 1)
assert(fs[2].n == 2)
"#);
    }
    #[test]
    fn nested_shadowing_and_for_cells_are_unchanged() {
        eval(r#"
let marker = 42
var i = 0
let fs = [nil, nil, nil]
while i < 3 {
    if true { let marker = i
        fs[i] = () => marker }
    i += 1
}
assert(marker == 42)
assert(fs[0]() == 0)
assert(fs[1]() == 1)
assert(fs[2]() == 2)
for n in range(3) { fs[n] = () => n }
assert(fs[0]() == 0)
assert(fs[1]() == 1)
assert(fs[2]() == 2)
"#);
    }
    #[test]
    fn declaration_free_loops_preserve_break_continue_return() {
        assert_eq!(eval(r#"
fn work() {
    var i = 0
    var total = 0
    while i < 10 {
        i += 1
        if i == 2 { continue }
        if i == 5 { return total }
        total += i
    }
    -1
}
assert(work() == 8)
var i = 0
while true {
    i += 1
    if i == 3 { break }
}
i
"#).as_int(), Some(3));
    }
    #[test]
    fn empty_scope_has_identical_visible_dump_order_and_values() {
        let interp = Interp::new();
        interp.globals.borrow_mut().define_named("a", Value::Int(7), true);
        interp.globals.borrow_mut().define_named("b", Value::Int(8), false);
        let scope = Scope { env: Rc::clone(&interp.globals), captured: Rc::new(Vec::new()), self_val: None };
        let outer = scope.child();
        outer.env.borrow_mut().define_named("a", Value::Int(9), true);
        let empty = outer.child();
        let visible = |s: &Scope| render_dump(&interp.visible_entries(s));
        assert_eq!(visible(&outer), "a ： 9\nb ： 8\n");
        assert_eq!(visible(&empty), visible(&outer));
    }
}

#[cfg(test)]
mod t13_call_join_tests {
    use super::*;

    fn run(src: &str) -> R<Value> {
        let tokens = crate::lexer::lex(src, 0)?;
        let program = crate::parser::parse(&tokens)?;
        eval_module(&program)
    }

    #[test]
    fn t13_callback_parameters_keep_fresh_cells_and_reassignment() {
        run(r#"
let fs = map((x) => { x += 10
    () => { x += 1
        x }
}, [1, 2, 3])
assert(fs[0]() == 12)
assert(fs[1]() == 13)
assert(fs[0]() == 13)
assert(fs[2]() == 14)
fn direct(x) { x += 1
    x }
assert(direct(8) == 9)
"#).unwrap();
    }

    #[test]
    fn t13_callbacks_keep_snapshot_container_alias_and_left_to_right() {
        run(r#"
let a = [1]
let b = [2]
let source = [a, b]
var order = ""
let results = map((x) => { order = order + str(x[0])
    x[0] += 10
    source[1] = [99]
    x
}, source)
assert(order == "12")
assert(a[0] == 11)
assert(b[0] == 12)
assert(results[0] == a)
assert(results[1] == b)
assert(source[1][0] == 99)
assert(reduce((acc, x) => acc * 10 + x, 0, [1, 2, 3]) == 123)
"#).unwrap();
    }

    #[test]
    fn t13_callback_keeps_captured_self_and_shadowing() {
        run(r#"
let o = { n: 7, build: () => map((x) => self.n + x, [1, 2]) }
assert(o.build() == [8, 9])
let shadow = (argv) => argv + 1
assert(map(shadow, [2, 3]) == [3, 4])
fn direct(x, y) => x + y
assert(direct(2, 3) == 5)
assert(map((x) => direct(x, 4), [1, 2]) == [5, 6])
"#).unwrap();
    }

    #[test]
    fn t13_callback_arity_error_keeps_call_site_without_pushing_frame() {
        let tokens = crate::lexer::lex("let f = (x, y) => x + y\nmap(f, [1])\n", 0).unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        let result = eval_module_traced(&program);
        assert_eq!(result.frames.len(), 1);
        let error = result.result.unwrap_err();
        assert_eq!(error.message(), "函数 <fn> 期待 2 个参数，得到 1");
        assert_eq!(error.span(), Some(Span::new(2, 1)));
    }

    #[test]
    fn t13_callback_body_error_keeps_callback_and_module_frames() {
        let tokens = crate::lexer::lex("fn bad(x) => x / 0\nmap(bad, [1])\n", 0).unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        let result = eval_module_traced(&program);
        assert_eq!(result.result.as_ref().unwrap_err().class_name(), "ZeroDivisionError");
        assert_eq!(result.frames.len(), 2);
        assert_eq!(result.frames[0].span, Span::new(2, 1));
        assert_eq!(result.frame_name(&result.frames[1]).as_ref(), "bad");
    }

    #[test]
    fn t13_join_keeps_unicode_empty_elements_and_source_alias() {
        run(r#"
let xs = ["", "中", "🦀", ""]
let alias = xs
let text = join("::", xs)
assert(text == "::中::🦀::")
assert(join("", ["中", "🦀"]) == "中🦀")
assert(join("/", []) == "")
assert(join("/", [""]) == "")
alias[1] = "changed"
assert(text == "::中::🦀::")
assert(xs[1] == "changed")
"#).unwrap();
    }

    #[test]
    fn t13_join_error_type_order_and_exact_span_are_unchanged() {
        let span = Span::new(9, 7);
        let error = builtins::call("join", &[Value::Int(1), Value::Nil], span).unwrap_err();
        assert_eq!(error.class_name(), "TypeError");
        assert!(error.message().contains("int"));
        assert_eq!(error.span(), Some(span));
        let items = Value::array(vec![Value::string("ok"), Value::Int(1), Value::Bool(true)]);
        let error = builtins::call("join", &[Value::string(","), items], span).unwrap_err();
        assert_eq!(error.class_name(), "TypeError");
        assert!(error.message().contains("int"));
        assert!(!error.message().contains("bool"));
        assert_eq!(error.span(), Some(span));
    }
}

#[cfg(test)]
mod t15_attempt_tests {
    use super::*;
    fn run(src: &str) -> R<Value> {
        let tokens = crate::lexer::lex(src, 0)?;
        let program = crate::parser::parse(&tokens)?;
        eval_module(&program)
    }
    #[test]
    fn t15_attempt_success_and_recovered_json() {
        run(r#"
let good = attempt(() => jsonParse("{\"n\":3}"))
assert(good.ok && good.value.n == 3 && good.error == nil)
let bad = attempt(() => jsonParse("{"))
assert(!bad.ok && bad.value == nil && bad.error["class"] == "ValueError")
assert(attempt(() => nil).ok)
"#).unwrap();
    }
    #[test]
    fn t15_many_failures_nesting_and_future_trace() {
        run(r#"
var i = 0
while i < 15000 {
 let r = attempt(() => 1/0)
 assert(!r.ok)
 i += 1
}
assert(attempt(() => attempt(() => fail("nested"))).value.ok == false)
fn sum(n) { if n == 0 { 0 } else { n + sum(n-1) } }
assert(sum(20) == 210)
"#).unwrap();
        let tokens=crate::lexer::lex("attempt(() => 1/0)\nfn fresh() => missing\nfresh()",0).unwrap();
        let p=crate::parser::parse(&tokens).unwrap();
        let outcome=eval_module_traced(&p);
        assert_eq!(outcome.result.unwrap_err().class_name(), "NameError");
        assert_eq!(outcome.frames.len(),2);
    }
    #[test]
    fn t15_alias_side_effects_and_escaped_closure_ids() {
        run(r#"
let xs=[0]
var escaped=()=>0
let bad=attempt(() => {
 xs[0]=7
 let local=8
 escaped=()=>local
 fail("after write")
})
assert(!bad.ok && xs[0] == 7 && escaped() == 8)
let good=attempt(()=>xs)
good.value[0]=9
assert(xs[0] == 9)
let f=attempt(() => (()=>42))
assert(f.value() == 42)
"#).unwrap();
    }
    #[test]
    fn t15_boundary_validation_and_shadowing() {
        for source in ["attempt()", "attempt(1)", "attempt((x)=>x)", "attempt(unknown)", "attempt(()=>1,2)"] {
            let e=run(source).unwrap_err();
            assert!(matches!(e.class_name(), "TypeError"|"NameError"));
        }
        run(r#"
assert(attempt(() => attempt(1)).error["class"] == "TypeError")
assert(attempt(argv).ok)
fn attempt(x)=>x+2
assert(attempt(3)==5)
"#).unwrap();
    }
    #[test]
    fn t15_recoverable_error_classes_and_json_data() {
        for (body, class) in [
            ("missing", "NameError"), ("1+true", "TypeError"),
            ("[1][9]", "IndexError"), ("({}).missing", "FieldError"),
            ("1/0", "ZeroDivisionError"), ("int(\"bad\")", "ValueError"),
            ("readFile(\"Temp/T15-runtime/not-existing-input.xyz\")", "IOError"),
            ("fail(\"x\")", "AssertionError"),
            ("9223372036854775807+1", "OverflowError")
        ] {
            let src=format!("let r=attempt(() => {body})\nassert(!r.ok && r.error[\"class\"] == \"{class}\")\njsonStringify(r)");
            assert!(run(&src).unwrap().as_str().is_some(),"{body}");
        }
    }
    #[test]
    fn t15_nonrecoverable_classification_and_preserved_state() {
        let span=Span::new(9,4);
        let errors=[LzError::Recursion {depth:10001,limit:10000,span},
            LzError::Overflow {span,msg:OverflowMsg::Capacity},
            LzError::CosmosAnswer {span},
            LzError::Syntax {span,msg:crate::error::SyntaxMsg::IncompleteExpr}];
        for error in errors { assert!(!Interp::attempt_recoverable(&error)); }
        let f=run("()=>1").unwrap();
        let mut interp=Interp::new();
        interp.depth=RECURSION_LIMIT;
        interp.trace.push(TraceFrame {func_id:0,span});
        let before=interp.trace.clone();
        assert_eq!(interp.attempt_call(&[f],span).unwrap_err().class_name(),"RecursionError");
        assert_eq!(interp.trace,before);
        assert_eq!(interp.depth,RECURSION_LIMIT);
    }
    #[test]
    fn t15_restores_complete_callers_span_on_success_and_failure() {
        for source in ["()=>1", "()=>1/0"] {
            let f=run(source).unwrap();
            let mut interp=Interp::new();
            let saved=TraceFrame {func_id:0,span:Span::new(33,9)};
            interp.trace.push(saved);
            let result=interp.attempt_call(&[f],Span::new(7,2)).unwrap();
            assert!(result.as_struct().is_some());
            assert_eq!(interp.trace, vec![saved]);
            assert_eq!(interp.depth,0);
        }
    }

}

#[cfg(test)]
mod t16_a_origin_tests {
    use super::*;
    use crate::source::{ModuleId, SourceId, SourceMap, SourceMetadataError as E, SourceRecord};
    fn run(text: &str) -> TracedRun {
        let tokens = crate::lexer::lex(text, 0).unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        eval_module_traced(&program)
    }
    fn sources() -> Rc<SourceMap> {
        let mut map = SourceMap::new();
        for name in ["entry.lfz", "lib.lfz"] {
            map.insert(SourceRecord { identity: None, display_path: name.into(), loaded: None }).unwrap();
        }
        Rc::new(map)
    }
    fn attach_error(run: &mut TracedRun) {
        run.attach_origins(sources(), vec![SourceId(0); run.frames.len()], Some(SourceId(0))).unwrap();
    }
    fn retained(run: &TracedRun) {
        assert_eq!(run.error_source().unwrap().display_path, "entry.lfz");
        assert_eq!(run.frame_source(0).unwrap().display_path, "entry.lfz");
    }
    #[test]
    fn t16_a_legacy_origins_are_absent() {
        for text in ["1", "missing", "fn f() => 1/0\nf()"] {
            let out = run(text);
            assert!(out.origins.is_none());
            assert!(out.frame_source(0).is_none());
            assert!(out.error_source().is_none());
        }
    }
    #[test]
    fn t16_a_home_module_is_integer_entry_for_all_legacy_closures() {
        for text in ["() => 1", "fn f() => 1\nf", "fn outer() => () => 1\nouter()"] {
            let out = run(text);
            let Value::Func(f) = out.result.unwrap() else { panic!("expected closure") };
            assert_eq!(f.user.as_ref().unwrap().home_module, ModuleId::ENTRY);
        }
        assert_eq!(Interp::new().active_module, ModuleId::ENTRY);
    }
    #[test]
    fn t16_a_valid_sources_use_frame_indices_and_can_replace_atomically() {
        let mut out = run("fn f() => 1/0\nf()");
        assert_eq!(out.frames.len(), 2);
        out.attach_origins(sources(), vec![SourceId(0), SourceId(1)], Some(SourceId(1))).unwrap();
        assert_eq!(out.frame_source(0).unwrap().display_path, "entry.lfz");
        assert_eq!(out.frame_source(1).unwrap().display_path, "lib.lfz");
        assert_eq!(out.error_source().unwrap().display_path, "lib.lfz");
        assert!(out.frame_source(2).is_none());
        assert!(out.frame_source(usize::MAX).is_none());
        attach_error(&mut out);
        retained(&out);
    }
    #[test]
    fn t16_a_rejected_error_metadata_preserves_old_sidecar_and_order() {
        let mut out = run("fn f() => 1/0\nf()");
        attach_error(&mut out);
        assert_eq!(out.attach_origins(sources(), vec![SourceId(99)], None), Err(E::FrameCount { expected: 2, actual: 1 }));
        retained(&out);
        assert_eq!(out.attach_origins(sources(), vec![SourceId(99), SourceId(98)], None), Err(E::UnknownSource { id: SourceId(99) }));
        retained(&out);
        assert_eq!(out.attach_origins(sources(), vec![SourceId(0), SourceId(1)], None), Err(E::MissingErrorSource));
        retained(&out);
        assert_eq!(out.attach_origins(sources(), vec![SourceId(0), SourceId(1)], Some(SourceId(99))), Err(E::UnknownSource { id: SourceId(99) }));
        retained(&out);
    }
    #[test]
    fn t16_a_success_metadata_rejects_error_source_before_its_lookup() {
        let mut out = run("1");
        out.attach_origins(sources(), vec![SourceId(1)], None).unwrap();
        assert_eq!(out.attach_origins(sources(), vec![SourceId(0)], Some(SourceId(99))), Err(E::UnexpectedErrorSource));
        assert_eq!(out.frame_source(0).unwrap().display_path, "lib.lfz");
        assert!(out.error_source().is_none());
    }
    #[test]
    fn t16_a_host_mutation_and_missing_loaded_text_remain_safe() {
        let mut out = run("missing");
        attach_error(&mut out);
        assert!(out.frame_source(0).unwrap().loaded.is_none());
        out.frames.clear();
        assert!(out.frame_source(0).is_none());
        out.result = Ok(Value::Nil);
        assert!(out.error_source().is_none());
        let mut out = run("1");
        out.attach_origins(sources(), vec![SourceId(0)], None).unwrap();
        out.result = Err(type_error(TypeMsg::NotCallable { t: "nil".into() }, Span::new(1, 1)));
        assert!(out.error_source().is_none());
    }
    #[test]
    fn t16_a_attempt_keeps_entry_and_fixed_old_record_shape() {
        let tokens = crate::lexer::lex("() => 1/0", 0).unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        let mut interp = Interp::new();
        let scope = Scope { env: Rc::clone(&interp.globals), captured: Rc::new(Vec::new()), self_val: None };
        let Flow::Value(f) = interp.exec_stmts(&program.stmts, &scope).unwrap() else { panic!("closure") };
        let saved = TraceFrame { func_id: 0, span: Span::new(9, 3) };
        interp.trace.push(saved);
        let result = interp.attempt_call(&[f], Span::new(5, 2)).unwrap();
        assert_eq!(interp.active_module, ModuleId::ENTRY);
        assert_eq!(interp.trace, vec![saved]);
        assert_eq!(interp.depth, 0);
        let json = crate::json_codec::stringify(&result, Span::new(1, 1)).unwrap();
        assert!(!json.contains("file"));
        run(r#"let r = attempt(() => 1/0)
assert(keys(r) == ["error", "ok", "value"])
assert(keys(r.error) == ["class", "col", "line", "message", "traceback"])
assert(keys(r.error.traceback[0]) == ["col", "func", "line"])
"#).result.unwrap();
    }
}

#[cfg(test)]
mod t16_b_execution_guard_tests {
    use super::*;
    fn parse(text: &str) -> Program {
        crate::parser::parse(&crate::lexer::lex(text, 0).unwrap()).unwrap()
    }
    fn check_error(error: &LzError, span: Span) {
        assert_eq!(error.class_name(), "SyntaxError");
        assert_eq!(error.message(), "模块执行尚未启用");
        assert_eq!(error.span(), Some(span));
    }
    #[test]
    fn t16_b_all_old_evaluation_entrypoints_preflight_entire_root() {
        let p = parse("let x = 1\nexport x");
        let at = Span::new(2, 1);
        check_error(&eval_module(&p).unwrap_err(), at);
        check_error(&run(&p).unwrap_err(), at);
        for out in [eval_module_traced(&p), eval_module_traced_with_args(&p, &["a".into()]),
            on_eval_stack(|| eval_module_traced_on_thread(&p)),
            on_eval_stack(|| eval_module_traced_on_thread_with_args(&p, &["a".into()]))] {
            check_error(out.result.as_ref().unwrap_err(), at);
            assert_eq!(out.frames, vec![TraceFrame { func_id: 0, span: at }]);
            assert_eq!(out.frame_name(&out.frames[0]).as_ref(), "<module>");
            assert!(out.frame_source(0).is_none());
        }
    }
    #[test]
    fn t16_b_late_export_cannot_overwrite_file_or_execute_earlier_failure() {
        let dir = std::path::Path::new("Temp/T16-B-runtime");
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join("preflight-sentinel.txt");
        std::fs::write(&path, b"original\0bytes").unwrap();
        let text = "writeFile(\"Temp/T16-B-runtime/preflight-sentinel.txt\", \"changed\")\nlet x=1\nexport x";
        let out = eval_module_traced(&parse(text));
        check_error(out.result.as_ref().unwrap_err(), Span::new(3, 1));
        assert_eq!(std::fs::read(&path).unwrap(), b"original\0bytes");
        let out = eval_module_traced(&parse("missing_name\nexport missing_name"));
        check_error(out.result.as_ref().unwrap_err(), Span::new(2, 1));
    }
    #[test]
    fn t16_b_first_declaration_determines_guard_without_loading_dependency() {
        let p = parse("import a from \"./does-not-exist.lfz\"\nexport a");
        check_error(eval_module_traced(&p).result.as_ref().unwrap_err(), Span::new(1, 1));
        let p = parse("let export = 2\nexport + 1");
        assert_eq!(eval_module(&p).unwrap().to_string(), "3");
    }
}

#[cfg(test)]
mod t16_c_tests {
 use super::*;
 use crate::modules::*;
 use crate::source::*;
 pub(super) fn graph(texts:&[&str]) -> ModuleGraph {
  let mut sources=SourceMap::new();let mut units=vec![];
  for (i,text) in texts.iter().enumerate(){
   let loaded=crate::loader::load_source(text).unwrap();let p=crate::parser::parse(&crate::lexer::lex(&loaded.text,0).unwrap()).unwrap();
   let sid=sources.insert(SourceRecord{identity:None,display_path:format!("m{i}.lfz"),loaded:Some(loaded)}).unwrap();
   let imports=p.stmts.iter().filter_map(|s|if let StmtKind::ImportDecl(d)=&s.node{Some(ResolvedImport{alias:d.alias.clone(),alias_span:d.alias_span,path_span:d.path_span,target:ModuleId(1),span:s.span})}else{None}).collect();
   let exports=p.stmts.iter().flat_map(|s|if let StmtKind::ExportDecl(d)=&s.node{d.names.iter().map(|n|ExportBinding{name:n.name.clone(),span:n.span}).collect::<Vec<_>>()}else{vec![]}).collect();
   units.push(ModuleUnit{id:ModuleId(i as u32),source_id:sid,program:p,imports,exports});
  }
  ModuleGraph{sources:Rc::new(sources),initialization_order:(0..texts.len()).rev().map(|i|ModuleId(i as u32)).collect(),entry:ModuleId::ENTRY,units}
 }
 #[test] fn t16_c_module_globals_and_session_isolation(){
  for _ in 0..2 {on_eval_stack(|| { let g=graph(&["import lib from \"./lib.lfz\"\nlet n=100\nassert(lib.inc()==4)\nassert(lib.inc()==5)","var n=3\nfn inc(){n+=1\n n}\nexport inc"]);
  eval_graph_traced_on_thread_with_args(&g,&[]).unwrap().result.unwrap(); });}
 }

 #[test] fn t16_c_defaults_use_home_and_capture_while_overrides_use_caller(){on_eval_stack(||{
  let g=graph(&["import lib from \"./lib.lfz\"\nlet n=100\nlet P=lib.P\nassert(P{}.x==7)\nassert(P{x:n}.x==100)\nlet maker=lib.maker\nlet Q=maker(9)\nassert(Q{}.x==9)","let n=7\nstruct P {x:n}\nfn maker(v){struct Q{x:v}\n Q}\nexport P,maker"]);
  eval_graph_traced_on_thread_with_args(&g,&[]).unwrap().result.unwrap();
 });}
 #[test] fn t16_c_attempt_restores_module_source_and_keeps_escaped_function_id(){on_eval_stack(||{
  let g=graph(&["import lib from \"./lib.lfz\"\nlet n=90\nlet r=attempt(()=>lib.bad())\nassert(!r.ok)\nassert(n==90)\nassert(lib.saved[0]()==8)\nmissing_final","let n=8\nlet saved=[nil]\nfn bad(){saved[0]=()=>n\n 1/0}\nexport bad,saved"]);
  let out=eval_graph_traced_on_thread_with_args(&g,&[]).unwrap();
  assert_eq!(out.result.as_ref().unwrap_err().class_name(),"NameError");
  assert_eq!(out.frames.len(),1);assert_eq!(out.error_source().unwrap().display_path,"m0.lfz");
 });}
 #[test] fn t16_c_error_and_default_frames_have_definition_source(){on_eval_stack(||{
  for (root,dep,expected) in [
   ("import lib from \"./lib.lfz\"\nlib.bad()","fn bad()=>1/0\nexport bad","bad"),
   ("import lib from \"./lib.lfz\"\nlet P=lib.P\nP{}","struct P{x:1/0}\nexport P","P::<defaults>")
  ]{
   let g=graph(&[root,dep]);let out=eval_graph_traced_on_thread_with_args(&g,&[]).unwrap();
   assert_eq!(out.result.as_ref().unwrap_err().class_name(),"ZeroDivisionError");
   assert_eq!(out.error_source().unwrap().display_path,"m1.lfz");
   assert_eq!(out.frame_source(0).unwrap().display_path,"m0.lfz");
   assert_eq!(out.frame_source(1).unwrap().display_path,"m1.lfz");
   assert_eq!(out.frame_name(&out.frames[1]).as_ref(),expected);
  }
 });}
 #[test] fn t16_c_alias_snapshot_self_and_argv_are_shared_per_session(){on_eval_stack(||{
  let g=graph(&["import lib from \"./lib.lfz\"\nlet same=lib\nassert(lib.a==1)\nassert(lib.inc()==2)\nassert(lib.a==1)\nsame.a=50\nassert(lib.a==50)\nassert(lib.arg()[0]==\"x\")\nassert(lib.method()==50)","var a=1\nfn inc(){a+=1\n a}\nfn arg()=>argv()\nfn method()=>self.a\nexport a,inc,arg,method"]);
  eval_graph_traced_on_thread_with_args(&g,&["x".into()]).unwrap().result.unwrap();
 });}
 #[test] fn t16_c_invalid_graphs_are_rejected_before_initialization(){on_eval_stack(||{
  for mutation in 0..7{
   let mut g=graph(&["import lib from \"./lib.lfz\"\nmissing_root","missing_dep\nexport missing_dep"]);
   match mutation{
    0=>g.entry=ModuleId(99),1=>g.units[1].id=ModuleId(0),2=>g.units[1].source_id=g.units[0].source_id,
    3=>g.initialization_order=vec![ModuleId(0),ModuleId(1)],4=>g.units[0].imports[0].alias="changed".into(),
    5=>g.units[1].exports[0].name="changed".into(),_=>g.units[0].imports[0].target=ModuleId(99)
   }
   assert!(matches!(eval_graph_traced_on_thread_with_args(&g,&[]),Err(GraphExecutionError::InvalidGraph{..})));
  }
 });}
 #[test] fn t16_c_initialization_failure_never_runs_entry(){on_eval_stack(||{
  let g=graph(&["import lib from \"./lib.lfz\"\nfail(\"entry must not execute\")","missing_dependency"]);
  let out=eval_graph_traced_on_thread_with_args(&g,&[]).unwrap();
  assert_eq!(out.result.as_ref().unwrap_err().class_name(),"NameError");
  assert_eq!(out.frames.len(),1);assert_eq!(out.error_source().unwrap().display_path,"m1.lfz");
 });}

 #[test] fn t16_c_defaults_resource_check_precedes_home_switch(){on_eval_stack(||{
  let g=graph(&["struct P{x:1}\nP"]);
  let value=eval_graph_traced_on_thread_with_args(&g,&[]).unwrap().result.unwrap();
  let mut i=Interp::new();i.globals.borrow_mut().define_named("P",value,false);
  i.modules=Some(vec![ModuleInstance{env:Rc::clone(&i.globals),namespace:None,state:ModuleState::Ready,source:SourceId(0)}]);
  let scope=Scope{env:Rc::clone(&i.globals),captured:Rc::new(vec![]),self_val:None};
  let p=crate::parser::parse(&crate::lexer::lex("P{}",0).unwrap()).unwrap();
  let StmtKind::Expr(expr)=&p.stmts[0].node else{panic!("expression")};
  let saved=TraceFrame{func_id:0,span:Span::new(11,7)};
  i.trace.push(saved);i.trace_sources.push(SourceId(0));i.depth=RECURSION_LIMIT;
  let error=match i.eval_expr(expr,&scope){Err(e)=>e,Ok(_)=>panic!("expected resource error")};assert_eq!(error.class_name(),"RecursionError");
  assert_eq!(i.depth,RECURSION_LIMIT);assert_eq!(i.trace.len(),1);assert_eq!(i.trace_sources,vec![SourceId(0)]);
  assert_eq!(i.active_module,ModuleId::ENTRY);
 });}
 #[test] fn t16_c_default_return_propagates_to_enclosing_function(){on_eval_stack(||{
  let g=graph(&["fn build(){struct P{x:if true {return 7}}\n P{}\n 99}\nassert(build()==7)"]);
  eval_graph_traced_on_thread_with_args(&g,&[]).unwrap().result.unwrap();
 });}
}

#[cfg(test)]
mod t17_tests {
 use super::*;
 #[test] fn t17_api_boundary() {
  let p=crate::parser::parse(&crate::lexer::lex("1",0).unwrap()).unwrap();
  let r=eval_module_traced_with_args_and_limits(&p,&[],EvalLimits{max_steps:Some(2),..Default::default()});
  assert!(r.result.is_ok());
 }
}

#[cfg(test)]
mod t17_boundaries {
 use super::*;
 use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
 use std::time::{Instant,Duration};
 fn run(text:&str,l:EvalLimits)->TracedRun {on_eval_stack(move || {
  let p=crate::parser::parse(&crate::lexer::lex(text,0).unwrap()).unwrap();
  eval_module_traced_on_thread_with_args_and_limits(&p,&[],l)
 })}
 fn budget(n:u64)->EvalLimits{EvalLimits{max_steps:Some(n),..Default::default()}}
 fn reason(r:TracedRun,want:crate::error::ExecutionLimitReason){assert!(matches!(*r.result.unwrap_err(),LzError::ExecutionLimit{reason,..} if reason==want));}
 #[test] fn exact_steps_and_no_final_poll(){
  use crate::error::ExecutionLimitReason::StepBudget;
  for _ in 0..3 {reason(run("1",budget(0)),StepBudget);reason(run("1",budget(1)),StepBudget);
   assert!(run("1",budget(2)).result.is_ok());assert!(run("let x=1\nx",budget(4)).result.is_ok());
   reason(run("let x=1\nx",budget(3)),StepBudget);assert!(run("1",budget(u64::MAX)).result.is_ok());}
 }
 #[test] fn empty_has_no_poll_and_cancel_priority(){
  let token=Arc::new(AtomicBool::new(true));let l=EvalLimits{max_steps:Some(0),deadline:Some(Instant::now()),cancel:Some(token.clone())};
  assert!(run("",l.clone()).result.is_ok());reason(run("1",l.clone()),crate::error::ExecutionLimitReason::Cancelled);
  reason(run("1",l),crate::error::ExecutionLimitReason::Cancelled);
  assert!(token.load(Ordering::Acquire));assert!(run("1",Default::default()).result.is_ok());
 }
 #[test] fn deadline_priority_and_future(){
  reason(run("1",EvalLimits{deadline:Some(Instant::now()),..budget(0)}),crate::error::ExecutionLimitReason::Deadline);
  assert!(run("1",EvalLimits{deadline:Some(Instant::now()+Duration::from_secs(60)),..budget(2)}).result.is_ok());
 }
 #[test] fn loops_recursion_hof_and_nested_attempt_stop(){
  for text in ["while true {}","fn f(){f()}\nf()","map((x)=>x+1,range(100))","attempt(()=>attempt(()=>{while true {}}))","while true {attempt(()=>1/0)}"] {
   reason(run(text,budget(40)),crate::error::ExecutionLimitReason::StepBudget);
  }
 }
 #[test] fn cancellation_between_nodes_and_late_flag(){
  let token=Arc::new(AtomicBool::new(false));let mut i=Interp::new();i.limits.cancel=Some(token.clone());
  i.poll(Span::new(1,1)).unwrap();token.store(true,Ordering::Release);
  assert!(matches!(*i.poll(Span::new(1,1)).unwrap_err(),LzError::ExecutionLimit{reason:crate::error::ExecutionLimitReason::Cancelled,..}));
  let r=run("1",Default::default());token.store(true,Ordering::Release);assert!(r.result.is_ok());
 }
 #[test] fn graph_budget_and_fault_source(){on_eval_stack(||{
  let g=super::t16_c_tests::graph(&["import m from \"./m.lfz\"\nm.f()","fn f(){while true {}}\nexport f"]);
  let r=eval_graph_traced_on_thread_with_args_and_limits(&g,&[],budget(40)).unwrap();
  assert_eq!(r.error_source().unwrap().display_path,"m1.lfz");reason(r,crate::error::ExecutionLimitReason::StepBudget);
 });}
 #[test] fn cross_thread_cancel_after_session_has_executed_and_run_isolation(){
  // 在真实LFZ首语句完成后握手；取消者是另一OS线程，不依赖sleep。
  let token=Arc::new(AtomicBool::new(false));
  let (started_tx,started_rx)=std::sync::mpsc::sync_channel(0);
  let (cancelled_tx,cancelled_rx)=std::sync::mpsc::sync_channel(0);
  let writer=token.clone();
  let host=std::thread::spawn(move || {started_rx.recv().unwrap();writer.store(true,Ordering::Release);cancelled_tx.send(()).unwrap();});
  on_eval_stack(move || {
   let p=crate::parser::parse(&crate::lexer::lex("var x=1\nwhile true {x+=1}",0).unwrap()).unwrap();
   let mut i=Interp::new();i.limits=EvalLimits{cancel:Some(token),..budget(100)};
   let scope=Scope{env:i.globals.clone(),captured:Rc::new(vec![]),self_val:None};
   assert!(i.exec_stmt(&p.stmts[0],&scope).is_ok());
   assert_eq!(i.globals.borrow().get_local(0).unwrap().as_int(),Some(1));
   started_tx.send(()).unwrap();cancelled_rx.recv().unwrap();
   let e=match i.exec_stmt(&p.stmts[1],&scope){Err(e)=>e,Ok(_)=>panic!("cancel must stop next statement")};
   assert!(matches!(*e,LzError::ExecutionLimit{reason:crate::error::ExecutionLimitReason::Cancelled,span} if span==p.stmts[1].span));
  });
  host.join().unwrap();
  assert!(run("1",EvalLimits{cancel:Some(Arc::new(AtomicBool::new(false))),..Default::default()}).result.is_ok());
 }
 #[test] fn all_seven_hofs_propagate_callback_limits_through_attempt(){
  for name in crate::builtins::HOF_NAMES {
   let args=if *name=="reduce" {"(a,x)=>{while true {}},0,[1]"} else {"(x)=>{while true {}},[1]"};
   let text=format!("attempt(()=>{name}({args}))");
   let r=run(&text,budget(50));
   assert!(r.frames.len()>=3,"{name}: callback frame must survive");
   reason(r,crate::error::ExecutionLimitReason::StepBudget);
  }
 }
 #[test] fn diamond_counts_shared_dependency_once_and_has_one_session_budget(){on_eval_stack(||{
  use crate::source::ModuleId;
  let mut g=super::t16_c_tests::graph(&[
   "import left from \"./left.lfz\"\nimport right from \"./right.lfz\"\n1",
   "import leaf from \"./leaf.lfz\"\nexport leaf",
   "import leaf from \"./leaf.lfz\"\nexport leaf",
   "let x=1\nexport x"]);
  g.units[0].imports[1].target=ModuleId(2);
  g.units[1].imports[0].target=ModuleId(3);g.units[2].imports[0].target=ModuleId(3);
  assert!(eval_graph_traced_on_thread_with_args_and_limits(&g,&[],budget(11)).unwrap().result.is_ok());
  let r=eval_graph_traced_on_thread_with_args_and_limits(&g,&[],budget(10)).unwrap();
  assert_eq!(r.error_source().unwrap().display_path,"m0.lfz");
  reason(r,crate::error::ExecutionLimitReason::StepBudget);
 });}
 #[test] fn template_default_limit_preserves_definition_source_and_frame(){on_eval_stack(||{
  let g=super::t16_c_tests::graph(&[
   "import lib from \"./lib.lfz\"\nlet P=lib.P\nattempt(()=>P{})",
   "struct P{x:if true {while true {}}}\nexport P"]);
  let r=eval_graph_traced_on_thread_with_args_and_limits(&g,&[],budget(50)).unwrap();
  assert_eq!(r.error_source().unwrap().display_path,"m1.lfz");
  let last=r.frames.len()-1;
  assert_eq!(r.frame_source(last).unwrap().display_path,"m1.lfz");
  assert!(r.func_names[r.frames[last].func_id as usize].as_ref().unwrap().contains("<defaults>"));
  assert_eq!(r.result.as_ref().unwrap_err().span().unwrap().line,1);
  reason(r,crate::error::ExecutionLimitReason::StepBudget);
 });}

}
#[cfg(test)]
mod t18_runtime_tests {
 use super::*;
 use crate::process_control::*;
 fn args()->Vec<Value>{let exe=if cfg!(windows){"C:\\test.exe"}else{"/bin/true"};vec![Value::array(vec![Value::string(exe)]),Value::Int(10),Value::Int(0)]}
 #[test] fn executable_validation_uses_backend_platform_rules_without_spawn(){
  let paths=if cfg!(windows){vec![(r"C:\test.exe",true),(r"\\server\share\tool.EXE",true),(r"C:test.exe",false),(r"\\?\C:\test.exe",false),(r"C:\tool.cmd",false)]}
   else {vec![("/bin/true",true),("/tmp/空 格",true),("bin/true",false),("",false)]};
  for (path,valid) in paths {
   let mut a=args();a[0]=Value::array(vec![Value::string(path)]);
   assert_eq!(crate::process_control::executable_valid(path),valid,"backend {path}");
   assert_eq!(validate_exec(&a,Span::START).is_ok(),valid,"runtime {path}");
  }
 }
 #[test] fn exec_validation_order_and_disabled_without_spawn(){
  let at=Span::START;let mut a=args();
  assert!(validate_exec(&a,at).is_ok());
  a[0]=Value::array(vec![Value::string("bad\0"),Value::Int(1)]);
  assert_eq!(validate_exec(&a,at).unwrap_err().class_name(),"TypeError");
  a[0]=Value::array(vec![]);assert_eq!(validate_exec(&a,at).unwrap_err().message(),"子进程 argv 不能为空");
  a=args();a[1]=Value::Int(0);a[2]=Value::Nil;
  assert_eq!(validate_exec(&a,at).unwrap_err().class_name(),"TypeError");
  for path in ["test.exe","C:foo.exe","C:\\foo.cmd","\\\\?\\C:\\foo.exe"] {
   a=args();a[0]=Value::array(vec![Value::string(path)]);
   assert_eq!(validate_exec(&a,at).unwrap_err().message(),"子进程程序必须是绝对可执行文件路径");
  }
  let mut i=Interp::new();assert_eq!(i.exec_process(&args(),at).unwrap_err().message(),"子进程能力未启用");
  assert_eq!(builtins::call("exec",&args(),at).unwrap_err().message(),"子进程能力未启用");
 }
 #[test] fn report_utf8_and_unsigned_exit_mapping(){
  let mut i=Interp::new();let at=Span::START;
  let value=i.consume_process_report(ProcessReport{result:Ok(ProcessOutput{stdout:b"ok".to_vec(),stderr:vec![],exit_code:Some(4294967295),signal:None}),diagnostics:vec![]},at).unwrap();
  assert_eq!(value.as_struct().unwrap().borrow().data_len(),4);
  let error=i.consume_process_report(ProcessReport{result:Ok(ProcessOutput{stdout:vec![255],stderr:vec![],exit_code:Some(0),signal:None}),diagnostics:vec![]},at).unwrap_err();
  assert_eq!(error.message(),"子进程输出不是合法 UTF-8");
 }
 #[test] fn global_termination_and_cleanup_diagnostics_are_not_business_result(){
  for failure in [ProcessFailure::Cancelled,ProcessFailure::Deadline]{
   let mut i=Interp::new();let error=i.consume_process_report(ProcessReport{result:Err(failure),diagnostics:vec!["cleanup witness".into()]},Span::START).unwrap_err();
   assert_eq!(error.class_name(),"ExecutionLimitError");assert!(!Interp::attempt_recoverable(&error));
   assert_eq!(i.host_diagnostics,vec!["cleanup witness"]);
  }
 }
 #[test] fn attempt_preserves_cleanup_diagnostics_and_context_defaults_off(){on_eval_stack(||{
  let p=crate::parser::parse(&crate::lexer::lex("attempt(()=>1/0)",0).unwrap()).unwrap();
  let mut i=Interp::new();
  let e=i.consume_process_report(ProcessReport{result:Err(ProcessFailure::Io{operation:"回收".into(),message:"injected".into()}),diagnostics:vec!["injected cleanup".into()]},Span::START).unwrap_err();
  assert!(Interp::attempt_recoverable(&e));
  let scope=Scope{env:i.globals.clone(),captured:Rc::new(vec![]),self_val:None};
  assert!(i.exec_stmts(&p.stmts,&scope).is_ok());assert_eq!(i.host_diagnostics,vec!["injected cleanup"]);
  assert!(!EvalContext::default().capabilities.subprocess);
  assert!(eval_module_traced_on_thread(&p).host_diagnostics().is_empty());
 });}
}

#[cfg(test)]
mod t18b_runtime_tests {
 use super::*;
 use crate::http_control::*;
 fn args()->Vec<Value>{vec![Value::string("GET"),Value::string("http://127.0.0.1/"),Value::array(vec![]),Value::string(""),Value::Int(100),Value::Int(1000)]}
 #[test] fn validation_order_and_shared_helpers_without_network(){
  let mut a=args();assert!(validate_http(&a,Span::START).is_ok());
  a[0]=Value::string("invalid");a[5]=Value::Nil;
  assert_eq!(validate_http(&a,Span::START).unwrap_err().class_name(),"TypeError");
  a=args();a[0]=Value::string("invalid");a[2]=Value::array(vec![Value::array(vec![Value::Int(1),Value::string("x")])]);
  assert_eq!(validate_http(&a,Span::START).unwrap_err().class_name(),"TypeError");
  a=args();a[0]=Value::string("get");a[1]=Value::string("invalid");
  assert_eq!(validate_http(&a,Span::START).unwrap_err().message(),"HTTP method 不受支持");
  a=args();a[2]=Value::array(vec![Value::array(vec![Value::string("Host"),Value::string("x")])]);
  assert_eq!(validate_http(&a,Span::START).unwrap_err().message(),"HTTP headers 非法或超出上限");
  a=args();a[3]=Value::string("x");assert_eq!(validate_http(&a,Span::START).unwrap_err().message(),"HTTP body 非法或超出上限");
  a=args();a[2]=Value::array(vec![Value::array(vec![Value::string("X-Test"),Value::string(" \tx\t ")])]);
  assert_eq!(validate_http(&a,Span::START).unwrap().headers,vec![("X-Test".into(),"x".into())]);
 }
 #[test] fn disabled_and_context_capabilities_are_independent(){
  for subprocess in [false,true]{
   let mut i=Interp::new();i.capabilities=EvalCapabilities{subprocess,http:false};
   assert_eq!(i.http_request(&args(),Span::START).unwrap_err().message(),"HTTP 能力未启用");
  }
  assert!(!EvalContext::default().capabilities.http);
  assert_eq!(builtins::call("httpRequest",&args(),Span::START).unwrap_err().message(),"HTTP 能力未启用");
 }
 #[test] fn non_success_http_status_is_value_duplicate_headers_and_strict_utf8(){
  let mut i=Interp::new();
  let value=i.consume_http_report(HttpReport{result:Ok(HttpResponse{status:404,headers:vec![("set-cookie".into(),"a".into()),("set-cookie".into(),"b".into())],body:b"not found".to_vec()}),diagnostics:vec![]},Span::START).unwrap();
  let object=value.as_struct().unwrap().borrow();
  assert_eq!(object.raw_fields().iter().map(|(k,_)|k.as_str()).collect::<Vec<_>>(),vec!["status","headers","body"]);
  assert_eq!(object.get("status").unwrap().as_int(),Some(404));
  assert_eq!(object.get("headers").unwrap().as_array().unwrap().borrow().len(),2);
  let e=i.consume_http_report(HttpReport{result:Ok(HttpResponse{status:200,headers:vec![],body:vec![255]}),diagnostics:vec![]},Span::START).unwrap_err();
  assert_eq!(e.message(),"HTTP 响应 body 不是合法 UTF-8");assert!(Interp::attempt_recoverable(&e));
 }
 #[test] fn http_diagnostics_survive_attempt_and_global_terminal_reports(){on_eval_stack(||{
  let mut i=Interp::new();i.host_diagnostics.push("process".into());
  let e=i.consume_http_report(HttpReport{result:Err(HttpFailure::Io{operation:"关闭".into(),message:"injected".into()}),diagnostics:vec!["http cleanup".into()]},Span::START).unwrap_err();
  assert!(Interp::attempt_recoverable(&e));assert_eq!(e.span(),Some(Span::START));
  let p=crate::parser::parse(&crate::lexer::lex("attempt(()=>1/0)",0).unwrap()).unwrap();
  let scope=Scope{env:i.globals.clone(),captured:Rc::new(vec![]),self_val:None};assert!(i.exec_stmts(&p.stmts,&scope).is_ok());
  assert_eq!(i.host_diagnostics,vec!["process"]);assert_eq!(i.http_diagnostics,vec!["http cleanup"]);
  for f in [HttpFailure::Cancelled,HttpFailure::Deadline]{let e=i.consume_http_report(HttpReport{result:Err(f),diagnostics:vec![]},Span::START).unwrap_err();assert_eq!(e.class_name(),"ExecutionLimitError");assert!(!Interp::attempt_recoverable(&e));}
  assert!(eval_module_traced_on_thread(&p).http_diagnostics().is_empty());
 });}
}
