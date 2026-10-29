import hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T14-independent-evidence';W=R/'Temp/T14-verifier';X=R/'dist/lfz.exe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
H='83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79'
assert sha(X)==sha(R/'target/release/lfz.exe')==H and X.stat().st_size==774656
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
log=(R/'docs/tooling/evidence/T14/cargo-test-release.txt').read_text(encoding='utf8')
paths=re.findall(r'Running [^\n]+\((target\\release\\deps\\[^)]+\.exe)\)',log)
assert len(paths)==7,paths
unit=[]
for i,name in enumerate(paths):
 p=R/name.replace('\\','/');r=subprocess.run([str(p)],capture_output=True,cwd=R,env=env,timeout=120)
 text=(r.stdout+r.stderr).decode('utf8');n=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',text)
 assert r.returncode==0 and n,(name,text[-600:])
 out=E/f'harness-{i+1}.txt';out.write_bytes(r.stdout+r.stderr)
 unit.append({'path':name,'sha256':sha(p),'passed':int(n[1]),'output':out.name})
assert sum(x['passed'] for x in unit)==542
(E/'harnesses.json').write_text(json.dumps(unit,indent=2)+'\n',encoding='utf8')
rows=[];existing=W/'exists.txt';missing=W/'not-created.txt';sentinel=b'KEEP\x00ORIGINAL\xff'
def invoke(args,data=b''):
 assert sha(X)==H
 return subprocess.run([str(X),*args],input=data,capture_output=True,cwd=R,env=env,timeout=8)
programs=[
 ('io_and_output',f'writeFile("{existing.as_posix()}","changed")\nappendFile("{existing.as_posix()}","append")\nwriteFile("{missing.as_posix()}","new")\nprint("bad")\neprint("bad")\n;;\ninput("prompt")',True),
 ('loop','while true {}',True),('recursive','fn again() => again()\nagain()',True),
 ('runtime_errors','unknownName\n1+true\n1/0\nassert(false)\nreadFile("no-such-file")',True),
 ('late_syntax',f'writeFile("{existing.as_posix()}","bad")\nprint(1 + )',False),
 ('unreachable_syntax','if false { print(1 + ) }',False),
 ('unicode','let text = "中文🙂"\nprint(text)',True),('empty','',True)]
for name,src,ok in programs:
 for origin in ('file','code','stdin'):
  for machine in (False,True):
   existing.write_bytes(sentinel)
   assert not missing.exists()
   path=W/'probe.lfz';path.write_text('#42\n'+src+'\n',encoding='utf8',newline='\n')
   operand=[str(path)] if origin=='file' else (['-e',src] if origin=='code' else ['-'])
   args=['check']+(['--json'] if machine else [])+operand
   r=invoke(args,src.encode('utf8') if origin=='stdin' else b'')
   assert r.returncode==(0 if ok else 2),(name,origin,r.stdout,r.stderr)
   assert existing.read_bytes()==sentinel and not missing.exists(),name
   if machine:
    obj=json.loads(r.stdout);assert obj['ok']==ok and obj['mode']=='check' and obj['executed'] is False and not r.stderr
    if ok:assert list(obj)==['ok','mode','executed']
    else:
     # Same front-end syntax error independently obtained via run's full parser before evaluation.
     runargs=['run','--json']+operand
     ref=invoke(runargs,src.encode('utf8') if origin=='stdin' else b'')
     error=json.loads(ref.stdout);assert ref.returncode==2
     assert {k:v for k,v in obj.items() if k not in ('mode','executed')}==error,(name,obj,error)
   else:assert not r.stdout and ((not r.stderr) if ok else bool(r.stderr))
   rows.append({'name':name,'origin':origin,'json':machine,'args':args,'source':src,'exit':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex(),'sentinel_unchanged':True,'missing_not_created':True})
# Operand protection: these exact strings must reach the lexer, not option handling.
for code in ('--json','--','check',''):
 r=invoke(['check','-e',code,'--json']);obj=json.loads(r.stdout)
 assert obj['mode']=='check' and obj['executed'] is False
 rows.append({'name':'code_operand','source':code,'exit':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
for tail in (['--'],['--','arg'],['--help'],[],['-e']):
 args=['check','--json']+tail;r=invoke(args)
 assert r.returncode==2 and not r.stdout and r.stderr
 rows.append({'name':'usage','args':args,'exit':r.returncode,'stderr_hex':r.stderr.hex()})
(E/'witnesses.json').write_text(json.dumps({'date':'2026-10-29','binary_sha256':H,'passed':len(rows),'rows':rows},ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(json.dumps({'unit_passed':542,'independent_process_witnesses':len(rows)}))
