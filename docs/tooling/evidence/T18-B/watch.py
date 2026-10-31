from pathlib import Path
import subprocess,os,re,json,hashlib
R=Path.cwd();W=R/'Temp/T18-B-tooling';names=['lfz-63397fd2d9985014.exe','lfz-243aeb8f5c3370f1.exe','cli-d19dbfa7dd3f9d73.exe','cli_t12-35b857587ead196a.exe','cli_t14-b528900585815e19.exe','cli_t16b-66b0b355c7f6f4d0.exe','cli_t16c-24a3e9d865ee9e42.exe','cli_t17-2cfc935946fdc938.exe','cli_t18a-2b1a6784d688e6c5.exe','cli_t18b-4fb747188dc55a23.exe','test_runner-a9d3da191eeb2958.exe','unit-ae562ae83f03ad63.exe']
env=os.environ.copy();env.update(TEMP=str(R/'Temp/toolchain'),TMP=str(R/'Temp/toolchain'),RUSTUP_HOME=str(R/'Temp/toolchain/rustup'),CARGO_HOME=str(R/'Temp/toolchain/cargo'))
rows=[]
for name in names:
 try:p=subprocess.run([str(R/'target/release/deps'/name)],cwd=R,env=env,capture_output=True,timeout=90)
 except subprocess.TimeoutExpired as e:
  (W/('watched-'+name+'.txt')).write_bytes((e.stdout or b'')+(e.stderr or b''));raise
 (W/('watched-'+name+'.txt')).write_bytes(p.stdout+p.stderr)
 m=re.search(rb'test result: ok\. (\d+) passed',p.stdout)
 row={'name':name,'exit':p.returncode,'passed':int(m[1]) if m else None};rows.append(row);print(row,flush=True)
 if p.returncode:print((p.stdout+p.stderr).decode('utf-8','replace')[-6000:]);break
(W/'watched-summary.json').write_text(json.dumps(rows,indent=2)+'\n',encoding='utf-8');assert all(r['exit']==0 for r in rows)
