import json,re,subprocess,hashlib
from pathlib import Path
root=Path.cwd()
def git(*a):return subprocess.run(['git',*a],capture_output=True,check=True).stdout
paths=sorted(set(filter(None,(git('diff','--name-only','-z')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0')))|{'docs/reports/T16-A-release-evidence/commit-groups.json'})
assert not git('diff','e1368de','--','Cargo.toml','Cargo.lock','docs/validation/agent-benchmark-v2','docs/reports/agent-benchmark-v2/v2-20261024-r01')
old=json.loads(git('show','e1368de:tests/cases.json'))['cases']; now=json.loads((root/'tests/cases.json').read_text(encoding='utf-8'))['cases'];assert now[:len(old)]==old
protect={'.opencode/team/DECISIONS.md':r'(?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)', '.opencode/team/PROJECT_STATE.md':r'(?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n', '.opencode/team/agents/team-lead/STATUS.md':r'(?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n'}
stage=root/'Temp/T16-A-release/stage';stage.mkdir(parents=True,exist_ok=True)
snapshots=[]
for p,pat in protect.items():
 t=(root/p).read_text(encoding='utf-8');assert len(re.findall(pat,t))==1,p
 out=stage/(p.replace('/','__'));out.write_text(re.sub(pat,'',t),encoding='utf-8')
 snapshots.append({'path':p,'staged_copy':out.relative_to(root).as_posix(),'removed_matches':1,'copy_sha256':hashlib.sha256(out.read_bytes()).hexdigest()})
def group(p):
 if p.startswith('docs/spec/') or p in ['docs/reports/T16-A-design.md','.opencode/team/REQUIREMENTS.md']:return 1
 if p in ['src/source.rs','src/lib.rs'] or p.startswith('docs/reports/T16-A-core'):return 2
 if p in ['src/value.rs','src/evaluator.rs'] or p.startswith('docs/reports/T16-A-runtime'):return 3
 if p.startswith(('docs/reports/T16-A-independent','tests/')):return 4
 if p.startswith('.opencode/team/') or p in ['README.md','docs/reports/delivery-checklist.md','docs/reports/T16-B-design-draft.md'] or p.startswith('docs/reports/T16-A-release-evidence/'):return 5
 raise ValueError(p)
msgs={1:'docs(spec): define source identity foundations',2:'feat(source): add source identity metadata',3:'feat(runtime): attach optional source origins',4:'test(source): verify metadata compatibility',5:'chore(checkpoint): preserve T16-A candidate'}
r={'date':'2026-10-29','baseline':'e1368de','status':'T16-A foundations and compatibility independently PASS; no import execution; R304 OPEN; no timing/tag/Git writes','original_cases_preserved':len(old),'snapshots':snapshots,'groups':[{'sequence':n,'message':msgs[n],'direct_stage_paths':[p for p in paths if group(p)==n and p not in protect],'staged_snapshot_paths':[p for p in paths if group(p)==n and p in protect]} for n in range(1,6)]}
(root/'docs/reports/T16-A-release-evidence/commit-groups.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'paths':len(paths),'groups':[(g['sequence'],len(g['direct_stage_paths']),len(g['staged_snapshot_paths'])) for g in r['groups']],'user_matches':[1,1,1],'original_cases':len(old)}))