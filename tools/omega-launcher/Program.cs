using System.Diagnostics;
using System.Drawing.Drawing2D;
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
    private static readonly Color Panel = Color.FromArgb(35, 29, 51);
    private static readonly Color Accent = Color.FromArgb(140, 232, 225);
    private static readonly string StateDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Omega Launcher");
    private static readonly string StateFile = Path.Combine(StateDir, "settings.json");
    private readonly string? repo;
    private readonly TextBox folder = new();
    private readonly ComboBox profile = new();
    private readonly CheckBox rays = new();
    private readonly CheckBox original = new();
    private readonly CheckBox shadowBoost = new();
    private readonly CheckBox memory = new();
    private readonly CheckBox diagnostics = new();
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
        ClientSize = new Size(780, 590);
        MinimumSize = new Size(700, 615);
        StartPosition = FormStartPosition.CenterScreen;
        BackColor = Color.FromArgb(22, 16, 30);
        ForeColor = Ink;
        Font = new Font("Segoe UI", 10);

        var layout = new TableLayoutPanel
        {
            Dock = DockStyle.Fill, Padding = new Padding(28, 22, 28, 20),
            ColumnCount = 1, RowCount = 5
        };
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 104));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 130));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 126));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 108));
        layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
        Controls.Add(layout);
        layout.Controls.Add(Hero(), 0, 0);
        layout.Controls.Add(FolderCard(), 0, 1);
        layout.Controls.Add(ProfileCard(), 0, 2);
        layout.Controls.Add(OptionsCard(), 0, 3);
        layout.Controls.Add(Footer(), 0, 4);

        var saved = ReadSettings();
        folder.Text = saved.Folder ?? FindLocalGame() ?? "";
        profile.SelectedIndex = saved.SeriesX ? 1 : 0;
        rays.Checked = saved.RayTracing;
        original.Checked = saved.OriginalGraphics;
        shadowBoost.Checked = saved.ShadowBoost;
        shadowBoost.AccessibleDescription = "Lifts dark areas when ray tracing and Original graphics are off.";
        memory.AccessibleDescription = "Uses smaller memory windows for buffer copies.";
        memory.Checked = saved.LowMemory;
        diagnostics.Checked = saved.Diagnostics;
        diagnostics.CheckedChanged += (_, _) => RefreshStatus();
        folder.TextChanged += (_, _) => RefreshStatus();
        profile.SelectedIndexChanged += (_, _) => RefreshStatus();
        rays.CheckedChanged += (_, _) => RefreshStatus();
        original.CheckedChanged += (_, _) => RefreshStatus();
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
        copy.Controls.Add(LabelFor("Configure your installed game and launch profile.", 10, Muted));
        hero.Controls.Add(copy, 1, 0);
        return hero;
    }

    private Control FolderCard()
    {
        var card = Card();
        card.Controls.Add(LabelFor("Game library", 13, Ink, bold: true), 0, 0);
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
        card.Controls.Add(LabelFor("Rendering profile", 13, Ink, bold: true), 0, 0);
        var help = LabelFor("Series S targets 1080p. Series X may render at a higher internal resolution.", 9, Muted);
        card.Controls.Add(help, 0, 1);
        card.SetColumnSpan(help, 2);
        profile.Items.AddRange(["Series S · 1080p", "Series X"]);
        profile.DropDownStyle = ComboBoxStyle.DropDownList;
        profile.Dock = DockStyle.Fill;
        profile.Margin = new Padding(3, 6, 10, 0);
        profile.BackColor = Color.FromArgb(24, 18, 34);
        profile.ForeColor = Ink;
        card.Controls.Add(profile, 0, 2);
        rays.Text = "Ray tracing";
        rays.AccessibleDescription = "Experimental. Leave off if you see rendering artifacts.";
        rays.Dock = DockStyle.Fill;
        card.Controls.Add(rays, 1, 2);
        return card;
    }

    private Control OptionsCard()
    {
        var card = Card();
        var title = LabelFor("Launch options", 13, Ink, bold: true);
        card.Controls.Add(title, 0, 0);
        card.SetColumnSpan(title, 2);
        var options = new FlowLayoutPanel { Dock = DockStyle.Fill, WrapContents = true };
        foreach (var (check, label) in new[] {
            (original, "Original graphics"), (shadowBoost, "Shadow boost"), (memory, "Low-memory mode"),
            (diagnostics, "FPS HUD and logs") })
        {
            check.Text = label;
            check.AutoSize = true;
            check.Margin = new Padding(3, 4, 17, 2);
            options.Controls.Add(check);
        }
        card.Controls.Add(options, 0, 1);
        card.SetColumnSpan(options, 2);
        card.SetRowSpan(options, 2);
        return card;
    }

    private Control Footer()
    {
        var footer = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 3, RowCount = 1 };
        footer.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
        footer.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 108));
        footer.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 142));
        status.Dock = DockStyle.Fill;
        status.ForeColor = Muted;
        status.TextAlign = ContentAlignment.MiddleLeft;
        footer.Controls.Add(status, 0, 0);
        StyleButton(play, "Launch game", true);
        play.Click += (_, _) => Launch();
        var logs = MakeButton("Open logs", false);
        logs.Click += (_, _) =>
        {
            try
            {
                Directory.CreateDirectory(StateDir);
                Process.Start(new ProcessStartInfo(StateDir) { UseShellExecute = true })?.Dispose();
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or System.ComponentModel.Win32Exception)
            {
                status.Text = "Could not open logs. " + e.Message;
            }
        };
        footer.Controls.Add(logs, 1, 0);
        footer.Controls.Add(play, 2, 0);
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
        RoundCorners(card, 16);
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
        button.ForeColor = primary ? Color.FromArgb(12, 39, 43) : Ink;
        button.Dock = DockStyle.Fill;
        button.Margin = new Padding(3, 3, 0, 3);
        button.Cursor = Cursors.Hand;
        RoundCorners(button, 10);
    }

    private static void RoundCorners(Control control, int radius)
    {
        void Update(object? sender, EventArgs e)
        {
            int diameter = Math.Min(radius * 2 * control.DeviceDpi / 96,
                Math.Min(control.ClientSize.Width, control.ClientSize.Height));
            if (diameter <= 0) return;
            int right = control.ClientSize.Width - diameter;
            int bottom = control.ClientSize.Height - diameter;
            using var path = new GraphicsPath();
            path.AddArc(0, 0, diameter, diameter, 180, 90);
            path.AddArc(right, 0, diameter, diameter, 270, 90);
            path.AddArc(right, bottom, diameter, diameter, 0, 90);
            path.AddArc(0, bottom, diameter, diameter, 90, 90);
            path.CloseFigure();
            var previous = control.Region;
            control.Region = new Region(path);
            previous?.Dispose();
        }
        control.SizeChanged += Update;
        Update(control, EventArgs.Empty);
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
        (diagnostics.Checked ? new[] { "diag" } : new[] { "release", "diag" }).FirstOrDefault(p => File.Exists(Path.Combine(repo, "target", p, "victory.exe")));

    private void RefreshStatus()
    {
        shadowBoost.Enabled = !rays.Checked && !original.Checked;
        bool ready = repo is not null && BuildProfile() is not null && GamePath() is not null;
        play.Enabled = ready && !launchPending;
        if (launchPending)
            return;
        status.Text = repo is null ? "Place Omega Launcher inside the Victory Omega checkout."
            : BuildProfile() is null ? diagnostics.Checked ? "Build the diagnostics executable to use the FPS HUD." : "Build Victory Omega before launching."
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
            var selection = new Settings(Path.GetDirectoryName(exe), profile.SelectedIndex == 1, rays.Checked,
                original.Checked, memory.Checked, diagnostics.Checked, shadowBoost.Checked);
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
            if (selection.OriginalGraphics) start.ArgumentList.Add("-OriginalGraphics");
            if (selection.ShadowBoost && shadowBoost.Enabled) start.ArgumentList.Add("-ShadowBoost");
            if (selection.LowMemory) start.ArgumentList.Add("-LowMemory");
            if (selection.Diagnostics) start.ArgumentList.Add("-PerformanceLog");
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
        return new[] { Environment.CurrentDirectory, root }.FirstOrDefault(path =>
            path is not null && File.Exists(Path.Combine(path, "data", "last-game-executable.txt")));
    }

    private string? FindLocalGame()
    {
        var root = LocalRoot();
        try
        {
            var current = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "victory", "data", "last-game-executable.txt");
            if (File.Exists(current))
                return Path.GetDirectoryName(File.ReadAllText(current).Trim());
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
        start.ArgumentList.Add("-LegacyRoot");
        start.ArgumentList.Add(root);
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

    private sealed record Settings(string? Folder = null, bool SeriesX = false, bool RayTracing = false,
        bool OriginalGraphics = false, bool LowMemory = false, bool Diagnostics = false, bool ShadowBoost = false);
}
