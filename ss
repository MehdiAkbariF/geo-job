# ۱. حذف فایل ۲ گیگابایتی قبلی
Remove-Item "project_codebase_export.txt" -ErrorAction SilentlyContinue

# ۲. ساخت فایل جدید مختص کدهای منبع، کانفیگ‌ها و مایگریشن‌ها
New-Item -ItemType File -Force -Path "project_codebase_export.txt"

# ۳. استخراج هوشمند فقط فایل‌های متنی و سورس‌کد (بدون target و data و git)
Get-ChildItem -Recurse -File | Where-Object { 
    $_.FullName -notmatch '\\target\\' -and 
    $_.FullName -notmatch '\\\.git\\' -and 
    $_.FullName -notmatch '\\data\\' -and 
    $_.Extension -in '.rs', '.toml', '.sql', '.html', '.md', '.yml' 
} | ForEach-Object {
    "======================================================================"
    "FILE: $($_.FullName)"
    "======================================================================"
    Get-Content $_.FullName -Raw
    "`n`n"
} | Out-File -Encoding utf8 "project_codebase_export.txt"