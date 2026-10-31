//! Pure validation and owned HTTP boundary; platform code never touches LFZ values.
use std::{sync::{Arc,atomic::AtomicBool},time::Instant};
#[derive(Debug)] pub(crate) struct HttpRequest{pub(crate) method:String,pub(crate) url:String,pub(crate) headers:Vec<(String,String)>,pub(crate) body:Vec<u8>,pub(crate) timeout_ms:u64,pub(crate) max_response_bytes:usize}
#[derive(Clone,Debug)] pub(crate) struct HttpControl{pub(crate) deadline:Option<Instant>,pub(crate) cancel:Option<Arc<AtomicBool>>}
#[derive(Debug)] pub(crate) struct HttpResponse{pub(crate) status:u16,pub(crate) headers:Vec<(String,String)>,pub(crate) body:Vec<u8>}
#[derive(Debug)] pub(crate) enum HttpFailure{Cancelled,Deadline,Timeout,ResponseLimit,Unsupported,Io{operation:String,message:String}}
#[derive(Debug)] pub(crate) struct HttpReport{pub(crate) result:Result<HttpResponse,HttpFailure>,pub(crate) diagnostics:Vec<String>}
pub(crate) fn valid_method(s:&str)->bool{matches!(s,"GET"|"HEAD"|"POST"|"PUT"|"PATCH"|"DELETE"|"OPTIONS")}
pub(crate) fn valid_headers(h:&[(String,String)])->bool{
 if h.len()>256{return false;}let mut total=0usize;
 for (name,value) in h{if name.is_empty()||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b"!#$%&'*+-.^_`|~".contains(&b))||!value.bytes().all(|b|b==b'\t'||(32..=126).contains(&b)){return false;}
 if matches!(name.to_ascii_lowercase().as_str(),"host"|"content-length"|"transfer-encoding"|"connection"|"proxy-authorization"|"proxy-connection"|"accept-encoding"|"upgrade"){return false;}
 let Some(n)=total.checked_add(name.len()).and_then(|n|n.checked_add(value.len())).and_then(|n|n.checked_add(4))else{return false;};total=n;if total>65536{return false;}}
 true
}
pub(crate) fn valid_url(s:&str)->bool{
 if s.len()>16384||!s.is_ascii()||s.bytes().any(|b|b<=32||b==127||b==b'\\'||b==b'#'){return false;}
 let Some((scheme,rest))=s.split_once("://")else{return false;};if !scheme.eq_ignore_ascii_case("http")&&!scheme.eq_ignore_ascii_case("https"){return false;}
 let bytes=rest.as_bytes();let mut i=0;while i<bytes.len(){if bytes[i]==b'%' {if i+2>=bytes.len()||!bytes[i+1].is_ascii_hexdigit()||!bytes[i+2].is_ascii_hexdigit(){return false;}i+=3;}else{i+=1;}}
 let authority=rest.split(['/', '?']).next().unwrap_or("");if authority.is_empty()||authority.contains('@'){return false;}
 let port=if authority.starts_with('['){let Some(end)=authority.find(']')else{return false;};if authority[1..end].parse::<std::net::Ipv6Addr>().is_err(){return false;}
 let tail=&authority[end+1..];if tail.is_empty(){None}else if let Some(p)=tail.strip_prefix(':'){Some(p)}else{return false;}
 }else{let (host,port)=authority.split_once(':').map_or((authority,None),|(h,p)|(h,Some(p)));if host.is_empty()||host.bytes().any(|b|!b.is_ascii_alphanumeric()&&!b"-._~%!$&'()*+,;=".contains(&b)){return false;}port};
 port.is_none_or(|p|!p.is_empty()&&p.bytes().all(|b|b.is_ascii_digit())&&p.parse::<u16>().is_ok_and(|n|n>0))
}
pub(crate) fn run(r:HttpRequest,c:HttpControl)->HttpReport{
 if !valid_method(&r.method)||!valid_url(&r.url)||!valid_headers(&r.headers)||r.body.len()>16777216||(matches!(r.method.as_str(),"GET"|"HEAD")&&!r.body.is_empty())||!(1..=600000).contains(&r.timeout_ms)||r.max_response_bytes>16777216{return HttpReport{result:Err(HttpFailure::Io{operation:"请求校验".into(),message:"无效的受控 HTTP 请求".into()}),diagnostics:vec![]};}
 #[cfg(windows)]{windows::run(r,c)}
 #[cfg(not(windows))]{let _=(r,c);HttpReport{result:Err(HttpFailure::Unsupported),diagnostics:vec![]}}
}
#[cfg(windows)]mod windows;
