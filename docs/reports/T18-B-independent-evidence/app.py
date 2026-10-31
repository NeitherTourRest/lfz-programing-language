"""T16 four-module JSONL application oracle. No compilation or external scratch paths."""
import argparse, datetime, hashlib, json, os, subprocess
from pathlib import Path
import sys
sys.path.insert(0,str(Path.cwd()/'app'))
from verify_jsonbatch import strict
ROOT=Path.cwd()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--lfz',type=Path,required=True);ap.add_argument('--expected-sha',required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
    exe=a.lfz.resolve();out=a.output.resolve();expected=a.expected_sha.lower()
    assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists() and sha(exe)==expected
    w=ROOT/'Temp/T18-B-verifier/app';w.mkdir(parents=True,exist_ok=True);env=dict(os.environ,TEMP=str(w),TMP=str(w),PYTHONDONTWRITEBYTECODE='1')
    app=ROOT/'app/jsonbatch_modules/main.lfz';rows=[]
    def invoke(name,command):
        assert sha(exe)==expected
        p=subprocess.run([str(exe),*map(str,command)],capture_output=True,cwd=ROOT,env=env,timeout=15)
        row={'name':name,'command':list(map(str,command)),'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode(),'passed':False};rows.append(row);return p,row
    p,r=invoke('check_graph_no_initialization',['check','--json',app]);assert p.returncode==0 and not p.stderr and json.loads(p.stdout)=={'ok':True,'mode':'check','executed':False};r['passed']=True
    samples=[('mixed',(ROOT/'app/fixtures/t15/mixed.jsonl').read_bytes()),('empty',b''),('one',b'42'),('final_lf',b'42\n'),('blank',b'\n'),('two_blank',b'\n\n'),('middle_blank',b'1\n\n2\n'),('crlf',b'1\r\ninvalid\r\nnull\r\n'),('nil_false_zero',b'null\nfalse\n0\n'),('unicode','{"text":"中文🙂"}\n'.encode()),('i64',b'9223372036854775808\n9007199254740993\n'),('duplicate',b'{"a":1,"a":2}\n{}\n'),('recovery',b'wrong\n'*80+b'{"ok":true}\n')]
    for name,data in samples:
        ip=w/'in.jsonl';op=w/'out.json';ip.write_bytes(data);op.write_bytes(b'KEEP')
        last=None
        for k in range(2):
            p,r=invoke(name+('_again' if k else ''),['run',app,'--',ip,op]);assert p.returncode==0 and not p.stderr,r
            doc=json.loads(op.read_bytes());text=data.decode().replace('\r\n','\n');lines=[] if not text else text.split('\n')
            if text.endswith('\n'):lines.pop()
            assert doc['total']==len(lines) and len(doc['rows'])==len(lines) and doc['moduleCalls']==len(lines),doc
            good=0
            for n,(line,item) in enumerate(zip(lines,doc['rows']),1):
                assert item['line']==n and set(item)=={'line','ok','value','error'}
                try:v=strict(line);ok=True
                except ValueError:v=None;ok=False
                assert item['ok'] is ok
                if ok: assert item['value']==v and item['error'] is None;good+=1
                else:
                    er=item['error'];assert item['value'] is None and er['class']=='ValueError' and set(er)=={'class','message','line','col','traceback'}
                    assert er['traceback'][-1]['func']=='<fn>' and er['line']>0 and er['col']>0
            assert doc['passed']==good and doc['failed']==len(lines)-good and ip.read_bytes()==data
            assert json.loads(p.stdout)==dict(total=len(lines),passed=good,failed=len(lines)-good,path=str(op))
            if last is not None:assert last==op.read_bytes(),'session/cache/results not isolated'
            last=op.read_bytes();r['passed']=True;r['result']=doc
    for name,args in [('no_args',[]),('extra',[w/'in.jsonl',w/'out.json','x']),('empty',['',w/'out.json']),('same',[w/'in.jsonl',w/'in.jsonl']),('missing',[w/'missing.jsonl',w/'out.json'])]:
        ip=w/'in.jsonl';op=w/'out.json';ip.write_bytes(b'1\n');op.write_bytes(b'KEEP');p,r=invoke(name,['run',app,'--',*args]);assert p.returncode==2 and ip.read_bytes()==b'1\n' and op.read_bytes()==b'KEEP',r
        if name in ('no_args','extra','empty','same'):assert 'jsonbatch-modules:' in r['stderr'] and 'entry-local-do-not-use' not in r['stderr']
        r['passed']=True
    ip.write_bytes(b'\xff');p,r=invoke('invalid_utf8',['run',app,'--',ip,op]);assert p.returncode==2 and op.read_bytes()==b'KEEP';r['passed']=True
    result={'date':'2026-10-30','binary_sha256':expected,'binary_bytes':exe.stat().st_size,'binary':str(exe.relative_to(ROOT)),'total':len(rows),'passed':sum(x['passed'] for x in rows),'source_hashes':{str(p.relative_to(ROOT)):sha(p) for p in sorted((ROOT/'app/jsonbatch_modules').glob('*.lfz'))},'rows':rows}
    out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({k:v for k,v in result.items() if k not in ('rows','source_hashes')}))
if __name__=='__main__':main()
