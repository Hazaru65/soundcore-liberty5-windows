param(
  [int]$Port = 4173
)

Set-Location $PSScriptRoot
Write-Host "Liberty 5 UI prototypes: http://localhost:$Port/index.html?variant=a"
py -m http.server $Port
