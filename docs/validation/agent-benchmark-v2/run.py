"""固定八题功能验收器（stdlib）；自动结果不等于 Agent 盲测成绩。"""
import argparse, collections, csv, hashlib, io, json, re, subprocess, sys, tempfile, time
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
IDS=["B%02d"%i for i in range(1,9)]
SCHEMA="LFZ-AGENT-BENCH-v2.1"
def sha(b): return hashlib.sha256(b).hexdigest()
def write_json(p,v): p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
def inside(p):
    p=Path(p).resolve()
    if not p.is_relative_to(ROOT): raise ValueError("Path outside project: "+str(p))
    return p
def literal(v):
    if v is None: return "nil"
    if isinstance(v,bool): return "true" if v else "false"
    if isinstance(v,int): return str(v)
    if isinstance(v,str):
        if any(ord(c)<32 and c not in "\n\r\t" for c in v):
            parts=re.split(r"([\x00-\x08\x0b\x0c\x0e-\x1f])",v)
            return "("+" + ".join("chr(%d)"%ord(x) if len(x)==1 and ord(x)<32 and x not in "\n\r\t" else literal(x) for x in parts if x)+")"
        s=v.replace("\\","\\\\").replace('"','\\"').replace("$","\\$")
        s=s.replace("\n","\\n").replace("\r","\\r").replace("\t","\\t")
        if any(ord(c)<32 for c in s): raise ValueError("Unsupported LFZ control character")
        return '"'+s+'"'
    if isinstance(v,list): return "["+",".join(literal(x) for x in v)+"]"
    if isinstance(v,dict): return "{"+",".join(literal(k)+":"+literal(x) for k,x in v.items())+"}"
    raise TypeError(type(v).__name__)
def expr_ref(s):
    ts=re.findall(r"\d+|[()+*/-]",s); at=0
    def atom():
        nonlocal at
        t=ts[at]; at+=1
        if t in ("+","-"):
            n=atom(); return n if t=="+" else -n
        if t=="(":
            n=add()
            assert ts[at]==")"
            at+=1; return n
        return int(t)
    def mul():
        nonlocal at
        n=atom()
        while at<len(ts) and ts[at] in ("*","/"):
            op=ts[at]; at+=1; r=atom(); n=n*r if op=="*" else n//r
        return n
    def add():
        nonlocal at
        n=mul()
        while at<len(ts) and ts[at] in ("+","-"):
            op=ts[at]; at+=1; r=mul(); n=n+r if op=="+" else n-r
        return n
    n=add(); assert at==len(ts); return n
def bfs_ref(grid,start,goal):
    start,goal=tuple(start),tuple(goal)
    queue=collections.deque([start]); prev={start:None}
    while queue:
        p=queue.popleft()
        if p==goal:
            out=[]
            while p is not None: out.append(list(p)); p=prev[p]
            return out[::-1]
        for dr,dc in [(-1,0),(0,1),(1,0),(0,-1)]:
            v=(p[0]+dr,p[1]+dc)
            if 0<=v[0]<len(grid) and 0<=v[1]<len(grid[0]) and grid[v[0]][v[1]]=="." and v not in prev:
                prev[v]=p; queue.append(v)
    return []
def dp_ref(items,cap):
    rows=[[0]*(cap+1)]
    for w,v in items:
        old=rows[-1]; rows.append([max(old[c],old[c-w]+v) if c>=w else old[c] for c in range(cap+1)])
    c=cap; out=[]
    for i in range(len(items),0,-1):
        if rows[i][c]!=rows[i-1][c]: out.append(i-1); c-=items[i-1][0]
    return {"value":rows[-1][cap],"indices":out[::-1]}
def reference(task,args):
    if task=="B01": return expr_ref(args[0])
    if task=="B02":
        return [r if r else [""] for r in csv.reader(io.StringIO(args[0],newline=""),strict=True)] if args[0] else []
    if task=="B03":
        tab={}; out=[]
        for op in args[0]:
            if op[0]=="set": tab[op[1]]=op[2]; out.append(None)
            elif op[0]=="get": out.append(tab.get(op[1]))
            elif op[0]=="has": out.append(op[1] in tab)
            elif op[0]=="delete": out.append(op[1] in tab); tab.pop(op[1],None)
            elif op[0]=="size": out.append(len(tab))
            else: raise ValueError(op)
        return out
    if task=="B04": return bfs_ref(*args)
    if task=="B05": return dp_ref(*args)
    if task=="B06":
        tab={}; rejected=0
        for op in args[0]:
            if op[0]=="deposit": tab[op[1]]=tab.get(op[1],0)+op[2]
            else:
                _,src,dst,v=op; tab.setdefault(src,0); tab.setdefault(dst,0)
                if tab[src]<v: rejected+=1
                else: tab[src]-=v; tab[dst]+=v
        return {"balances":tab,"rejected":rejected}
    if task=="B07":
        words=[w.lower() for w in re.findall("[A-Za-z]+",args[0])]; ix=collections.defaultdict(list)
        for i,w in enumerate(words): ix[w].append(i)
        return [[q.lower(),len(ix[q.lower()]),ix[q.lower()]] for q in args[1]]
    if task=="B08":
        nums=[int(s.strip()) for s in args[0].splitlines() if s.strip()]
        total=0; lines=["index,value,running\n"]
        for i,v in enumerate(nums,1): total+=v; lines.append("%d,%d,%d\n"%(i,v,total))
        return {"count":len(nums),"total":total},"".join(lines)
    raise ValueError(task)
def load_cases():
    d=json.loads((HERE/"cases.json").read_text(encoding="utf-8"))
    assert d["schema"]==SCHEMA and set(d["tasks"])==set(IDS)
    return d
def manifest_files(lfz):
    paths=[HERE/x for x in ["TASKS.md","STUDENT_PROMPT.md","cases.json","run.py"]]
    paths += [ROOT/".opencode/skills/lfz-programming"/x for x in ["SKILL.md","README.md","VERIFICATION.md","prompt-template.md"]]
    paths += [inside(lfz)]
    return {p.relative_to(ROOT).as_posix():sha(p.read_bytes()) for p in paths}
def freeze(lfz):
    write_json(HERE/"manifest.json",{"schema":SCHEMA,"files":manifest_files(lfz),
               "note":"Excludes manifest itself. Re-freeze explicitly before a new measurement only."})
def verify_freeze(lfz):
    raw=(HERE/"manifest.json").read_bytes(); d=json.loads(raw)
    if d["schema"]!=SCHEMA or d["files"]!=manifest_files(lfz):
        raise ValueError("Frozen manifest mismatch; review and explicitly re-freeze before measuring")
    return sha(raw)
def execute(lfz,p,timeout=10):
    t=time.monotonic()
    try:
        r=subprocess.run([str(lfz),"run",str(p)],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=timeout)
        code,out,err,timed=r.returncode,r.stdout,r.stderr,False
    except subprocess.TimeoutExpired as e:
        code,out,err,timed=None,e.stdout or b"",e.stderr or b"",True
    return {"exit":code,"stdout":out.decode("utf-8",errors="replace"),"stderr":err.decode("utf-8",errors="replace"),
            "stdout_sha256":sha(out),"stderr_sha256":sha(err),"timed_out":timed,"elapsed_seconds":round(time.monotonic()-t,6)}
def equal_expression(task,name,value):
    if task=="B06":
        # Compare arbitrary account keys as entries; LFZ struct literals only allow literal keys.
        pairs=[[k,v] for k,v in sorted(value["balances"].items())]
        return '(type('+name+') == "struct" && len('+name+') == 2 && '+name+'.rejected == '+literal(value["rejected"])+' && entries('+name+'.balances) == '+literal(pairs)+')'
    return name+" == "+literal(value)
def harness(task,case,args):
    lines=["let __bench_arg_%d = %s"%(i,literal(v)) for i,v in enumerate(args)]
    lines+=["let __bench_result = solve("+",".join("__bench_arg_%d"%i for i in range(len(args)))+")",
            "assert(%s,%s)"%(equal_expression(task,"__bench_result",case["expected"]),literal(case["id"]))]
    if task=="B04":
        lines += ["assert(__bench_arg_%d == %s,%s)"%(i,literal(v),literal("input unchanged %d"%i)) for i,v in enumerate(args)]
    if task!="B08":
        lines.append("let __bench_repeat = solve("+",".join(literal(v) for v in args)+")")
        lines.append("assert(%s,%s)"%(equal_expression(task,"__bench_repeat",case["expected"]),literal("repeat independent solve")))
    return "\n".join(lines)+"\n"
def case_hash(c): return sha(json.dumps(c,ensure_ascii=False,sort_keys=True).encode("utf-8"))
def run_case(task,c,source,lfz,folder):
    folder.mkdir(parents=True,exist_ok=True)
    result={"id":c["id"],"source_sha256":sha(source),"case_sha256":case_hash(c)}
    try:
        lines=source.decode("utf-8").splitlines()
        if not lines or lines[0]!="#42": raise ValueError("First line must be exact #42")
    except (UnicodeError,ValueError) as e:
        result.update(status="INVALID_SOURCE",detail=str(e),attempts=[]); return result
    body="\n".join(lines[1:])+"\n"; attempts=[]
    for i in range(2 if task=="B08" else 1):
        args=c["args"]
        if task=="B08":
            inp=folder/"输入 文件.txt"; outp=folder/"输出 文件.csv"; original=args[0].encode("utf-8")
            if i==0: inp.write_bytes(original)
            outp.write_bytes(b"STALE OUTPUT MUST BE TRUNCATED\n"*20)
            args=[inp.relative_to(ROOT).as_posix(),outp.relative_to(ROOT).as_posix()]
        combined="#42\n"+body+"\n"+harness(task,c,args)
        p=folder/("combined-%d.lfz"%i); p.write_bytes(combined.encode("utf-8"))
        r=execute(lfz,p,timeout=10); r.update(program_sha256=sha(p.read_bytes()),program_path=p.relative_to(ROOT).as_posix(),command=[str(lfz),"run",str(p)])
        if task=="B08":
            actual=outp.read_bytes() if outp.is_file() else None
            after=inp.read_bytes() if inp.is_file() else None; expected=c["expected_output"].encode("utf-8")
            r.update(output_matches=actual==expected,input_unchanged=after==original,
                     expected_output_sha256=sha(expected),output_sha256=sha(actual) if actual is not None else None,
                     expected_input_sha256=sha(original),input_sha256=sha(after) if after is not None else None)
        r["passed"]=r["exit"]==0 and not r["timed_out"] and r.get("output_matches",True) and r.get("input_unchanged",True)
        attempts.append(r)
    result.update(status="PASS" if all(r["passed"] for r in attempts) else "FAIL",attempts=attempts)
    return result
def run_suite(solutions,lfz,work,data):
    tasks=[]
    for task in IDS:
        p=inside(solutions/task/"solution.lfz"); cases=data["tasks"][task]
        if not p.is_file():
            records=[{"id":c["id"],"status":"MISSING","source_sha256":None,"case_sha256":case_hash(c),
                      "exit":None,"stdout":"","stderr":"","attempts":[],"detail":str(p)} for c in cases]
        else:
            source=p.read_bytes(); folder=work/task; folder.mkdir(parents=True,exist_ok=True)
            (folder/"submitted-snapshot.lfz").write_bytes(source)
            records=[run_case(task,c,source,lfz,folder/c["id"]) for c in cases]
        passed=sum(c["status"]=="PASS" for c in records)
        tasks.append({"task":task,"functional_status":"FUNCTIONAL_PASS" if passed==len(cases) else "FAIL",
                      "source_review":"REVIEW_REQUIRED","protocol_review":"REVIEW_REQUIRED","overall_task_status":"NOT_CERTIFIED",
                      "passed_cases":passed,"total_cases":len(cases),"cases":records})
    return {"schema":SCHEMA,"kind":"FUNCTIONAL_EVALUATION_ONLY","denominator":8,
            "warning":"Not an eight-Agent blind-test result. Independent source and protocol review are required.",
            "functional_pass_tasks":sum(t["functional_status"]=="FUNCTIONAL_PASS" for t in tasks),
            "certified_pass_tasks":None,"tasks":tasks}
def validate_data(d):
    out=[]
    for task in IDS:
        cases=d["tasks"][task]; assert len(cases)>=12 and len({c["id"] for c in cases})==len(cases)
        for c in cases:
            a=reference(task,c["args"])
            assert a==((c["expected"],c["expected_output"]) if task=="B08" else c["expected"]),c["id"]
            literal(c["args"])
        out.append({"task":task,"cases":len(cases),"reference_matches":True})
    assert [c["expected"] for c in d["tasks"]["B01"]]==[9,-4,12,3,9,5,10,-4,3,24,0,15,3,9007199254740995,9223372036854775806,3]
    assert reference("B07",["aKB İx ſY",["a","b","akb","i","x","s","y"]])==[["a",1,[0]],["b",1,[1]],["akb",0,[]],["i",0,[]],["x",1,[2]],["s",0,[]],["y",1,[3]]]
    assert expr_ref("2 * (3 + 4) - 5")==9 and expr_ref("-7 / 2")==-4
    assert reference("B02",["\n"])==[[""]]
    assert reference("B02",['"a\r\nb",c\r\n'])==[["a\r\nb","c"]]
    assert bfs_ref(["..",".."],[0,0],[1,1])==[[0,0],[0,1],[1,1]]
    assert dp_ref([[1,4],[1,4]],1)=={"value":4,"indices":[0]}
    assert reference("B06",[[["deposit","a",5],["transfer","a","a",4]]])=={"balances":{"a":5},"rejected":0}
    assert reference("B07",["a中B1a",["A","B"]])==[["a",2,[0,2]],["b",1,[1]]]
    assert reference("B08",["3\n-1\n"])==({"count":2,"total":2},"index,value,running\n1,3,3\n2,-1,2\n")
    for c in d["tasks"]["B03"]:
        ops=c["args"][0]
        assert len(ops)<=128 and len({o[1] for o in ops if len(o)>1})<=32
        assert all(len(o[1])<=128 for o in ops if len(o)>1)
    for c in d["tasks"]["B04"]:
        grid,start,goal=c["args"]
        assert 1<=len(grid)<=20 and 1<=len(grid[0])<=20
        assert all(len(r)==len(grid[0]) and set(r)<=set(".#") for r in grid)
        assert all(0<=p[0]<len(grid) and 0<=p[1]<len(grid[0]) and grid[p[0]][p[1]]=="." for p in [start,goal])
    for c in d["tasks"]["B06"]:
        assert len(c["args"][0])<=128
        for op in c["args"][0]:
            assert all(len(k)<=128 for k in (op[1:2] if op[0]=="deposit" else op[1:3]))
    for c in d["tasks"]["B07"]:
        text,queries=c["args"]
        assert len(text)<=4096 and len(queries)<=32
        assert all(re.fullmatch("[A-Za-z]{1,64}",q) for q in queries)
    for c in d["tasks"]["B01"]: assert len(c["args"][0])<=256
    for c in d["tasks"]["B02"]: assert len(c["args"][0])<=2048
    for c in d["tasks"]["B08"]: assert len(c["args"][0].splitlines())<=128
    for c in d["tasks"]["B05"]:
        items,cap=c["args"]
        assert len(items)<=24 and 0<=cap<=80 and all(w>0 and v>=0 for w,v in items)
        assert sum(v for w,v in items)<=9223372036854775807
        if len(items)<=12:
            candidates=[(sum(v for i,(w,v) in enumerate(items) if m>>i&1),-m,m) for m in range(1<<len(items))
                        if sum(w for i,(w,v) in enumerate(items) if m>>i&1)<=cap]
            _,_,m=max(candidates)
            assert c["expected"]["indices"]==[i for i in range(len(items)) if m>>i&1]
    return out
def self_test(lfz,data):
    work=Path(tempfile.mkdtemp(prefix="benchmark-v2-selftest-",dir=ROOT/"Temp"))
    refs=validate_data(data); c={"id":"ST-expression","args":["2 * (3 + 4) - 5"],"expected":9}
    controls={}
    sources={"positive":b"#42\nfn solve(s) => 9\n","wrong":b"#42\nfn solve(s) => 0\n",
             "missing_function":b"#42\n","nonzero":b'#42\nfail("deliberate self-test")\n',
             "invalid_preamble":b"fn solve(s) => 9\n"}
    for k,source in sources.items(): controls[k]=run_case("B01",c,source,lfz,work/k)
    assert controls["positive"]["status"]=="PASS"
    assert all(controls[k]["status"]=="FAIL" for k in ["wrong","missing_function","nonzero"])
    assert controls["invalid_preamble"]["status"]=="INVALID_SOURCE"
    mutation_case={"id":"ST-grid-mutation","args":[["."],[0,0],[0,0]],"expected":[[0,0]]}
    mutation_source=b'#42\nfn solve(g,s,t) {\ng[0] = "changed"\nreturn [[0,0]]\n}\n'
    grid_mutation=run_case("B04",mutation_case,mutation_source,lfz,work/"grid-mutation")
    assert grid_mutation["status"]=="FAIL"
    state_source=b'#42\nvar count = 8\nfn solve(s) {\ncount = count + 1\nreturn count\n}\n'
    state_leak=run_case("B01",c,state_source,lfz,work/"state-leak")
    assert state_leak["status"]=="FAIL"
    empty=work/"empty-solutions"; empty.mkdir()
    missing=run_suite(empty,lfz,work/"missing-suite",data)
    assert missing["functional_pass_tasks"]==0 and len(missing["tasks"])==8
    assert all(c["status"]=="MISSING" for t in missing["tasks"] for c in t["cases"])
    tricky=chr(36)+'{notInterpolation} " slash\\ 中文🙂\n\r\t\x00\x01'
    p=work/"literal.lfz"
    p.write_bytes(("#42\nassert(len("+literal(tricky)+') == '+str(len(tricky))+',"scalar roundtrip")\n'+
                  "assert("+literal(tricky)+" == "+literal(tricky)+',"literal equality")\n').encode("utf-8"))
    escaped=execute(lfz,p); assert escaped["exit"]==0,escaped
    p=work/"timeout.lfz"; p.write_text("#42\nwhile true {\n}\n",encoding="utf-8")
    timed=execute(lfz,p,timeout=0.2); assert timed["timed_out"] and timed["exit"] is None
    c={"id":"ST-io","args":["3\n-1\n"],"expected":{"count":2,"total":2},
       "expected_output":"index,value,running\n1,3,3\n2,-1,2\n"}
    good=('#42\nfn solve(p,q) {\nwriteFile(q,'+literal(c["expected_output"])+')\nreturn {"count":2,"total":2}\n}\n').encode("utf-8")
    wrong=b'#42\nfn solve(p,q) {\nwriteFile(q,"wrong")\nreturn {"count":2,"total":2}\n}\n'
    changed=good.replace(b'writeFile(q,',b'writeFile(p,"changed")\nwriteFile(q,')
    io_controls={k:run_case("B08",c,s,lfz,work/k) for k,s in [("io_positive",good),("io_wrong",wrong),("io_input_mutation",changed)]}
    assert io_controls["io_positive"]["status"]=="PASS" and len(io_controls["io_positive"]["attempts"])==2
    assert io_controls["io_wrong"]["status"]==io_controls["io_input_mutation"]["status"]=="FAIL"
    # A constant ledger control verifies arbitrary-key comparison, not an LFZ algorithm solution.
    ledger_case={"id":"ST-ledger-nul","args":[[["deposit","a\x00b",8],["transfer","a\x00b","\x01",3]]],
                 "expected":{"balances":{"a\x00b":5,"\x01":3},"rejected":0}}
    ledger_source=('#42\nfn solve(events) {\nlet balances = {}\nbalances['+literal("a\x00b")+'] = 5\nbalances['+
                   literal("\x01")+'] = 3\nreturn {"balances":balances,"rejected":0}\n}\n').encode("utf-8")
    ledger_control=run_case("B06",ledger_case,ledger_source,lfz,work/"ledger-nul")
    assert ledger_control["status"]=="PASS",ledger_control
    assert all("AssertionError" in controls[k]["attempts"][0]["stderr"] for k in ["wrong","nonzero"])
    assert "NameError" in controls["missing_function"]["attempts"][0]["stderr"]
    assert "AssertionError" in grid_mutation["attempts"][0]["stderr"]
    assert "AssertionError" in state_leak["attempts"][0]["stderr"]
    assert all(a["exit"]==0 and not a["output_matches"] for a in io_controls["io_wrong"]["attempts"])
    assert all(a["exit"]==0 and not a["input_unchanged"] for a in io_controls["io_input_mutation"]["attempts"])
    raw_path=work/"literal-roundtrip.txt"
    p=work/"literal-bytes.lfz"
    p.write_bytes(("#42\nwriteFile("+literal(raw_path.relative_to(ROOT).as_posix())+","+literal(tricky)+")\n").encode("utf-8"))
    bytes_control=execute(lfz,p)
    assert bytes_control["exit"]==0 and raw_path.read_bytes()==tricky.encode("utf-8")
    result={"kind":"RUNNER_SELF_TEST_ONLY","ok":True,"references":refs,"controls":controls,"missing_tasks_counted":8,
            "literal_roundtrip":escaped,"timeout_detected":timed,"io_controls":io_controls,
            "grid_mutation_detected":grid_mutation,"state_leak_detected":state_leak,
            "ledger_nul_harness":ledger_control,"literal_bytes_roundtrip":bytes_control,
            "work_directory":work.relative_to(ROOT).as_posix(),"note":"Constant stubs are harness controls, never student scores."}
    write_json(work/"self-test.json",result); return result
def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--solutions",type=Path); ap.add_argument("--lfz",type=Path,required=True)
    ap.add_argument("--output",type=Path); ap.add_argument("--freeze",action="store_true"); ap.add_argument("--self-test",action="store_true")
    o=ap.parse_args(); lfz=inside(o.lfz)
    if not lfz.is_file(): ap.error("--lfz file missing")
    (ROOT/"Temp").mkdir(exist_ok=True); d=load_cases()
    if o.freeze:
        freeze(lfz); print("manifest frozen; excludes itself")
        if not o.self_test and not o.solutions: return 0
    manifest_sha=verify_freeze(lfz); output=inside(o.output) if o.output else None
    if output and (output.suffix.lower()!=".json" or output in [HERE/"cases.json",HERE/"manifest.json"]):
        ap.error("--output must be a report .json path, not cases/manifest")
    if output and output.exists(): ap.error("output exists: choose a new report path to preserve results")
    if o.self_test:
        r=self_test(lfz,d); r["manifest_sha256"]=manifest_sha
        if output: output.parent.mkdir(parents=True,exist_ok=True); write_json(output,r)
        print(json.dumps({"kind":r["kind"],"ok":True,"cases":sum(len(x) for x in d["tasks"].values()),
                          "work_directory":r["work_directory"]})); return 0
    if not o.solutions or not output: ap.error("--solutions and --output are required")
    solutions=inside(o.solutions)
    if not solutions.is_dir(): ap.error("--solutions directory missing; individual missing tasks count MISSING")
    work=Path(tempfile.mkdtemp(prefix="benchmark-v2-run-",dir=ROOT/"Temp"))
    r=run_suite(solutions,lfz,work,d); r.update(manifest_sha256=manifest_sha,interpreter_sha256=sha(lfz.read_bytes()),
                                             work_directory=work.relative_to(ROOT).as_posix())
    output.parent.mkdir(parents=True,exist_ok=True); write_json(output,r)
    print(json.dumps({"functional_pass_tasks":r["functional_pass_tasks"],"denominator":8,"source_review":"REVIEW_REQUIRED",
                      "protocol_review":"REVIEW_REQUIRED","certified_pass_tasks":None}))
    return 0 if r["functional_pass_tasks"]==8 else 1
if __name__=="__main__":
    try: sys.exit(main())
    except (OSError,ValueError,AssertionError,KeyError) as e:
        print("runner error: "+str(e),file=sys.stderr); sys.exit(2)
