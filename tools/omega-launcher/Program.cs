using System.Diagnostics;
using System.Text.Json;

namespace OmegaLauncher;

internal static class Program
{
    [STAThread]
    private static void Main()
    {
        ApplicationConfiguration.Initialize();
        Application.Run(new Launcher());
    }
}

internal sealed class Launcher : Form
{
    private const string GameExe = "game_scarlett_master.exe";
    private static readonly Color Ink = Color.FromArgb(241, 234, 246);
    private static readonly Color Muted = Color.FromArgb(191, 177, 202);
    private static readonly Color Panel = Color.FromArgb(36, 26, 47);
    private static readonly Color Accent = Color.FromArgb(255, 129, 183);
    private static readonly string StateDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Omega Launcher");
    private static readonly string StateFile = Path.Combine(StateDir, "settings.json");
    private readonly string? repo;
    private readonly TextBox folder = new();
    private readonly ComboBox profile = new();
    private readonly CheckBox rays = new() { Dock = DockStyle.Fill, AutoEllipsis = true };
    private readonly CheckBox waterWave64 = new() { Dock = DockStyle.Fill, AutoEllipsis = true };
    private readonly Label status = new();
    private readonly Button play = new();
    private readonly Image logo;
    private bool launchPending;

    public Launcher()
    {
        using var art = typeof(Launcher).Assembly.GetManifestResourceStream("omega.png")!;
        using var decoded = Image.FromStream(art);
        logo = new Bitmap(decoded);
        Icon = Icon.ExtractAssociatedIcon(Application.ExecutablePath);
        repo = FindCheckout();

        Text = "Omega Launcher";
        ClientSize = new Size(760, 496);
        MinimumSize = new Size(650, 530);
        StartPosition = FormStartPosition.CenterScreen;
        BackColor = Color.FromArgb(22, 16, 30);
        ForeColor = Ink;
        Font = new Font("Segoe UI", 10);

        var layout = new TableLayoutPanel
        {
            Dock = DockStyle.Fill, Padding = new Padding(28, 22, 28, 20),
            ColumnCount = 1, RowCount = 4
        };
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 120));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 130));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 126));
        layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
        Controls.Add(layout);
        layout.Controls.Add(Hero(), 0, 0);
        layout.Controls.Add(FolderCard(), 0, 1);
        layout.Controls.Add(ProfileCard(), 0, 2);
        layout.Controls.Add(Footer(), 0, 3);

        var saved = ReadSettings();
        folder.Text = saved.Folder ?? FindLocalGame() ?? "";
        profile.SelectedIndex = saved.SeriesX ? 1 : 0;
        rays.Checked = saved.RayTracing;
        waterWave64.Checked = saved.WaterWave64;
        folder.TextChanged += (_, _) => RefreshStatus();
        profile.SelectedIndexChanged += (_, _) => RefreshStatus();
        rays.CheckedChanged += (_, _) => RefreshStatus();
        RefreshStatus();
    }

    private Control Hero()
    {
        var hero = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1 };
        hero.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 96));
        hero.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
        hero.Controls.Add(new PictureBox
        {
            Image = logo, SizeMode = PictureBoxSizeMode.Zoom,
            Dock = DockStyle.Fill, Margin = new Padding(0, 4, 16, 20)
        }, 0, 0);
        var copy = new FlowLayoutPanel
        {
            FlowDirection = FlowDirection.TopDown, WrapContents = false,
            Dock = DockStyle.Fill, Padding = new Padding(0, 10, 0, 0)
        };
        var name = LabelFor("Omega Launcher", 28, Ink, bold: true);
        copy.Controls.Add(name);
        copy.Controls.Add(LabelFor("Choose your game folder, pick a profile, then launch.", 10, Muted));
        hero.Controls.Add(copy, 1, 0);
        return hero;
    }

    private Control FolderCard()
    {
        var card = Card();
        card.Controls.Add(LabelFor("1. Game folder", 13, Ink, bold: true), 0, 0);
        card.SetColumnSpan(card.GetControlFromPosition(0, 0)!, 2);
        var help = LabelFor("Choose your own Xbox game folder containing " + GameExe + ".", 9, Muted);
        card.Controls.Add(help, 0, 1);
        card.SetColumnSpan(help, 2);
        folder.PlaceholderText = @"D:\Games\MyDump";
        folder.Dock = DockStyle.Fill;
        folder.BackColor = Color.FromArgb(24, 18, 34);
        folder.ForeColor = Ink;
        folder.BorderStyle = BorderStyle.FixedSingle;
        folder.Margin = new Padding(3, 6, 10, 0);
        card.Controls.Add(folder, 0, 2);
        var browse = MakeButton("Browse…", false);
        browse.Click += (_, _) =>
        {
            using var dialog = new FolderBrowserDialog
            {
                Description = "Choose the folder containing " + GameExe,
                UseDescriptionForTitle = true,
                SelectedPath = Directory.Exists(folder.Text) ? folder.Text : ""
            };
            if (dialog.ShowDialog(this) == DialogResult.OK)
                folder.Text = dialog.SelectedPath;
        };
        card.Controls.Add(browse, 1, 2);
        return card;
    }

    private Control ProfileCard()
    {
        var card = Card();
        card.Controls.Add(LabelFor("2. Rendering profile", 13, Ink, bold: true), 0, 0);
        var help = LabelFor("Both profiles output at up to 1080p. Series X renders at higher internal resolutions.", 9, Muted);
        card.Controls.Add(help, 0, 1);
        card.SetColumnSpan(help, 2);
        profile.Items.AddRange(["Series S · 1080p", "Series X · 1080p output"]);
        profile.DropDownStyle = ComboBoxStyle.DropDownList;
        profile.Dock = DockStyle.Fill;
        profile.Margin = new Padding(3, 6, 10, 0);
        profile.BackColor = Color.FromArgb(24, 18, 34);
        profile.ForeColor = Ink;
        card.Controls.Add(profile, 0, 2);
        rays.Text = "Ray tracing";
        rays.AccessibleDescription = "Experimental. Leave off if you see rendering artifacts.";
        waterWave64.Text = "Water Wave64";
        waterWave64.AccessibleDescription = "Experimental paired-wave translation for non-tiled water light.";
        card.ColumnStyles[0].Width = 52;
        card.ColumnStyles[1].SizeType = SizeType.Percent;
        card.ColumnStyles[1].Width = 48;
        var toggles = new TableLayoutPanel
        {
            Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty
        };
        toggles.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
        toggles.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
        toggles.Controls.Add(rays, 0, 0);
        toggles.Controls.Add(waterWave64, 1, 0);
        card.Controls.Add(toggles, 1, 2);
        return card;
    }

    private Control Footer()
    {
        var footer = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1 };
        footer.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
        footer.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 142));
        status.Dock = DockStyle.Fill;
        status.AutoEllipsis = true;
        status.ForeColor = Muted;
        status.TextAlign = ContentAlignment.MiddleLeft;
        footer.Controls.Add(status, 0, 0);
        StyleButton(play, "Launch game", true);
        play.Click += (_, _) => Launch();
        footer.Controls.Add(play, 1, 0);
        AcceptButton = play;
        return footer;
    }

    private static TableLayoutPanel Card()
    {
        var card = new TableLayoutPanel
        {
            Dock = DockStyle.Fill, Margin = new Padding(0, 0, 0, 12),
            Padding = new Padding(13, 8, 13, 8), ColumnCount = 2,
            RowCount = 3, BackColor = Panel
        };
        card.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
        card.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 112));
        card.RowStyles.Add(new RowStyle(SizeType.Absolute, 30));
        card.RowStyles.Add(new RowStyle(SizeType.Absolute, 25));
        card.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
        return card;
    }

    private static Label LabelFor(string text, float size, Color color, bool bold = false) =>
        new() { Text = text, AutoSize = true, ForeColor = color, Font = new Font("Segoe UI", size, bold ? FontStyle.Bold : FontStyle.Regular) };

    private static Button MakeButton(string text, bool primary)
    {
        var button = new Button();
        StyleButton(button, text, primary);
        return button;
    }

    private static void StyleButton(Button button, string text, bool primary)
    {
        button.Text = text;
        button.FlatStyle = FlatStyle.Flat;
        button.FlatAppearance.BorderSize = 0;
        button.BackColor = primary ? Accent : Color.FromArgb(67, 44, 82);
        button.ForeColor = primary ? Color.FromArgb(36, 13, 30) : Ink;
        button.Dock = DockStyle.Fill;
        button.Margin = new Padding(3, 3, 0, 3);
        button.Cursor = Cursors.Hand;
    }

    private string? GamePath()
    {
        string value = folder.Text.Trim();
        if (value.Length == 0)
            return null;
        var exe = Path.Combine(value, GameExe);
        return File.Exists(exe) ? Path.GetFullPath(exe) : null;
    }

    private string? BuildProfile() => repo is null ? null :
        new[] { "release", "diag" }.FirstOrDefault(p => File.Exists(Path.Combine(repo, "target", p, "victory.exe")));

    private void RefreshStatus()
    {
        bool ready = repo is not null && BuildProfile() is not null && GamePath() is not null;
        play.Enabled = ready && !launchPending;
        if (launchPending)
            return;
        status.Text = repo is null ? "Place Omega Launcher inside the Victory Omega checkout."
            : BuildProfile() is null ? "Build Victory Omega before launching the game."
            : GamePath() is null ? "Select the folder with the game executable."
            : rays.Checked ? "Ready. Ray tracing is experimental."
            : "Ready to launch.";
    }

    private void Launch()
    {
        var exe = GamePath();
        var build = BuildProfile();
        if (exe is null || build is null || repo is null)
            return;

        var active = Process.GetProcessesByName("victory");
        bool running = active.Length != 0;
        foreach (var process in active)
            process.Dispose();
        if (running)
        {
            status.Text = "Victory is already running. Close it before starting another profile.";
            return;
        }

        try
        {
            Directory.CreateDirectory(StateDir);
            var selection = new Settings(Path.GetDirectoryName(exe), profile.SelectedIndex == 1, rays.Checked, waterWave64.Checked);
            File.WriteAllText(StateFile, JsonSerializer.Serialize(selection));
            var start = new ProcessStartInfo("powershell.exe")
            {
                WorkingDirectory = repo, UseShellExecute = false, CreateNoWindow = true,
                RedirectStandardOutput = true, RedirectStandardError = true
            };
            string[] arguments = ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "RemoteSigned",
                "-File", Path.Combine(repo, "scripts", "play.ps1"), "-Exe", exe,
                "-Console", selection.SeriesX ? "series-x" : "series-s", "-Build", build];
            foreach (string argument in arguments)
                start.ArgumentList.Add(argument);
            if (selection.RayTracing)
                start.ArgumentList.Add("-RayTracing");
            if (selection.WaterWave64)
                start.ArgumentList.Add("-WaterWave64");
            UseLocalData(start);
            TrackRun(start);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or System.ComponentModel.Win32Exception)
        {
            status.Text = "Could not start Victory. " + e.Message;
        }
    }

    private async void TrackRun(ProcessStartInfo start)
    {
        launchPending = true;
        play.Enabled = false;
        status.Text = "Starting Victory Omega…";
        try
        {
            using var stdout = File.Create(Path.Combine(StateDir, "game-stdout.log"));
            using var stderr = File.Create(Path.Combine(StateDir, "game-stderr.log"));
            using var run = Process.Start(start) ?? throw new IOException("Could not start PowerShell.");
            await Task.WhenAll(run.StandardOutput.BaseStream.CopyToAsync(stdout),
                run.StandardError.BaseStream.CopyToAsync(stderr), run.WaitForExitAsync());
            launchPending = false;
            RefreshStatus();
            status.Text = run.ExitCode == 0 ? "Victory exited." : $"Victory exited with code {run.ExitCode}. See game-stderr.log in {StateDir}.";
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or System.ComponentModel.Win32Exception)
        {
            launchPending = false;
            RefreshStatus();
            status.Text = "Could not start Victory. " + e.Message;
        }
    }

    private static string? FindCheckout()
    {
        for (var dir = new DirectoryInfo(AppContext.BaseDirectory); dir is not null; dir = dir.Parent)
            if (File.Exists(Path.Combine(dir.FullName, "scripts", "play.ps1")))
                return dir.FullName;
        return null;
    }

    private string? LocalRoot()
    {
        string? root = repo is null ? null : Directory.GetParent(repo)?.FullName;
        return root is not null && File.Exists(Path.Combine(root, "data", "last-game-executable.txt")) ? root : null;
    }

    private string? FindLocalGame()
    {
        var root = LocalRoot();
        try
        {
            return root is null ? null : Path.GetDirectoryName(
                File.ReadAllText(Path.Combine(root, "data", "last-game-executable.txt")).Trim());
        }
        catch (IOException) { return null; }
    }

    private void UseLocalData(ProcessStartInfo start)
    {
        var root = LocalRoot();
        if (root is null)
            return;
        start.Environment.TryAdd("VICTORY_DATA_DIR", Path.Combine(root, "data"));
        start.Environment.TryAdd("VICTORY_CACHE_DIR", Path.Combine(root, "cache", "cache-coolfox"));
    }

    private static Settings ReadSettings()
    {
        try
        {
            return File.Exists(StateFile)
                ? JsonSerializer.Deserialize<Settings>(File.ReadAllText(StateFile)) ?? new()
                : new();
        }
        catch (IOException) { return new(); }
        catch (JsonException) { return new(); }
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
            logo.Dispose();
        base.Dispose(disposing);
    }

    private sealed record Settings(string? Folder = null, bool SeriesX = false, bool RayTracing = false, bool WaterWave64 = false);
}
