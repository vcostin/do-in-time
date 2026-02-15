# Convert icon-design.svg to all required Tauri icon formats
#
# Prerequisites:
#   Install ImageMagick: https://imagemagick.org/script/download.php
#   Windows: choco install imagemagick (or download installer)
#   Or use: winget install ImageMagick.ImageMagick

Write-Host "Converting icon-design.svg to all required formats..." -ForegroundColor Green

$svgFile = "icon-design.svg"
$iconDir = "src-tauri/icons"

# Check if ImageMagick is available
$magickCmd = Get-Command magick -ErrorAction SilentlyContinue
if (-not $magickCmd) {
    Write-Host "ERROR: ImageMagick not found!" -ForegroundColor Red
    Write-Host "Please install ImageMagick first:" -ForegroundColor Yellow
    Write-Host "  choco install imagemagick" -ForegroundColor Yellow
    Write-Host "  OR download from: https://imagemagick.org/script/download.php" -ForegroundColor Yellow
    exit 1
}

Write-Host "Creating PNG files..." -ForegroundColor Cyan

# Standard sizes
magick $svgFile -resize 32x32 -background none "$iconDir/32x32.png"
magick $svgFile -resize 128x128 -background none "$iconDir/128x128.png"
magick $svgFile -resize 256x256 -background none "$iconDir/128x128@2x.png"
magick $svgFile -resize 512x512 -background none "$iconDir/icon.png"

# Windows Store logos
magick $svgFile -resize 30x30 -background none "$iconDir/Square30x30Logo.png"
magick $svgFile -resize 44x44 -background none "$iconDir/Square44x44Logo.png"
magick $svgFile -resize 71x71 -background none "$iconDir/Square71x71Logo.png"
magick $svgFile -resize 89x89 -background none "$iconDir/Square89x89Logo.png"
magick $svgFile -resize 107x107 -background none "$iconDir/Square107x107Logo.png"
magick $svgFile -resize 142x142 -background none "$iconDir/Square142x142Logo.png"
magick $svgFile -resize 150x150 -background none "$iconDir/Square150x150Logo.png"
magick $svgFile -resize 284x284 -background none "$iconDir/Square284x284Logo.png"
magick $svgFile -resize 310x310 -background none "$iconDir/Square310x310Logo.png"
magick $svgFile -resize 44x44 -background none "$iconDir/StoreLogo.png"

Write-Host "Creating ICO file (Windows)..." -ForegroundColor Cyan
magick $svgFile -define icon:auto-resize=256,128,96,64,48,32,16 -background none "$iconDir/icon.ico"

Write-Host "Creating ICNS file (macOS)..." -ForegroundColor Cyan
# Create temporary iconset directory
$iconsetDir = "icon.iconset"
New-Item -ItemType Directory -Force -Path $iconsetDir | Out-Null

# Generate all required macOS icon sizes
magick $svgFile -resize 16x16 -background none "$iconsetDir/icon_16x16.png"
magick $svgFile -resize 32x32 -background none "$iconsetDir/icon_16x16@2x.png"
magick $svgFile -resize 32x32 -background none "$iconsetDir/icon_32x32.png"
magick $svgFile -resize 64x64 -background none "$iconsetDir/icon_32x32@2x.png"
magick $svgFile -resize 128x128 -background none "$iconsetDir/icon_128x128.png"
magick $svgFile -resize 256x256 -background none "$iconsetDir/icon_128x128@2x.png"
magick $svgFile -resize 256x256 -background none "$iconsetDir/icon_256x256.png"
magick $svgFile -resize 512x512 -background none "$iconsetDir/icon_256x256@2x.png"
magick $svgFile -resize 512x512 -background none "$iconsetDir/icon_512x512.png"
magick $svgFile -resize 1024x1024 -background none "$iconsetDir/icon_512x512@2x.png"

# Convert to ICNS (requires iconutil on macOS or png2icns on Windows)
if (Get-Command iconutil -ErrorAction SilentlyContinue) {
    iconutil -c icns $iconsetDir -o "$iconDir/icon.icns"
} elseif (Get-Command png2icns -ErrorAction SilentlyContinue) {
    png2icns "$iconDir/icon.icns" "$iconsetDir/*.png"
} else {
    Write-Host "Note: Could not create ICNS file automatically." -ForegroundColor Yellow
    Write-Host "On macOS, run: iconutil -c icns $iconsetDir -o $iconDir/icon.icns" -ForegroundColor Yellow
    Write-Host "Or upload PNG files to: https://cloudconvert.com/png-to-icns" -ForegroundColor Yellow
}

# Clean up
Remove-Item -Recurse -Force $iconsetDir -ErrorAction SilentlyContinue

Write-Host "`nDone! All icons generated in $iconDir" -ForegroundColor Green
Write-Host "`nIcon files created:" -ForegroundColor Cyan
Get-ChildItem $iconDir | ForEach-Object { Write-Host "  $($_.Name)" }
