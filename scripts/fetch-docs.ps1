# Downloads report documents from the iDrive E2 bucket into var/docs.
#
#   $env:S3_ENDPOINT / S3_REGION / S3_ACCESS_KEY / S3_SECRET_KEY
#   powershell -File .\scripts\fetch-docs.ps1
#
# Temporary research helper - keys stay in the environment.

param(
    [string]$Bucket = 'udsm',
    [string]$Prefix = 'dsm-waste-report-pack',
    [string]$Dest = ''
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
if (-not $Dest) { $Dest = Join-Path $root 'var\docs' }

$access = $env:S3_ACCESS_KEY
$secret = $env:S3_SECRET_KEY
$endpoint = $env:S3_ENDPOINT
$region = $env:S3_REGION

if (-not $access -or -not $secret -or -not $endpoint -or -not $region) {
    Write-Error 'Set S3_ACCESS_KEY, S3_SECRET_KEY, S3_ENDPOINT and S3_REGION first.'
    exit 1
}

$files = @(
    'final-report/FINAL_REPORT_Smart_Waste_Collection_System.md',
    'analysis-md/SMART_TRACS_System_Note.md',
    'analysis-md/05_Contractor_LGA_Obligations_Permits.md',
    'analysis-md/09_Full_Lifecycle_Districts_DMDP.md',
    'analysis-md/10_Contractor_Profiles_Overview.md',
    'analysis-md/06_Smart_Waste_System_Proposal.md'
)

New-Item -ItemType Directory -Force -Path $Dest | Out-Null

function Get-Sha256Hex([byte[]]$bytes) {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    ($sha.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') }) -join ''
}

function Get-Hmac([byte[]]$key, [string]$data) {
    $h = [System.Security.Cryptography.HMACSHA256]::new($key)
    $h.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($data))
}

foreach ($name in $files) {
    $uri = "/$Bucket/$Prefix/$name"
    $amzDate = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')
    $dateStamp = $amzDate.Substring(0, 8)
    $payloadHash = Get-Sha256Hex ([System.Text.Encoding]::UTF8.GetBytes(''))

    $canonicalHeaders = "host:$endpoint`nx-amz-content-sha256:$payloadHash`nx-amz-date:$amzDate`n"
    $signedHeaders = 'host;x-amz-content-sha256;x-amz-date'
    $canonicalRequest = "GET`n$uri`n`n$canonicalHeaders`n$signedHeaders`n$payloadHash"

    $scope = "$dateStamp/$region/s3/aws4_request"
    $stringToSign = "AWS4-HMAC-SHA256`n$amzDate`n$scope`n" +
        (Get-Sha256Hex ([System.Text.Encoding]::UTF8.GetBytes($canonicalRequest)))

    $kDate = Get-Hmac ([System.Text.Encoding]::UTF8.GetBytes("AWS4$secret")) $dateStamp
    $kRegion = Get-Hmac $kDate $region
    $kService = Get-Hmac $kRegion 's3'
    $kSigning = Get-Hmac $kService 'aws4_request'
    $signature = ((Get-Hmac $kSigning $stringToSign) | ForEach-Object { $_.ToString('x2') }) -join ''

    $authorization = "AWS4-HMAC-SHA256 Credential=$access/$scope, SignedHeaders=$signedHeaders, Signature=$signature"

    $response = Invoke-WebRequest -Uri "https://$endpoint$uri" -Method Get -Headers @{
        'x-amz-date'           = $amzDate
        'x-amz-content-sha256' = $payloadHash
        'Authorization'        = $authorization
    } -UseBasicParsing -TimeoutSec 120

    $save = Split-Path -Leaf $name
    $bytes = if ($response.Content -is [byte[]]) {
        $response.Content
    } else {
        [System.Text.Encoding]::UTF8.GetBytes([string]$response.Content)
    }
    [IO.File]::WriteAllBytes((Join-Path $Dest $save), $bytes)
    Write-Host ("{0,-52} {1,6} KB" -f $save, [math]::Round($bytes.Length / 1KB))
}

Write-Host "`ndocs in $Dest"
