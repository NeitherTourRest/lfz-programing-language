//! T16-B：只构建静态模块图；调用者在大栈内构建和释放AST。
use crate::ast::{Program, StmtKind, ImportDecl};
use crate::error::{self, LzError, SyntaxMsg};
use crate::loader::{self, Loaded};
use crate::source::{SourceId, ModuleId, SourceMap, SourceRecord, LocatedFailure};
use crate::span::Span;
use std::{collections::{HashMap, HashSet}, path::{Path,PathBuf}, rc::Rc, io::Read};
pub const MAX_MODULES: usize=1024;
pub const MAX_MODULE_DEPTH: usize=128;
pub const MAX_SOURCE_BYTES: usize=2*1024*1024;
pub const MAX_GRAPH_BYTES: usize=16*1024*1024;
#[derive(Debug)]
pub struct ParsedEntry { pub program:Program,pub loaded:Loaded,pub display_path:String,pub identity:Option<PathBuf>,pub base_dir:PathBuf,pub raw_bytes:usize }
#[derive(Clone,Debug)]
pub struct ResolvedImport { pub alias:String,pub alias_span:Span,pub path_span:Span,pub target:ModuleId,pub span:Span }
#[derive(Clone,Debug)]
pub struct ExportBinding { pub name:String,pub span:Span }
#[derive(Debug)]
pub struct ModuleUnit { pub id:ModuleId,pub source_id:SourceId,pub program:Program,pub imports:Vec<ResolvedImport>,pub exports:Vec<ExportBinding> }
#[derive(Debug)]
pub struct ModuleGraph { pub sources:Rc<SourceMap>,pub units:Vec<ModuleUnit>,pub entry:ModuleId,pub initialization_order:Vec<ModuleId> }
#[derive(Debug)]
pub struct GraphFailure { pub sources:Rc<SourceMap>,pub failure:LocatedFailure }

pub fn module_declaration_span(program:&Program)->Option<Span> {
    program.stmts.iter().find(|s|matches!(s.node,StmtKind::ImportDecl(_)|StmtKind::ExportDecl(_))).map(|s|s.span)
}
/// 同时拒绝跨宿主盘符/UNC语法；不执行文件IO或canonicalize。
pub fn valid_module_path(path:&str)->bool {
    !path.is_empty() && !path.contains('\0') && !Path::new(path).is_absolute()
        && !path.starts_with(['/', '\\']) && !path.contains(':') && loader::is_lfz(path)
}
fn limit(kind:&str,n:usize,span:Span)->Box<LzError>{error::syntax(SyntaxMsg::ModuleLimit{kind:kind.into(),limit:n},span)}
fn admit(raw:usize,total:usize,span:Span)->Result<usize,Box<LzError>> {
    if raw>MAX_SOURCE_BYTES {return Err(limit("单文件字节",MAX_SOURCE_BYTES,span));}
    total.checked_add(raw).filter(|n|*n<=MAX_GRAPH_BYTES).ok_or_else(||limit("源码总字节",MAX_GRAPH_BYTES,span))
}
fn local(program:&Program)->Result<Vec<ExportBinding>,Box<LzError>> {
    let mut all=HashSet::new(); let mut imported=HashSet::new();
    for s in &program.stmts { match &s.node {
        StmtKind::Decl{name,..}=>{all.insert(name.clone());},
        StmtKind::FnDecl(f)=>{all.insert(f.name.clone());},
        StmtKind::StructDecl(f)=>{all.insert(f.name.clone());},
        StmtKind::ImportDecl(i)=>{all.insert(i.alias.clone());imported.insert(i.alias.clone());},_=>{}
    }}
    let mut seen=HashSet::new();let mut exports=HashSet::new();let mut out=vec![];
    for s in &program.stmts {
        let binding=match &s.node {
            StmtKind::ImportDecl(i)=>Some((&i.alias,i.alias_span)),
            StmtKind::Decl{name,..}=>Some((name,s.span)),
            StmtKind::FnDecl(f)=>Some((&f.name,s.span)),
            StmtKind::StructDecl(f)=>Some((&f.name,s.span)),_=>None
        };
        if let Some((name,span))=binding {if imported.contains(name)&&!seen.insert(name.clone()){return Err(error::syntax(SyntaxMsg::ImportConflict{name:name.clone()},span));}}
        if let StmtKind::ExportDecl(e)=&s.node {for n in &e.names {
            if !exports.insert(n.name.clone()){return Err(error::syntax(SyntaxMsg::DuplicateExport{name:n.name.clone()},n.span));}
            if !all.contains(&n.name){return Err(error::syntax(SyntaxMsg::MissingExport{name:n.name.clone()},n.span));}
            out.push(ExportBinding{name:n.name.clone(),span:n.span});
        }}
    }
    Ok(out)
}
struct Frame { id:ModuleId,next:usize }
struct Builder { sources:SourceMap,units:Vec<ModuleUnit>,pending:Vec<Vec<(Span,ImportDecl)>>,bases:Vec<PathBuf>,ready:Vec<bool>,cache:HashMap<PathBuf,ModuleId>,order:Vec<ModuleId>,total:usize }
impl Builder {
    fn source(&mut self,record:SourceRecord)->SourceId { self.sources.insert(record).expect("图节点上限保证SourceId可表示") }
    fn failure(self,id:SourceId,error:Box<LzError>)->GraphFailure {GraphFailure{sources:Rc::new(self.sources),failure:LocatedFailure{source_id:id,error}}}
    fn push(&mut self,program:Program,source_id:SourceId,base:PathBuf,exports:Vec<ExportBinding>)->ModuleId {
        let id=ModuleId(u32::try_from(self.units.len()).expect("图节点上限"));
        let imports=program.stmts.iter().filter_map(|s|if let StmtKind::ImportDecl(i)=&s.node {Some((s.span,i.clone()))}else{None}).collect();
        self.pending.push(imports);self.bases.push(base);self.ready.push(false);
        self.units.push(ModuleUnit{id,source_id,program,imports:vec![],exports});id
    }
    fn io(&mut self,target:String,importer:SourceId,span:Span)->(SourceId,Box<LzError>) {
        let from=&self.sources.get(importer).expect("内部来源").display_path;
        let msg=format!("无法读取模块：{target}（导入于 {from}:{}:{}）",span.line,span.col);
        let sid=self.source(SourceRecord{identity:None,display_path:target,loaded:None});(sid,error::io(msg,None))
    }
}
pub fn build_graph(entry:ParsedEntry)->Result<ModuleGraph,GraphFailure> {
    let ParsedEntry{program,loaded,display_path,identity,base_dir,raw_bytes}=entry;
    let mut b=Builder{sources:SourceMap::new(),units:vec![],pending:vec![],bases:vec![],ready:vec![],cache:HashMap::new(),order:vec![],total:0};
    let sid=b.source(SourceRecord{identity:identity.clone(),display_path,loaded:Some(loaded)});
    b.total=match admit(raw_bytes,0,Span::START){Ok(n)=>n,Err(e)=>return Err(b.failure(sid,e))};
    let exports=match local(&program){Ok(x)=>x,Err(e)=>return Err(b.failure(sid,e))};
    let root=b.push(program,sid,base_dir,exports);if let Some(p)=identity{b.cache.insert(p,root);}
    let mut stack=vec![Frame{id:root,next:0}];
    while let Some(frame)=stack.last_mut() {
        let idx=frame.id.0 as usize;
        if frame.next==b.pending[idx].len(){b.ready[idx]=true;b.order.push(frame.id);stack.pop();continue;}
        let (span,import)=b.pending[idx][frame.next].clone();frame.next+=1;
        let source=b.units[idx].source_id;
        if !valid_module_path(&import.path){return Err(b.failure(source,error::syntax(SyntaxMsg::ModulePath,import.path_span)));}
        let lexical=b.bases[idx].join(&import.path);
        let Some(lexical_text)=lexical.to_str() else {return Err(b.failure(source,error::syntax(SyntaxMsg::ModulePath,import.path_span)));};
        let identity=match std::fs::canonicalize(&lexical){Ok(p)=>p,Err(_)=>{let (id,e)=b.io(lexical_text.into(),source,span);return Err(b.failure(id,e));}};
        let Some(path)=identity.to_str().map(str::to_owned) else{return Err(b.failure(source,error::syntax(SyntaxMsg::ModulePath,import.path_span)));};
        let target=if let Some(&target)=b.cache.get(&identity) {
            if !b.ready[target.0 as usize] {
                let pos=stack.iter().position(|f|f.id==target).expect("visiting在DFS栈中");
                let mut chain:Vec<String>=stack[pos..].iter().map(|f|b.sources.get(b.units[f.id.0 as usize].source_id).unwrap().display_path.clone()).collect();
                chain.push(b.sources.get(b.units[target.0 as usize].source_id).unwrap().display_path.clone());
                return Err(b.failure(source,error::syntax(SyntaxMsg::ModuleCycle{chain:chain.join(" → ")},span)));
            } target
        }else{
            if b.units.len()>=MAX_MODULES{return Err(b.failure(source,limit("节点数",MAX_MODULES,span)));}
            if stack.len()>=MAX_MODULE_DEPTH{return Err(b.failure(source,limit("依赖深度",MAX_MODULE_DEPTH,span)));}
            let max=MAX_SOURCE_BYTES.min(MAX_GRAPH_BYTES-b.total);
            let mut bytes=Vec::new();
            let read=std::fs::File::open(&identity).and_then(|f|f.take((max+1) as u64).read_to_end(&mut bytes));
            if read.is_err(){let(id,e)=b.io(path,source,span);return Err(b.failure(id,e));}
            let total=match admit(bytes.len(),b.total,span){Ok(n)=>n,Err(e)=>return Err(b.failure(source,e))};
            let loaded=match loader::load_bytes(&path,bytes){Ok(x)=>x,Err(e)=>{let id=b.source(SourceRecord{identity:Some(identity),display_path:path,loaded:None});return Err(b.failure(id,e));}};
            let sid=b.source(SourceRecord{identity:Some(identity.clone()),display_path:path,loaded:Some(loaded.clone())});
            let program=match crate::lexer::lex(&loaded.text,loaded.line_base).and_then(|t|crate::parser::parse(&t)){Ok(p)=>p,Err(e)=>return Err(b.failure(sid,e))};
            let exports=match local(&program){Ok(x)=>x,Err(e)=>return Err(b.failure(sid,e))};
            b.total=total;let base=identity.parent().unwrap_or(Path::new("")).to_path_buf();
            let target=b.push(program,sid,base,exports);b.cache.insert(identity,target);stack.push(Frame{id:target,next:0});target
        };
        b.units[idx].imports.push(ResolvedImport{alias:import.alias,alias_span:import.alias_span,path_span:import.path_span,target,span});
    }
    Ok(ModuleGraph{sources:Rc::new(b.sources),units:b.units,entry:root,initialization_order:b.order})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64,Ordering};
    static SERIAL:AtomicU64=AtomicU64::new(0);
    fn parse(text:&str)->crate::error::R<Program>{crate::lexer::lex(text,0).and_then(|t|crate::parser::parse(&t))}
    fn work()->PathBuf {let p=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Temp/T16-B-core/tests").join(format!("{}-{}",std::process::id(),SERIAL.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir_all(&p).unwrap();p}
    fn write(p:&Path,text:&str){std::fs::write(p,format!("#42\n{text}")).unwrap();}
    fn graph(p:&Path)->Result<ModuleGraph,GraphFailure>{crate::evaluator::on_eval_stack(||{let path=std::fs::canonicalize(p).unwrap();let c=loader::load_file_counted(path.to_str().unwrap()).unwrap();let program=crate::lexer::lex(&c.loaded.text,c.loaded.line_base).and_then(|t|crate::parser::parse(&t)).unwrap();build_graph(ParsedEntry{program,loaded:c.loaded,display_path:path.to_str().unwrap().into(),identity:Some(path.clone()),base_dir:path.parent().unwrap().into(),raw_bytes:c.raw_bytes})})}
    #[test]
    fn t16b_parser_declarations_and_export_identifiers(){
        let p=parse("import a from \"./a.LFZ\"\nexport a\n").unwrap();assert_eq!(p.stmts.len(),2);assert!(matches!(p.stmts[0].node,StmtKind::ImportDecl(_)));
        for old in ["let export=1\nexport = 2","export(1)","export.foo","export[\"x\"]","export + 1","export","let x=export","fn export(export)=>export","let x={export:1}"]{assert!(parse(old).is_ok(),"{old}");}
    }
    #[test]
    fn t16b_parser_rejects_nested_order_and_dynamic_paths(){
        for code in ["if true { import a from \"a.lfz\" }","while true { export x }","fn f(){import a from \"a.lfz\"}","let f=()=>{export x}"]{assert_eq!(parse(code).unwrap_err().message(),"模块声明只能出现在文件顶层");}
        assert_eq!(parse("let x=1\nimport a from \"a.lfz\"").unwrap_err().message(),"import 必须位于其它顶层语句之前");
        for path in ["", "a.txt", "/a.lfz", "C:a.lfz", "\\\\host\\a.lfz"]{assert!(!valid_module_path(path));}
        for code in ["import a from \"${1}.lfz\"","import a from \"\"","import a from \"a\\0.lfz\""]{assert!(parse(code).is_err());}
        assert!(valid_module_path("../lib/a.lfz"));
        assert!(parse("export a,").is_err());
    }
    #[test]
    fn t16b_counted_loader_preserves_legacy_and_raw_bytes(){
        let text="\u{feff}#42\r\nlet x=1\r\n";let d=work();let p=d.join("a.lfz");std::fs::write(&p,text).unwrap();let c=loader::load_file_counted(p.to_str().unwrap()).unwrap();assert_eq!(c.raw_bytes,text.len());assert_eq!(c.loaded,loader::load_file(p.to_str().unwrap()).unwrap());assert_eq!(c.loaded.line_base,1);
        assert_eq!(loader::load_source_counted(text).unwrap().loaded,loader::load_source(text).unwrap());
        let e=loader::load_bytes("a.lfz",vec![255]).unwrap_err();assert_eq!(e.class_name(),"SyntaxError");
    }
    #[test]
    fn t16b_diamond_order_shares_snapshot_without_initialization(){
        let d=work();write(&d.join("main.lfz"),"import a from \"a.lfz\"\nimport b from \"b.lfz\"\nexport a,b\nwhile true {}\n");write(&d.join("a.lfz"),"import c from \"c.lfz\"\nexport c\n");write(&d.join("b.lfz"),"import c from \"./c.lfz\"\nexport c\n");write(&d.join("c.lfz"),"let value=3\nexport value\ninput()\n");
        let g=graph(&d.join("main.lfz")).unwrap();assert_eq!(g.units.len(),4);assert_eq!(g.initialization_order,vec![ModuleId(2),ModuleId(1),ModuleId(3),ModuleId(0)]);assert_eq!(g.units[1].imports[0].target,g.units[3].imports[0].target);
        std::fs::write(d.join("c.lfz"),"changed").unwrap();assert!(g.sources.get(g.units[2].source_id).unwrap().loaded.as_ref().unwrap().text.contains("value=3"));
    }
    #[test]
    fn t16b_static_conflicts_forward_exports_and_first_error(){
        let d=work();let p=d.join("main.lfz");
        for (text,msg) in [("export x\nlet x=1\nexport x","重复导出：x"),("export missing","导出名没有顶层声明：missing"),("import x from \"absent.lfz\"\nlet x=1","模块导入名冲突：x"),("import x from \"absent.lfz\"\nimport x from \"absent2.lfz\"","模块导入名冲突：x")]{write(&p,text);assert_eq!(graph(&p).unwrap_err().failure.error.message(),msg);}
        write(&p,"export x\nlet x=1");assert_eq!(graph(&p).unwrap().units[0].exports[0].name,"x");
        write(&p,"let x=1\nlet x=2\nexport x");assert!(graph(&p).is_ok());
    }
    #[test]
    fn t16b_cycle_and_missing_module_have_actual_sources(){
        let d=work();let a=d.join("a.lfz");write(&a,"import b from \"b.lfz\"");write(&d.join("b.lfz"),"import a from \"a.lfz\"");let e=graph(&a).unwrap_err();assert!(matches!(*e.failure.error,LzError::Syntax{msg:SyntaxMsg::ModuleCycle{..},..}));assert!(e.failure.error.message().contains(" → "));
        write(&a,"import b from \"missing.lfz\"");let e=graph(&a).unwrap_err();assert_eq!(e.failure.error.class_name(),"IOError");let source=e.sources.get(e.failure.source_id).unwrap();assert!(source.loaded.is_none());assert!(source.display_path.ends_with("missing.lfz"));assert!(e.failure.error.message().contains("（导入于 "));
    }
    #[test]
    fn t16b_dependency_parse_error_points_to_dependency(){let d=work();let a=d.join("a.lfz");write(&a,"import b from \"b.lfz\"");write(&d.join("b.lfz"),"let x=\n");let e=graph(&a).unwrap_err();assert_eq!(e.failure.error.class_name(),"SyntaxError");let r=e.sources.get(e.failure.source_id).unwrap();assert!(r.display_path.ends_with("b.lfz"));assert!(r.loaded.is_some());assert_eq!(e.failure.error.span().unwrap().line,2);}
    #[test]
    fn t16b_depth_boundaries_are_iterative(){for n in [127,128,129]{let d=work();for i in 0..n {let body=if i+1<n {format!("import x from \"{}.lfz\"",i+1)}else{"export v\nlet v=1".into()};write(&d.join(format!("{i}.lfz")),&body);}let g=graph(&d.join("0.lfz"));if n<=128 {assert_eq!(g.unwrap().units.len(),n);}else{assert_eq!(g.unwrap_err().failure.error.message(),"模块图超限：依赖深度（上限 128）");}}}
    #[test]
    fn t16b_node_boundaries_are_unique_ids(){for n in [1023,1024,1025]{let d=work();let mut root=String::new();for i in 1..n {root.push_str(&format!("import n{i} from \"{i}.lfz\"\n"));write(&d.join(format!("{i}.lfz")),"let x=1\nexport x");}write(&d.join("0.lfz"),&root);let g=graph(&d.join("0.lfz"));if n<=1024{assert_eq!(g.unwrap().units.len(),n);}else{assert_eq!(g.unwrap_err().failure.error.message(),"模块图超限：节点数（上限 1024）");}}}
    #[test]
    fn t16b_byte_admission_exact_limits_and_precedence(){assert_eq!(admit(MAX_SOURCE_BYTES,MAX_GRAPH_BYTES-MAX_SOURCE_BYTES,Span::START).unwrap(),MAX_GRAPH_BYTES);assert_eq!(admit(MAX_SOURCE_BYTES+1,MAX_GRAPH_BYTES,Span::START).unwrap_err().message(),"模块图超限：单文件字节（上限 2097152）");assert_eq!(admit(1,MAX_GRAPH_BYTES,Span::START).unwrap_err().message(),"模块图超限：源码总字节（上限 16777216）");assert!(admit(0,MAX_GRAPH_BYTES,Span::START).is_ok());}
    #[test]
    fn t16b_actual_bounded_single_file_bytes() {
        let d=work();let root=d.join("root.lfz");let dep=d.join("dep.lfz");write(&root,"import d from \"dep.lfz\"");
        let mut bytes=b"#42\n//".to_vec();bytes.resize(MAX_SOURCE_BYTES,b' ');std::fs::write(&dep,&bytes).unwrap();
        assert_eq!(graph(&root).unwrap().units.len(),2);
        bytes.push(255);std::fs::write(&dep,&bytes).unwrap();
        assert_eq!(graph(&root).unwrap_err().failure.error.message(),"模块图超限：单文件字节（上限 2097152）");
    }
    #[test]
    fn t16b_actual_bounded_total_bytes() {
        let d=work();let root=d.join("root.lfz");let mut imports=String::new();
        for i in 0..8 {imports.push_str(&format!("import m{i} from \"{i}.lfz\"\n"));}
        write(&root,&imports);let root_bytes=std::fs::read(&root).unwrap().len();
        for i in 0..8 {let size=if i==7 {MAX_SOURCE_BYTES-root_bytes}else{MAX_SOURCE_BYTES};let mut bytes=b"#42\n//".to_vec();bytes.resize(size,b' ');std::fs::write(d.join(format!("{i}.lfz")),bytes).unwrap();}
        assert_eq!(graph(&root).unwrap().units.len(),9);
        let p=d.join("7.lfz");use std::io::Write;std::fs::OpenOptions::new().append(true).open(p).unwrap().write_all(&[255]).unwrap();
        assert_eq!(graph(&root).unwrap_err().failure.error.message(),"模块图超限：源码总字节（上限 16777216）");
    }
}
