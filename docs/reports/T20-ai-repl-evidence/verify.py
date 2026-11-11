"""T20 AI REPL聚焦，最终hash GO前仅prepare-only。"""
from pathlib import Path
import argparse,json,hashlib,re,subprocess
R=Path(__file__).resolve().parents[3]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--prepare-only',action='store_true');ap.add_argument('--binary',type=Path);ap.add_argument('--sha256');ap.add_argument('--output',type=Path,required=True);a=ap.parse_args();out=(R/a.output).resolve();assert out.is_relative_to(R)and not out.exists();out.parent.mkdir(parents=True,exist_ok=True)
 docs=['.opencode/skills/lfz-programming/'+n for n in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md'];hashes={n:sha(R/n)for n in docs};cases=[]
 for i,n in enumerate([docs[0],docs[-1]]):
  t=(R/n).read_text(encoding='utf8').split('## T20 REPL 工作流')[1];src=re.search(r'```repl\n(.*?)\n```',t,re.S)[1]+'\n';cases.append((f'extracted{i}',[],src,0,'2\n5\n',None))
 cases += [('multiline',[],'fn twice(x) {\nreturn x*2\n}\ntwice(4)\n',0,'8\n',None),('error_continue',[],'missing\n7\n',2,'7\n','NameError'),('old_source',[],'fn bad() {\nfail("old")\n}\nbad()\n9\n',2,'9\n','<repl:1>'),('incomplete_eof',[],'fn f() {\n',2,'','SyntaxError'),('input',[],'input()\n:quit\n4\n',0,':quit\n4\n',None),('argv',['--','中文'],'argv()[0]\n',0,'中文\n',None)]
 v=dict(mode='prepare-only'if a.prepare_only else'final',document_hashes_before=hashes,cases=len(cases),runs=[])
 def save():out.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 if a.prepare_only:v['lfz_launches']=0;save();print('prepared',len(cases),'LFZ launches 0');return
 x=(R/a.binary).resolve();assert x.is_relative_to(R)and sha(x)==a.sha256;v.update(binary_sha256=sha(x),binary_bytes=x.stat().st_size)
 for name,args,src,code,stdout,err in cases:
  assert sha(x)==a.sha256
  row=dict(name=name,input=src,args=args);v['runs'].append(row);save()
  p=subprocess.run([str(x),'repl',*args],input=src.encode(),cwd=R,capture_output=True,timeout=15);s=p.stdout.decode();e=p.stderr.decode();row.update(exit=p.returncode,stdout=s,stderr=e,passed=p.returncode==code and s==stdout and (err in e if err else not e));save()
 v['document_hashes_after']={n:sha(R/n)for n in docs};v['documents_unchanged']=v['document_hashes_after']==hashes;save();assert all(r['passed']for r in v['runs'])and v['documents_unchanged']
if __name__=='__main__':main()
