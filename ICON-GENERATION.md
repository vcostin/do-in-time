# Icon Generation Guide

This guide explains how to convert the `icon-design.svg` to all required formats for the Tauri application.

## Option 1: Automated Script (Recommended)

### Windows

1. **Install ImageMagick:**
   ```powershell
   # Using Chocolatey
   choco install imagemagick

   # OR using winget
   winget install ImageMagick.ImageMagick

   # OR download installer from: https://imagemagick.org/script/download.php
   ```

2. **Run the conversion script:**
   ```powershell
   .\generate-icons.ps1
   ```

### macOS/Linux

1. **Install ImageMagick:**
   ```bash
   # macOS
   brew install imagemagick

   # Ubuntu/Debian
   sudo apt-get install imagemagick

   # Fedora
   sudo dnf install imagemagick
   ```

2. **Run conversion commands:**
   ```bash
   # Make script executable (if using bash version)
   chmod +x generate-icons.sh
   ./generate-icons.sh
   ```

## Option 2: Online Tools (No Installation Required)

If you prefer not to install ImageMagick, use these online tools:

### 1. Convert SVG to PNG sizes
**Tool:** https://cloudconvert.com/svg-to-png
- Upload `icon-design.svg`
- Set each required size (32x32, 128x128, 256x256, 512x512, etc.)
- Download and rename files according to list below

### 2. Create ICO file (Windows icon)
**Tool:** https://cloudconvert.com/png-to-ico
- Upload the 512x512 PNG
- Download as `icon.ico`

### 3. Create ICNS file (macOS icon)
**Tool:** https://cloudconvert.com/png-to-icns
- Upload multiple PNG sizes (16, 32, 64, 128, 256, 512, 1024)
- Download as `icon.icns`

### 4. Alternative: Icon Kitchen
**Tool:** https://icon.kitchen/
- Upload `icon-design.svg`
- Select "Desktop" platform
- Generates all required sizes automatically
- Download and extract to `src-tauri/icons/`

## Required Icon Files

Place all generated icons in `src-tauri/icons/`:

### Standard Icons
- `32x32.png` - 32×32 pixels
- `128x128.png` - 128×128 pixels
- `128x128@2x.png` - 256×256 pixels (2x retina)
- `icon.png` - 512×512 pixels (main icon)

### Windows
- `icon.ico` - Multi-size ICO file
- `Square30x30Logo.png`
- `Square44x44Logo.png`
- `Square71x71Logo.png`
- `Square89x89Logo.png`
- `Square107x107Logo.png`
- `Square142x142Logo.png`
- `Square150x150Logo.png`
- `Square284x284Logo.png`
- `Square310x310Logo.png`
- `StoreLogo.png` - 44×44 pixels

### macOS
- `icon.icns` - ICNS file (contains multiple sizes)

## Verification

After generating icons, verify they exist:

```bash
# Windows PowerShell
ls src-tauri/icons/

# macOS/Linux
ls -lh src-tauri/icons/
```

You should see all the files listed above.

## Troubleshooting

### ImageMagick not found after installation
- **Windows**: Restart PowerShell/Terminal after installation
- **Check PATH**: Ensure ImageMagick is in your system PATH
- **Test**: Run `magick -version` to verify installation

### ICNS creation fails on Windows
- Use online tool: https://cloudconvert.com/png-to-icns
- Or skip for now - ICNS is only needed for macOS builds

### Icons look blurry
- Ensure SVG is clean (no errors)
- Use `-background none` to preserve transparency
- Check source SVG resolution is adequate

## Manual Conversion Commands

If the script doesn't work, run these commands manually:

```bash
# Basic PNG sizes
magick icon-design.svg -resize 32x32 -background none src-tauri/icons/32x32.png
magick icon-design.svg -resize 128x128 -background none src-tauri/icons/128x128.png
magick icon-design.svg -resize 256x256 -background none src-tauri/icons/128x128@2x.png
magick icon-design.svg -resize 512x512 -background none src-tauri/icons/icon.png

# ICO file (Windows)
magick icon-design.svg -define icon:auto-resize=256,128,96,64,48,32,16 -background none src-tauri/icons/icon.ico
```
