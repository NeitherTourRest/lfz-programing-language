"""T20 人指南真实 stdin 验证；prepare-only 不启动产品。"""
import argparse,hashlib,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
DOCS=[ROOT/'docs/guide'/n for n in ['README.md','tutorial.md','reference.md','errors.md','testing.md','repl.md']]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
CASES=[
 ('persistent',[], 'var n=1\nfn read()=>n\nn=2\nread()\n',0,'2\n',[]),
 ('multiline',[],'fn add(x) {\nreturn x+1\n}\nadd(2)\n',0,'3\n',[]),
 ('ifelse',[],'if true {\nprint("yes")\n} else {\nprint("no")\n}\n',0,'yes\n',[]),
 ('partial-effects',[],'var state={n:0}\nfn failLater(){\nstate.n+=1\n1/0\n}\nfailLater()\nstate.n\nprint("continued")\n',2,'1\ncontinued\n',['<repl:2>','<repl:3>','^']),
 ('default-source',[],'struct P{x:1/0}\nP{}\nprint("after")\n',2,'after\n',['<repl:1>','<repl:2>','^']),
 ('shared-stdin',[],'input()\n:quit\nprint("continued")\n:quit\n',0,':quit\ncontinued\n',[]),
 ('reset-argv',['--','--json','hello'],'argv()[0]\nvar x=1\n:unknown\n:reset\nargv()[1]\nx\n:quit\n',2,'--json\nhello\n',['repl: 未知命令','NameError']),
 ('steps',['--max-steps','100'],'while true {}\n1\n',2,'1\n',['ExecutionLimitError']),
 ('relative-timeout',['--timeout-ms','0'],'1\n:quit\n',2,'',['ExecutionLimitError']),
 ('incomplete-eof',[],'fn incomplete(){\n',2,'',['SyntaxError','<repl:1>']),
 ('empty-eof',[],'',0,'',[]),
 ('print-once',[],'print("once")\n1\n2\n',0,'once\n1\n2\n',[]),
 ('json-rejected',['--json'],'',2,'',['repl 不支持 --json']),
 ('import-rejected',[],'import lib from "./missing.lfz"\nprint("after")\n',2,'after\n',['REPL 暂不支持 import/export']),
 ('syntax-continue',[],'let =\nprint("after")\n',2,'after\n',['SyntaxError','<repl:1>','^']),
 ('raw-cap',[],' ' * 1048576+'x\nprint("never")\n',2,'',['cell bytes'])
]
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--prepare-only',action='store_true');ap.add_argument('--binary',type=Path);ap.add_argument('--sha256');ap.add_argument('--output',type=Path,default=ROOT/'docs/reports/T20-guide-evidence/runs.json');a=ap.parse_args()
 manifest={str(p.relative_to(ROOT)):sha(p) for p in DOCS}
 if a.prepare_only:print(json.dumps({'prepared':len(CASES),'executed':False,'docs':manifest}));return
 assert a.binary and a.sha256
 exe=a.binary.resolve();out=a.output.resolve();assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists();assert sha(exe)==a.sha256.lower()
 work=ROOT/'Temp/T20-guide/actual';work.mkdir(parents=True,exist_ok=True)
 rows=[];data={'binary_sha256':sha(exe),'binary_bytes':exe.stat().st_size,'docs':manifest,'runs':rows,'executed':True}
 for name,flags,source,code,stdout,need in CASES:
  assert sha(exe)==a.sha256.lower();p=subprocess.run([str(exe),'repl',*flags],input=source.encode(),capture_output=True,cwd=work,timeout=12)
  actual=p.stdout.decode('utf8');err=p.stderr.decode('utf8');ok=p.returncode==code and actual.replace('\r\n','\n')==stdout and all(n in err for n in need) and (bool(need) or not err)
  rows.append({'name':name,'args':['repl',*flags],'stdin_sha256':hashlib.sha256(source.encode()).hexdigest(),'stdin_bytes':len(source.encode()),'exit':p.returncode,'stdout':actual,'stderr':err,'pass':ok})
  out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 data['passed']=sum(r['pass'] for r in rows);data['final_binary_sha256']=sha(exe);data['final_docs']={str(p.relative_to(ROOT)):sha(p) for p in DOCS};assert data['final_docs']==manifest
 out.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':data['passed'],'total':len(rows)}));raise SystemExit(data['passed']!=len(rows))
if __name__=='__main__':main()