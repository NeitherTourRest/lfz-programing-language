use std::{io::{Read,Write},time::{Duration,Instant}};
fn main(){let a:Vec<String>=std::env::args().collect();match a.get(1).map(|s|s.as_str()).unwrap_or("none"){
 "echo"=>{for s in &a[2..]{let _=write!(std::io::stdout(),"{}:",s.len());std::io::stdout().write_all(s.as_bytes()).unwrap();println!();}},
 "dual"=>{let n:usize=a[2].parse().unwrap();let block=[b'x';8192];let mut left=n;while left>0{let k=left.min(block.len());std::io::stdout().write_all(&block[..k]).unwrap();std::io::stderr().write_all(&block[..k]).unwrap();left-=k;}},
 "utf8"=>{std::io::stdout().write_all(&[0xef,0xbb,0xbf,b'a',b'\r',b'\n']).unwrap();},
 "invalid"=>{std::io::stdout().write_all(&[0xff]).unwrap();},
 "stdin"=>{let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();print!("{}",b.len());},
 "exit"=>std::process::exit(a[2].parse().unwrap()),
 "sleep"=>std::thread::sleep(Duration::from_millis(a[2].parse().unwrap())),
 "mark_sleep"=>{std::fs::write(&a[2],b"started").unwrap();std::thread::sleep(Duration::from_secs(10));},
 "mark"=>std::fs::write(&a[2],b"ran").unwrap(),
 "descendant"=>{std::fs::write(&a[2],std::process::id().to_string()).unwrap();loop{std::thread::sleep(Duration::from_millis(100));}},
 "parent"=>{let mut c=std::process::Command::new(std::env::current_exe().unwrap());c.args(["descendant",&a[2]]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::inherit()).stderr(std::process::Stdio::inherit());#[cfg(windows)]{use std::os::windows::process::CommandExt;c.creation_flags(0x08000000);}let mut child=c.spawn().unwrap();let end=Instant::now()+Duration::from_secs(2);while !std::path::Path::new(&a[2]).exists(){if Instant::now()>=end{let _=child.kill();let _=child.wait();std::process::exit(3);}std::thread::sleep(Duration::from_millis(2));}print!("{}",child.id());},
 _=>{}
}}
