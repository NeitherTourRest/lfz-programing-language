from pathlib import Path
import os,subprocess
R=Path.cwd();env=os.environ.copy();env.update(RUSTUP_HOME=str(R/'Temp/toolchain/rustup'),CARGO_HOME=str(R/'Temp/toolchain/cargo'),TEMP=str(R/'Temp/toolchain'),TMP=str(R/'Temp/toolchain'))
for name,args in [('lfz-243aeb8f5c3370f1.exe',['t20_frontend_tests']),('cli_t20-be52399e4be58bfb.exe',[])]:
 p=subprocess.run([str(R/'target/release/deps'/name),*args,'--test-threads=1','--nocapture'],env=env,capture_output=True,timeout=40)
 (R/'Temp/T20-tooling'/('second-'+name+'.txt')).write_bytes(p.stdout+p.stderr);print((p.stdout+p.stderr).decode('utf-8','replace'));assert p.returncode==0,p.returncode
