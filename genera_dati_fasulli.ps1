# Genera l'archivio di dati fasulli usato come esempio da Thermite.
#
# Data unica: 17/10/2026. Un registro per ora, da 00:00:00 a 24:00:00
# (25 righe: la mezzanotte finale chiude la giornata, come in ISO 8601).
# Quattro sensori con valori casuali fra 2 e 8 gradi.
#
# Il separatore e la virgola decimale sono quelli di un archivio europeo,
# così il file serve anche a provare il rilevamento automatico del
# separatore.
#
# Seme fisso: rigenerando il file si ottiene sempre lo stesso contenuto,
# altrimenti ogni esecuzione riscriverebbe il file e creerebbe differenze
# inutili nel repository.
#
# Uso:
#   powershell -ExecutionPolicy Bypass -File .\genera_dati_fasulli.ps1

param(
    [string]$Uscita = "",
    [int]$Seme = 20261017,
    [int]$Sensori = 4,
    [double]$Min = 2.0,
    [double]$Max = 8.0,
    [string]$Data = "17/10/2026"
)

# Percorso di default accanto a questo script, così funziona da qualunque
# cartella di lavoro.
if ([string]::IsNullOrWhiteSpace($Uscita)) {
    $cartella = Split-Path -Parent $MyInvocation.MyCommand.Path
    $Uscita = Join-Path $cartella "data\dati_fasulli_17-10-2026.csv"
}

$rng = New-Object System.Random($Seme)

# Intestazione: colonna 1 data, colonna 2 ora, poi una serie per sensore.
$nomi = @("Data", "Ora") + (1..$Sensori | ForEach-Object { "Sensore_$_" })
$righe = New-Object System.Collections.Generic.List[string]
$righe.Add(($nomi -join ";"))

# 0..24 = 25 registrazioni, comprende sia 00:00:00 sia 24:00:00.
foreach ($h in 0..24) {
    $ora = "{0:D2}:00:00" -f $h
    $valori = for ($s = 0; $s -lt $Sensori; $s++) {
        $v = $Min + ($rng.NextDouble() * ($Max - $Min))
        # Virgola decimale, come negli archivi europei.
        ("{0:F1}" -f $v).Replace('.', ',')
    }
    $righe.Add((@($Data, $ora) + $valori) -join ";")
}

Set-Content -Path $Uscita -Value $righe -Encoding UTF8

"scritte: " + $righe.Count + " righe (1 intestazione + 25 dati)"
"sensori: " + $Sensori + "  intervallo: $Min-$Max gradi"
"file   : " + $Uscita
