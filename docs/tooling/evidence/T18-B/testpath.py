from pathlib import Path
p=Path('tests/cli_t18b.rs');s=p.read_text(encoding='utf-8').replace('C:\\\\not-present.exe','C:/not-present.exe');p.write_text(s,encoding='utf-8')
