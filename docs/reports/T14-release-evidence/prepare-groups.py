import subprocess,json,re
from pathlib import Path
root=Path.cwd()
def git(*a):return subprocess.run(['git',*a],check=True,capture_output=True).stdout.decode().split('\0')
paths=sorted(set(filter(None,git('diff','--name-only','-z')+git('ls-files','--others','--exclude-standard','-z')))|{'docs/reports/T14-release-evidence/commit-groups.json'})
protected={'.opencode/team/DECISIONS.md':r'(?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)', '.opencode/team/PROJECT_STATE.md':r'(?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n', '.opencode/team/agents/team-lead/STATUS.md':r'(?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n'}
counts={p:len(re.findall(pat,(root/p).read_text(encoding='utf-8'))) for p,pat in protected.items()}
assert all(n==1 for n in counts.values()),counts
assert not [p for p in paths if p.startswith(('docs/validation/agent-benchmark-v2/','docs/reports/agent-benchmark-v2/v2-20261024-r01/'))]
def group(p):
 if p in ['src/builtins.rs','src/evaluator.rs'] or p.startswith(('benchmarks/t13/','docs/reports/T13')):return 1
 if p.startswith('docs/spec/') or p=='docs/reports/T14-check-design.md':return 2
 if p in ['src/cli.rs','src/main.rs','tests/cli_t14.rs'] or p.startswith('docs/tooling/') or p=='docs/reports/T14-tooling.md':return 3
 if p.startswith('tests/') or p.startswith(('docs/reports/T14-blackbox','docs/reports/T14-independent')):return 4
 if p.startswith(('docs/guide/','.opencode/skills/','docs/reports/T14-guide','docs/reports/T14-skill')):return 5
 if p.startswith('.opencode/team/') or p in ['README.md','docs/reports/delivery-checklist.md'] or p.startswith('docs/reports/T14-release-evidence/'):return 6
 raise ValueError(p)
names={1:'test(runtime): preserve rejected optimization evidence',2:'docs(spec): define non-executing check command',3:'feat(cli): add front-end check command',4:'test(cli): verify check has no execution effects',5:'docs(guide): teach check before execution',6:'chore(checkpoint): record T14 candidate delivery'}
r={'date':'2026-10-29','status':'preparation only; final T14 public and independent verification pending; R304 OPEN; no Git writes','protected_user_match_counts':counts,'refresh_note':'active reviewer/role records may add paths; refresh before stage, preserve original user28/1/1 worktree additions','groups':[{'sequence':n,'message':names[n],'direct_stage_paths':[p for p in paths if group(p)==n and p not in protected],'staged_snapshot_paths':[p for p in paths if group(p)==n and p in protected]}for n in range(1,7)]}
(root/'docs/reports/T14-release-evidence/commit-groups.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'paths':len(paths),'groups':[(x['sequence'],len(x['direct_stage_paths']),len(x['staged_snapshot_paths']))for x in r['groups']],'protected':counts},ensure_ascii=False))