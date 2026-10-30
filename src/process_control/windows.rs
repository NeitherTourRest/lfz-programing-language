//! Win32 handles are single-owner RAII; no shell/PATH lookup or unsupervised fallback.
use super::*;
use std::{ffi::c_void,ptr::{null,null_mut},mem::{size_of,zeroed},sync::atomic::Ordering,time::Duration};
type H=*mut c_void;
#[cfg(test)] std::thread_local! { static FAULT:std::cell::Cell<Option<&'static str>>=const{std::cell::Cell::new(None)}; }
fn injected(op:&str)->bool { #[cfg(test)] {return FAULT.with(|f|f.get().is_some_and(|points|points.split(',').any(|point|point==op)));} #[cfg(not(test))] {let _=op;false} }
fn injection(op:&str)->ProcessFailure{ProcessFailure::Io{operation:op.into(),message:"故障注入".into()}}

#[repr(C)]struct Sa{size:u32,descriptor:H,inherit:i32}
#[repr(C)]struct Startup{cb:u32,reserved:*mut u16,desktop:*mut u16,title:*mut u16,x:u32,y:u32,xsize:u32,ysize:u32,xchars:u32,ychars:u32,fill:u32,flags:u32,show:u16,reserved_size:u16,reserved_ptr:*mut u8,input:H,output:H,error:H}
#[repr(C)]struct StartupEx{info:Startup,attributes:H}
#[repr(C)]struct Pi{process:H,thread:H,pid:u32,tid:u32}
#[repr(C)]struct Basic{process_time:i64,job_time:i64,flags:u32,min_ws:usize,max_ws:usize,active:u32,affinity:usize,priority:u32,scheduling:u32}
#[repr(C)]struct Extended{basic:Basic,io:[u64;6],process_memory:usize,job_memory:usize,peak_process:usize,peak_job:usize}
#[repr(C)]struct Accounting{times:[i64;4],faults:u32,total:u32,active:u32,terminated:u32}
#[link(name="kernel32")]unsafe extern "system"{
 fn CloseHandle(h:H)->i32; fn CreatePipe(read:*mut H,write:*mut H,sa:*const Sa,size:u32)->i32;fn SetHandleInformation(h:H,mask:u32,flags:u32)->i32;
 fn CreateFileW(name:*const u16,access:u32,share:u32,sa:*const Sa,creation:u32,flags:u32,template:H)->H;
 fn CreateJobObjectW(sa:*const Sa,name:*const u16)->H;fn SetInformationJobObject(job:H,class:i32,data:H,len:u32)->i32;fn QueryInformationJobObject(job:H,class:i32,data:H,len:u32,returned:*mut u32)->i32;
 fn InitializeProcThreadAttributeList(list:H,count:u32,flags:u32,size:*mut usize)->i32;fn UpdateProcThreadAttribute(list:H,flags:u32,attribute:usize,value:H,size:usize,previous:H,returned:*mut usize)->i32;fn DeleteProcThreadAttributeList(list:H);
 fn CreateProcessW(app:*const u16,command:*mut u16,psa:*const Sa,tsa:*const Sa,inherit:i32,flags:u32,env:H,cwd:*const u16,start:*const Startup,pi:*mut Pi)->i32;
 fn AssignProcessToJobObject(job:H,process:H)->i32;fn ResumeThread(thread:H)->u32;fn TerminateJobObject(job:H,code:u32)->i32;fn TerminateProcess(process:H,code:u32)->i32;
 fn WaitForSingleObject(h:H,ms:u32)->u32;fn GetExitCodeProcess(h:H,code:*mut u32)->i32;
 fn PeekNamedPipe(pipe:H,buffer:H,size:u32,read:*mut u32,available:*mut u32,left:*mut u32)->i32;fn ReadFile(file:H,buffer:H,size:u32,read:*mut u32,overlap:H)->i32;
}
// Each run owns the registry. Explicit close_all runs after execute on every Result
// path; Drop only covers unwinding/unexpected abandonment. Slots are consumed before
// closing, so neither a reported OS failure nor an injected failure can double-close.
struct Handle(H,usize);
#[derive(Default)]struct Handles{slots:std::cell::RefCell<Vec<H>>,diagnostics:std::cell::RefCell<Vec<String>>}
impl Handles{
 fn add(&self,h:H)->Handle{let mut slots=self.slots.borrow_mut();let id=slots.len();slots.push(h);Handle(h,id)}
 fn close(&self,h:Handle){let raw=std::mem::replace(&mut self.slots.borrow_mut()[h.1],null_mut());if raw.is_null()||raw as isize == -1{return;}
  let closed=unsafe{CloseHandle(raw)};let error=if closed==0{Some(std::io::Error::last_os_error())}else{None};
  // Injection reports a failure but still performs the real close, never leaks a
  // valid kernel handle and never retries a potentially consumed handle value.
  if let Some(error)=error{self.diagnostics.borrow_mut().push(format!("关闭句柄失败：{error}"));}
  else if injected("close"){self.diagnostics.borrow_mut().push("关闭句柄失败：故障注入，实际句柄已关闭".into());}
 }
 fn close_all(&self){let len=self.slots.borrow().len();for id in (0..len).rev(){let raw=self.slots.borrow()[id];self.close(Handle(raw,id));}}
}
impl Drop for Handles{fn drop(&mut self){self.close_all();}}
struct Attributes{_storage:Vec<usize>,list:H}impl Drop for Attributes{fn drop(&mut self){unsafe{DeleteProcThreadAttributeList(self.list);}}}
fn os(op:&str)->ProcessFailure{ProcessFailure::Io{operation:op.into(),message:std::io::Error::last_os_error().to_string()}}
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
fn quote(s:&str)->String{let mut o=String::from("\"");let mut slashes=0;for c in s.chars(){if c=='\\'{slashes+=1;continue;}if c=='"'{o.push_str(&"\\".repeat(slashes*2+1));}else{o.push_str(&"\\".repeat(slashes));}slashes=0;o.push(c);}o.push_str(&"\\".repeat(slashes*2));o.push('"');o}
fn poll(c:&ProcessControl,local:Option<Instant>)->Option<ProcessFailure>{if c.cancel.as_ref().is_some_and(|t|t.load(Ordering::Acquire)){Some(ProcessFailure::Cancelled)}else if c.deadline.is_some_and(|d|Instant::now()>=d){Some(ProcessFailure::Deadline)}else if local.is_some_and(|d|Instant::now()>=d){Some(ProcessFailure::Timeout)}else{None}}
fn pipe(sa:&Sa,handles:&Handles)->Result<(Handle,Handle),ProcessFailure>{if injected("pipe"){return Err(injection("管道创建"));}unsafe{let(mut r,mut w)=(null_mut(),null_mut());if CreatePipe(&mut r,&mut w,sa,8192)==0{return Err(os("管道创建"));}let pair=(handles.add(r),handles.add(w));if SetHandleInformation(r,1,0)==0{return Err(os("句柄继承设置"));}Ok(pair)}}
fn attrs(handles:&mut[H])->Result<Attributes,ProcessFailure>{unsafe{let mut bytes=0;InitializeProcThreadAttributeList(null_mut(),1,0,&mut bytes);if bytes==0{return Err(os("继承列表准备"));}let mut storage=vec![0usize;bytes.div_ceil(size_of::<usize>())];let list=storage.as_mut_ptr() as H;if InitializeProcThreadAttributeList(list,1,0,&mut bytes)==0{return Err(os("继承列表创建"));}let a=Attributes{_storage:storage,list};if UpdateProcThreadAttribute(list,0,0x20002,handles.as_mut_ptr() as H,std::mem::size_of_val(handles),null_mut(),null_mut())==0{return Err(os("继承列表设置"));}Ok(a)}}
// Peek + a single owner reader means ReadFile never asks for unavailable bytes.
fn drain(pipe:H,out:&mut Vec<u8>,other_len:usize,max:usize)->Result<(bool,bool),ProcessFailure>{if injected("read"){return Err(injection("管道读取"));}unsafe{let mut available=0;if PeekNamedPipe(pipe,null_mut(),0,null_mut(),&mut available,null_mut())==0{let e=std::io::Error::last_os_error();if e.raw_os_error()==Some(109){return Ok((true,false));}return Err(ProcessFailure::Io{operation:"管道读取".into(),message:e.to_string()});}if available==0{return Ok((false,false));}let mut scratch=[0u8;8192];let mut read=0;let n=available.min(8192);if ReadFile(pipe,scratch.as_mut_ptr() as H,n,&mut read,null_mut())==0{return Err(os("管道读取"));}let allowed=max.saturating_sub(out.len()+other_len);let keep=(read as usize).min(allowed);out.reserve_exact(keep.saturating_sub(out.capacity()-out.len()));out.extend_from_slice(&scratch[..keep]);if read as usize>allowed{return Err(ProcessFailure::OutputLimit);}Ok((false,read>0))}}
fn cleanup(job:H,process:H,assigned:bool,diagnostics:&mut Vec<String>){unsafe{
 if injected("kill"){diagnostics.push("终止Job失败：故障注入，执行可靠回退终止".into());}
 if injected("cleanup_wait"){diagnostics.push("等待失败：故障注入，继续实际回收确认".into());}
 if assigned {if TerminateJobObject(job,1)==0{diagnostics.push(format!("终止Job失败：{}",std::io::Error::last_os_error()));}}else if TerminateProcess(process,1)==0{diagnostics.push(format!("终止挂起进程失败：{}",std::io::Error::last_os_error()));}
 let until=Instant::now()+Duration::from_secs(2);
 loop {let direct=WaitForSingleObject(process,0);if direct==0xffffffff{diagnostics.push(format!("等待直系失败：{}",std::io::Error::last_os_error()));break;}
 let mut a:Accounting=zeroed();let empty=if assigned{if QueryInformationJobObject(job,1,&mut a as *mut _ as H,size_of::<Accounting>()as u32,null_mut())==0{diagnostics.push(format!("查询Job回收失败：{}",std::io::Error::last_os_error()));break;}a.active==0}else{true};
 if direct==0&&empty{break;}if Instant::now()>=until{diagnostics.push("等待子进程回收失败：回收等待超时".into());break;}std::thread::sleep(Duration::from_millis(5));
 }
}}
pub(super)fn run(r:ProcessRequest,c:ProcessControl)->ProcessReport{
 let handles=Handles::default();let mut diagnostics=vec![];
 let mut result=unsafe{execute(r,c,&mut diagnostics,&handles)};
 handles.close_all();diagnostics.extend(std::mem::take(&mut *handles.diagnostics.borrow_mut()));
 if result.is_ok()&&!diagnostics.is_empty(){result=Err(ProcessFailure::Io{operation:"回收".into(),message:"子进程资源回收未确认".into()});}
 ProcessReport{result,diagnostics}
}
unsafe fn execute(r:ProcessRequest,c:ProcessControl,diagnostics:&mut Vec<String>,handles:&Handles)->Result<ProcessOutput,ProcessFailure>{
 if let Some(e)=poll(&c,None){return Err(e);}
 if injected("job"){return Err(injection("Job创建"));}
 let job=handles.add(CreateJobObjectW(null(),null()));if job.0.is_null(){return Err(os("Job创建"));}
 let mut limit:Extended=zeroed();limit.basic.flags=0x2000;if SetInformationJobObject(job.0,9,&mut limit as *mut _ as H,size_of::<Extended>()as u32)==0{return Err(os("Job设置"));}
 let sa=Sa{size:size_of::<Sa>()as u32,descriptor:null_mut(),inherit:1};let(or,ow)=pipe(&sa,handles)?;let(er,ew)=pipe(&sa,handles)?;
 let input=handles.add(CreateFileW(wide("NUL").as_ptr(),0x80000000,3,&sa,3,0x80,null_mut()));if input.0 as isize == -1{return Err(os("标准输入创建"));}
 let mut inherited=[input.0,ow.0,ew.0];let attributes=attrs(&mut inherited)?;
 let mut startup:StartupEx=zeroed();startup.info.cb=size_of::<StartupEx>()as u32;startup.info.flags=0x100;startup.info.input=input.0;startup.info.output=ow.0;startup.info.error=ew.0;startup.attributes=attributes.list;
 if let Some(e)=poll(&c,None){return Err(e);}
 let app=wide(&r.argv[0]);let mut command=wide(&r.argv.iter().map(|s|quote(s)).collect::<Vec<_>>().join(" "));let mut pi:Pi=zeroed();
 if CreateProcessW(app.as_ptr(),command.as_mut_ptr(),null(),null(),1,0x00080000|4|0x08000000,null_mut(),null(),&startup.info,&mut pi)==0{return Err(os("启动"));}
 let process=handles.add(pi.process);let thread=handles.add(pi.thread);let local=match Instant::now().checked_add(Duration::from_millis(r.timeout_ms)){Some(d)=>Some(d),None=>{cleanup(job.0,process.0,false,diagnostics);return Err(ProcessFailure::Io{operation:"截止时间建立".into(),message:"平台时间范围溢出".into()});}};handles.close(ow);handles.close(ew);handles.close(input);drop(attributes);
 let mut assigned=false;let initial=if let Some(e)=poll(&c,local){Err(e)}else if injected("assign"){Err(injection("Job纳管"))}else if AssignProcessToJobObject(job.0,process.0)==0{Err(os("Job纳管"))}else{assigned=true;if let Some(e)=poll(&c,local){Err(e)}else if ResumeThread(thread.0)==u32::MAX{Err(os("恢复"))}else{Ok(())}};
 if let Err(e)=initial{cleanup(job.0,process.0,assigned,diagnostics);return Err(e);}handles.close(thread);
 let(mut stdout,mut stderr)=(Vec::new(),Vec::new());let(mut oe,mut ee)=(false,false);let mut next_error=None;let mut terminated=false;let mut exit_code=0;let mut turn=false;
 let result=loop{
  if let Some(e)=poll(&c,local){break Err(e);}if let Some(e)=next_error.take(){break Err(e);}
  if injected("wait"){break Err(injection("等待"));}
  let state=WaitForSingleObject(process.0,0);if state==0xffffffff{break Err(os("等待"));}
  if state==0&&!terminated{if GetExitCodeProcess(process.0,&mut exit_code)==0{break Err(os("退出码读取"));}cleanup(job.0,process.0,true,diagnostics);terminated=true;}
  let mut progress=false;
  for n in 0..2 {let which=(n==0)^turn;let read=if which {drain(or.0,&mut stdout,stderr.len(),r.max_output_bytes)}else{drain(er.0,&mut stderr,stdout.len(),r.max_output_bytes)};
   match read{Ok((e,p))=>{if which{oe=e;}else{ee=e;}progress|=p;},Err(e)=>{if matches!(e,ProcessFailure::OutputLimit)||next_error.is_none(){next_error=Some(e);}}}
   if matches!(next_error,Some(ProcessFailure::OutputLimit)){break;}
  }turn=!turn;
  if next_error.is_some(){continue;}
  if terminated&&oe&&ee{if let Some(e)=poll(&c,local){break Err(e);}break Ok(ProcessOutput{stdout,stderr,exit_code:Some(i64::from(exit_code)),signal:None});}
  if !progress{std::thread::sleep(Duration::from_millis(5));}
 };
 if !terminated{cleanup(job.0,process.0,true,diagnostics);}
 if diagnostics.is_empty(){result}else{match result{Err(e@ProcessFailure::Cancelled)|Err(e@ProcessFailure::Deadline)|Err(e@ProcessFailure::Timeout)|Err(e@ProcessFailure::OutputLimit)=>Err(e),_=>Err(ProcessFailure::Io{operation:"回收".into(),message:"子进程资源回收未确认".into()})}}
}
#[cfg(test)]mod tests{use super::*;#[test]fn crt_quoting(){assert_eq!(quote(""),"\"\"");assert_eq!(quote("a b"),"\"a b\"");assert_eq!(quote("a\\"),"\"a\\\\\"");assert_eq!(quote("a\"b"),"\"a\\\"b\"");}}

#[cfg(test)]mod process_tests {
 use super::*;use std::sync::{OnceLock,Arc,atomic::AtomicBool};
 fn fixture()->String {static EXE:OnceLock<String>=OnceLock::new();EXE.get_or_init(||{
  let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let dir=root.join("Temp/T18-A-tooling/unit-fixture");std::fs::create_dir_all(&dir).unwrap();let src=dir.join("fixture.rs");let exe=dir.join("fixture.exe");
  std::fs::write(&src,include_str!("fixture.rs")).unwrap();let output=std::process::Command::new(root.join("Temp/toolchain/cargo/bin/rustc.exe")).args(["--edition=2021","-C","opt-level=1","-C","debuginfo=0"]).arg(&src).arg("-o").arg(&exe).env("TEMP",&dir).env("TMP",&dir).output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));exe.to_str().unwrap().into()
 }).clone()}
 pub(super) fn request(args:&[&str],max:usize,ms:u64)->ProcessRequest{let mut argv=vec![fixture()];argv.extend(args.iter().map(|s|s.to_string()));ProcessRequest{argv,timeout_ms:ms,max_output_bytes:max}}
 pub(super) fn control()->ProcessControl{ProcessControl{deadline:None,cancel:None}}
 #[test]fn args_empty_space_unicode_slash_and_quote(){let values=["","a b","中文🙂","a\\","a\"b"];let mut a=vec!["echo"];a.extend(values);let r=run(request(&a,1000,5000),control());assert!(r.diagnostics.is_empty());let o=r.result.unwrap();let expected=values.iter().map(|s|format!("{}:{s}\n",s.len())).collect::<String>();assert_eq!(o.stdout,expected.as_bytes());}
 #[test]fn fair_dual_pipes_exact_budget_and_zero(){let r=run(request(&["dual","131072"],262144,5000),control());assert!(r.diagnostics.is_empty(),"{:?}",r);let o=r.result.unwrap();assert_eq!((o.stdout.len(),o.stderr.len()),(131072,131072));assert!(matches!(run(request(&["dual","131072"],262143,5000),control()).result,Err(ProcessFailure::OutputLimit)));assert!(run(request(&["none"],0,5000),control()).result.is_ok());assert!(matches!(run(request(&["echo","x"],0,5000),control()).result,Err(ProcessFailure::OutputLimit)));}
 #[test]fn raw_bytes_nonzero_exit_stdin_and_timeout(){assert_eq!(run(request(&["invalid"],1,5000),control()).result.unwrap().stdout,vec![255]);assert_eq!(run(request(&["utf8"],6,5000),control()).result.unwrap().stdout,b"\xef\xbb\xbfa\r\n");assert_eq!(run(request(&["exit","-1"],0,5000),control()).result.unwrap().exit_code,Some(4294967295));assert_eq!(run(request(&["stdin"],1,5000),control()).result.unwrap().stdout,b"0");assert!(matches!(run(request(&["sleep","10000"],0,20),control()).result,Err(ProcessFailure::Timeout)));}
 #[test]fn cancel_live_native_process_after_fixture_started(){let marker=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Temp/T18-A-tooling/live-cancel-marker.txt");let _=std::fs::remove_file(&marker);let r=request(&["mark_sleep",marker.to_str().unwrap()],0,5000);let token=Arc::new(AtomicBool::new(false));let t=token.clone();let m=marker.clone();let host=std::thread::spawn(move ||{let end=Instant::now()+Duration::from_secs(3);while !m.exists()&&Instant::now()<end{std::thread::sleep(Duration::from_millis(2));}let started=m.exists();t.store(true,Ordering::Release);started});let result=run(r,ProcessControl{cancel:Some(token),deadline:None});assert!(host.join().unwrap(),"fixture did not reach execution before cancellation");assert!(matches!(result.result,Err(ProcessFailure::Cancelled)));assert!(result.diagnostics.is_empty());}
 #[test]fn pre_cancel_and_deadline_priority(){let c=ProcessControl{cancel:Some(Arc::new(AtomicBool::new(true))),deadline:Some(Instant::now())};assert!(matches!(run(request(&["none"],0,1000),c).result,Err(ProcessFailure::Cancelled)));let c=ProcessControl{cancel:None,deadline:Some(Instant::now())};assert!(matches!(run(request(&["none"],0,1000),c).result,Err(ProcessFailure::Deadline)));}
 #[test]fn descendant_holding_pipe_is_reaped_before_success(){let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let marker=root.join(format!("Temp/T18-A-tooling/descendant-{}.txt",std::process::id()));let _=std::fs::remove_file(&marker);let r=run(request(&["parent",marker.to_str().unwrap()],128,5000),control());assert!(r.diagnostics.is_empty(),"{:?}",r);let out=r.result.unwrap();let pid:u32=String::from_utf8(out.stdout).unwrap().parse().unwrap();unsafe{#[link(name="kernel32")]unsafe extern "system"{fn OpenProcess(access:u32,inherit:i32,pid:u32)->H;}let h=OpenProcess(0x00100000,0,pid);if !h.is_null(){let handles=Handles::default();let h=handles.add(h);assert_eq!(WaitForSingleObject(h.0,0),0);handles.close(h);assert!(handles.diagnostics.borrow().is_empty());}else{assert_eq!(std::io::Error::last_os_error().raw_os_error(),Some(87));}}}
 #[test]fn injected_job_assign_pipe_read_wait_and_cleanup(){for point in ["job","assign","pipe","read","wait","kill","cleanup_wait"]{let marker=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("Temp/T18-A-tooling/forbidden-{point}.txt"));let _=std::fs::remove_file(&marker);FAULT.with(|f|f.set(Some(point)));let r=run(request(&["mark",marker.to_str().unwrap()],0,5000),control());FAULT.with(|f|f.set(None));assert!(matches!(r.result,Err(ProcessFailure::Io{..})),"{point}: {r:?}");if matches!(point,"job"|"assign"|"pipe"){assert!(!marker.exists(),"fixture ran before admission: {point}");}if matches!(point,"kill"|"cleanup_wait"){assert!(!r.diagnostics.is_empty());}}}
}

#[cfg(test)]mod close_tests{
 use super::*;
 #[test]fn close_diagnostics_reverse_order_and_no_double_close(){unsafe{
  let handles=Handles::default();let first=handles.add(CreateJobObjectW(null(),null()));let second=handles.add(CreateJobObjectW(null(),null()));
  assert!(!first.0.is_null()&&!second.0.is_null());let first_raw=first.0;let second_raw=second.0;
  FAULT.with(|f|f.set(Some("close")));handles.close(second);handles.close_all();handles.close_all();FAULT.with(|f|f.set(None));
  assert_eq!(handles.diagnostics.borrow().len(),2);assert!(handles.slots.borrow().iter().all(|h|h.is_null()));
  // OS confirms both real handles closed, including the injected early-close.
  assert_eq!(WaitForSingleObject(first_raw,0),u32::MAX);assert_eq!(WaitForSingleObject(second_raw,0),u32::MAX);
 }}
 #[test]fn close_failure_on_success_pre_spawn_error_and_locked_cancellation(){
  // A normal completion cannot hide a close diagnostic.
  FAULT.with(|f|f.set(Some("close")));let success=run(super::process_tests::request(&["none"],0,5000),super::process_tests::control());FAULT.with(|f|f.set(None));
  assert!(matches!(success.result,Err(ProcessFailure::Io{..})));assert!(!success.diagnostics.is_empty());
  // Failing CreateProcess still closes every earlier pipe/Job/input handle.
  let mut missing=super::process_tests::request(&["none"],0,5000);missing.argv[0].push_str(".missing.exe");
  FAULT.with(|f|f.set(Some("close")));let failed=run(missing,super::process_tests::control());FAULT.with(|f|f.set(None));
  assert!(matches!(failed.result,Err(ProcessFailure::Io{ref operation,..})if operation=="启动"));assert_eq!(failed.diagnostics.len(),6);
  // Admission errors close owned handles in the same explicit path; the
  // suspended fixture cannot execute when Assign fails.
  for (point,operation,count) in [("assign,close","Job纳管",8),("pipe,close","管道创建",1)]{
   let r=super::process_tests::request(&["none"],0,5000);FAULT.with(|f|f.set(Some(point)));let report=run(r,super::process_tests::control());FAULT.with(|f|f.set(None));
   assert!(matches!(report.result,Err(ProcessFailure::Io{operation:ref op,..})if op==operation));assert_eq!(report.diagnostics.len(),count,"{point}: {report:?}");
  }
  // Cancellation is triggered only after the real fixture reached execution.
  let marker=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Temp/T18-A-tooling/close-cancel-marker.txt");let _=std::fs::remove_file(&marker);
  let r=super::process_tests::request(&["mark_sleep",marker.to_str().unwrap()],0,5000);
  let token=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));let t=token.clone();let m=marker.clone();
  let host=std::thread::spawn(move ||{let until=Instant::now()+Duration::from_secs(3);while !m.exists()&&Instant::now()<until{std::thread::sleep(Duration::from_millis(2));}let started=m.exists();t.store(true,Ordering::Release);started});
  FAULT.with(|f|f.set(Some("close")));let ended=run(r,ProcessControl{deadline:None,cancel:Some(token)});FAULT.with(|f|f.set(None));
  assert!(host.join().unwrap());assert!(matches!(ended.result,Err(ProcessFailure::Cancelled)));assert!(!ended.diagnostics.is_empty());
 }
}
