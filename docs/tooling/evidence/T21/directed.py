from pathlib import Path
import os,subprocess
R=Path.cwd();D=R/'Temp/T21-tooling';env=os.environ.copy();env.update(TEMP=str(R/'Temp/toolchain'),TMP=str(R/'Temp/toolchain'),RUSTUP_HOME=str(R/'Temp/toolchain/rustup'),CARGO_HOME=str(R/'Temp/toolchain/cargo'))
for label,argv in [('formatter',['lfz-63397fd2d9985014.exe','formatter::tests']),('cli',['cli_t21-f65aa2b7afce5cb1.exe'])]:
 p=subprocess.run([str(R/'target/release/deps'/argv[0]),*argv[1:],'--test-threads=1'],cwd=R,env=env,capture_output=True,timeout=30);(D/f'directed-first-{label}.txt').write_bytes(p.stdout+p.stderr);print(p.stdout.decode('utf-8','replace'));assert p.returncode==0,p.returncode
