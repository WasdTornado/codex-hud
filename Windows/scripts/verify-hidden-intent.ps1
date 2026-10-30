# 人工观察记录器，不伪装成自动 UI 验收；只启动演示宿主，不读取账号。
param([string]$OutputPath = '')
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw '需要可交互 Windows 环境；此处未执行不等于通过。' }
$ProjectRoot = Split-Path $PSScriptRoot -Parent
$Executable = Join-Path $ProjectRoot 'src-tauri\target\debug\codex-hud-windows.exe'
if (-not (Test-Path -LiteralPath $Executable)) { throw '先执行 verify-windows.ps1 构建宿主。' }
if (Get-Process -Name 'codex-hud-windows' -ErrorAction SilentlyContinue) { throw '请先自行退出已有 HUD，避免多实例影响判定；脚本不会终止已有进程。' }
if (-not $OutputPath) { $OutputPath = Join-Path $ProjectRoot '.artifacts\windows-hidden-intent.json' }
if (Test-Path -LiteralPath $OutputPath) { throw '结果文件已存在，请用 -OutputPath 指定新文件；不会覆盖旧证据。' }
$Steps = @(
    @{ id = 'hide'; instruction = '通过托盘显示后隐藏浮窗；确认浮窗和详情都消失，托盘仍可用。' },
    @{ id = 'manual_refresh'; instruction = '保持隐藏，从托盘刷新额度；确认仍隐藏，托盘同步状态更新。' },
    @{ id = 'background_refresh'; instruction = '保持隐藏至少 6 分钟；观察托盘仍可用且浮窗未出现。仅凭等待不能证明真实后台刷新，另按验收清单记录同步证据。' },
    @{ id = 'restart'; instruction = '用托盘退出，重新运行屏幕上给出的同一 EXE（不加 --real-cli）；确认启动没有闪现浮窗且仍隐藏。' },
    @{ id = 'theme'; instruction = '在系统设置中切换应用/任务栏外观，再恢复原设置；确认两次切换都保持隐藏。未实际切换请选择 s。' },
    @{ id = 'wake'; instruction = '让 Windows 实际睡眠再唤醒；确认保持隐藏。没有实际睡眠请选择 s，发送模拟消息不能替代。' },
    @{ id = 'restore'; instruction = '从托盘主动显示；确认能打开详情、Esc 收起；再隐藏并通过托盘退出测试程序。' }
)
$Results = @($Steps | ForEach-Object { [pscustomobject]@{ id = $_.id; status = 'not_run'; evidence = '尚未观察' } })
Write-Output "仅演示模式：$Executable"
Write-Output '每一步输入 y=实际观察符合，n=失败，s=未执行；没有观察不得填 y。不要记录真实额度或凭据。'
try {
    Start-Process -FilePath $Executable -WorkingDirectory (Split-Path $Executable -Parent)
    for ($Index = 0; $Index -lt $Steps.Count; $Index++) {
        Write-Output $Steps[$Index].instruction
        $Answer = Read-Host '结果 [y/n/s]'
        if ($Answer -eq 'y') { $Results[$Index].status = 'passed'; $Results[$Index].evidence = '操作者确认实际观察符合' }
        elseif ($Answer -eq 'n') { $Results[$Index].status = 'failed'; $Results[$Index].evidence = '操作者确认与预期不符' }
        else { $Results[$Index].status = 'not_run'; $Results[$Index].evidence = '未确认实际观察' }
    }
} finally {
    $ParentDirectory = Split-Path ([IO.Path]::GetFullPath($OutputPath)) -Parent
    [IO.Directory]::CreateDirectory($ParentDirectory) | Out-Null
    $Report = [pscustomobject]@{
        kind = 'manual_windows_demo_hidden_intent'; timestampUtc = [DateTime]::UtcNow.ToString('o')
        windowsVersion = [Environment]::OSVersion.VersionString; executable = $Executable; steps = $Results
    }
    # CreateNew 保证不覆盖另一轮并发生成的记录。
    $Bytes = [Text.UTF8Encoding]::new($false).GetBytes(($Report | ConvertTo-Json -Depth 5))
    $Stream = [IO.File]::Open($OutputPath,[IO.FileMode]::CreateNew)
    try { $Stream.Write($Bytes,0,$Bytes.Length) } finally { $Stream.Dispose() }
    Write-Output "本地记录：$OutputPath。若中断，请自行退出本次演示宿主并恢复系统外观。"
}
if (@($Results | Where-Object { $_.status -eq 'failed' }).Count) { throw '隐藏意图验收存在失败。' }
if (@($Results | Where-Object { $_.status -eq 'not_run' }).Count) { throw '隐藏意图验收不完整；not_run 不计通过。' }
Write-Output '仅人工确认演示模式隐藏意图通过；不是完整 Windows 或真实账号验收。'
