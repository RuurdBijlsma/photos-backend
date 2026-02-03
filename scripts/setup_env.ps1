Push-Location $( Resolve-Path (Join-Path $PSScriptRoot "..") )

scripts/start_postgres.ps1

Pop-Location

llama-server -hf unsloth/Qwen3-VL-4B-Instruct-GGUF:Q4_K_M `
    --n-gpu-layers 99 --jinja --top-p 0.8 --temp 0.7 --min-p 0.0 --flash-attn on `
    --presence-penalty 1.5 --ctx-size 8192 --models-max 1 --sleep-idle-seconds 60
