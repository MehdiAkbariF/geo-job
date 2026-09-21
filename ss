$outFile = "dump_codebase.txt"
if (Test-Path $outFile) { Remove-Item $outFile -Force }

"====================================" | Out-File -FilePath $outFile -Encoding utf8
"DIRECTORY STRUCTURE"                 | Out-File -FilePath $outFile -Append -Encoding utf8
"====================================" | Out-File -FilePath $outFile -Append -Encoding utf8
Get-ChildItem -Recurse -File | Where-Object { 
    $_.FullName -notmatch '\\(target|\.git|node_modules)\\' -and $_.Name -ne $outFile 
} | Resolve-Path -Relative | Out-File -FilePath $outFile -Append -Encoding utf8

"`n====================================" | Out-File -FilePath $outFile -Append -Encoding utf8
"FILES CONTENT"                       | Out-File -FilePath $outFile -Append -Encoding utf8
"====================================" | Out-File -FilePath $outFile -Append -Encoding utf8
Get-ChildItem -Recurse -File | Where-Object { 
    $_.FullName -notmatch '\\(target|\.git|node_modules)\\' -and 
    $_.Name -ne $outFile -and 
    $_.Extension -match '\.(rs|toml|sql|json|yaml|yml|md|env)$' 
} | ForEach-Object {
    $relPath = Resolve-Path -Path $_.FullName -Relative
    "`n--------------------------------------------------" | Out-File -FilePath $outFile -Append -Encoding utf8
    "FILE: $relPath"                                     | Out-File -FilePath $outFile -Append -Encoding utf8
    "--------------------------------------------------"   | Out-File -FilePath $outFile -Append -Encoding utf8
    Get-Content -Path $_.FullName -Raw | Out-File -FilePath $outFile -Append -Encoding utf8
}