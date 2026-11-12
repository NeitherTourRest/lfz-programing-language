from pathlib import Path
import hashlib, subprocess, json, os
root=Path.cwd(); work=root/'Temp/T21-verifier/own'; work.mkdir(parents=True,exist_ok=False)
candidate=root/'target/release/lfz.exe'; baseline=root/'dist/lfz.exe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(candidate)=='76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e'
assert sha(baseline)=='a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c'
env=dict(os.environ,TEMP=str(work),TMP=str(work)); result=[]; logs=[]
def run(binary,cwd,*argv):
 p=subprocess.run([str(binary),*argv],cwd=cwd,env=env,capture_output=True,timeout=10)
 logs.append(dict(exe=sha(binary),cwd=str(cwd),args=argv,exit=p.returncode,stdout_hex=p.stdout.hex(),stderr=p.stderr.decode('utf8','replace')));return p
def expect_format(name,raw,expected):
 src=work/name; src.write_bytes(raw); p=run(candidate,work,'fmt',name)
 assert (p.returncode,p.stdout,p.stderr,src.read_bytes())==(0,expected,b'',raw),(name,logs[-1]); second=work/('formatted-'+name);second.write_bytes(p.stdout)
 p=run(candidate,work,'fmt',second.name);assert (p.returncode,p.stdout,p.stderr,second.read_bytes())==(0,expected,b'',expected); result.append(name)
try:
 expect_format('raw.lfz','\ufeff#42\r\tlet x = 0xff \t\r\t \n\tprint("雪 ${\t"中 ${x:>4}" \t} \t") \t\r\n\t/*注 \t\r\n\t中 \t\r尾 */ \t\n\t//保留 \t\n\t x |> str() \t\n\t;; \t'.encode(), '\ufeff#42\r let x = 0xff\r\t \n print("雪 ${\t"中 ${x:>4}" \t} \t")\r\n /*注 \t\r\n\t中 \t\r尾 */\n //保留 \t\n  x |> str()\n ;;'.encode())
 expect_format('comment.txt','\ufeff\t/*开 \t\r内 \t\r\n收 */\t 9 \t\n\t//末 \t'.encode(),'\ufeff /*开 \t\r内 \t\r\n收 */\t 9\n //末 \t'.encode())
 expect_format('module.lfz', b'#42\n\timport dep from "./never-load.lfz" \t\n\tprint("DO NOT EXECUTE") \t\n',b'#42\n import dep from "./never-load.lfz"\n print("DO NOT EXECUTE")\n');assert not (work/'never-load.lfz').exists()
 expect_format('prelude.lfz',b'\xef\xbb\xbf#42\r',b'\xef\xbb\xbf#42\r')
 raw=b'#42\r\n\tfn probe() { \t\n\t\treturn unbound() \t\r\n\t} \t\r\t \n\tlet saved=attempt(()=>probe()) \t\n\tprint(jsonStringify({line:saved.error.line,col:saved.error.col,traceback:saved.error.traceback})) \t\n\tappendFile("effect.txt","x") /* no execution in fmt */ \t\r\n\tprobe() \t'
 expected=b'#42\r\n fn probe() {\n  return unbound()\r\n }\r\t \n let saved=attempt(()=>probe())\n print(jsonStringify({line:saved.error.line,col:saved.error.col,traceback:saved.error.traceback}))\n appendFile("effect.txt","x") /* no execution in fmt */\r\n probe()'
 expect_format('position.lfz',raw,expected);assert not (work/'effect.txt').exists()
 results=[]
 for variant,binary,data in [('baseline',baseline,raw),('current_raw',candidate,raw),('current_formatted',candidate,expected)]:
  cwd=work/variant;cwd.mkdir();(cwd/'subject.lfz').write_bytes(data);p=run(binary,cwd,'--json','subject.lfz');assert p.returncode==2 and (cwd/'effect.txt').read_bytes()==b'x';assert (cwd/'subject.lfz').read_bytes()==data
  external=json.loads(p.stdout); internal=json.loads(p.stderr); assert external['line']==internal['line']==3 and external['col']==internal['col']==10 and len(external['traceback'])>=2 and len(internal['traceback'])>=2,(external,internal);results.append((p.stdout,p.stderr))
 assert results[0]==results[1]==results[2];result.append('behavior_positions_trace_side_effect')
 for name,data in [('bad-utf8.lfz',b'#42\n\xff'),('bad-prefix.lfz',b'#42 \n1'),('bad-lex.lfz',b'#42\n@'),('bad-syntax.lfz',b'#42\nlet ='),('missing.lfz',None)]:
  src=work/name
  if data is not None:src.write_bytes(data)
  p=run(candidate,work,'fmt',name);assert p.returncode==2 and p.stdout==b'' and p.stderr and (src.read_bytes()==data if data is not None else not src.exists());result.append(name)
 legacy=work/'fmt';legacy.write_bytes(b'print(jsonStringify(argv()))\n')
 for opts in [('fmt',),('fmt','--json'),('--json','fmt'),('fmt','--'),('fmt','--','two'),('fmt','--allow-process'),('fmt','--allow-http'),('run','fmt')]:
  a=run(baseline,work,*opts);b=run(candidate,work,*opts);assert (a.returncode,a.stdout,a.stderr)==(b.returncode,b.stdout,b.stderr);result.append('legacy '+repr(opts))
 for opts in [('fmt','raw.lfz','extra'),('--json','fmt','raw.lfz'),('fmt','raw.lfz','--','extra')]:
  p=run(candidate,work,*opts);assert p.returncode==2 and p.stdout==b'' and p.stderr;result.append('rejected '+repr(opts))
finally:
 (work/'results.json').write_text(json.dumps(dict(candidate_sha256=sha(candidate),baseline_sha256=sha(baseline),passed=result,logs=logs),ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(len(result),'independent assertions groups')
