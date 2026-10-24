import importlib.util,json,subprocess,sys,time,hashlib,shutil
from pathlib import Path
root=Path.cwd();work=root/'Temp/verifier-benchmark-v2';pkg=root/'docs/validation/agent-benchmark-v2';exe=root/'target/release/lfz.exe'
s=importlib.util.spec_from_file_location('r',pkg/'run.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
results=[]
def check(name,condition,details):
 results.append({'name':name,'pass':bool(condition),'details':details})
def cli(name,args):
 cmd=[sys.executable,'-B',str(pkg/'run.py'),'--lfz',str(exe),*args];r=subprocess.run(cmd,capture_output=True,cwd=root,timeout=40)
 d={'command':cmd,'exit':r.returncode,'stdout':r.stdout.decode('utf-8',errors='replace'),'stderr':r.stderr.decode('utf-8',errors='replace')};(work/(name+'.json')).write_text(json.dumps(d,ensure_ascii=False,indent=2),encoding='utf-8');return r,d
missing=work/'empty';missing.mkdir(exist_ok=True)
r,ev=cli('missing-cli',['--solutions',str(missing),'--output',str(work/'missing-results.json')]);d=json.loads((work/'missing-results.json').read_text(encoding='utf-8'))
check('CLI missing submissions refused',r.returncode==1 and d['functional_pass_tasks']==0 and d['denominator']==8 and all(c['status']=='MISSING' for t in d['tasks'] for c in t['cases']),ev)
wrong=work/'wrong-submissions';(wrong/'B01').mkdir(parents=True,exist_ok=True);(wrong/'B01/solution.lfz').write_text('#42\nfn solve(s) => nil\n',encoding='utf-8')
r,ev=cli('wrong-cli',['--solutions',str(wrong),'--output',str(work/'wrong-results.json')]);d=json.loads((work/'wrong-results.json').read_text(encoding='utf-8'))
check('CLI wrong solution refused without certification',r.returncode==1 and d['functional_pass_tasks']==0 and all(c['status']=='FAIL' for c in d['tasks'][0]['cases']) and d['certified_pass_tasks'] is None and all(t['overall_task_status']=='NOT_CERTIFIED' and t['source_review']==t['protocol_review']=='REVIEW_REQUIRED' for t in d['tasks']),ev)
p=work/'timeout-default.lfz';p.write_text('#42\nwhile true {\n}\n',encoding='utf-8');t=time.monotonic();r=m.execute(exe,p);elapsed=time.monotonic()-t
check('default ten-second process timeout',r['timed_out'] and r['exit'] is None and 9.5<=elapsed<15,r)
c={'id':'independent-io','args':['7\n-2\n'],'expected':{'count':2,'total':5},'expected_output':'index,value,running\n1,7,7\n2,-2,5\n'}
sources={
 'good':('#42\nfn solve(p,q) {\nassert(readFile(p) == "7\\n-2\\n")\nwriteFile(q,'+m.literal(c['expected_output'])+')\nreturn {count:2,total:5}\n}\n').encode(),
 'append':('#42\nfn solve(p,q) {\nappendFile(q,'+m.literal(c['expected_output'])+')\nreturn {count:2,total:5}\n}\n').encode(),
 'wrong-return':('#42\nfn solve(p,q) {\nwriteFile(q,'+m.literal(c['expected_output'])+')\nreturn {count:2,total:6}\n}\n').encode(),
 'crlf':('#42\nfn solve(p,q) {\nwriteFile(q,'+m.literal(c['expected_output'].replace('\n','\r\n'))+')\nreturn {count:2,total:5}\n}\n').encode()}
for name,src in sources.items():
 rr=m.run_case('B08',c,src,exe,work/('io-'+name));(work/('io-'+name+'.json')).write_text(json.dumps(rr,ensure_ascii=False,indent=2),encoding='utf-8')
 check('B08 '+name,rr['status']==('PASS' if name=='good' else 'FAIL') and len(rr['attempts'])==2 and all((root/a['program_path']).resolve().is_relative_to((root/'Temp').resolve()) for a in rr['attempts']),rr)
# verify_freeze must bind the interpreter path as well as its bytes without modifying frozen artifacts.
copy=work/'lfz-copy.exe';shutil.copyfile(exe,copy)
try:m.verify_freeze(copy);ok=False;msg='unexpected success'
except ValueError as e:ok='manifest mismatch' in str(e);msg=str(e)
check('manifest path mismatch rejected',ok,msg)
r,ev=cli('output-preservation',['--self-test','--output',str(work/'cli-self-test.json')]);check('existing report not overwritten',r.returncode==2 and 'output exists' in ev['stderr'],ev)
summary={'kind':'INDEPENDENT_RUNNER_REVIEW_NOT_AGENT_SCORE','manifest_sha256':m.verify_freeze(exe),'runner_sha256':hashlib.sha256((pkg/'run.py').read_bytes()).hexdigest(),'cases_sha256':hashlib.sha256((pkg/'cases.json').read_bytes()).hexdigest(),'checks':results,'total':len(results),'passed':sum(x['pass'] for x in results)}
(work/'independent-controls.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'checks':summary['total'],'passed':summary['passed'],'failed':[x['name'] for x in results if not x['pass']],'manifest_sha256':summary['manifest_sha256']},ensure_ascii=True))