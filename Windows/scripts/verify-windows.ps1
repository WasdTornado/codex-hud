# 在已有开发环境的 Windows 中手动运行；不安装工具、不触发 CI、不上传结果。
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw '此脚本必须在 Windows 中运行，非 Windows 不算通过。' }
$ProjectRoot = Split-Path $PSScriptRoot -Parent
$HadPython = Test-Path Env:HUD_TEST_PYTHON
$PreviousPython = $env:HUD_TEST_PYTHON
Push-Location $ProjectRoot
try {
    if (-not (Get-Command cargo -CommandType Application -ErrorAction SilentlyContinue)) { throw '缺少 Rust/Cargo。请先由用户配置官方工具链。' }
    $Python = Get-Command python -CommandType Application -ErrorAction SilentlyContinue
    if (-not $Python -or $Python.Source -match 'WindowsApps') { throw '需要已有可运行的 Python，不能使用商店占位入口。脚本不会安装。' }
    $env:HUD_TEST_PYTHON = $Python.Source
    & $env:HUD_TEST_PYTHON --version
    if ($LASTEXITCODE -ne 0) { throw '现有 Python 无法运行。' }

    # 先确认专属用例确实被收集，避免 feature/cfg 导致 0 tests 却被误算成功。
    $Cases = @(cargo test --locked --features test-fixtures --manifest-path Cargo.toml --test windows_native -- --list)
    if ($LASTEXITCODE -ne 0) { throw 'Windows 专属测试编译/收集失败。' }
    $WindowsCases = @($Cases | Where-Object { $_ -match '^windows_.+: test$' })
    if ($WindowsCases.Count -ne 7) { throw "预期收集 7 个 Windows 专属用例，实际 $($WindowsCases.Count)；不能算通过。" }
    $WindowsCases | ForEach-Object { Write-Output $_ }

    cargo test --locked --features test-fixtures --manifest-path Cargo.toml --lib --test protocol --test native_fixture
    if ($LASTEXITCODE -ne 0) { throw '核心或共用无账号协议测试失败。' }
    cargo test --locked --features test-fixtures --manifest-path Cargo.toml --test windows_native -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Windows exe/cmd/进程树专属测试失败。' }
    cargo test --locked --manifest-path src-tauri/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw '宿主绘制、隐藏偏好持久化或离线验签测试失败。' }
    cargo build --locked --manifest-path src-tauri/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Windows 宿主构建失败。' }
    Write-Output '仅完成 Windows 本地构建和无账号自动测试；请核对各段均为 failed=0、ignored=0。'
    Write-Output '隐藏意图交互验收仍未执行：运行 scripts/verify-hidden-intent.ps1 并按真实观察记录。'
    Write-Output '真实账号、透明磨砂/托盘/DPI、真实唤醒与更新安装仍需独立验收。'
} finally {
    if ($HadPython) { $env:HUD_TEST_PYTHON = $PreviousPython }
    else { Remove-Item Env:HUD_TEST_PYTHON -ErrorAction SilentlyContinue }
    Pop-Location
}
