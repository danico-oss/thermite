# Generates the fake data archive used as an example by Thermite.
#
# Single date: 17/10/2026. One record per hour, from 00:00:00 to 24:00:00
# (25 rows: the final midnight closes the day, as in ISO 8601).
# Four sensors with random values between 2 and 8 degrees.
#
# The separator and the decimal comma are those of a European archive,
# so the file also serves to test the automatic detection of the
# separator.
#
# Fixed seed: regenerating the file always yields the same content,
# otherwise every run would rewrite the file and create useless
# differences in the repository.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\genera_dati_fasulli.ps1

param(
    [string]$Uscita = "",
    [int]$Seme = 20261017,
    [int]$Sensori = 4,
    [double]$Min = 2.0,
    [double]$Max = 8.0,
    [string]$Data = "17/10/2026"
)

# Default path next to this script, so it works from any
# working directory.
if ([string]::IsNullOrWhiteSpace($Uscita)) {
    $cartella = Split-Path -Parent $MyInvocation.MyCommand.Path
    $Uscita = Join-Path $cartella "data\example_data.csv"
}

$rng = New-Object System.Random($Seme)

# Header: column 1 date, column 2 time, then one series per sensor.
$nomi = @("Data", "Ora") + (1..$Sensori | ForEach-Object { "Sensore_$_" })
$righe = New-Object System.Collections.Generic.List[string]
$righe.Add(($nomi -join ";"))

# 0..24 = 25 records, includes both 00:00:00 and 24:00:00.
foreach ($h in 0..24) {
    $ora = "{0:D2}:00:00" -f $h
    $valori = for ($s = 0; $s -lt $Sensori; $s++) {
        $v = $Min + ($rng.NextDouble() * ($Max - $Min))
        # Decimal comma, as in European archives.
        ("{0:F1}" -f $v).Replace('.', ',')
    }
    $righe.Add((@($Data, $ora) + $valori) -join ";")
}

Set-Content -Path $Uscita -Value $righe -Encoding UTF8

"scritte: " + $righe.Count + " righe (1 intestazione + 25 dati)"
"sensori: " + $Sensori + "  intervallo: $Min-$Max gradi"
"file   : " + $Uscita
