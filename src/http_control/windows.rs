//! Async WinHTTP. Only the calling thread issues APIs; callbacks publish one bounded completion.
use super::*;
use std::{ffi::c_void,ptr::{null,null_mut},sync::{Mutex,Condvar,atomic::{AtomicUsize,Ordering}},time::Duration};
type H=*mut c_void;
#[cfg(test)]std::thread_local!{static CLOSE_FAULT:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};static OBSERVED:std::cell::RefCell<Option<(String,Vec<u16>)>>=const{std::cell::RefCell::new(None)};}
#[link(name="winhttp")]unsafe extern "system"{
 fn WinHttpOpen(agent:*const u16,access:u32,proxy:*const u16,bypass:*const u16,flags:u32)->H;
 fn WinHttpConnect(session:H,host:*const u16,port:u16,reserved:u32)->H;
 fn WinHttpOpenRequest(connect:H,verb:*const u16,path:*const u16,version:*const u16,referer:*const u16,accept:*const *const u16,flags:u32)->H;
 fn WinHttpSetStatusCallback(handle:H,callback:Option<unsafe extern "system" fn(H,usize,u32,*mut c_void,u32)>,flags:u32,reserved:usize)->usize;
 fn WinHttpSetOption(handle:H,option:u32,buffer:*const c_void,len:u32)->i32;
 fn WinHttpSetTimeouts(handle:H,resolve:i32,connect:i32,send:i32,receive:i32)->i32;
 fn WinHttpSendRequest(handle:H,headers:*const u16,len:u32,body:*const c_void,body_len:u32,total:u32,context:usize)->i32;
 fn WinHttpReceiveResponse(handle:H,reserved:*mut c_void)->i32;
 fn WinHttpQueryHeaders(handle:H,info:u32,name:*const u16,buffer:*mut c_void,len:*mut u32,index:*mut u32)->i32;
 fn WinHttpReadData(handle:H,buffer:*mut c_void,len:u32,read:*mut u32)->i32;
 fn WinHttpCloseHandle(handle:H)->i32;
}
#[derive(Default)]struct State{event:Option<(u32,u32)>,closing:bool}
struct Context{state:Mutex<State>,wake:Condvar,active:AtomicUsize}
impl Context{fn new()->Self{Self{state:Mutex::new(State::default()),wake:Condvar::new(),active:AtomicUsize::new(0)}}}
#[repr(C)]struct AsyncError{operation:usize,error:u32}
unsafe extern "system" fn callback(_:H,context:usize,status:u32,info:*mut c_void,len:u32){
 if context==0{return;}let c=&*(context as *const Context);c.active.fetch_add(1,Ordering::AcqRel);
 // No user code or fallible allocation is executed across this FFI boundary.
 {let mut s=c.state.lock().unwrap_or_else(|e|e.into_inner());if status==0x800{s.closing=true;}else if matches!(status,0x400000|0x20000|0x80000|0x200000){let value=if status==0x200000&&!info.is_null()&&len as usize>=std::mem::size_of::<AsyncError>(){(*(info as *const AsyncError)).error}else{len};s.event=Some((status,value));}c.active.fetch_sub(1,Ordering::AcqRel);c.wake.notify_all();}
}
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
fn io(operation:&str)->HttpFailure{HttpFailure::Io{operation:operation.into(),message:std::io::Error::last_os_error().to_string()}}
fn protocol(s:&str)->HttpFailure{HttpFailure::Io{operation:"协议".into(),message:s.into()}}
fn poll(c:&HttpControl,local:Instant)->Option<HttpFailure>{if c.cancel.as_ref().is_some_and(|t|t.load(Ordering::Acquire)){Some(HttpFailure::Cancelled)}else if c.deadline.is_some_and(|d|Instant::now()>=d){Some(HttpFailure::Deadline)}else if Instant::now()>=local{Some(HttpFailure::Timeout)}else{None}}
fn wait(ctx:&Context,c:&HttpControl,local:Instant,wanted:u32)->Result<u32,HttpFailure>{let mut s=ctx.state.lock().unwrap_or_else(|e|e.into_inner());loop{if let Some(e)=poll(c,local){return Err(e);}if let Some((status,n))=s.event.take(){if status==0x200000{return Err(HttpFailure::Io{operation:"异步请求".into(),message:format!("WinHTTP error {n}")});}if status==wanted{return Ok(n);}return Err(protocol("意外完成状态"));}s=ctx.wake.wait_timeout(s,Duration::from_millis(5)).unwrap_or_else(|e|e.into_inner()).0;}}
struct Handles{session:H,connect:H,request:H,registered:bool}
impl Handles{
 fn close(&mut self,ctx:&Context,diag:&mut Vec<String>){unsafe{
  if !self.request.is_null(){let h=std::mem::replace(&mut self.request,null_mut());if WinHttpCloseHandle(h)==0{diag.push(format!("关闭HTTP请求失败：{}",std::io::Error::last_os_error()));}#[cfg(test)]if CLOSE_FAULT.with(|f|f.get()){diag.push("关闭HTTP请求失败：故障注入，实际关闭仍执行".into());}
   if self.registered{let mut s=ctx.state.lock().unwrap_or_else(|e|e.into_inner());while !s.closing||ctx.active.load(Ordering::Acquire)!=0{s=ctx.wake.wait_timeout(s,Duration::from_millis(5)).unwrap_or_else(|e|e.into_inner()).0;}}}
  for h in [&mut self.connect,&mut self.session]{if !h.is_null(){let raw=std::mem::replace(h,null_mut());if WinHttpCloseHandle(raw)==0{diag.push(format!("关闭HTTP父句柄失败：{}",std::io::Error::last_os_error()));}}}
 }}
}
pub(super)fn run(r:HttpRequest,c:HttpControl)->HttpReport{
 let Some(local)=Instant::now().checked_add(Duration::from_millis(r.timeout_ms))else{return HttpReport{result:Err(protocol("截止时间溢出")),diagnostics:vec![]};};
 let ctx=Box::new(Context::new());let mut handles=Handles{session:null_mut(),connect:null_mut(),request:null_mut(),registered:false};let mut diagnostics=vec![];
 let mut scratch=Box::new([0u8;8192]);let execution=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||unsafe{execute(&r,&c,local,&ctx,&mut handles,&mut scratch)}));
 // All referenced send/read buffers and context remain live through the final
 // HANDLE_CLOSING callback and its exit. No worker is detached on cancellation.
 handles.close(&ctx,&mut diagnostics);
 #[cfg(test)]if handles.registered{assert!(ctx.state.lock().unwrap().closing);assert_eq!(ctx.active.load(Ordering::Acquire),0);}
 let mut result=match execution{Ok(result)=>result,Err(panic)=>std::panic::resume_unwind(panic)};
 if result.is_ok()&&!diagnostics.is_empty(){result=Err(HttpFailure::Io{operation:"回收".into(),message:"HTTP资源回收未确认".into()});}
 HttpReport{result,diagnostics}
}
unsafe fn option(h:H,n:u32,v:u32)->Result<(),HttpFailure>{if WinHttpSetOption(h,n,&v as *const _ as *const c_void,4)==0{Err(io("设置选项"))}else{Ok(())}}
unsafe fn execute(r:&HttpRequest,c:&HttpControl,local:Instant,ctx:&Context,h:&mut Handles,scratch:&mut [u8;8192])->Result<HttpResponse,HttpFailure>{
 if let Some(e)=poll(c,local){return Err(e);}
 let (_,rest)=r.url.split_once("://").ok_or_else(||protocol("URL"))?;let split=rest.find(['/', '?']).unwrap_or(rest.len());let authority=&rest[..split];let suffix=&rest[split..];let path=if suffix.is_empty(){"/".into()}else if suffix.starts_with('?'){format!("/{suffix}")}else{suffix.to_string()};let tls=r.url[..5].eq_ignore_ascii_case("https");
 let (host,port)=if authority.starts_with('['){let end=authority.find(']').ok_or_else(||protocol("IPv6"))?;(&authority[1..end],authority[end+1..].strip_prefix(':'))}else{authority.split_once(':').map_or((authority,None),|(a,b)|(a,Some(b)))};let port=port.map(|p|p.parse::<u16>().unwrap()).unwrap_or(if tls{443}else{80});
 h.session=WinHttpOpen(wide("LFZ/1.4").as_ptr(),1,null(),null(),0x10000000);if h.session.is_null(){return Err(io("创建会话"));}
 let ms=r.timeout_ms.min(i32::MAX as u64)as i32;if WinHttpSetTimeouts(h.session,ms,ms,ms,ms)==0{return Err(io("设置超时"));}
 option(h.session,5,1)?; // one connect attempt; no application resend loop
 h.connect=WinHttpConnect(h.session,wide(host).as_ptr(),port,0);if h.connect.is_null(){return Err(io("连接句柄"));}
 h.request=WinHttpOpenRequest(h.connect,wide(&r.method).as_ptr(),wide(&path).as_ptr(),null(),null(),null(),if tls{0x800000}else{0});if h.request.is_null(){return Err(io("创建请求"));}
 option(h.request,63,1|2|4)?; // cookies, redirects, authentication disabled
 option(h.request,88,0)?;option(h.request,91,65536)?;
 // Bind context before registering callbacks, so early cancellation also has a
 // closing notification even if SendRequest never ran.
 let context=ctx as *const Context as usize;
 if WinHttpSetOption(h.request,45,&context as *const _ as *const c_void,std::mem::size_of::<usize>()as u32)==0{return Err(io("设置上下文"));}
 if WinHttpSetStatusCallback(h.request,Some(callback),0xffffffff,0)==usize::MAX{return Err(io("设置回调"));}h.registered=true;
 if let Some(e)=poll(c,local){return Err(e);}
 let headers=r.headers.iter().map(|(k,v)|format!("{k}: {}\r\n",v.trim_matches([' ','\t']))).collect::<String>();let headers=wide(&headers);
 if WinHttpSendRequest(h.request,headers.as_ptr(),(headers.len()-1)as u32,r.body.as_ptr()as *const c_void,r.body.len()as u32,r.body.len()as u32,context)==0{return Err(io("发送"));}wait(ctx,c,local,0x400000)?;
 if let Some(e)=poll(c,local){return Err(e);}if WinHttpReceiveResponse(h.request,null_mut())==0{return Err(io("接收"));}wait(ctx,c,local,0x20000)?;
 if let Some(e)=poll(c,local){return Err(e);}let mut status=0u32;let mut n=4;if WinHttpQueryHeaders(h.request,19|0x20000000,null(),&mut status as *mut _ as *mut c_void,&mut n,null_mut())==0{return Err(io("状态"));}if !(100..=599).contains(&status){return Err(protocol("HTTP状态越界"));}
 let mut bytes=0;WinHttpQueryHeaders(h.request,22,null(),null_mut(),&mut bytes,null_mut());if bytes>65536{return Err(HttpFailure::ResponseLimit);}if bytes<2||bytes%2!=0{return Err(protocol("头长度非法"));}
 let mut raw=vec![0u16;bytes as usize/2];if WinHttpQueryHeaders(h.request,22,null(),raw.as_mut_ptr()as *mut c_void,&mut bytes,null_mut())==0{return Err(io("响应头"));}
 if bytes%2!=0||bytes as usize/2>=raw.len(){return Err(protocol("头长度非法"));}raw.truncate(bytes as usize/2);if raw.contains(&0){return Err(protocol("响应头内嵌NUL"));}let text=String::from_utf16(&raw).map_err(|_|protocol("响应头UTF16非法"))?;
 #[cfg(test)]OBSERVED.with(|slot|*slot.borrow_mut()=Some((text.clone(),raw.clone())));
 if text.len()>r.max_response_bytes{return Err(HttpFailure::ResponseLimit);}let mut headers=vec![];for line in text.split("\r\n").skip(1){if line.is_empty(){continue;}let Some((key,value))=line.split_once(':')else{return Err(protocol("响应头缺冒号"));};if key.is_empty(){return Err(protocol("响应头名为空"));}headers.push((key.to_ascii_lowercase(),value.trim_matches([' ','\t']).into()));if headers.len()>256{return Err(HttpFailure::ResponseLimit);}}
 let mut body=vec![];loop{if let Some(e)=poll(c,local){return Err(e);}if WinHttpReadData(h.request,scratch.as_mut_ptr()as *mut c_void,8192,null_mut())==0{return Err(io("读取"));}let count=wait(ctx,c,local,0x80000)? as usize;if count>8192{return Err(protocol("读取长度非法"));}if count>r.max_response_bytes-text.len()-body.len(){return Err(HttpFailure::ResponseLimit);}if count==0{break;}body.reserve_exact(count);body.extend_from_slice(&scratch[..count]);}
 if let Some(e)=poll(c,local){return Err(e);}Ok(HttpResponse{status:status as u16,headers,body})
}
#[cfg(test)]mod tests{
 use super::*;use std::{io::{Read,Write},net::TcpListener,sync::{Arc,atomic::{AtomicBool,Ordering}},thread};
 struct Server{url:String,seen:Arc<AtomicBool>,stop:Arc<AtomicBool>,worker:Option<thread::JoinHandle<()>>,race:Option<Arc<std::sync::Barrier>>}
 impl Server{fn new(mode:&'static str)->Self{let socket=TcpListener::bind("127.0.0.1:0").unwrap();socket.set_nonblocking(true).unwrap();let url=format!("http://{}/x?y=1",socket.local_addr().unwrap());let seen=Arc::new(AtomicBool::new(false));let stop=Arc::new(AtomicBool::new(false));let (s,t)=(seen.clone(),stop.clone());let race=(mode=="race").then(||Arc::new(std::sync::Barrier::new(2)));let gate=race.clone();let worker=thread::spawn(move||{while !t.load(Ordering::Acquire){match socket.accept(){Ok((mut stream,_))=>{stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();stream.set_write_timeout(Some(Duration::from_millis(100))).unwrap();let mut input=vec![];let mut b=[0;1024];while !t.load(Ordering::Acquire){match stream.read(&mut b){Ok(0)=>break,Ok(n)=>{input.extend_from_slice(&b[..n]);if input.windows(4).any(|w|w==b"\r\n\r\n"){break;}},Err(_)=>{}}}s.store(true,Ordering::Release);if let Some(gate)=&gate{gate.wait();}
 if mode=="slowhead"{while !t.load(Ordering::Acquire){thread::sleep(Duration::from_millis(2));}break;}
 if mode=="drip"{let _=stream.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n");while !t.load(Ordering::Acquire){if stream.write_all(b"1\r\nx\r\n").is_err(){break;}thread::sleep(Duration::from_millis(10));}break;}
 if mode=="witness"{let _=stream.write_all(b"HTTP/1.1 200 Test\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");break;}
 let body=if mode=="invalid"{&b"\xff"[..]}else{&b"abc"[..]};let headers=format!("HTTP/1.1 404 Missing\r\nX-Dup: a\r\nX-Dup: b\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());let _=stream.write_all(headers.as_bytes());if mode=="slowbody"{while !t.load(Ordering::Acquire){thread::sleep(Duration::from_millis(2));}}else{let _=stream.write_all(body);}break;},Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>thread::sleep(Duration::from_millis(2)),Err(e)=>panic!("{e}")}}});Self{url,seen,stop,worker:Some(worker),race}}
 fn request(&self,ms:u64,max:usize)->HttpRequest{HttpRequest{method:"GET".into(),url:self.url.clone(),headers:vec![],body:vec![],timeout_ms:ms,max_response_bytes:max}}}
 impl Drop for Server{fn drop(&mut self){self.stop.store(true,Ordering::Release);self.worker.take().unwrap().join().unwrap();}}
 fn control()->HttpControl{HttpControl{deadline:None,cancel:None}}
 #[test]fn raw_headers_exact_n_duplicate_status_and_utf16_evidence(){let s=Server::new("normal");let response=run(s.request(5000,4096),control());assert!(response.diagnostics.is_empty(),"{response:?}");let response=response.result.unwrap();assert_eq!(response.status,404);assert_eq!(response.body,b"abc");assert_eq!(response.headers.iter().filter(|(n,_)|n=="x-dup").count(),2);drop(s);
 let dir=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Temp/T18-B-tooling");let (text,raw_units)=OBSERVED.with(|slot|slot.borrow_mut().take().unwrap());std::fs::create_dir_all(&dir).unwrap();let raw=text.as_bytes();let units=raw_units.iter().flat_map(|n|n.to_le_bytes()).collect::<Vec<_>>();std::fs::write(dir.join("raw-header-observed.txt"),raw).unwrap();std::fs::write(dir.join("raw-header-observed.u16le"),&units).unwrap();let n=raw.len()+3;std::fs::write(dir.join("raw-header-count.txt"),format!("native_u16_bytes_excluding_NUL={}\nheader_utf8_bytes={}\nbody_bytes=3\nN={}\n",units.len(),raw.len(),n)).unwrap();let exact=Server::new("normal");assert!(run(exact.request(5000,n),control()).result.is_ok());drop(exact);let less=Server::new("normal");assert!(matches!(run(less.request(5000,n-1),control()).result,Err(HttpFailure::ResponseLimit)));}
 #[test]fn public_exact_n_native_witness(){let mut server=Server::new("witness");let authority=server.url.split("/x?").next().unwrap().to_string();server.url=format!("{authority}/");let report=run(server.request(5000,4096),control());assert_eq!(report.result.unwrap().body,b"ok");let (text,units)=OBSERVED.with(|slot|slot.borrow_mut().take().unwrap());let dir=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Temp/T18-B-tooling");std::fs::write(dir.join("public-raw-headers.txt"),text.as_bytes()).unwrap();std::fs::write(dir.join("public-raw-headers.u16le"),units.iter().flat_map(|n|n.to_le_bytes()).collect::<Vec<_>>()).unwrap();let n=text.len()+2;drop(server);let server=Server::new("witness");assert!(run(server.request(5000,n),control()).result.is_ok());drop(server);let server=Server::new("witness");assert!(matches!(run(server.request(5000,n-1),control()).result,Err(HttpFailure::ResponseLimit)));}
 #[test]fn total_timeout_slow_headers_body_and_drip(){for mode in ["slowhead","slowbody","drip"]{let server=Server::new(mode);let report=run(server.request(80,4096),control());assert!(server.seen.load(Ordering::Acquire));assert!(matches!(report.result,Err(HttpFailure::Timeout)),"{mode}: {report:?}");assert!(report.diagnostics.is_empty(),"{report:?}");}}
 #[test]fn live_cancel_and_global_deadline(){let server=Server::new("slowhead");let seen=server.seen.clone();let token=Arc::new(AtomicBool::new(false));let t=token.clone();let worker=thread::spawn(move||{let end=Instant::now()+Duration::from_secs(3);while !seen.load(Ordering::Acquire)&&Instant::now()<end{thread::sleep(Duration::from_millis(2));}assert!(seen.load(Ordering::Acquire));t.store(true,Ordering::Release);});let report=run(server.request(5000,4096),HttpControl{deadline:None,cancel:Some(token)});worker.join().unwrap();assert!(matches!(report.result,Err(HttpFailure::Cancelled)),"{report:?}");assert!(report.diagnostics.is_empty());drop(server);let server=Server::new("slowbody");assert!(matches!(run(server.request(5000,4096),HttpControl{deadline:Some(Instant::now()+Duration::from_millis(80)),cancel:None}).result,Err(HttpFailure::Deadline)));}
 #[test]fn close_diagnostics_never_hide_success_or_global_reason(){let server=Server::new("normal");CLOSE_FAULT.with(|f|f.set(true));let report=run(server.request(5000,4096),control());CLOSE_FAULT.with(|f|f.set(false));assert!(matches!(report.result,Err(HttpFailure::Io{..})));assert_eq!(report.diagnostics.len(),1);drop(server);let server=Server::new("slowbody");CLOSE_FAULT.with(|f|f.set(true));let report=run(server.request(5000,4096),HttpControl{deadline:Some(Instant::now()+Duration::from_millis(80)),cancel:None});CLOSE_FAULT.with(|f|f.set(false));assert!(matches!(report.result,Err(HttpFailure::Deadline)));assert_eq!(report.diagnostics.len(),1);}
 #[test]fn synchronized_real_cancellation_vs_response_completion(){for _ in 0..40{let server=Server::new("race");let seen=server.seen.clone();let gate=server.race.as_ref().unwrap().clone();let token=Arc::new(AtomicBool::new(false));let t=token.clone();let worker=thread::spawn(move||{let until=Instant::now()+Duration::from_secs(3);while !seen.load(Ordering::Acquire)&&Instant::now()<until{thread::yield_now();}assert!(seen.load(Ordering::Acquire));gate.wait();t.store(true,Ordering::Release);});let report=run(server.request(5000,4096),HttpControl{deadline:None,cancel:Some(token)});worker.join().unwrap();assert!(matches!(report.result,Ok(_)|Err(HttpFailure::Cancelled)),"{report:?}");assert!(report.diagnostics.is_empty());}}
 #[test]fn cancellation_completion_races_repeat_without_callbacks_left(){for _ in 0..20{let server=Server::new("normal");let report=run(server.request(5000,4096),control());assert!(report.result.is_ok()&&report.diagnostics.is_empty(),"{report:?}");}}
}
