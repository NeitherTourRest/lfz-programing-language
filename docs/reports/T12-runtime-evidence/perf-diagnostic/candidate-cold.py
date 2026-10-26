from pathlib import Path
import shutil
root=Path('Temp/runtime-t12/perf-lab/cold-ident');(root/'src').mkdir(parents=True,exist_ok=True)
for p in Path('src').glob('*.rs'):shutil.copyfile(p,root/p)
for p in ['Cargo.toml','Cargo.lock']:shutil.copyfile(p,root/p)
p=root/'src/evaluator.rs';s=p.read_text(encoding='utf8');old='''                None if name == "argv" => Ok(Flow::Value(self.argv_function.clone())),
                None => Err(name_error(name.clone(), span)),''';assert old in s;s=s.replace(old,'''                None => self.eval_unbound_ident(name, span),''')
marker='''    /// [`Interp::eval_expr`] 的求值体（不含帧 `span` 覆盖）。''';assert marker in s;s=s.replace(marker,'''    // 内置函数值/未定义名字均是普通变量读取的罕见分支，隔离出热表达式派发。
    #[cold]
    #[inline(never)]
    fn eval_unbound_ident(&self, name: &str, span: Span) -> R<Flow> {
        if name == "argv" { Ok(Flow::Value(self.argv_function.clone())) }
        else { Err(name_error(name.to_string(), span)) }
    }

'''+marker);p.write_text(s,encoding='utf8')
