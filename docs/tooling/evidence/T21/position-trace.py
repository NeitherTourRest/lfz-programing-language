from pathlib import Path
import hashlib,json,os,subprocess
R=Path.cwd(); D=R/'Temp/T21-tooling'; env=os.environ.copy(); env.update(TEMP=str(R/'Temp/toolchain'),TMP=str(R/'Temp/toolchain'),RUSTUP_HOME=str(R/'Temp/toolchain/rustup'),CARGO_HOME=str(R/'Temp/toolchain/cargo'))
def run(binary,*args):return subprocess.run([str(binary),*args],cwd=D,env=env,capture_output=True,timeout=10)
exe=R/'target/release/lfz.exe'; baseline=R/'dist/lfz.exe'
raw=b'#42\r\n\tfn fail() { \t\r\n\t\treturn missing() \t\r\n\t} \t\n\tlet r=attempt(()=>fail()) \t\n\tprint(r.error.line) \t\n\tprint(r.error.col) \t\n\tprint(jsonStringify(r.error.traceback)) \t\n\tappendFile("position-effect.txt","x") /* kept */ \t\n\tfail() \t'
name='position-and-trace.lfz'; p=D/name; effect=D/'position-effect.txt'; p.write_bytes(raw); effect.write_bytes(b'')
a=run(baseline,'--json',name); assert a.returncode==2 and effect.read_bytes()==b'x'; effect.write_bytes(b'')
b=run(exe,'fmt',name); assert b.returncode==0 and not b.stderr and p.read_bytes()==raw and effect.read_bytes()==b''
p.write_bytes(b.stdout); c=run(exe,'--json',name); assert c.returncode==2 and effect.read_bytes()==b'x'; assert a.stdout==c.stdout and a.stderr==c.stderr,(a,c)
e=json.loads(c.stdout); assert e['error']=='NameError' and e['line']==3 and len(e['traceback'])>=2
(D/'position-original.lfz').write_bytes(raw);(D/'position-formatted.lfz').write_bytes(b.stdout)
(D/'position-trace-witness.json').write_text(json.dumps({'baseline_sha256':hashlib.sha256(baseline.read_bytes()).hexdigest(),'candidate_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'exit':c.returncode,'stdout_error':e,'stderr_script_output':c.stderr.decode('utf-8'),'effect':'x','formatted_bytes':len(b.stdout)},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
