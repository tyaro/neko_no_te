# LLM 応答性能検証スクリプト
# Usage: .\scripts\verify-llm.ps1 [-Model "phi4-mini:3.8b"] [-EnableMcp] [-Verbose]

param(
    [string]$Model = "phi4-mini:3.8b",
    [switch]$EnableMcp,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "=== LLM Response Verification Script ===" -ForegroundColor Cyan
Write-Host "Model: $Model"
Write-Host "MCP Tools: $(if ($EnableMcp) { 'Enabled' } else { 'Disabled' })"
Write-Host ""

# Test prompts
$testCases = @(
    @{
        Name = "Simple greeting"
        Prompt = "こんにちは、簡単に挨拶してください。"
        ExpectTool = $false
    },
    @{
        Name = "Weather query (requires MCP)"
        Prompt = "東京の今日の天気を教えて"
        ExpectTool = $true
    },
    @{
        Name = "Code explanation"
        Prompt = "Rustのライフタイムとは何か、1文で説明して"
        ExpectTool = $false
    }
)

$results = @()

foreach ($test in $testCases) {
    Write-Host "[TEST] $($test.Name)" -ForegroundColor Yellow
    Write-Host "Prompt: $($test.Prompt)"
    
    $args = @(
        "chat"
        "--prompt", $test.Prompt
        "--model", $Model
        "--format", "json"
    )
    
    if (-not $EnableMcp) {
        $args += "--no-mcp"
    }
    
    if ($test.ExpectTool -and -not $EnableMcp) {
        Write-Host "[SKIP] Test requires MCP but it's disabled" -ForegroundColor Gray
        continue
    }
    
    try {
        # Capture only stdout, filter out warnings
        $output = & cargo run -p neko-assistant --quiet -- @args 2>$null
        
        # Parse JSON from clean stdout
        if ($output) {
            $jsonText = $output | Out-String
            $json = $jsonText | ConvertFrom-Json
            
            $result = @{
                Test = $test.Name
                Success = $true
                ElapsedMs = $json.elapsed_ms
                ToolCalls = $json.tool_calls
                ResponseLength = $json.response.Length
            }
            
            Write-Host "✓ Success" -ForegroundColor Green
            Write-Host "  Response: $($json.response.Substring(0, [Math]::Min(60, $json.response.Length)))..."
            Write-Host "  Elapsed: $($json.elapsed_ms)ms | Tool calls: $($json.tool_calls)"
            
        } else {
            throw "Failed to parse JSON output"
        }
    }
    catch {
        Write-Host "✗ Failed: $_" -ForegroundColor Red
        $result = @{
            Test = $test.Name
            Success = $false
            Error = $_.Exception.Message
        }
    }
    
    $results += [PSCustomObject]$result
    Write-Host ""
}

# Summary
Write-Host "=== Summary ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize

$totalTests = $results.Count
$successTests = ($results | Where-Object { $_.Success }).Count
$avgElapsed = if ($successTests -gt 0) {
    ($results | Where-Object { $_.Success } | Measure-Object -Property ElapsedMs -Average).Average
} else { 0 }

Write-Host ""
Write-Host "Total: $totalTests | Success: $successTests | Avg time: $([Math]::Round($avgElapsed))ms" -ForegroundColor Cyan

if ($successTests -eq $totalTests) {
    Write-Host "✓ All tests passed" -ForegroundColor Green
    exit 0
} else {
    Write-Host "✗ Some tests failed" -ForegroundColor Red
    exit 1
}
