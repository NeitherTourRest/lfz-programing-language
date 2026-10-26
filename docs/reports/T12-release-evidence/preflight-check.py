import hashlib,json,re,subprocess,zipfile
from pathlib import Path
root=Path.cwd()
def git(*args): return subprocess.run(['git',*args],capture_output=True,check=True).stdout
def sha(b): return hashlib.sha256(b).hexdigest()
tracked=git('diff','--name-only','-z').decode().split('\0')
new=git('ls-files','--others','--exclude-standard','-z').decode().split('\0')
paths=sorted(set(p for p in tracked+new if p))
extras=['docs/reports/T12-release-evidence/checkpoint-preflight.json','docs/reports/T12-release-evidence/commit-groups.json']
paths=sorted(set(paths+extras))
old=json.loads(git('show','HEAD:tests/cases.json'))['cases']
now=json.loads((root/'tests/cases.json').read_text(encoding='utf-8'))['cases']
oldtests=[p for p in tracked if p and p.startswith('tests/') and p not in ('tests/REPORT.md','tests/cases.json','tests/coverage-matrix.md')]
frozen=[p for p in tracked if p and (p.startswith('docs/validation/agent-benchmark-v2/') or p.startswith('docs/reports/agent-benchmark-v2/v2-20261024-r01/'))]
protect={'.opencode/team/DECISIONS.md':r'(?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)', '.opencode/team/PROJECT_STATE.md':r'(?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n', '.opencode/team/agents/team-lead/STATUS.md':r'(?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n'}
protected={p:len(re.findall(pat,(root/p).read_text(encoding='utf-8'))) for p,pat in protect.items()}
secret_patterns=[r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----',r'gh[pousr]_[A-Za-z0-9]{30,}',r'github_pat_[A-Za-z0-9_]{30,}',r'AKIA[A-Z0-9]{16}',r'sk-(?:proj-)?[A-Za-z0-9_-]{30,}']
hits=[]
for p in paths:
 f=root/p
 if not f.is_file(): continue
 try:s=f.read_bytes().decode('utf-8')
 except UnicodeDecodeError:continue
 if any(re.search(pat,s) for pat in secret_patterns):hits.append(p)
zip_audit=[]
for p in paths:
 if not p.endswith('.zip') or not (root/p).is_file():continue
 with zipfile.ZipFile(root/p) as z:
  names=z.namelist()
  unsafe=[n for n in names if n.startswith(('/', '\\')) or '..' in Path(n).parts or re.match(r'^[A-Za-z]:',n)]
  credentials=[]
  for n in names:
   if n.endswith('/'):continue
   try:s=z.read(n).decode('utf-8')
   except UnicodeDecodeError:continue
   if any(re.search(pat,s) for pat in secret_patterns):credentials.append(n)
  assert not unsafe and not credentials,(p,unsafe,credentials)
  zip_audit.append({'path':p,'members':len(names),'unsafe_path_entries':unsafe,'credential_pattern_hit_members':credentials})
def group(p):
 if p.startswith('docs/spec/') or p=='.opencode/team/REQUIREMENTS.md' or p.startswith('docs/reports/T12-design'):return 1
 if p in ['src/cli.rs','src/main.rs','tests/cli_t12.rs'] or p.startswith('docs/tooling/'):return 3
 if p.startswith('docs/reports/T12-runtime-evidence/perf-diagnostic/') or p.startswith('docs/reports/T12-runtime-evidence/startup-candidates/'):return 7
 if p.startswith('src/') or p=='docs/reports/T12-runtime-evidence.md' or p.startswith('docs/reports/T12-runtime-evidence/'):return 2
 if p.startswith('tests/') or p.startswith('docs/reports/T12-blackbox') or p.startswith('docs/reports/T12-independent') or p=='docs/reports/T12-cargo-full.txt':return 4
 if p.startswith('app/') or p.startswith('docs/reports/T12-app'):return 5
 if p.startswith('docs/guide/') or p.startswith('.opencode/skills/lfz-programming/') or p.startswith('docs/reports/T12-guide') or p.startswith('docs/reports/T12-skill'):return 6
 if p.startswith('benchmarks/t12/') or p.startswith('docs/reports/T12-performance') or p.startswith('docs/reports/T12-runtime-performance') or p.startswith('docs/reports/T12-runtime-startup'):return 7
 if p.startswith('.opencode/team/') or p in ['README.md','Cargo.toml','Cargo.lock','docs/reports/delivery-checklist.md','docs/reports/T12-release-plan.md'] or p.startswith('docs/reports/T12-release-evidence/'):return 8
 raise ValueError('Unclassified path '+p)
messages={1:'docs(spec): define agent data workflow contracts',2:'feat(runtime): add json and argument contexts',3:'feat(cli): add inline and stdin script execution',4:'test(agent): cover json and script entry points',5:'feat(app): add configurable json data workflow',6:'docs(guide): teach agent json script workflows',7:'perf(runtime): record bounded workflow benchmarks',8:'chore(checkpoint): save 1.4.0 candidate evidence'}
groups=[]
for n in range(1,9):
 items=[p for p in paths if group(p)==n]
 groups.append({'sequence':n,'message':messages[n],'direct_stage_paths':[p for p in items if p not in protect],'staged_snapshot_paths':[p for p in items if p in protect]})
assert not oldtests and not frozen and now[:len(old)]==old and all(v==1 for v in protected.values()) and not hits
outgroups=root/'docs/reports/T12-release-evidence/commit-groups.json'
outgroups.write_text(json.dumps({'status':'candidate checkpoint only; no formal v1.4.0 tag; refresh after final verifier artifacts','groups':groups},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
files=[{'path':p,'bytes':(root/p).stat().st_size,'sha256':sha((root/p).read_bytes())} for p in paths if (root/p).is_file() and p!=extras[0]]
r={'date':'2026-10-25','status':'candidate checkpoint preflight snapshot; final functional PASS bound 47ED; R304 OPEN; no formal release tag; no Git writes','changed_paths':paths,'files_excluding_this_manifest':files,'existing_lfz_or_rust_tests_modified':oldtests,'frozen_package_or_historical_report_modified':frozen,'cases_original_count':len(old),'cases_now_count':len(now),'cases_original_decoded_prefix_unchanged':now[:len(old)]==old,'protected_user_block_match_counts':protected,'credential_pattern_hit_paths':hits,'zip_audit':zip_audit,'scan_limit':'pattern-based UTF8 text only, not proof of all possible sensitive material','old_manifest_sha256':sha((root/'docs/validation/agent-benchmark-v2/manifest.json').read_bytes()),'archived_zip_sha256':sha((root/'docs/reports/agent-benchmark-v2/v2-20261024-r01/raw-inputs-submissions.zip').read_bytes()),'binary_sha256':sha((root/'dist/lfz.exe').read_bytes()),'binary_bytes':(root/'dist/lfz.exe').stat().st_size}
(root/'docs/reports/T12-release-evidence/checkpoint-preflight.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'paths':len(paths),'groups':[(g['sequence'],len(g['direct_stage_paths']),len(g['staged_snapshot_paths'])) for g in groups],'original_cases':len(old),'new_cases':len(now)-len(old),'original_tests_modified':oldtests,'frozen_modified':frozen,'user_blocks':protected,'credential_hits':hits},ensure_ascii=False))