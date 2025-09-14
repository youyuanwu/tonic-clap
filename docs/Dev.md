# Auto complete
```ps1
# Generate PowerShell completion script
.\ctr.exe --generate-completion powershell > target/ctr-completion.ps1

# Generate PowerShell completion script
.\ctr.exe --generate-completion powershell > target/ctr-completion.ps1

# Generate and load completion directly
.\ctr.exe --generate-completion powershell | Invoke-Expression
```

```ps1
cargo run --bin hwgencli -q -- --generate-completion powershell > target/hwgencli-completion.ps1
```