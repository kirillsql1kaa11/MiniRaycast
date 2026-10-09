param (
    [Parameter(Mandatory=$true)]
    [string]$Action,
    [string]$Target = ""
)

switch ($Action.ToLower()) {
    "flushdns" {
        Clear-DnsClientCache -ErrorAction SilentlyContinue
        & ipconfig /flushdns | Out-Null
        [PSCustomObject]@{
            success = $true
            message = "DNS-кэш успешно очищен"
        } | ConvertTo-Json -Compress
    }
    "get_ip" {
        $localIp = ""
        try {
            $addr = Get-NetIPAddress -AddressFamily IPv4 -ErrorAction Stop |
                Where-Object { $_.InterfaceAlias -notlike '*Loopback*' -and $_.IPAddress -notlike '169.254*' } |
                Select-Object -First 1 -ExpandProperty IPAddress
            $localIp = $addr
        } catch {
            $localIp = "127.0.0.1"
        }

        $extIp = ""
        try {
            $extIp = (Invoke-RestMethod -Uri "https://api.ipify.org" -TimeoutSec 2 -ErrorAction Stop).Trim()
        } catch {
            $extIp = "Не удалось определить"
        }

        [PSCustomObject]@{
            success = $true
            local_ip = $localIp
            external_ip = $extIp
        } | ConvertTo-Json -Compress
    }
    "ping" {
        $hostToPing = if ($Target) { $Target } else { "ya.ru" }
        try {
            $pingRes = Test-Connection -ComputerName $hostToPing -Count 1 -ErrorAction Stop
            $timeMs = $pingRes.ResponseTime
            [PSCustomObject]@{
                success = $true
                host = $hostToPing
                time_ms = $timeMs
                status = "Доступен"
            } | ConvertTo-Json -Compress
        } catch {
            [PSCustomObject]@{
                success = $false
                host = $hostToPing
                time_ms = -1
                status = "Недоступен"
            } | ConvertTo-Json -Compress
        }
    }
    "bluetooth_on" {
        try {
            Start-Service -Name "bthserv" -ErrorAction SilentlyContinue
            Start-Process "ms-settings:bluetooth"
            [PSCustomObject]@{
                success = $true
                message = "Bluetooth включен"
            } | ConvertTo-Json -Compress
        } catch {
            [PSCustomObject]@{
                success = $false
                message = "Ошибка включения Bluetooth"
            } | ConvertTo-Json -Compress
        }
    }
    "bluetooth_off" {
        try {
            Stop-Service -Name "bthserv" -Force -ErrorAction SilentlyContinue
            [PSCustomObject]@{
                success = $true
                message = "Bluetooth выключен"
            } | ConvertTo-Json -Compress
        } catch {
            [PSCustomObject]@{
                success = $false
                message = "Ошибка выключения Bluetooth"
            } | ConvertTo-Json -Compress
        }
    }
    "bluetooth_toggle" {
        try {
            $svc = Get-Service -Name "bthserv" -ErrorAction SilentlyContinue
            if ($svc -and $svc.Status -eq "Running") {
                Stop-Service -Name "bthserv" -Force -ErrorAction SilentlyContinue
                [PSCustomObject]@{
                    success = $true
                    state = "off"
                    message = "Bluetooth отключен"
                } | ConvertTo-Json -Compress
            } else {
                Start-Service -Name "bthserv" -ErrorAction SilentlyContinue
                Start-Process "ms-settings:bluetooth"
                [PSCustomObject]@{
                    success = $true
                    state = "on"
                    message = "Bluetooth включен"
                } | ConvertTo-Json -Compress
            }
        } catch {
            Start-Process "ms-settings:bluetooth"
            [PSCustomObject]@{
                success = $true
                state = "settings"
                message = "Открыты настройки Bluetooth"
            } | ConvertTo-Json -Compress
        }
    }
}
