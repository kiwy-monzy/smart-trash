# Downloads the Dar es Salaam map layers from the iDrive E2 bucket into
# var/geo-raw (outside the app bundle, so it is never shipped). Then run:
#   python scripts/prepare-map.py
#
# Credentials come from the environment and are never written to the repo:
#
#   $env:S3_ENDPOINT   = 's3.eu-west-3.idrivee2.com'
#   $env:S3_REGION     = 'eu-west-3'
#   $env:S3_ACCESS_KEY = '...'
#   $env:S3_SECRET_KEY = '...'
#   powershell -File .\scripts\fetch-map-data.ps1

param(
    [string]$Bucket = 'udsm',
    [string]$Prefix = 'dsm-waste-preview-source/public/geojson',
    [string]$Dest = ''
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
if (-not $Dest) { $Dest = Join-Path $root 'var\geo-raw' }

$access = $env:S3_ACCESS_KEY
$secret = $env:S3_SECRET_KEY
$endpoint = $env:S3_ENDPOINT
$region = $env:S3_REGION

if (-not $access -or -not $secret -or -not $endpoint -or -not $region) {
    Write-Error 'Set S3_ACCESS_KEY, S3_SECRET_KEY, S3_ENDPOINT and S3_REGION first.'
    exit 1
}

# One entry per source layer we actually draw. roads_minor is 48 MB raw; the
# prepare step simplifies it before the app ever sees it.
$files = @(
    'region.geojson',
    'districts.geojson',
    'wards_official.geojson',
    'dumpsites.geojson',
    'roads_major.geojson',
    'roads_minor.geojson',
    'summary.json',
    'ward_list_official.json'
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
    } -UseBasicParsing -TimeoutSec 300

    $bytes = if ($response.Content -is [byte[]]) {
        $response.Content
    } else {
        [System.Text.Encoding]::UTF8.GetBytes([string]$response.Content)
    }

    [IO.File]::WriteAllBytes((Join-Path $Dest $name), $bytes)
    Write-Host ("{0,-26} {1,7} KB" -f $name, [math]::Round($bytes.Length / 1KB))
}

Write-Host "`nraw layers in $Dest"
Write-Host 'next: python scripts/prepare-map.py'
