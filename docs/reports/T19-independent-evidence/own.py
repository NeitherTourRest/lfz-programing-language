import ctypes,json,subprocess,time,os,runpy
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T19-independent-evidence';W=R/'Temp/T19-verifier/own';W.mkdir(parents=True,exist_ok=True);X=R/'target/release/lfz.exe';exclusive=runpy.run_path(str(E/'public47.py'))['exclusive'];rows=[];q=lambda p:json.dumps(str(p).replace('\\','/'),ensure_ascii=False)
def run(name,data,cap,callback,check):
 f=W/(name+'.txt');f.write_bytes(data);src='var seen=[]\nlet r=forEachLine('+callback+','+str(cap)+','+q(f)+')\n'+check;p=subprocess.run([str(X),'--json','-e',src],capture_output=True,timeout=8);row=dict(name=name,src=src,exit=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode());rows.append(row);assert p.returncode==0,row;row['passed']=True
run('utf8_cross',b'x'*8191+'🙂\n'.encode(),8195,'(s,n)=>{seen=push(len(s),seen)\ntrue}','assert(seen==[8192] && r.lines==1)')
run('bom_cr_eof','\ufeffa\r\nb\r'.encode(),5,'(s,n)=>{seen=push(s,seen)\ntrue}','assert(seen==["\ufeffa","b\\r"] && !r.stopped)')
run('false_poison',b'ok\n'+b'\xff'*100,2,'(s,n)=>false','assert(r.lines==1 && r.stopped)')
run('json_attempts',b'bad\n1\n\n2\n',20,'(s,n)=>{seen=push(attempt(()=>jsonParse(s)).ok,seen)\ntrue}','assert(seen==[false,true,false,true] && r.lines==4)')
for kind,data,cap,cb,expected in [('eof',b'x\n',2,'(s,n)=>true','assert(r.ok)'),('utf8',b'\xff\n',2,'(s,n)=>true','assert(!r.ok && r.error.message=="第 1 行不是合法 UTF-8")'),('toolong',b'abc\r\n',3,'(s,n)=>true','assert(!r.ok && r.error.message=="第 1 行超过逐行字节上限 3")'),('outerattempt',b'x\n',2,'(s,n)=>1/0','assert(!r.ok)')]:
 f=W/(kind+'.txt');f.write_bytes(data);mark=W/(kind+'.returned');gate=W/(kind+'.gate');gate.write_text('wait');mark.unlink(missing_ok=True);src='let r=attempt(()=>forEachLine('+cb+','+str(cap)+','+q(f)+'))\n'+expected+'\nwriteFile('+q(mark)+',"returned")\nwhile readFile('+q(gate)+')!="go" {}';p=subprocess.Popen([str(X),'--json','--timeout-ms','5000','-e',src],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 try:
  end=time.monotonic()+3
  while not mark.exists() and p.poll() is None and time.monotonic()<end:time.sleep(.005)
  assert mark.exists() and p.poll() is None,kind;probe=exclusive(f);assert probe['ok'],(kind,probe);gate.write_text('go');so,se=p.communicate(timeout=5);assert p.returncode==0,(kind,so,se);rows.append(dict(name='samePID_closed_'+kind,pid=p.pid,alive_when_probed=True,probe=probe,stdout=so.decode(),passed=True))
 finally:
  if p.poll() is None:p.kill();p.communicate()
(E/'own.json').write_text(json.dumps(dict(total=len(rows),passed=len(rows),rows=rows),ensure_ascii=False,indent=2),encoding='utf8');print(len(rows),'independent file/samePID witnesses PASS')
