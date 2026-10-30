import json,re,subprocess,hashlib,zipfile
from pathlib import Path
root=Path.cwd()
def git(*a):return subprocess.run(['git',*a],capture_output=True,check=True).stdout
paths=sorted(set(filter(None,(git('diff','--name-only','-z')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0')))|{'docs/reports/T16-C-release-evidence/commit-groups.json'})
assert not git('diff','9d7e1aa','--','Cargo.toml','Cargo.lock','docs/validation/agent-benchmark-v2','docs/reports/agent-benchmark-v2/v2-20261024-r01')
old=json.loads(git('show','9d7e1aa:tests/cases.json'))['cases']; now=json.loads((root/'tests/cases.json').read_text(encoding='utf-8'))['cases'];assert now[:len(old)]==old
protect={'.opencode/team/DECISIONS.md':r'(?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)', '.opencode/team/PROJECT_STATE.md':r'(?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n', '.opencode/team/agents/team-lead/STATUS.md':r'(?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n'}
stage=root/'Temp/T16-C-release/stage';stage.mkdir(parents=True,exist_ok=True)
snapshots=[]
for p,pat in protect.items():
 t=(root/p).read_text(encoding='utf-8');assert len(re.findall(pat,t))==1,p
 out=stage/(p.replace('/','__'));out.write_text(re.sub(pat,'',t),encoding='utf-8')
 snapshots.append({'path':p,'staged_copy':out.relative_to(root).as_posix(),'removed_matches':1,'copy_sha256':hashlib.sha256(out.read_bytes()).hexdigest()})
planning={'docs/reports/T17-design.md','docs/reports/T17-design-draft.md','docs/reports/T17-resource-error-proposal.md'}
excluded=[p for p in paths if 't17' in p.lower() and p not in planning]
paths=[p for p in paths if p not in excluded and not p.startswith(('src/','tests/'))]
archive=root/'docs/reports/T16-C-release-evidence/candidate-snapshot.zip'
assert hashlib.sha256(archive.read_bytes()).hexdigest()=='8a6f9d0acadb86fa56ca49bb7e0a82650f7fd00902348dbf46b4bfbe7b8a9532'
with zipfile.ZipFile(archive)as z:
 frozen={n:z.read(n)for n in z.namelist()if n.startswith(('src/','tests/'))}
assert len(frozen)==21
for name,b in frozen.items():
 out=stage/name.replace('/','__');out.write_bytes(b)
 snapshots.append({'path':name,'staged_copy':out.relative_to(root).as_posix(),'kind':'frozen_C_source_or_test','copy_sha256':hashlib.sha256(b).hexdigest()})
 prior=subprocess.run(['git','show','9d7e1aa:'+name],capture_output=True)
 if prior.returncode or prior.stdout.replace(b'\r\n',b'\n')!=b.replace(b'\r\n',b'\n'):paths.append(name)
paths=sorted(set(paths));snapshot_paths={s['path']for s in snapshots}
def group(p):
 if p.startswith('docs/spec/') or p=='.opencode/team/REQUIREMENTS.md':return 1
 if p.startswith('src/')or p.startswith('docs/reports/T16-C-runtime'):return 2
 if p.startswith('tests/')or p.startswith(('docs/reports/T16-C-blackbox','docs/reports/T16-C-independent','docs/reports/T16-C-tooling','docs/tooling/')):return 3
 if p.startswith('app/'):return 4
 if p.startswith(('.opencode/skills/','docs/guide/','docs/reports/T16-C-guide','docs/reports/T16-C-skill')):return 5
 if p.startswith('.opencode/team/')or p in planning or p in ['README.md','docs/reports/delivery-checklist.md']or p.startswith('docs/reports/T16-C-release-evidence/'):return 6
 raise ValueError(p)
msgs={1:'docs(spec): record module execution contracts',2:'feat(runtime): execute modules in defining context',3:'test(modules): verify execution and file diagnostics',4:'feat(app): split JSONL workflow into modules',5:'docs(guide): teach module execution workflows',6:'chore(checkpoint): freeze C candidate and T17 plans'}
r={'date':'2026-10-30','baseline':'9d7e1aa','status':'C independent PASS; T17 planning only; R304 OPEN/no timing/tag/Git','excluded_T17_paths':excluded,'snapshots':snapshots,'groups':[{'sequence':n,'message':msgs[n],'direct_stage_paths':[p for p in paths if group(p)==n and p not in snapshot_paths],'staged_snapshot_paths':[p for p in paths if group(p)==n and p in snapshot_paths]}for n in range(1,7)]}
(root/'docs/reports/T16-C-release-evidence/commit-groups.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'paths':len(paths),'groups':[(g['sequence'],len(g['direct_stage_paths']),len(g['staged_snapshot_paths']))for g in r['groups']],'snapshots':len(snapshots)}))