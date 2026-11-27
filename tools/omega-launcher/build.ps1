$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$obj = Join-Path $PSScriptRoot 'obj'
$null = New-Item -ItemType Directory -Path $obj -Force
Add-Type -AssemblyName System.Drawing
$bitmap = [Drawing.Bitmap]::new(256, 256)
$graphics = [Drawing.Graphics]::FromImage($bitmap)
$graphics.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
$background = [Drawing.Drawing2D.LinearGradientBrush]::new(
    [Drawing.Rectangle]::new(0, 0, 256, 256),
    [Drawing.Color]::FromArgb(86, 48, 153), [Drawing.Color]::FromArgb(32, 20, 55), 52)
$outline = [Drawing.Pen]::new([Drawing.Color]::FromArgb(210, 128, 247), 7)
$glyph = [Drawing.Drawing2D.GraphicsPath]::new()
$center = [Drawing.Drawing2D.Matrix]::new()
$face = [Drawing.FontFamily]::new('Segoe UI')
$glow = [Drawing.Drawing2D.LinearGradientBrush]::new(
    [Drawing.Rectangle]::new(0, 42, 256, 173),
    [Drawing.Color]::FromArgb(182, 255, 245), [Drawing.Color]::FromArgb(86, 206, 244), 90)
try {
    $graphics.FillEllipse($background, 9, 9, 238, 238)
    $graphics.DrawEllipse($outline, 9, 9, 238, 238)
    $glyph.AddString([string][char]0x03A9, $face, [int][Drawing.FontStyle]::Bold,
        192, [Drawing.PointF]::Empty, [Drawing.StringFormat]::GenericTypographic)
    $bounds = $glyph.GetBounds()
    $center.Translate(128 - $bounds.X - $bounds.Width / 2, 128 - $bounds.Y - $bounds.Height / 2)
    $glyph.Transform($center)
    $graphics.FillPath($glow, $glyph)
    $png = Join-Path $obj 'launcher.png'
    $bitmap.Save($png, [Drawing.Imaging.ImageFormat]::Png)
    $art = [IO.File]::ReadAllBytes($png)
} finally {
    $glow.Dispose()
    $face.Dispose()
    $center.Dispose()
    $glyph.Dispose()
    $outline.Dispose()
    $background.Dispose()
    $graphics.Dispose()
    $bitmap.Dispose()
}
# ICO directory for one 256×256 PNG-encoded image.
$bytes = [IO.MemoryStream]::new()
$writer = [IO.BinaryWriter]::new($bytes)
try {
    $writer.Write([uint16]0)
    $writer.Write([uint16]1)
    $writer.Write([uint16]1)
    $writer.Write([uint32]0)
    $writer.Write([uint16]1)
    $writer.Write([uint16]32)
    $writer.Write([uint32]$art.Length)
    $writer.Write([uint32]22)
    $writer.Write($art)
    [IO.File]::WriteAllBytes((Join-Path $obj 'omega.ico'), $bytes.ToArray())
} finally {
    $writer.Dispose()
}
dotnet publish (Join-Path $PSScriptRoot 'OmegaLauncher.csproj') -c Release -r win-x64 --self-contained false -p:PublishSingleFile=true -p:NuGetAudit=false --ignore-failed-sources -o (Join-Path $repo 'target/omega-launcher')
exit $LASTEXITCODE