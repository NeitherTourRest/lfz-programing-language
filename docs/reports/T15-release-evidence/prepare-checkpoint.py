import json,re,subprocess,hashlib
from pathlib import Path
root=Path.cwd()
def git(*a):return subprocess.run(['git',*a],capture_output=True,check=True).stdout
paths=sorted(set(filter(None,(git('diff','--name-only','-z')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0')))|{'docs/reports/T15-release-evidence/commit-groups.json'})
assert not git('diff','2c0e079','--','Cargo.toml','Cargo.lock','docs/validation/agent-benchmark-v2','docs/reports/agent-benchmark-v2/v2-20261024-r01')
old=json.loads(git('show','2c0e079:tests/cases.json'))['cases']; now=json.loads((root/'tests/cases.json').read_text(encoding='utf-8'))['cases'];assert now[:len(old)]==old
protect={'.opencode/team/DECISIONS.md':r'(?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)', '.opencode/team/PROJECT_STATE.md':r'(?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n', '.opencode/team/agents/team-lead/STATUS.md':r'(?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n'}
stage=root/'Temp/T15-release/stage';stage.mkdir(parents=True,exist_ok=True)
snapshots=[]
for p,pat in protect.items():
 t=(root/p).read_text(encoding='utf-8');assert len(re.findall(pat,t))==1,p
 out=stage/(p.replace('/','__'));out.write_text(re.sub(pat,'',t),encoding='utf-8')
 snapshots.append({'path':p,'staged_copy':out.relative_to(root).as_posix(),'removed_matches':1,'copy_sha256':hashlib.sha256(out.read_bytes()).hexdigest()})
def group(p):
 if p.startswith('docs/spec/') or p in ['docs/reports/T15-design.md','.opencode/team/REQUIREMENTS.md']:return 1
 if p.startswith('src/') or p.startswith('docs/reports/T15-runtime'):return 2
 if p.startswith('tests/') or p.startswith(('docs/reports/T15-blackbox','docs/reports/T15-independent')):return 3
 if p.startswith('app/'):return 4
 if p.startswith(('docs/guide/','.opencode/skills/','docs/reports/T15-guide','docs/reports/T15-skill')):return 5
 if p.startswith('.opencode/team/') or p in ['README.md','docs/reports/delivery-checklist.md','docs/reports/NEXT-ITERATIONS.md','docs/reports/T16-design-draft.md'] or p.startswith('docs/reports/T15-release-evidence/'):return 6
 raise ValueError(p)
msgs={1:'docs(spec): define controlled error recovery',2:'feat(runtime): add bounded attempt recovery',3:'test(runtime): cover attempt recovery boundaries',4:'feat(app): process mixed JSONL batches',5:'docs(guide): teach controlled error results',6:'chore(checkpoint): preserve T15 candidate and plans'}
r={'date':'2026-10-29','baseline':'2c0e079','status':'T15 final functional PASS; R304 OPEN; no T15 timing; candidate checkpoint only; no formal tag; no Git writes','original_cases_preserved':len(old),'snapshots':snapshots,'groups':[{'sequence':n,'message':msgs[n],'direct_stage_paths':[p for p in paths if group(p)==n and p not in protect],'staged_snapshot_paths':[p for p in paths if group(p)==n and p in protect]} for n in range(1,7)]}
(root/'docs/reports/T15-release-evidence/commit-groups.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'paths':len(paths),'groups':[(g['sequence'],len(g['direct_stage_paths']),len(g['staged_snapshot_paths'])) for g in r['groups']],'user_matches':[1,1,1],'original_cases':len(old)}))