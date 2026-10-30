//! 纯owned受控进程边界；Windows实现，其他平台明确Unsupported。
use std::sync::{Arc,atomic::AtomicBool};
use std::time::Instant;
#[derive(Debug)] pub(crate) struct ProcessRequest {pub(crate) argv:Vec<String>,pub(crate) timeout_ms:u64,pub(crate) max_output_bytes:usize}
#[derive(Clone,Debug)] pub(crate) struct ProcessControl {pub(crate) deadline:Option<Instant>,pub(crate) cancel:Option<Arc<AtomicBool>>}
#[derive(Debug)] pub(crate) struct ProcessOutput {pub(crate) stdout:Vec<u8>,pub(crate) stderr:Vec<u8>,pub(crate) exit_code:Option<i64>,pub(crate) signal:Option<i64>}
#[derive(Debug)] pub(crate) enum ProcessFailure {Cancelled,Deadline,Timeout,OutputLimit,Unsupported,Io{operation:String,message:String}}
#[derive(Debug)] pub(crate) struct ProcessReport {pub(crate) result:Result<ProcessOutput,ProcessFailure>,pub(crate) diagnostics:Vec<String>}
pub(crate) fn executable_valid(path:&str)->bool {
 if path.contains('\0'){return false;}
 if !cfg!(windows){return std::path::Path::new(path).is_absolute();}
 if !path.to_ascii_lowercase().ends_with(".exe"){return false;}
 let b=path.as_bytes();
 let drive=b.len()>3&&b[0].is_ascii_alphabetic()&&b[1]==b':'&&matches!(b[2],b'\\'|b'/');
 let unc=path.starts_with("\\\\")&&!path.starts_with("\\\\?\\")&&!path.starts_with("\\\\.\\")&&path[2..].split('\\').filter(|s|!s.is_empty()).count()>=3;
 drive||unc
}
pub(crate) fn run(request:ProcessRequest,control:ProcessControl)->ProcessReport {
 if request.argv.is_empty()||request.argv.iter().any(|s|s.contains('\0'))||!executable_valid(&request.argv[0])||!(1..=600000).contains(&request.timeout_ms)||request.max_output_bytes>16777216 {
  return ProcessReport{result:Err(ProcessFailure::Io{operation:"请求校验".into(),message:"无效的受控子进程请求".into()}),diagnostics:vec![]};
 }
 #[cfg(windows)] {windows::run(request,control)}
 #[cfg(not(windows))] {let _=(request,control);ProcessReport{result:Err(ProcessFailure::Unsupported),diagnostics:vec![]}}
}
#[cfg(windows)] mod windows;
