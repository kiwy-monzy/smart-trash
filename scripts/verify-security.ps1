# Security checks for the HTTP gateway.
#
# Start both processes first, then run:
#   powershell -ExecutionPolicy Bypass -File .\scripts\verify-security.ps1
#
# It checks authentication, the role rules, per-customer scoping, token
# tampering, SQL-injection probes and the response headers. Exit code 1 means
# at least one check failed.

param([string]$BaseUrl = 'http://127.0.0.1:8080')

$passed = 0
$failed = 0

function Invoke-Api {
    param([string]$Method, [string]$Path, [string]$Token, [hashtable]$Body)

    $headers = @{}
    if ($Token) { $headers['Authorization'] = "Bearer $Token" }

    $params = @{
        Method      = $Method
        Uri         = "$BaseUrl$Path"
        Headers     = $headers
        ErrorAction = 'Stop'
    }
    if ($Body) {
        $params['Body'] = ($Body | ConvertTo-Json -Compress)
        $params['ContentType'] = 'application/json'
    }

    try {
        $response = Invoke-WebRequest @params -UseBasicParsing
        return @{ Code = [int]$response.StatusCode; Text = $response.Content }
    }
    catch [System.Net.WebException] {
        $response = $_.Exception.Response
        $reader = New-Object System.IO.StreamReader($response.GetResponseStream())
        return @{ Code = [int]$response.StatusCode; Text = $reader.ReadToEnd() }
    }
}

function Check {
    param([string]$Name, [string]$Method, [string]$Path, [string]$Token,
          [hashtable]$Body, [int]$Expected)

    $result = Invoke-Api -Method $Method -Path $Path -Token $Token -Body $Body
    if ($result.Code -eq $Expected) {
        $script:passed++
        Write-Host ("  PASS  {0} ({1})" -f $Name, $result.Code) -ForegroundColor Green
    }
    else {
        $script:failed++
        Write-Host ("  FAIL  {0} -> {1}, expected {2}: {3}" -f $Name, $result.Code, $Expected, $result.Text) -ForegroundColor Red
    }
    return $result
}

function Login {
    param([string]$UserId)
    $result = Invoke-Api -Method 'POST' -Path '/api/auth/login' -Body @{ user_id = $UserId }
    if ($result.Code -ne 200) { throw "login for $UserId failed: $($result.Code) $($result.Text)" }
    return ($result.Text | ConvertFrom-Json)
}

Write-Host "`nSmart Trash gateway checks against $BaseUrl" -ForegroundColor Cyan

# 1. public surface
Write-Host "`nPublic routes" -ForegroundColor Cyan
Check 'health'          'GET' '/health'       $null $null 200 | Out-Null
Check 'prototype index' 'GET' '/'             $null $null 200 | Out-Null

# 2. no credentials
Write-Host "`nWithout a token" -ForegroundColor Cyan
Check 'list pickups'    'GET' '/api/pickups'  $null $null 401 | Out-Null
Check 'create pickup'   'POST' '/api/pickups' $null @{ customer_id = 'HOUSEHOLD-0421'; waste_type = 'general'; latitude = -6.8; longitude = 39.2; estimated_volume = 1 } 401 | Out-Null
Check 'council stats'   'GET' '/api/stats'    $null $null 401 | Out-Null
Check 'who am i'        'GET' '/api/auth/me'  $null $null 401 | Out-Null

# 3. sign-in
Write-Host "`nSign-in" -ForegroundColor Cyan
$household = Login 'HOUSEHOLD-0421'
$business = Login 'BUSINESS-12'
$driver = Login 'DRIVER-07'
$council = Login 'COUNCIL-01'
Check 'unknown user'    'POST' '/api/auth/login' $null @{ user_id = 'HOUSEHOLD-9999' } 401 | Out-Null
Check 'who am i'        'GET' '/api/auth/me' $household.token $null 200 | Out-Null

# 4. household
Write-Host "`nHousehold" -ForegroundColor Cyan
$list = Check 'own pickups' 'GET' '/api/pickups?page_size=20' $household.token $null 200
$foreign = ($list.Text | ConvertFrom-Json) | Where-Object { $_.customer_id -ne 'HOUSEHOLD-0421' }
if ($foreign) { $failed++; Write-Host '  FAIL  household saw another customer''s rows' -ForegroundColor Red }
else { $passed++; Write-Host '  PASS  every row belongs to HOUSEHOLD-0421' -ForegroundColor Green }

$created = Check 'create own pickup' 'POST' '/api/pickups' $household.token @{
    customer_id = 'HOUSEHOLD-0421'; waste_type = 'recyclable'
    latitude = -6.7565; longitude = 39.2049; estimated_volume = 2.5
} 201
Write-Host ("        created {0} (this demo row stays in the database)" -f ($created.Text | ConvertFrom-Json).pickup_id)

Check 'book for somebody else' 'POST' '/api/pickups' $household.token @{
    customer_id = 'BUSINESS-12'; waste_type = 'general'
    latitude = -6.8; longitude = 39.2; estimated_volume = 1
} 403 | Out-Null

Check 'assign a truck'  'POST' '/api/pickups/PICKUP-00001/assign' $household.token @{ vehicle_id = 'VEH-001'; driver_id = 'DRIVER-07' } 403 | Out-Null
Check 'council stats'   'GET' '/api/stats?zone=Kinondoni' $household.token $null 403 | Out-Null

# 5. council
Write-Host "`nCouncil" -ForegroundColor Cyan
Check 'all pickups'     'GET' '/api/pickups?page_size=5' $council.token $null 200 | Out-Null
Check 'collection stats' 'GET' '/api/stats?zone=Kinondoni' $council.token $null 200 | Out-Null
Check 'bad month'       'GET' "/api/stats?zone=Kinondoni&month=2026-13" $council.token $null 400 | Out-Null
Check 'report gps'      'POST' '/api/tracking/locations' $council.token @{ vehicle_id = 'VEH-001'; driver_id = 'DRIVER-07'; latitude = -6.78; longitude = 39.21; speed_mps = 1 } 403 | Out-Null

# 6. driver
Write-Host "`nDriver" -ForegroundColor Cyan
$route = Check 'own route' 'GET' '/api/pickups?status=assigned' $driver.token $null 200
$wrong = ($route.Text | ConvertFrom-Json) | Where-Object { $_.driver_id -ne 'DRIVER-07' }
if ($wrong) { $failed++; Write-Host '  FAIL  driver saw another driver''s work' -ForegroundColor Red }
else { $passed++; Write-Host '  PASS  every row is assigned to DRIVER-07' -ForegroundColor Green }

Check 'gps ping'        'POST' '/api/tracking/locations' $driver.token @{ vehicle_id = 'VEH-001'; driver_id = 'DRIVER-07'; latitude = -6.78; longitude = 39.21; speed_mps = 7.2 } 202 | Out-Null
Check 'gps as somebody else' 'POST' '/api/tracking/locations' $driver.token @{ vehicle_id = 'VEH-001'; driver_id = 'DRIVER-11'; latitude = -6.78; longitude = 39.21; speed_mps = 7.2 } 403 | Out-Null
Check 'council stats'   'GET' '/api/stats' $driver.token $null 403 | Out-Null

# 7. business
Write-Host "`nBusiness" -ForegroundColor Cyan
Check 'own records'     'GET' '/api/pickups?page_size=50' $business.token $null 200 | Out-Null
Check 'read a household pickup' 'GET' '/api/pickups/PICKUP-00001' $business.token $null 403 | Out-Null
Check 'council stats'   'GET' '/api/stats' $business.token $null 403 | Out-Null

# 8. token tampering
Write-Host "`nTampered tokens" -ForegroundColor Cyan
$forged = $household.token -replace '\.household\.', '.council.'
Check 'role swapped in token' 'GET' '/api/pickups' $forged $null 401 | Out-Null
Check 'random string'         'GET' '/api/pickups' 'not-a-token' $null 401 | Out-Null
Check 'extra character'       'GET' '/api/pickups' ($household.token + 'x') $null 401 | Out-Null

# 9. injection probes
Write-Host "`nInjection probes" -ForegroundColor Cyan
Check 'quote in pickup id'   'GET' "/api/pickups/PICKUP-00001'%20OR%20'1'='1" $household.token $null 400 | Out-Null
Check 'stacked statement'    'GET' '/api/pickups/1;DROP%20TABLE%20pickups;--' $household.token $null 400 | Out-Null
Check 'null byte'            'GET' '/api/pickups/PICKUP-00001%00' $household.token $null 400 | Out-Null
Check 'quote in status'      'GET' "/api/pickups?status=pending'--" $household.token $null 400 | Out-Null
Check 'huge page size'       'GET' '/api/pickups?page_size=999999999' $household.token $null 200 | Out-Null

# 10. headers
Write-Host "`nResponse headers" -ForegroundColor Cyan
$headers = (Invoke-WebRequest -Uri "$BaseUrl/health" -UseBasicParsing).Headers
foreach ($name in 'X-Content-Type-Options', 'X-Frame-Options', 'Referrer-Policy', 'Content-Security-Policy') {
    if ($headers[$name]) {
        $passed++
        Write-Host ("  PASS  {0}: {1}" -f $name, $headers[$name]) -ForegroundColor Green
    }
    else {
        $failed++
        Write-Host ("  FAIL  {0} is missing" -f $name) -ForegroundColor Red
    }
}

Write-Host ("`npassed {0}, failed {1}" -f $passed, $failed) -ForegroundColor Cyan
if ($failed -gt 0) { exit 1 }
