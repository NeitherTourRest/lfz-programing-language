from pathlib import Path
import ast, subprocess,json,re
R=Path.cwd(); W=R/'Temp/T21-tooling';s=(R/'tests/cli_t21.rs').read_text(encoding='utf-8-sig'); raw=ast.literal_eval(re.search(r'let raw = (b"#42.*?);',s).group(1));(W/'column-oracle.lfz').write_bytes(raw)
p=subprocess.run([str(R/'dist/lfz.exe'),'column-oracle.lfz'],cwd=W,capture_output=True,timeout=5);print(repr(p.stdout),repr(p.stderr),p.returncode);assert p.returncode==0
(W/'baseline-position.json').write_text(json.dumps({'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode(),'effect':(W/'effect.txt').read_text()},indent=2),encoding='utf-8')
