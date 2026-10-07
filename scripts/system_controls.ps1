param (
    [Parameter(Mandatory=$true)]
    [string]$Action,
    [string]$Value = ""
)

switch ($Action.ToLower()) {
    "lock" {
        rundll32.exe user32.dll,LockWorkStation
    }
    "sleep" {
        Add-Type -Assembly System.Windows.Forms
        [System.Windows.Forms.Application]::SetSuspendState([System.Windows.Forms.PowerState]::Suspend, $false, $false)
    }
    "empty_trash" {
        Clear-RecycleBin -Force -ErrorAction SilentlyContinue
    }
    "volume_up" {
        $wscript = New-Object -ComObject WScript.Shell
        1..5 | ForEach-Object { $wscript.SendKeys([char]175) }
    }
    "volume_down" {
        $wscript = New-Object -ComObject WScript.Shell
        1..5 | ForEach-Object { $wscript.SendKeys([char]174) }
    }
    "mute" {
        $wscript = New-Object -ComObject WScript.Shell
        $wscript.SendKeys([char]173)
    }
    "shutdown" {
        shutdown /s /t 0
    }
    "restart" {
        shutdown /r /t 0
    }
    default {
        Write-Error "Unknown action: $Action"
        exit 1
    }
}
