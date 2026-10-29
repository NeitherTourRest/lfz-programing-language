import json,re,subprocess,hashlib,zipfile
from pathlib import Path
root=Path.cwd()
def git(*a):return subprocess.run(['git',*a],capture_output=True,check=True).stdout
paths=sorted(set(filter(None,(git('diff','--name-only','-z')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0')))|{'docs/reports/T16-B-release-evidence/commit-groups.json'})
# C execution work is active: only these explicit C planning documents may enter B.
c_planning={'docs/reports/T16-C-design-draft.md','docs/reports/T16-C-design.md','docs/reports/T16-C-interface-proposal.md'}
excluded_C_nonplanning_paths=[p for p in paths if 'T16-C' in p and p not in c_planning]
paths=[p for p in paths if p not in excluded_C_nonplanning_paths]
assert not git('diff','738e198','--','Cargo.toml','Cargo.lock','docs/validation/agent-benchmark-v2','docs/reports/agent-benchmark-v2/v2-20261024-r01')
old=json.loads(git('show','738e198:tests/cases.json'))['cases']; now=json.loads((root/'tests/cases.json').read_text(encoding='utf-8'))['cases'];assert now[:len(old)]==old
protect={'.opencode/team/DECISIONS.md':r'(?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)', '.opencode/team/PROJECT_STATE.md':r'(?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n', '.opencode/team/agents/team-lead/STATUS.md':r'(?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n'}
stage=root/'Temp/T16-B-release/stage';stage.mkdir(parents=True,exist_ok=True)
snapshots=[]
for p,pat in protect.items():
 t=(root/p).read_text(encoding='utf-8');assert len(re.findall(pat,t))==1,p
 out=stage/(p.replace('/','__'));out.write_text(re.sub(pat,'',t),encoding='utf-8')
 snapshots.append({'path':p,'staged_copy':out.relative_to(root).as_posix(),'removed_matches':1,'copy_sha256':hashlib.sha256(out.read_bytes()).hexdigest()})
# B production source always comes from the immutable B archive, even if C is active.
archive=root/'docs/reports/T16-B-release-evidence/candidate-snapshot.zip'
assert hashlib.sha256(archive.read_bytes()).hexdigest()=='db4ae08e66dd43d7a90c9ddd33706b170ee36c014114840922431c36292556ca'
with zipfile.ZipFile(archive) as z:
 bsource={n:z.read(n) for n in z.namelist() if n.startswith('src/')}
assert len(bsource)==18
expected_sources=json.loads((root/'docs/reports/T16-B-release-evidence/source-hashes.json').read_text(encoding='utf-8'))
for item in expected_sources:
 if item['path'].startswith('src/'):
  assert hashlib.sha256(bsource[item['path']]).hexdigest()==item['sha256'],item['path']
paths=[p for p in paths if not p.startswith('src/')]
for name,b in bsource.items():
 prior=subprocess.run(['git','show','738e198:'+name],capture_output=True)
 out=stage/name.replace('/','__');out.write_bytes(b)
 snapshots.append({'path':name,'staged_copy':out.relative_to(root).as_posix(),'kind':'frozen_B_source','copy_sha256':hashlib.sha256(b).hexdigest()})
 if prior.returncode!=0 or prior.stdout.replace(b'\r\n',b'\n')!=b.replace(b'\r\n',b'\n'):paths.append(name)

testpath='tests/cli_t16b.rs'
b=(root/'docs/reports/T16-B-release-evidence/cli_t16b.rs.txt').read_bytes()
out=stage/'tests__cli_t16b.rs';out.write_bytes(b)
snapshots.append({'path':testpath,'staged_copy':out.relative_to(root).as_posix(),'kind':'frozen_B_test','copy_sha256':hashlib.sha256(b).hexdigest()})
paths=sorted(set(p for p in paths if not p.startswith('tests/') or p in ['tests/cli_t16b.rs','tests/t16b_cli_check.py','tests/REPORT.md','tests/coverage-matrix.md','tests/.gitattributes'])); snapshot_paths={s['path'] for s in snapshots}
def group(p):
 if p.startswith('docs/spec/') or p in ['docs/reports/T16-B-design.md','.opencode/team/REQUIREMENTS.md']:return 1
 if p in ['src/ast.rs','src/error.rs','src/lib.rs','src/loader.rs','src/parser.rs','src/modules.rs'] or p.startswith('docs/reports/T16-B-core'):return 2
 if p in ['src/cli.rs','src/evaluator.rs'] or p.startswith('docs/tooling/') or p in ['docs/reports/T16-B-tooling.md','docs/reports/T16-B-runtime.md']:return 3
 if p in ['tests/cli_t16b.rs','tests/t16b_cli_check.py','tests/REPORT.md','tests/coverage-matrix.md','tests/.gitattributes'] or p.startswith(('docs/reports/T16-B-independent','docs/reports/T16-B-blackbox')):return 4
 if p.startswith('.opencode/team/') or p in ['README.md','docs/reports/delivery-checklist.md','docs/reports/T16-C-design-draft.md','docs/reports/T16-C-design.md','docs/reports/T16-C-interface-proposal.md','docs/reports/T17-design-draft.md','docs/reports/T17-resource-error-proposal.md'] or p.startswith('docs/reports/T16-B-release-evidence/'):return 5
 raise ValueError(p)
msgs={1:'docs(spec): define module graph checking',2:'feat(modules): build static module dependency graph',3:'feat(cli): check module graphs without execution',4:'test(modules): verify non-executing graph checks',5:'chore(checkpoint): preserve B candidate and plans'}
r={'date':'2026-10-30','baseline':'738e198','status':'B graph-check candidate; C documents planning only; production snapshots from B ZIP; R304 OPEN; no tag/Git','excluded_C_nonplanning_paths':excluded_C_nonplanning_paths,'snapshots':snapshots,'groups':[{'sequence':n,'message':msgs[n],'direct_stage_paths':[p for p in paths if group(p)==n and p not in snapshot_paths],'staged_snapshot_paths':[p for p in paths if group(p)==n and p in snapshot_paths]} for n in range(1,6)]}
(root/'docs/reports/T16-B-release-evidence/commit-groups.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'paths':len(paths),'groups':[(g['sequence'],len(g['direct_stage_paths']),len(g['staged_snapshot_paths'])) for g in r['groups']],'snapshots':len(snapshots)}))