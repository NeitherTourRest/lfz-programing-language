from pathlib import Path
p=Path('src/http_control/windows.rs');s=p.read_text(encoding='utf-8')
s=s.replace('handles.close(&ctx,&mut diagnostics);','handles.close(&ctx,&mut diagnostics);\n #[cfg(test)]if handles.registered{assert!(ctx.state.lock().unwrap().closing);assert_eq!(ctx.active.load(Ordering::Acquire),0);}')
s=s.replace('worker:Option<thread::JoinHandle<()>>}', 'worker:Option<thread::JoinHandle<()>>,race:Option<Arc<std::sync::Barrier>>}')
s=s.replace('let (s,t)=(seen.clone(),stop.clone());let worker=', 'let (s,t)=(seen.clone(),stop.clone());let race=(mode=="race").then(||Arc::new(std::sync::Barrier::new(2)));let gate=race.clone();let worker=')
s=s.replace('s.store(true,Ordering::Release);','s.store(true,Ordering::Release);if let Some(gate)=&gate{gate.wait();}')
s=s.replace('Self{url,seen,stop,worker:Some(worker)}','Self{url,seen,stop,worker:Some(worker),race}')
s=s.replace(' #[test]fn cancellation_completion_races_repeat_without_callbacks_left()', ''' #[test]fn synchronized_real_cancellation_vs_response_completion(){for _ in 0..40{let server=Server::new("race");let seen=server.seen.clone();let gate=server.race.as_ref().unwrap().clone();let token=Arc::new(AtomicBool::new(false));let t=token.clone();let worker=thread::spawn(move||{let until=Instant::now()+Duration::from_secs(3);while !seen.load(Ordering::Acquire)&&Instant::now()<until{thread::yield_now();}assert!(seen.load(Ordering::Acquire));gate.wait();t.store(true,Ordering::Release);});let report=run(server.request(5000,4096),HttpControl{deadline:None,cancel:Some(token)});worker.join().unwrap();assert!(matches!(report.result,Ok(_)|Err(HttpFailure::Cancelled)),"{report:?}");assert!(report.diagnostics.is_empty());}}
 #[test]fn cancellation_completion_races_repeat_without_callbacks_left()''')
p.write_text(s,encoding='utf-8')
