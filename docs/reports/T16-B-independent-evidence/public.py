"""Public T16-B CLI acceptance. No product imports; project-local fixtures only."""
import argparse,datetime,hashlib,json,os,subprocess,sys
from pathlib import Path
ROOT=Path.cwd()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--expected-sha');ap.add_argument('--output',type=Path,required=True);ap.add_argument('--smoke-only',action='store_true');a=ap.parse_args()
 exe=a.binary.resolve();out=a.output.resolve();assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists()
 digest=sha(exe);assert not a.expected_sha or digest==a.expected_sha.lower()
 w=ROOT/'Temp/T16-B-verifier/public';w.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
 env=os.environ.copy();env.update(TEMP=str(w),TMP=str(w),PYTHONDONTWRITEBYTECODE='1')
 rows=[];q=lambda s:json.dumps(str(s),ensure_ascii=False).replace('\\u0000','\x00')
 def file(name,body,raw=None):
  p=w/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw if raw is not None else ('#42\n'+body).encode());return p
 def run(name,args,*,data=b'',code=0,msg=None,cls=None,where=None,line=None,col=None,normal=False,stdout=None,watch=None):
  assert sha(exe)==digest
  before={str(p):p.read_bytes().hex() if p.exists() else None for p in watch or []}
  try:
   r=subprocess.run([str(exe),*args],input=data,capture_output=True,cwd=w,env=env,timeout=8)
   text=r.stdout.decode('utf8');err=r.stderr.decode('utf8');checks={'exit':r.returncode==code}
   obj=None
   if normal:
    checks['streams']=text==(stdout or '') and (not err if code==0 else bool(err))
   else:
    obj=json.loads(text);checks['stderr_empty']=not err
    keys=['ok','mode','executed']+(['error','message','file','line','col','traceback'] if code else [])
    checks['keys']=list(obj)==keys;checks['mode']=obj.get('mode')=='check' and obj.get('executed') is False;checks['ok']=obj.get('ok')==(code==0)
    checks['one_line']=text.count('\n')==1
    if msg is not None:checks['message']=obj.get('message')==msg
    if cls:checks['class']=obj.get('error')==cls
    if where:checks['file']=obj.get('file') in ([str(where),chr(92)*2+'?'+chr(92)+str(where)] if os.name=='nt' else [str(where)])
    if line is not None:checks['line']=obj.get('line')==line
    if col is not None:checks['col']=obj.get('col')==col
   after={str(p):p.read_bytes().hex() if p.exists() else None for p in watch or []};checks['sentinels']=before==after
   rows.append({'name':name,'args':args,'stdin_hex':data.hex(),'exit':r.returncode,'stdout':text,'stderr':err,'checks':checks,'passed':all(checks.values()),'before':before,'after':after})
  except (subprocess.TimeoutExpired,ValueError,UnicodeError) as e:rows.append({'name':name,'args':args,'passed':False,'exception':repr(e)})
 def ck(name,p,**kw):run(name,['check','--json',str(p)],**kw)
 dep=file('lib/dep.lfz','let n=2\nexport n\n');root=file('main.lfz','import d from "./lib/dep.lfz"\nlet n=d.n\nexport n\n')
 ck('module_smoke',root)
 if not a.smoke_only:
  other=file('lib/other.lfz','import d from "./dep.lfz"\nlet z=d.n\nexport z\n')
  root=file('main.lfz','import a from "./lib/dep.lfz"\nimport b from "./lib/other.lfz"\nimport same from "./lib/../lib/dep.lfz"\nlet n=1\nexport n\n')
  ck('diamond_alias',root);run('human_success',['check',str(root)],normal=True)
  code='import d from "./lib/dep.lfz"\nexport d\n'
  run('eval_base',['check','--json','-e',code]);run('stdin_base',['check','--json','-'],data=code.encode())
  cases=[('if','if true { export x }','模块声明只能出现在文件顶层'),('while','while false { import d from "./lib/dep.lfz" }','模块声明只能出现在文件顶层'),('fn','fn f() { export x }','模块声明只能出现在文件顶层'),('lambda','let f=()=>{ import d from "./lib/dep.lfz" }','模块声明只能出现在文件顶层'),('order','let x=1\nimport d from "./lib/dep.lfz"','import 必须位于其它顶层语句之前')]
  for name,src,msg in cases:ck('syntax_'+name,file('bad.lfz',src),code=2,cls='SyntaxError',msg=msg)
  for name,path in [('empty',''),('extension','./dep.txt'),('absolute',dep.as_posix()),('nul','a\0.lfz')]:
   ck('path_'+name,file('bad.lfz','import d from '+q(path)+'\n'),code=2,cls='SyntaxError',msg='模块路径必须是非空的相对 .lfz 字符串（不含插值或 NUL）')
  ck('interpolated',file('bad.lfz','import d from "./${1}.lfz"\n'),code=2,cls='SyntaxError',msg='模块路径必须是非空的相对 .lfz 字符串（不含插值或 NUL）')
  for name,src,msg,ln,cl in [('import_conflict','import a from "./lib/dep.lfz"\nimport a from "./lib/dep.lfz"','模块导入名冲突：a',3,8),('binding_conflict','import a from "./lib/dep.lfz"\nlet a=1','模块导入名冲突：a',3,1),('duplicate_export','let x=1\nexport x,x','重复导出：x',3,10),('missing_export','export missing','导出名没有顶层声明：missing',2,8),('builtin_export','export print','导出名没有顶层声明：print',2,8)]:
   ck(name,file('bad.lfz',src),code=2,cls='SyntaxError',msg=msg,line=ln,col=cl)
  ck('forward_export',file('forward.lfz','export x\nlet x=3\n'))
  run('export_identifier_run',['-e','var export=1\nexport = 2\nassert(export + 1 == 3)\nlet s={export:4}\nassert(s.export==4)'],normal=True)
  ck('export_identifier_check',file('legacy.lfz','var export=1\nexport=2\nexport + 1\n'))
  for name,raw,cls in [('parse',b'#42\nlet x =\n','SyntaxError'),('utf8',b'#42\n\xff','SyntaxError'),('preamble',b'let x=1\n','CosmosAnswerError')]:
   p=file('lib/bad.lfz','',raw=raw);entry=file('err-entry.lfz','import b from "./lib/bad.lfz"\n')
   ck('dependency_'+name,entry,code=2,cls=cls,where=p.resolve())
  missing=(w/'lib/absent.lfz').resolve();entry=file('missing-entry.lfz','import b from "./lib/absent.lfz"\n')
  ck('missing_dependency',entry,code=2,cls='IOError',where=missing)
  aa=file('cycle/a.lfz','import b from "./b.lfz"\n');bb=file('cycle/b.lfz','import a from "./a.lfz"\n');ck('cycle',aa,code=2,cls='SyntaxError',where=bb.resolve(),line=2,col=1)
  old=w/'sentinel.bin';old.write_bytes(b'ORIGINAL\x00\xff');new=w/'never.bin'
  if new.exists():new.unlink()
  for name,side in [('output','print("NO")\neprint("NO")\n;;\ninput("NO")'),('io','writeFile('+q(old.as_posix())+',"NO")\nappendFile('+q(old.as_posix())+',"NO")\nwriteFile('+q(new.as_posix())+',"NO")'),('loop','while true {}')]:
   file('lib/side.lfz',side+'\nlet x=1\nexport x\n');entry=file('side-entry.lfz','import s from "./lib/side.lfz"\n')
   ck('no_eval_'+name,entry,watch=[old,new])
  for name,src in [('late_export','writeFile('+q(old.as_posix())+',"NO")\nprint("NO")\nlet x=1\nexport x'),('early_import','import d from "./lib/dep.lfz"\nwriteFile('+q(old.as_posix())+',"NO")')]:
   p=file('run-guard.lfz',src);run('run_guard_'+name,['run','--json',str(p)],code=2,normal=True,watch=[old,new])
   # Execution JSON retains its original protocol, unlike check. Verify error explicitly.
   row=rows[-1]
   if row.get('stdout'):
    result=json.loads(row['stdout']);row['checks']['execution_error']=result.get('ok') is False and result.get('error')=='SyntaxError' and result.get('message')=='模块执行尚未启用' and 'mode' not in result;row['checks']['streams']=not row['stderr'];row['passed']=all(row['checks'].values())
  # Large comments preserve low AST size and do not alter legacy acceptance.
  limit=2*1024*1024
  p=file('large-legacy.lfz','',raw=b'#42\n//'+b'x'*limit+b'\n');ck('legacy_large_root',p);p.unlink()
  prefix=b'#42\nexport x\nlet x=1\n//';p=file('large-module.lfz','',raw=prefix+b'x'*(limit-len(prefix)));ck('module_exact_bytes',p)
  p.write_bytes(p.read_bytes()+b'x');ck('module_bytes_plus_one',p,code=2,cls='SyntaxError',msg='模块图超限：单文件字节（上限 2097152）');p.unlink()
  for i in range(129):file('chain/'+str(i)+'.lfz',('import n from "./'+str(i+1)+'.lfz"\n' if i<127 else 'let x=1\nexport x\n'))
  ck('depth128',w/'chain/0.lfz');file('chain/127.lfz','import n from "./128.lfz"\n');ck('depth129',w/'chain/0.lfz',code=2,cls='SyntaxError',msg='模块图超限：依赖深度（上限 128）')
 assert sha(exe)==digest
 result={'date':datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).date().isoformat(),'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':str(exe.relative_to(ROOT)),'binary_sha256':digest,'bytes':exe.stat().st_size,'schema':'LFZ-T16-B-CLI-1','checks_total':len(rows),'checks_passed':sum(r['passed'] for r in rows),'rows':rows,'scope':'public CLI graph checking only, not module execution or performance'}
 out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
 print(json.dumps({k:result[k] for k in ('checks_total','checks_passed','binary_sha256')},ensure_ascii=False));return 0 if result['checks_total']==result['checks_passed'] else 1
if __name__=='__main__':sys.exit(main())
