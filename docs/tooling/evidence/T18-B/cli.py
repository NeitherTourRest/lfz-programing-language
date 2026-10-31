from pathlib import Path
p=Path('src/http_control/windows.rs');s=p.read_text(encoding='utf-8-sig');s=s.replace('s.event=Some((status,value));}}\n c.active.fetch_sub(1,Ordering::AcqRel);c.wake.notify_all();','s.event=Some((status,value));}c.active.fetch_sub(1,Ordering::AcqRel);c.wake.notify_all();}')
p.write_text(s,encoding='utf-8')
p=Path('src/cli.rs');s=p.read_text(encoding='utf-8-sig');s=s.replace('allow_process:bool, first_capability:Option<bool>','allow_process:bool, allow_http:bool, first_capability:Option<u8>')
s=s.replace('||self.allow_process}','||self.allow_process||self.allow_http}')
s=s.replace('if self.first_capability==Some(true){','if self.first_capability==Some(2){"HTTP 能力选项只适用于运行命令"}else if self.first_capability==Some(1){')
s=s.replace('subprocess:self.allow_process}','subprocess:self.allow_process,http:self.allow_http}')
s=s.replace('get_or_insert(true)','get_or_insert(1)').replace('get_or_insert(false)','get_or_insert(0)')
s=s.replace('        else if a=="--allow-process"', '''        else if a=="--allow-http" {if limits.allow_http{return Err("选项 '--allow-http' 不能重复".into());}limits.allow_http=true;limits.first_capability.get_or_insert(2);i+=1;}
        else if a.starts_with("--allow-http="){return Err("选项 '--allow-http' 不接受值".into());}
        else if a=="--allow-process"''')
s=s.replace('  --allow-process      ', '  --allow-http         仅本次执行启用直连HTTP/HTTPS，返回响应值；能力不是沙箱\n  --allow-process      ')
s=s.replace('    WithDiagnostics { inner:Box<CaseEval>, diagnostics:Vec<String> },','    WithDiagnostics { inner:Box<CaseEval>, diagnostics:Vec<String> },\n    HttpDiagnostics { inner:Box<CaseEval>, diagnostics:Vec<String> },')
s=s.replace('Self::WithDiagnostics{inner,..}', 'Self::WithDiagnostics{inner,..}|Self::HttpDiagnostics{inner,..}')
s=s.replace('CaseEval::WithDiagnostics{inner,..}', 'CaseEval::WithDiagnostics{inner,..}|CaseEval::HttpDiagnostics{inner,..}')
s=s.replace('Self::Located{inner,..}=>inner.cleanup_diagnostics()', 'Self::Located{inner,..}|Self::HttpDiagnostics{inner,..}=>inner.cleanup_diagnostics()')
needle='    pub(crate) fn passed(&self)'
pos=s.index(needle)
s=s[:pos]+'''    pub(crate) fn http_diagnostics(&self)->&[String]{match self{Self::HttpDiagnostics{diagnostics,..}=>diagnostics,Self::Located{inner,..}|Self::WithDiagnostics{inner,..}=>inner.http_diagnostics(),_=>&[]}}
    pub(crate) fn has_diagnostics(&self)->bool{!self.cleanup_diagnostics().is_empty()||!self.http_diagnostics().is_empty()}
    pub(crate) fn cleanup_title(&self)->&'static str{if self.cleanup_diagnostics().is_empty(){"lfz: HTTP 资源回收失败"}else{"lfz: 子进程资源回收失败"}}
    pub(crate) fn write_diagnostics(&self,err:&mut dyn Write){write_cleanup_diagnostics(err,self.cleanup_diagnostics());for line in self.http_diagnostics(){let _=writeln!(err,"lfz: HTTP 回收诊断：{line}");}}
'''+s[pos:]
s=s.replace('let diagnostics=traced.host_diagnostics().to_vec();','let diagnostics=traced.host_diagnostics().to_vec();let http=traced.http_diagnostics().to_vec();')
s=s.replace('with_diagnostics(inner,diagnostics)\n','with_http_diagnostics(with_diagnostics(inner,diagnostics),http)\n')
s=s.replace('fn with_diagnostics(inner:', 'fn with_http_diagnostics(inner:CaseEval,diagnostics:Vec<String>)->CaseEval{if diagnostics.is_empty(){inner}else{CaseEval::HttpDiagnostics{inner:Box::new(inner),diagnostics}}}\nfn with_diagnostics(inner:')
s=s.replace('if eval.passed() && !diagnostics.is_empty(){let _=writeln!(err,"lfz: 子进程资源回收失败");write_cleanup_diagnostics(err,diagnostics);return EXIT_ERROR;}','if eval.passed() && eval.has_diagnostics(){let _=writeln!(err,"{}",eval.cleanup_title());eval.write_diagnostics(err);return EXIT_ERROR;}')
s=s.replace('    let diagnostics=eval.cleanup_diagnostics();\n','')
s=s.replace('    write_cleanup_diagnostics(err,diagnostics);','    eval.write_diagnostics(err);')
p.write_text(s,encoding='utf-8')
