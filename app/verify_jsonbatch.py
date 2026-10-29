"""Independent JSONL oracle and process checks. All scratch data stays in project Temp."""
import argparse, datetime, hashlib, json, math, os, subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def strict(text):
    def integer(s):
        v=int(s)
        if not -(2**63)<=v<2**63: raise ValueError('i64')
        return v
    def floating(s):
        v=float(s)
        if not math.isfinite(v): raise ValueError('finite')
        return v
    def pairs(items):
        out={}
        for k,v in items:
            if k in out: raise ValueError('duplicate')
            out[k]=v
        return out
    def bad(s): raise ValueError(s)
    return json.loads(text,parse_int=integer,parse_float=floating,parse_constant=bad,object_pairs_hook=pairs)
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--lfz',type=Path,required=True);ap.add_argument('--expected-sha');ap.add_argument('--output',type=Path,required=True);ap.add_argument('--smoke-only',action='store_true');a=ap.parse_args()
    exe=a.lfz.resolve();out=a.output.resolve()
    assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists()
    expected=a.expected_sha.lower() if a.expected_sha else sha(exe); assert sha(exe)==expected
    work=ROOT/'Temp/T15-app';work.mkdir(parents=True,exist_ok=True)
    env=dict(os.environ,TEMP=str(work),TMP=str(work),PYTHONDONTWRITEBYTECODE='1')
    rows=[];source=ROOT/'app/jsonbatch.lfz'
    def run(name,args):
        assert sha(exe)==expected
        p=subprocess.run([str(exe),'run',str(source),'--',*map(str,args)],capture_output=True,cwd=ROOT,env=env,timeout=15)
        row={'name':name,'args':list(map(str,args)),'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8'),'passed':False};rows.append(row)
        return p,row
    samples=[('mixed',(ROOT/'app/fixtures/t15/mixed.jsonl').read_bytes()),('empty',b''),('one',b'42'),('terminal_lf',b'42\n'),('blank',b'\n'),('two_blanks',b'\n\n'),('blank_inside',b'1\n\n2\n'),('crlf','{"x":"中"}\r\ninvalid\r\nnull\r\n'.encode()),('lone_cr',b'1\r2'),('nil',b'null\nfalse\n0\n""\n'),('unicode','"🙂"\n{"x":"中文"}\n'.encode()),('bounds',b'9223372036854775807\n9223372036854775808\n-9223372036854775808\n'),('duplicate',b'{"a":1,"a":2}\n3\n'),('trailing_garbage',b'{}x\n[]\n'),('many',b'broken\n'*200+b'{"last":true}\n')]
    if a.smoke_only:samples=samples[:1]
    for name,data in samples:
        ip=work/'input.jsonl';op=work/'output.json';ip.write_bytes(data);op.write_bytes(b'OLD')
        p,row=run(name,[ip,op]);assert p.returncode==0 and not p.stderr,row
        result=json.loads(op.read_bytes());text=data.decode().replace('\r\n','\n');lines=[] if not text else text.split('\n')
        if text.endswith('\n'):lines.pop()
        assert set(result)=={'total','passed','failed','rows'} and result['total']==len(lines)
        good=0
        for i,(line,item) in enumerate(zip(lines,result['rows']),1):
            assert set(item)=={'line','ok','value','error'} and item['line']==i
            try:value=strict(line);ok=True
            except (ValueError,json.JSONDecodeError):value=None;ok=False
            assert item['ok'] is ok
            if ok: assert item['value']==value and item['error'] is None;good+=1
            else:
                er=item['error'];assert item['value'] is None and set(er)=={'class','message','line','col','traceback'}
                assert er['class']=='ValueError' and er['message'].startswith('JSON 解析失败') and er['line']>0 and er['col']>0
                assert er['traceback'] and all(set(f)=={'func','line','col'} for f in er['traceback'])
        assert len(result['rows'])==len(lines) and result['passed']==good and result['failed']==len(lines)-good
        assert json.loads(p.stdout)==dict(total=len(lines),passed=good,failed=len(lines)-good,path=str(op)) and ip.read_bytes()==data
        row['passed']=True;row['result']=result
        original=op.read_bytes();p,row=run(name+'_repeat',[ip,op]);assert p.returncode==0 and not p.stderr and op.read_bytes()==original;row['passed']=True
    if not a.smoke_only:
        ip=work/'input.jsonl';op=work/'output.json';ip.write_bytes(b'1\n')
        cases=[('no_args',[]),('one_arg',[ip]),('extra',[ip,op,'extra']),('empty_path',['',op]),('missing',[work/'absent.jsonl',op]),('same_path',[ip,ip]),('output_parent_missing',[ip,work/'nonexistent-parent'/'result.json'])]
        for name,args in cases:
            op.write_bytes(b'KEEP');before=ip.read_bytes();p,row=run(name,args);assert p.returncode==2 and ip.read_bytes()==before and op.read_bytes()==b'KEEP';row['passed']=True
        ip.write_bytes(b'\xff');op.write_bytes(b'KEEP');p,row=run('non_utf8',[ip,op]);assert p.returncode==2 and op.read_bytes()==b'KEEP' and ip.read_bytes()==b'\xff';row['passed']=True
    result={'date':datetime.datetime.now(datetime.timezone.utc).date().isoformat(),'binary_sha256':expected,'binary':str(exe.relative_to(ROOT)),'script_sha256':sha(source),'total':len(rows),'passed':sum(r['passed'] for r in rows),'rows':rows}
    out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({k:v for k,v in result.items() if k!='rows'}))
if __name__=='__main__':main()
