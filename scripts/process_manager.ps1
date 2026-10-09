param (
    [Parameter(Mandatory=$true)]
    [string]$Action,
    [string]$Filter = "",
    [string]$PidToKill = ""
)

switch ($Action.ToLower()) {
    "list" {
        $procs = Get-Process -ErrorAction SilentlyContinue |
            Where-Object { $_.Id -ne 0 -and $_.Id -ne 4 } |
            Sort-Object WorkingSet64 -Descending |
            Select-Object -First 35 Id, ProcessName, WorkingSet64, CPU, Path

        $result = @()
        foreach ($p in $procs) {
            $ramMb = [math]::Round($p.WorkingSet64 / 1MB, 1)
            $cpuVal = if ($null -ne $p.CPU) { [math]::Round($p.CPU, 1) } else { 0.0 }
            $result += [PSCustomObject]@{
                id = $p.Id
                name = $p.ProcessName
                ram_mb = $ramMb
                cpu = $cpuVal
                path = if ($p.Path) { $p.Path } else { "" }
            }
        }
        $result | ConvertTo-Json -Compress
    }
    "kill" {
        if ($PidToKill) {
            Stop-Process -Id $PidToKill -Force -ErrorAction Stop
            Write-Output "OK"
        }
    }
}
