# Ollama Installation Script for Windows
# This script downloads and installs Ollama for unlimited free AI in your agentic CLI

Write-Host "🤖 Installing Ollama for Free AI Support..." -ForegroundColor Cyan

try {
    # Download Ollama for Windows
    $ollamaUrl = "https://github.com/ollama/ollama/releases/latest/download/ollama-windows-amd64.exe"
    $ollamaPath = "$env:TEMP\ollama-setup.exe"
    
    Write-Host "📥 Downloading Ollama..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $ollamaUrl -OutFile $ollamaPath
    
    Write-Host "🚀 Installing Ollama..." -ForegroundColor Yellow
    Start-Process -FilePath $ollamaPath -Wait
    
    Write-Host "✅ Ollama installed! Now downloading AI model..." -ForegroundColor Green
    
    # Add Ollama to PATH if not already there
    $env:PATH = $env:PATH + ";C:\Users\$env:USERNAME\AppData\Local\Programs\Ollama"
    
    # Download the gemma3 model
    Write-Host "📚 Downloading Gemma 3 model (this may take a few minutes)..." -ForegroundColor Yellow
    & "C:\Users\$env:USERNAME\AppData\Local\Programs\Ollama\ollama.exe" pull gemma3
    
    Write-Host "" -ForegroundColor White
    Write-Host "🎉 SUCCESS! Ollama is ready!" -ForegroundColor Green
    Write-Host "Your agentic CLI now has unlimited free AI support!" -ForegroundColor Green
    Write-Host "" -ForegroundColor White
    Write-Host "Test it now:" -ForegroundColor Cyan
    Write-Host "  ./target/release/agentic-cli.exe agent `"create a study plan for next week`"" -ForegroundColor White
    
} catch {
    Write-Host "❌ Error installing Ollama: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "" -ForegroundColor White
    Write-Host "Manual Installation:" -ForegroundColor Yellow
    Write-Host "1. Visit https://ollama.ai" -ForegroundColor White
    Write-Host "2. Download Ollama for Windows" -ForegroundColor White
    Write-Host "3. Run: ollama pull gemma3" -ForegroundColor White
}
