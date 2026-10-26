from pathlib import Path
import shutil
root=Path('Temp/runtime-t12/perf-lab/scope-elision');(root/'src').mkdir(parents=True,exist_ok=True)
for p in Path('src').glob('*.rs'):shutil.copyfile(p,root/p)
for p in ['Cargo.toml','Cargo.lock']:shutil.copyfile(p,root/p)
p=root/'src/evaluator.rs';s=p.read_text(encoding='utf8')
marker='''    fn exec_while(&mut self, cond: &Expr, body: &Block, scope: &Scope) -> R<Flow> {
        loop {''';assert marker in s;s=s.replace(marker,'''    fn exec_while(&mut self, cond: &Expr, body: &Block, scope: &Scope) -> R<Flow> {
        // 仅直接声明在此块创建绑定；嵌套块自行管理其作用域。
        let binds_locals = body.stmts.iter().any(|s| matches!(s.node,
            StmtKind::Decl { .. } | StmtKind::FnDecl(_) | StmtKind::StructDecl(_)));
        loop {''')
marker='''            match self.exec_block(body, scope)? {
                Flow::Value(_) => {}''';assert s.count(marker)==1;s=s.replace(marker,'''            let flow = if binds_locals { self.exec_block(body, scope)? }
                else { self.exec_stmts(&body.stmts, scope)? };
            match flow {
                Flow::Value(_) => {}''');p.write_text(s,encoding='utf8')
# 保留每个可执行产物、asm和所有源码/证据，清除已完成且不再运行的构建中间件。
base=Path('Temp/runtime-t12/perf-lab').resolve();freed=0
for variant in base.iterdir():
 if not variant.is_dir() or variant.name=='scope-elision':continue
 target=variant/'target'
 if not target.is_dir():continue
 release=target/'release'
 for asm in (release/'deps').glob('*.s') if (release/'deps').exists() else []:shutil.copyfile(asm,variant/asm.name)
 for q in [target/'debug',release/'deps',release/'.fingerprint',release/'incremental',release/'build']:
  if q.is_dir():
   assert q.resolve().is_relative_to(base)
   freed+=sum(f.stat().st_size for f in q.rglob('*') if f.is_file())
   shutil.rmtree(q)
print('Removed completed intermediates bytes',freed)
