# serve.ps1 - Petit serveur web en PowerShell
param(
    [int]$Port = 4000,
    [string]$Root = "."
)

Add-Type -AssemblyName System.Net.HttpListener
$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$Port/")
$listener.Start()
Write-Host "Serveur démarré sur http://localhost:$Port (racine: $Root)"

while ($listener.IsListening) {
    $ctx = $listener.GetContext()
    $req = $ctx.Request
    $res = $ctx.Response

    $path = $req.Url.LocalPath.TrimStart("/")
    if ([string]::IsNullOrWhiteSpace($path)) { $path = "index.html" }

    $file = Join-Path $Root $path
    if (Test-Path $file) {
        $bytes = [System.IO.File]::ReadAllBytes($file)
        # Détection simple du type MIME
        switch -regex ($file) {
            "\.html$" { $res.ContentType = "text/html; charset=utf-8" }
            "\.wasm$" { $res.ContentType = "application/wasm" }
            "\.js$"   { $res.ContentType = "application/javascript" }
            "\.css$"  { $res.ContentType = "text/css" }
            default   { $res.ContentType = "application/octet-stream" }
        }
        $res.OutputStream.Write($bytes, 0, $bytes.Length)
    } else {
        $res.StatusCode = 404
        $err = [Text.Encoding]::UTF8.GetBytes("404 Not Found")
        $res.OutputStream.Write($err, 0, $err.Length)
    }
    $res.Close()
}
