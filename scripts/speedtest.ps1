param()

$pingMs = 0
try {
    $pingRes = Test-Connection -ComputerName "ya.ru" -Count 1 -ErrorAction Stop
    $pingMs = $pingRes.ResponseTime
} catch {
    $pingMs = -1
}

$downMbps = 0.0
try {
    $downUrl = "https://speed.cloudflare.com/__down?bytes=5000000"
    $wc = New-Object System.Net.WebClient
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $bytes = $wc.DownloadData($downUrl)
    $sw.Stop()
    $elapsedSec = $sw.Elapsed.TotalSeconds
    if ($elapsedSec -gt 0 -and $bytes.Length -gt 0) {
        $bits = $bytes.Length * 8
        $downMbps = [math]::Round($bits / ($elapsedSec * 1000000), 1)
    }
} catch {
    $downMbps = 0.0
}

$upMbps = 0.0
try {
    $upUrl = "https://speed.cloudflare.com/__up"
    $uploadData = New-Object byte[] (1024 * 1024 * 1)
    (New-Object System.Random).NextBytes($uploadData)
    $wcUp = New-Object System.Net.WebClient
    $swUp = [System.Diagnostics.Stopwatch]::StartNew()
    $wcUp.UploadData($upUrl, "POST", $uploadData) | Out-Null
    $swUp.Stop()
    $elapsedUpSec = $swUp.Elapsed.TotalSeconds
    if ($elapsedUpSec -gt 0) {
        $bitsUp = $uploadData.Length * 8
        $upMbps = [math]::Round($bitsUp / ($elapsedUpSec * 1000000), 1)
    }
} catch {
    $upMbps = 0.0
}

[PSCustomObject]@{
    success = $true
    ping_ms = $pingMs
    download_mbps = $downMbps
    upload_mbps = $upMbps
    server = "Cloudflare Edge / ya.ru"
} | ConvertTo-Json -Compress
