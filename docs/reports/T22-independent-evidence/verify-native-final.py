from pathlib import Path
import json,hashlib,subprocess
r=Path.cwd();p=r/'Temp/T22-verifier/input-final';q=json.loads((p/'ordinary-console.json').read_text(encoding='utf8'));assert len(q['results'])==21 and 'harness_error'not in q['handles']; marks={'ordinary-out':'中文雪','ordinary-error':'未定义的名字','ordinary-repl':'中文雪','ordinary-repl-error':'repl: 未知命令','ordinary-repl-chinese-input':'3字符','write-only-out':'中文雪','write-only-error':'未定义的名字'}
for x in q['results']:
 v=x['variant'];assert x['exit']==(2 if 'error'in v else 0); assert x['actual_cp']==x['cp'];expect=not v.startswith('write-only') or x['cp']==65001;assert (marks[v]in x['text'])==expect,(v,x['cp']);assert hashlib.sha256((p/f"{x['cp']}-{v}.utf16le").read_bytes()).hexdigest()==x['raw_sha256'];print(x['cp'],v,'EXPECTED',x['exit'])
exe=r/'dist/lfz.exe';w=r/'Temp/T22-verifier';p=subprocess.run([str(exe),'-e','print("中文雪😀")'],cwd=w,capture_output=True,timeout=5); assert p.returncode==0 and p.stdout=='中文雪😀\n'.encode(); (w/'pipe-utf8.bin').write_bytes(p.stdout)
