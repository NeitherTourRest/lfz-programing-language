extern crate lfz;
use lfz::{evaluator as e, lexer, parser, value::{Value, StructObj}};
fn main() {
 let p=parser::parse(&lexer::lex("jsonStringify(argv())",0).unwrap()).unwrap();
 let inspect=|v:Value, want:&str| { if let Value::Str(s)=v {assert_eq!(s.as_ref(),want)} else {panic!("expected string")} };
 inspect(e::eval_module_traced_with_args(&p,&["alpha".into()]).result.unwrap(),"[\"alpha\"]");
 inspect(e::eval_module_traced_with_args(&p,&["中文".into(),"".into()]).result.unwrap(),"[\"中文\",\"\"]");
 inspect(e::eval_module(&p).unwrap(),"[]");
 inspect(e::eval_module_traced(&p).result.unwrap(),"[]");
 inspect(e::eval_module_traced_on_thread_with_args(&p,&["last".into()]).result.unwrap(),"[\"last\"]");
 inspect(e::eval_module_traced_on_thread(&p).result.unwrap(),"[]");
 let mut s=StructObj::from_fields(vec![("k".into(),Value::Int(1)),("k".into(),Value::Int(9))]);
 assert!(matches!(s.get("k"),Some(Value::Int(1))));
 s.set("k",Value::Int(2)); assert!(matches!(s.get("k"),Some(Value::Int(2))));
 assert_eq!(s.raw_fields().len(),2); assert!(matches!(s.raw_fields()[1].1,Value::Int(9)));
 for i in 0..2000 {s.set(&format!("x{i}"),Value::Int(i));}
 let clone=s.clone();
 for i in 0..2000 { assert!(matches!(clone.get(&format!("x{i}")),Some(Value::Int(v)) if *v==i));}
 assert!(!clone.contains("missing"));
 println!("PASS six independent evaluator contexts and StructObj first-match/index/clone");
}