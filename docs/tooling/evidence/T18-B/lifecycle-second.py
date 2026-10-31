import subprocess,os,json
from pathlib import Path
R=Path.cwd();env=os.environ.copy();env.update(RUSTUP_HOME=str(R/'Temp/toolchain/rustup'),CARGO_HOME=str(R/'Temp/toolchain/cargo'),TEMP=str(R/'Temp/toolchain'),TMP=str(R/'Temp/toolchain'))
p=subprocess.run([str(R/'target/release/deps/lfz-63397fd2d9985014.exe'),'http_control::windows::tests','--test-threads=1','--nocapture'],cwd=R,env=env,capture_output=True,timeout=40)
(R/'Temp/T18-B-tooling/lifecycle-second.txt').write_bytes(p.stdout+p.stderr)
print((p.stdout+p.stderr).decode('utf-8','replace'));assert p.returncode==0,p.returncode
