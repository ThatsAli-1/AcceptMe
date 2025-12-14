# PowerShell script to create minimal placeholder icons for Tauri
# This creates simple colored square PNG files

$iconDir = "src-tauri\icons"
if (-not (Test-Path $iconDir)) {
    New-Item -ItemType Directory -Path $iconDir -Force | Out-Null
}

# Create a simple base64 encoded 1x1 transparent PNG and resize it
# Actually, let's use a different approach - download or create using .NET

Add-Type -AssemblyName System.Drawing

$sizes = @(32, 128, 256, 384, 512)
$color = [System.Drawing.Color]::FromArgb(102, 126, 234)

foreach ($size in $sizes) {
    $bitmap = New-Object System.Drawing.Bitmap($size, $size)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.Clear($color)
    
    # Add a simple "A" text
    $font = New-Object System.Drawing.Font("Arial", ($size / 4), [System.Drawing.FontStyle]::Bold)
    $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = [System.Drawing.StringAlignment]::Center
    $format.LineAlignment = [System.Drawing.StringAlignment]::Center
    $graphics.DrawString("A", $font, $brush, ($size/2), ($size/2), $format)
    
    $bitmap.Save("$iconDir\$size`x$size.png", [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()
}

# Create the @2x version (256x256 for 128@2x)
$bitmap = New-Object System.Drawing.Bitmap(256, 256)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.Clear($color)
$font = New-Object System.Drawing.Font("Arial", 64, [System.Drawing.FontStyle]::Bold)
$brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$format = New-Object System.Drawing.StringFormat
$format.Alignment = [System.Drawing.StringAlignment]::Center
$format.LineAlignment = [System.Drawing.StringAlignment]::Center
$graphics.DrawString("A", $font, $brush, 128, 128, $format)
$bitmap.Save("$iconDir\128x128@2x.png", [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()

# Create icon.ico for Windows
$icoSizes = @(16, 32, 48, 256)
$icoBitmap = New-Object System.Drawing.Bitmap(256, 256)
$icoGraphics = [System.Drawing.Graphics]::FromImage($icoBitmap)
$icoGraphics.Clear($color)
$icoFont = New-Object System.Drawing.Font("Arial", 64, [System.Drawing.FontStyle]::Bold)
$icoBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$icoFormat = New-Object System.Drawing.StringFormat
$icoFormat.Alignment = [System.Drawing.StringAlignment]::Center
$icoFormat.LineAlignment = [System.Drawing.StringAlignment]::Center
$icoGraphics.DrawString("A", $icoFont, $icoBrush, 128, 128, $icoFormat)
$icoBitmap.Save("$iconDir\icon.ico", [System.Drawing.Imaging.ImageFormat]::Icon)
$icoGraphics.Dispose()
$icoBitmap.Dispose()

Write-Host "Icons created successfully in $iconDir"

