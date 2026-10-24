$ErrorActionPreference='Stop'
$root=(Get-Location).Path
$utf=New-Object System.Text.UTF8Encoding($false)
$cases=New-Object System.Collections.Generic.List[object]
function Case($id,$code,$error='',$message='',$cwd='') { $cases.Add([pscustomobject]@{id=$id;code=$code;error=$error;message=$message;cwd=$cwd}) }
Case unicode 'assert(indexOf("好", "😀你好好") == 2); assert(indexOf("", "😀") == 0); assert(indexOf("x", "😀") == -1); assert(endsWith("好", "😀好")); assert(endsWith("", "")); assert(!endsWith("好", "好😀")); assert(substring(1,3,"😀你好") == "你好"); assert(substring(-9,99,"😀你好") == "😀你好"); assert(substring(9,2,"😀你好") == ""); assert(substring(-9223372036854775808,9223372036854775807,"😀") == "😀");'
Case pad 'let s="中😀"; assert(padEnd(5,"甲🙂",s) == "中😀甲🙂甲"); assert(padStart(5,"甲🙂",s) == "甲🙂甲中😀"); assert(padEnd(-1,"x",s) == s); assert(padStart(1,"x",s) == s); assert(s == "中😀"); assert(padEnd(0,"x","") == "");'
Case codepoints 'assert(ord("😀") == 128512); assert(ord(chr(0)) == 0); assert(ord(chr(55295)) == 55295); assert(ord(chr(57344)) == 57344); assert(ord(chr(1114111)) == 1114111);'
Case range 'assert(range(-2,3) == [-2,-1,0,1,2]); assert(range(2,2) == []); assert(range(3,-2) == []); assert(range(-9223372036854775808,-9223372036854775806) == [-9223372036854775808,-9223372036854775807]); assert(range(9223372036854775805,9223372036854775807) == [9223372036854775805,9223372036854775806]); assert(range(9223372036854775807,-9223372036854775808) == []); assert(range(-1) == []);'
Case contains 'assert(contains([1,{x:2}], [[1,{x:2}]])); assert(!contains(3,[])); assert(contains(1.0,[1])); var a={x:nil}; a.x=a; var b={x:nil}; b.x=b; assert(contains(b,[a]));'
Case ieee 'assert(type(sin(0)) == "float"); assert(sin(0) == 0.0); assert(cos(0) == 1.0); assert(log(1) == 0.0); assert(exp(0) == 1.0); assert(str(log(0)) == "-inf"); assert(str(log(-1)) == "nan"); assert(str(exp(10000)) == "inf"); assert(str(exp(-10000)) == "0.0"); assert(str(sin(float("inf"))) == "nan"); assert(str(cos(float("-inf"))) == "nan"); assert(str(log(float("nan"))) == "nan");'
Case io 'let p="Temp/verifier-v11/data.txt"; assert(writeFile(p,"中文😀\n") == nil); assert(readFile(p) == "中文😀\n"); assert(appendFile(p,"尾") == nil); assert(readFile(p) == "中文😀\n尾"); writeFile(p,"短"); assert(readFile(p) == "短"); assert(appendFile("Temp/verifier-v11/new.txt","首") == nil); assert(readFile("Temp/verifier-v11/new.txt") == "首");'
Case cwd 'writeFile("cwd.txt","CWD"); assert(readFile("cwd.txt") == "CWD");' '' '' (Join-Path $root 'Temp/verifier-v11')
Case ord_empty 'ord("");' ValueError 'ord 的参数必须是单个字符（Unicode 标量数 0）'
Case ord_combining 'ord("é");' ValueError 'ord 的参数必须是单个字符（Unicode 标量数 2）'
Case ord_count 'ord("😀中文ab");' ValueError 'ord 的参数必须是单个字符（Unicode 标量数 5）'
foreach($n in @(-1,55296,57343,1114112,9223372036854775807)) { Case "chr_$n" "chr($n);" ValueError "chr 的参数不是合法的 Unicode 码点：$n" }
Case fill_empty 'padEnd(0,"","");' ValueError '填充串不能为空'
Case fill_start 'padStart(0,"","");' ValueError '填充串不能为空'
Case range_capacity 'range(-9223372036854775808,9223372036854775807);' OverflowError '容量溢出：所需容量超出可分配上限'
Case pad_capacity 'padEnd(9223372036854775807,"😀","a");' OverflowError '容量溢出：所需容量超出可分配上限'
$types=@{range='range(1,2.0)';indexOf='indexOf(1,"a")';endsWith='endsWith("a",1)';padEnd='padEnd(1.0,"a","")';padStart='padStart(2,1,"")';substring='substring(0,1.0,"")';ord='ord(1)';chr='chr(1.0)';contains='contains("x",{x:1})';sin='sin(true)';cos='cos("0")';log='log(nil)';exp='exp([])';readFile='readFile(1)';writeFile='writeFile("Temp/verifier-v11/data.txt",42)';appendFile='appendFile(1,"x)'}
$types.appendFile='appendFile(1,"x")'
foreach($name in $types.Keys) { Case "type_$name" ($types[$name]+';') TypeError }
$arity=@{indexOf=2;endsWith=2;padEnd=3;padStart=3;substring=3;ord=1;chr=1;contains=2;sin=1;cos=1;log=1;exp=1;readFile=1;writeFile=2;appendFile=2}
foreach($name in $arity.Keys) { Case "arity_${name}_zero" "$name();" TypeError "函数 $name 期待 $($arity[$name]) 个参数，得到 0"; $argsList=(@('nil') * ($arity[$name]+1)) -join ','; Case "arity_${name}_extra" "$name($argsList);" TypeError "函数 $name 期待 $($arity[$name]) 个参数，得到 $($arity[$name]+1)" }
Case range_zero 'range();' TypeError '函数 range 期待 1 个参数，得到 0'
Case range_three 'range(1,2,3);' TypeError '函数 range 期待 2 个参数，得到 3'
[IO.File]::WriteAllBytes((Join-Path $root 'Temp/verifier-v11/invalid.txt'),[byte[]]@(255,254,128))
Case io_invalid 'readFile("Temp/verifier-v11/invalid.txt");' IOError '无法读取：Temp/verifier-v11/invalid.txt（不是合法的 UTF-8 编码）'
Case io_missing 'readFile("Temp/verifier-v11/absent.txt");' IOError '无法读取：Temp/verifier-v11/absent.txt'
Case io_parent 'writeFile("Temp/verifier-v11/no-parent/data.txt","x");' IOError '无法写入：Temp/verifier-v11/no-parent/data.txt'
Case io_append_parent 'appendFile("Temp/verifier-v11/no-parent/data.txt","x");' IOError '无法追加：Temp/verifier-v11/no-parent/data.txt'
Case io_no_tilde 'readFile("~/verifier-v11-absent.txt");' IOError '无法读取：~/verifier-v11-absent.txt'
Case json_channels 'print("stdout marker"); eprint("stderr marker"); let z=42; __DUMP__' 
$results=foreach($c in $cases) {
 $fixtureName='new-'+[Guid]::NewGuid().ToString('N')+'.txt'; $c.code=$c.code.Replace('new.txt',$fixtureName); $c.code=$c.code.Replace(';;','__DUMP__').Replace(';',"`n").Replace('__DUMP__',';;'); $file=Join-Path $root ("Temp/verifier-v11/"+$c.id+'.lfz'); [IO.File]::WriteAllText($file,"#42`n"+$c.code+"`n",$utf)
 $psi=New-Object Diagnostics.ProcessStartInfo; $psi.FileName=Join-Path $root 'target/debug/lfz.exe'; $psi.Arguments='--json run "'+$file+'"'; $psi.WorkingDirectory=$root; if($c.cwd){$psi.WorkingDirectory=$c.cwd}; $psi.UseShellExecute=$false; $psi.RedirectStandardOutput=$true; $psi.RedirectStandardError=$true; $psi.StandardOutputEncoding=$utf; $psi.StandardErrorEncoding=$utf
 $p=[Diagnostics.Process]::Start($psi); $out=$p.StandardOutput.ReadToEnd(); $err=$p.StandardError.ReadToEnd(); $p.WaitForExit(); $pass=$false; $obj=$null
 try { $obj=$out | ConvertFrom-Json; if($c.error){$pass=($p.ExitCode -eq 2 -and $obj.error -eq $c.error -and (!$c.message -or $obj.message -eq $c.message))} else {$pass=($p.ExitCode -eq 0 -and $obj.ok -eq $true)} } catch {}
 if($c.id -eq 'json_channels'){$pass=$pass -and $err.Contains('stdout marker') -and $err.Contains('stderr marker') -and $err.Contains('42')}
 [pscustomobject]@{id=$c.id;pass=$pass;expectedError=$c.error;expectedMessage=$c.message;exit=$p.ExitCode;stdout=$out;stderr=$err;command=$psi.FileName+' '+$psi.Arguments;cwd=$psi.WorkingDirectory;source=$c.code}
}
$results | ConvertTo-Json -Depth 8 | ForEach-Object { [IO.File]::WriteAllText((Join-Path $root 'docs/reports/T11-v11-reverification-evidence/witness.json'),$_,$utf) }
$results | Where-Object { !$_.pass } | Format-List
"TOTAL=$($results.Count) PASS=$(@($results | Where-Object pass).Count) FAIL=$(@($results | Where-Object { !$_.pass }).Count)"