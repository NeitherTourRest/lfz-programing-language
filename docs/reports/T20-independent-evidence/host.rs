extern crate lfz;
use lfz::evaluator::*;
use std::time::{Instant,Duration};
fn main(){on_eval_stack(||{
 let deadline=Instant::now()+Duration::from_millis(100);
 let mut s=ReplSession::new(ReplConfig{context:EvalContext{limits:EvalLimits{deadline:Some(deadline),..Default::default()},..Default::default()},timeout_ms:Some(5000),..Default::default()}).unwrap();
 assert!(s.eval_cell("1").unwrap().unwrap().result.is_ok());
 std::thread::sleep(Duration::from_millis(150));
 for _ in 0..2 {let r=s.eval_cell("1").unwrap().unwrap();assert!(matches!(r.result.unwrap_err().as_ref(),lfz::error::LzError::ExecutionLimit{reason:lfz::error::ExecutionLimitReason::Deadline,..}));s.reset();}
 println!("absolute deadline retained across cells/reset, earlier than relative 5000ms: PASS");
 let mut s=ReplSession::new(Default::default()).unwrap();
 s.eval_cell("fn old()=>9").unwrap().unwrap().result.unwrap();
 for n in 0..1000{assert_eq!(s.eval_cell(&n.to_string()).unwrap().unwrap().result.unwrap().as_int(),Some(n));}
 let fail=s.eval_cell("missing").unwrap().unwrap();assert_eq!(fail.error_source().unwrap().display_path,"<repl:1002>");
 s.reset();let fail=s.eval_cell("old()").unwrap().unwrap();assert_eq!(fail.error_source().unwrap().display_path,"<repl:1>");assert_eq!(fail.result.unwrap_err().class_name(),"NameError");
 println!("1000 real cells, reset numbering and definition removal: PASS");
});}
