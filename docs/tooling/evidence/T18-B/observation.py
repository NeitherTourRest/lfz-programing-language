from pathlib import Path
p=Path('src/http_control/windows.rs');s=p.read_text(encoding='utf-8-sig');s=s.replace('type H=*mut c_void;', '''type H=*mut c_void;
#[cfg(test)]std::thread_local!{static OBSERVED:std::cell::RefCell<Option<(String,Vec<u16>)>>=const{std::cell::RefCell::new(None)};}''')
a=s.index(' #[cfg(test)]{let dir=');b=s.index('\n if text.len()',a)
s=s[:a]+''' #[cfg(test)]OBSERVED.with(|slot|*slot.borrow_mut()=Some((text.clone(),raw.clone())));'''+s[b:]
s=s.replace('let raw=std::fs::read(dir.join("raw-header-observed.txt")).unwrap();let units=std::fs::read(dir.join("raw-header-observed.u16le")).unwrap();', '''let (text,raw_units)=OBSERVED.with(|slot|slot.borrow_mut().take().unwrap());std::fs::create_dir_all(&dir).unwrap();let raw=text.as_bytes();let units=raw_units.iter().flat_map(|n|n.to_le_bytes()).collect::<Vec<_>>();std::fs::write(dir.join("raw-header-observed.txt"),raw).unwrap();std::fs::write(dir.join("raw-header-observed.u16le"),&units).unwrap();''')
p.write_text(s,encoding='utf-8')
