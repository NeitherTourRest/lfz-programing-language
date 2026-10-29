use std::{path::PathBuf,rc::Rc};
use lfz::{evaluator::{eval_module_traced,TracedRun},source::*,value::Value};
fn run(s:&str)->TracedRun {let ts=lfz::lexer::lex(s,0).unwrap();let p=lfz::parser::parse(&ts).unwrap();eval_module_traced(&p)}
fn record(path:&str,loaded:Option<lfz::loader::Loaded>)->SourceRecord{SourceRecord{identity:Some(PathBuf::from(path)),display_path:path.into(),loaded}}
fn main(){
 let mut map=SourceMap::new();assert!(map.is_empty());assert!(map.get(SourceId::ENTRY).is_none());
 let a=map.insert(record("<root>",Some(lfz::loader::Loaded{text:"raw\r\ntext".into(),line_base:7}))).unwrap();
 let b=map.insert(record("<missing>",None)).unwrap();assert_eq!(a,SourceId::ENTRY);assert_eq!(b,SourceId(1));assert_eq!(map.len(),2);
 assert_eq!(map.get(a).unwrap().loaded.as_ref().unwrap().text,"raw\r\ntext");assert_eq!(map.get(a).unwrap().loaded.as_ref().unwrap().line_base,7);assert!(map.get(b).unwrap().loaded.is_none());assert!(map.get(SourceId(u32::MAX)).is_none());
 let map=Rc::new(map);let mut good=run("42");assert!(good.frame_source(0).is_none());assert!(good.error_source().is_none());
 let n=good.frames.len();assert!(n>0);good.attach_origins(map.clone(),vec![a;n],None).unwrap();assert_eq!(good.frame_source(0).unwrap().display_path,"<root>");
 assert_eq!(good.attach_origins(map.clone(),vec![],Some(SourceId(99))),Err(SourceMetadataError::FrameCount{expected:n,actual:0}));
 assert_eq!(good.attach_origins(map.clone(),vec![SourceId(99);n],Some(a)),Err(SourceMetadataError::UnknownSource{id:SourceId(99)}));
 assert_eq!(good.attach_origins(map.clone(),vec![a;n],Some(SourceId(99))),Err(SourceMetadataError::UnexpectedErrorSource));assert_eq!(good.frame_source(0).unwrap().display_path,"<root>");
 let mut bad=run("fn f()=>1/0\nf()");assert!(bad.result.is_err());assert!(bad.error_source().is_none());let k=bad.frames.len();assert!(k>=2);
 let mut ids=vec![a;k];ids[k-1]=b;bad.attach_origins(map.clone(),ids,None).unwrap_err();
 assert_eq!(bad.attach_origins(map.clone(),vec![a;k],None),Err(SourceMetadataError::MissingErrorSource));
 assert_eq!(bad.attach_origins(map.clone(),vec![a;k],Some(SourceId(99))),Err(SourceMetadataError::UnknownSource{id:SourceId(99)}));
 let mut ids=vec![a;k];ids[k-1]=b;bad.attach_origins(map.clone(),ids,Some(b)).unwrap();assert_eq!(bad.frame_source(k-1).unwrap().display_path,"<missing>");assert!(bad.error_source().unwrap().loaded.is_none());
 assert_eq!(bad.attach_origins(map.clone(),vec![],None),Err(SourceMetadataError::FrameCount{expected:k,actual:0}));assert_eq!(bad.error_source().unwrap().display_path,"<missing>");
 assert!(bad.frame_source(k).is_none());assert!(bad.frame_source(usize::MAX).is_none());bad.frames.pop();assert!(bad.frame_source(0).is_none());bad.result=Ok(Value::Nil);assert!(bad.error_source().is_none());
 let f=run("()=>1");if let Ok(Value::Func(cl))=&f.result{assert_eq!(cl.user.as_ref().unwrap().home_module,ModuleId::ENTRY)}else{panic!("expected function")}
 println!("T16-A independent API witnesses PASS");
}
