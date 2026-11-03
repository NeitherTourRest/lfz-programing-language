"""Independent bounded JSONL application oracle; small fixtures only."""
import argparse,ast,hashlib,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 a=argparse.ArgumentParser();a.add_argument('--prepare-only',action='store_true');a.add_argument('--binary',type=Path);a.add_argument('--expected-sha');a.add_argument('--output',type=Path);o=a.parse_args();app=ROOT/'app/jsonlstream.lfz'
 if o.prepare_only:
  print(json.dumps(dict(prepared=True,source_sha256=sha(app),interpreter_processes=0)));return
 assert o.binary and o.expected_sha and o.output
 exe=o.binary.resolve();report=o.output.resolve();assert exe.is_relative_to(ROOT) and report.is_relative_to(ROOT) and not report.exists() and sha(exe)==o.expected_sha
 w=ROOT/'Temp/T19-app';w.mkdir(parents=True,exist_ok=True);report.parent.mkdir(parents=True,exist_ok=True)
 inp=w/'input.jsonl';out=w/'output.jsonl';summary=w/'summary.json';rows=[]
 evidence=dict(binary_sha256=sha(exe),source_sha256=sha(app),rows=rows,status='running')
 def save():report.write_text(json.dumps(evidence,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 def run(name,data,take=0,cap=4096,error=None,expected=None,flags=(),keep=False):
  inp.write_bytes(data);out.write_bytes(b'KEEP');summary.write_bytes(b'KEEP')
  cmd=[str(exe),'run','--json',*flags,str(app),'--',str(inp),str(out),str(summary),str(take),str(cap)]
  p=subprocess.run(cmd,cwd=ROOT,capture_output=True,timeout=15,env=dict(os.environ,TEMP=str(w),TMP=str(w)),creationflags=0x08000000 if os.name=='nt' else 0)
  r=dict(name=name,argv=cmd,exit=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode(),output_hex=out.read_bytes().hex(),summary_hex=summary.read_bytes().hex(),passed=False);rows.append(r);save()
  assert not p.stderr and p.returncode==(2 if error else 0)
  if error:
   e=json.loads(p.stdout);assert e['error']==error
   if keep:assert out.read_bytes()==b'KEEP'
   elif expected=='prefix':
    prefix=[json.loads(x) for x in out.read_bytes().splitlines()];assert prefix==[dict(line=i,ok=True,value=1,error=None) for i in range(1,257)]
   else:assert out.read_bytes()==(expected or b'')
   assert summary.read_bytes()==b'KEEP'
  else:
   raw=data.split(b'\n');raw=raw[:-1] if data.endswith(b'\n') else raw
   if not data:raw=[]
   raw=[x[:-1] if x.endswith(b'\r') and (i<len(raw)-1 or data.endswith(b'\n')) else x for i,x in enumerate(raw)]
   selected=raw[:take] if take else raw;actual=[json.loads(x) for x in out.read_text(encoding='utf8').splitlines()];assert len(actual)==len(selected)
   good=0
   for n,(text,row) in enumerate(zip(selected,actual),1):
    assert row['line']==n
    try:value=json.loads(text.decode(),parse_constant=lambda x: (_ for _ in ()).throw(ValueError(x)));valid=True
    except (ValueError,UnicodeError):valid=False
    assert row['ok']==valid
    if valid:assert row['value']==value and row['error'] is None;good+=1
    else:assert row['value'] is None and row['error']['class']=='ValueError' and isinstance(row['error']['line'],int)
   want=dict(total=len(selected),passed=good,failed=len(selected)-good,lines=len(selected),stopped=bool(take and len(selected)==take))
   assert json.loads(summary.read_bytes())==want and json.loads(p.stdout)==want
  r['passed']=True;save()
 try:
  for name,data in [('empty',b''),('mixed','1\nBAD\n\n{"字":"🙂"}\n'.encode()),('crlf',b'1\r\nfalse\r\n'),('no_tail',b'null\ntrue'),('two_blank',b'\n\n'),('bom','\ufeff1\n2'.encode())]:run(name,data)
  run('early_false_skips_invalid',b'1\n\xff'+b'x'*5000,take=1)
  run('exact_limit',b'1234\n',cap=4)
  run('cr_counts_in_limit',b'1234\r\n',cap=4,error='ValueError')
  run('invalid_utf8',b'1\n\xff',error='ValueError')
  run('long_line',b'1\n12345',cap=4,error='ValueError')
  run('batch_flush',b'1\n'*600)
  # First complete batch persists; pending record 257 is discarded when line 258 fails.
  run('flushed_prefix_survives_read_error',b'1\n'*257+b'\xff',error='ValueError',expected='prefix')
  run('global_budget_before_output',b'1\n',error='ExecutionLimitError',flags=['--max-steps','0'],keep=True)
  evidence['status']='completed'
 except BaseException as e:evidence.update(status='failed',error=repr(e));raise
 finally:evidence.update(total=len(rows),passed=sum(x['passed'] for x in rows));save()
 print(json.dumps(dict(status=evidence['status'],total=len(rows),passed=evidence['passed'])))
if __name__=='__main__':main()
