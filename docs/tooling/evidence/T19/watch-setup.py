from pathlib import Path
s=Path('Temp/T18-B-tooling/watch.py').read_text(encoding='utf-8-sig').replace("W=R/'Temp/T18-B-tooling'","W=R/'Temp/T19-tooling'").replace("[str(R/'target/release/deps'/name)]","[str(R/'target/release/deps'/name),'--test-threads=1']")
Path('Temp/T19-tooling/watch.py').write_text(s,encoding='utf-8')
