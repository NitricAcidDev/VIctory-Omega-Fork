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
    private static readonly Color Input = Color.FromArgb(24, 18, 34);
    private static readonly Color Selected = Color.FromArgb(67, 44, 82);
    private static readonly Color Accent = Color.FromArgb(140, 232, 225);
    private static readonly string StateDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Omega Launcher");
    private static readonly string StateFile = Path.Combine(StateDir, "settings.json");
    private static readonly string NvidiaRunnerDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "victory", "nvidia-runner", "omega-options-v1");
    private static bool NvidiaInstalled => File.Exists(Path.Combine(NvidiaRunnerDir, "victory.exe"))
        && File.Exists(Path.Combine(NvidiaRunnerDir, "omega-branding-v1"));
    private readonly string? repo;
    private readonly TextBox folder = new();
    private readonly ComboBox profile = new();
    private readonly CheckBox rays = new DarkCheckBox();
    private readonly CheckBox original = new DarkCheckBox();
    private readonly CheckBox shadowBoost = new DarkCheckBox();
    private readonly CheckBox memory = new DarkCheckBox();
    private readonly CheckBox diagnostics = new DarkCheckBox();
    private readonly CheckBox audioTrace = new DarkCheckBox();
    private readonly CheckBox waterLight = new DarkCheckBox();
    private readonly CheckBox waterWave64 = new DarkCheckBox();
    private readonly CheckBox unifiedTuning = new DarkCheckBox();
    private readonly CheckBox trainer = new DarkCheckBox();
    private readonly Label trainerStatus = new();
    private readonly CheckBox nvidia = new DarkCheckBox();
    private readonly ComboBox frameLimit = new();
    private readonly ComboBox dlss = new();
    private readonly ComboBox frameGeneration = new();
    private readonly Label status = new();
    private readonly Button play = new();
    private readonly Image logo;
    private Settings lastSaved;
    private bool launchPending;
    private static readonly string?[] FrameLimits = [null, "15", "30", "60", "unlimited"];
    private static readonly string[] DlssModes = ["off", "dlaa", "quality", "balanced", "performance"];
    private static readonly int[] FrameMultipliers = [0, 2, 3, 4, 5, 6];

    public Launcher()
    {
        using var art = typeof(Launcher).Assembly.GetManifestResourceStream("omega.png")!;
        using var decoded = Image.FromStream(art);
        logo = new Bitmap(decoded);
        Icon = Icon.ExtractAssociatedIcon(Application.ExecutablePath);
        repo = FindCheckout();

        Text = "Omega Launcher";
        ClientSize = new Size(780, 752);
        MinimumSize = SizeFromClientSize(new Size(684, 736));
        StartPosition = FormStartPosition.CenterScreen;
        BackColor = Color.FromArgb(22, 16, 30);
        ForeColor = Ink;
        Font = new Font("Segoe UI", 10);

        var layout = new TableLayoutPanel
        {
            Dock = DockStyle.Fill, Padding = new Padding(28, 22, 28, 20),
            ColumnCount = 1, RowCount = 7
        };
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 104));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 130));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 126));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 126));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 114));
        layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 58));
        layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
        Controls.Add(Navigation(layout, ModsPanel()));
        layout.Controls.Add(Hero(), 0, 0);
        layout.Controls.Add(FolderCard(), 0, 1);
        layout.Controls.Add(ProfileCard(), 0, 2);
        layout.Controls.Add(OptionsCard(), 0, 3);
        layout.Controls.Add(RunnerCard(), 0, 4);
        layout.Controls.Add(Footer(), 0, 5);

        var saved = ReadSettings();
        frameLimit.SelectedIndex = Math.Max(0, Array.IndexOf(FrameLimits, saved.FrameLimit));
        audioTrace.Checked = saved.AudioTrace;
        waterLight.Checked = saved.WaterLight;
        waterWave64.Checked = saved.WaterWave64;
        unifiedTuning.Checked = saved.UnifiedTuning;
        trainer.Checked = saved.Trainer;
        nvidia.Checked = saved.Nvidia && NvidiaInstalled;
        dlss.SelectedIndex = Math.Max(0, Array.IndexOf(DlssModes, saved.Dlss ?? "quality"));
        frameGeneration.SelectedIndex = Math.Max(0, Array.IndexOf(FrameMultipliers, saved.FrameGeneration));
        folder.Text = saved.Folder ?? FindLocalGame() ?? "";
        profile.SelectedIndex = saved.SeriesX ? 1 : 0;
        rays.Checked = saved.RayTracing;
        original.Checked = saved.OriginalGraphics;
        shadowBoost.Checked = saved.ShadowBoost;
        shadowBoost.AccessibleDescription = "Lifts dark areas when ray tracing and Original graphics are off.";
        memory.AccessibleDescription = "Uses smaller memory windows for buffer copies.";
        memory.Checked = saved.LowMemory;
        diagnostics.Checked = saved.Diagnostics;
        lastSaved = CurrentSettings();
        diagnostics.CheckedChanged += (_, _) => RefreshStatus();
        folder.TextChanged += (_, _) => RefreshStatus();
        profile.SelectedIndexChanged += (_, _) => RefreshStatus();
        rays.CheckedChanged += (_, _) => RefreshStatus();
        original.CheckedChanged += (_, _) => RefreshStatus();
        memory.CheckedChanged += (_, _) => RefreshStatus();
        nvidia.CheckedChanged += (_, _) => RefreshStatus();
        dlss.SelectedIndexChanged += (_, _) => RefreshStatus();
        RefreshStatus();
    }

    private Control Navigation(Control homePage, Control modsPage)
    {
        var pages = new Panel { Dock = DockStyle.Fill };
        pages.Controls.Add(homePage);
        pages.Controls.Add(modsPage);
        var bar = new FlowLayoutPanel
        {
            Dock = DockStyle.Top, Height = 36,
            Padding = new Padding(28, 4, 0, 0), WrapContents = false
        };
        var home = MakeButton("Home", false);
        var mods = MakeButton("Mods", false);
        foreach (var button in new[] { home, mods })
        {
            button.Dock = DockStyle.None;
            button.Size = new Size(82, 28);
            button.Margin = new Padding(0, 0, 6, 0);
            bar.Controls.Add(button);
        }

        void ShowPage(bool showHome)
        {
            homePage.Visible = showHome;
            modsPage.Visible = !showHome;
            home.BackColor = showHome ? Selected : Panel;
            mods.BackColor = showHome ? Panel : Selected;
        }

        home.Click += (_, _) => ShowPage(true);
        mods.Click += (_, _) => ShowPage(false);
        ShowPage(true);
        var navigation = new Panel { Dock = DockStyle.Fill };
        navigation.Controls.Add(pages);
        navigation.Controls.Add(bar);
        return navigation;
    }

    private Control ModsPanel()
    {
        var panel = new FlowLayoutPanel
        {
            Dock = DockStyle.Fill, FlowDirection = FlowDirection.TopDown,
            WrapContents = false, Padding = new Padding(28, 22, 28, 20)
        };
        panel.Controls.Add(LabelFor("ScriptHook VI", 18, Ink, bold: true));
        trainer.Text = "Enable trainer";
        trainer.AutoSize = true;
        trainer.ForeColor = Ink;
        panel.Controls.Add(trainer);
        trainerStatus.AutoSize = true;
        trainerStatus.ForeColor = Muted;
        panel.Controls.Add(trainerStatus);
        return panel;
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
        folder.BackColor = Input;
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
        StyleChoice(profile);
        profile.Margin = new Padding(3, 6, 10, 0);
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
        var options = new TableLayoutPanel
        {
            Dock = DockStyle.Fill, ColumnCount = 3, RowCount = 3, Margin = Padding.Empty
        };
        for (int column = 0; column < options.ColumnCount; column++)
            options.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100f / options.ColumnCount));
        for (int row = 0; row < options.RowCount; row++)
            options.RowStyles.Add(new RowStyle(SizeType.Percent, 100f / options.RowCount));
        var choices = new[] {
            (original, "Original graphics"), (shadowBoost, "Shadow boost"), (memory, "Low-memory mode"),
            (diagnostics, "FPS HUD and logs"), (audioTrace, "Audio tracing"), (waterLight, "Water Light"),
            (waterWave64, "Water Wave64") };
        waterLight.AccessibleDescription = "Enables experimental NVIDIA water lighting. May cause dark or green water tiles.";
        waterWave64.AccessibleDescription = "Tests paired-wave translation for non-tiled water lighting in the built-in runner.";
        for (int index = 0; index < choices.Length; index++)
        {
            var (check, label) = choices[index];
            check.Text = label;
            check.Dock = DockStyle.Fill;
            options.Controls.Add(check, index % options.ColumnCount, index / options.ColumnCount);
        }
        card.Controls.Add(options, 0, 1);
        card.SetColumnSpan(options, 2);
        card.SetRowSpan(options, 2);
        return card;
    }

    private Control RunnerCard()
    {
        var card = Card();
        card.ColumnCount = 3;
        card.ColumnStyles.Clear();
        for (int column = 0; column < 3; column++)
            card.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100f / 3));

        nvidia.Text = NvidiaInstalled ? "Use NVIDIA runner (F1 graphics menu)" : "NVIDIA runner is not installed";
        nvidia.Enabled = NvidiaInstalled;
        nvidia.Dock = DockStyle.Fill;
        card.Controls.Add(nvidia, 0, 0);
        card.SetColumnSpan(nvidia, 2);
        unifiedTuning.Text = "Unified tuning";
        unifiedTuning.Dock = DockStyle.Fill;
        unifiedTuning.AccessibleDescription = "Use six graphics recording workers and up to 1 GiB of Resizable BAR upload memory.";
        card.Controls.Add(unifiedTuning, 2, 0);

        void AddChoice(ComboBox field, int column, string label, string[] values)
        {
            card.Controls.Add(LabelFor(label, 10, Ink), column, 1);
            field.AccessibleName = label;
            StyleChoice(field);
            field.Items.AddRange(values);
            card.Controls.Add(field, column, 2);
        }

        AddChoice(frameLimit, 0, "Frame limit", ["Game default", "15 FPS", "30 FPS", "60 FPS", "Unlimited"]);
        AddChoice(dlss, 1, "DLSS", ["Off", "DLAA", "Quality", "Balanced", "Performance"]);
        AddChoice(frameGeneration, 2, "Frame Generation", ["Off", "2×", "3×", "4×", "5×", "6×"]);
        return card;
    }

    private static void StyleChoice(ComboBox field)
    {
        field.DropDownStyle = ComboBoxStyle.DropDownList;
        field.FlatStyle = FlatStyle.Flat;
        field.BackColor = Input;
        field.ForeColor = Ink;
        field.Dock = DockStyle.Fill;
        field.DrawMode = DrawMode.OwnerDrawFixed;
        field.DrawItem += (_, e) =>
        {
            bool selected = (e.State & DrawItemState.Selected) != 0
                && (e.State & DrawItemState.ComboBoxEdit) == 0;
            using var fill = new SolidBrush(selected ? Selected : Input);
            e.Graphics.FillRectangle(fill, e.Bounds);
            if (e.Index >= 0)
            {
                var bounds = e.Bounds;
                bounds.X += 4;
                bounds.Width -= 4;
                TextRenderer.DrawText(e.Graphics, field.GetItemText(field.Items[e.Index]), field.Font, bounds,
                    field.Enabled ? Ink : Muted, TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine |
                    TextFormatFlags.EndEllipsis | TextFormatFlags.NoPrefix);
            }
            e.DrawFocusRectangle();
        };
    }

    private sealed class DarkCheckBox : CheckBox
    {
        private const TextFormatFlags LabelFlags =
            TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine | TextFormatFlags.EndEllipsis;
        private int GlyphSize => 13 * DeviceDpi / 96;
        private int LabelLeft => 1 + GlyphSize + 5 * DeviceDpi / 96;

        public DarkCheckBox() => FlatStyle = FlatStyle.Flat;

        public override Size GetPreferredSize(Size proposedSize)
        {
            var label = TextRenderer.MeasureText(Text, Font, Size.Empty, LabelFlags);
            return new Size(LabelLeft + label.Width, Math.Max(GlyphSize + 2, label.Height));
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            e.Graphics.Clear(BackColor);
            int glyph = GlyphSize;
            var box = new Rectangle(1, (ClientSize.Height - glyph) / 2, glyph, glyph);
            using var fill = new SolidBrush(Checked && Enabled ? Selected : Input);
            using var border = new Pen(Focused && Enabled ? Accent : Muted);
            e.Graphics.FillRectangle(fill, box);
            e.Graphics.DrawRectangle(border, box);
            if (Checked)
            {
                using var tick = new Pen(Enabled ? Accent : Muted, Math.Max(1f, DeviceDpi / 96f));
                e.Graphics.DrawLines(tick, new Point[] {
                    new(box.Left + glyph / 4, box.Top + glyph / 2),
                    new(box.Left + glyph * 2 / 5, box.Top + glyph * 3 / 4),
                    new(box.Left + glyph * 3 / 4, box.Top + glyph / 4)
                });
            }
            var label = new Rectangle(LabelLeft, 0, Math.Max(0, ClientSize.Width - LabelLeft), ClientSize.Height);
            TextRenderer.DrawText(e.Graphics, Text, Font, label, Enabled ? Ink : Muted, LabelFlags);
            if (Focused && ShowFocusCues)
                ControlPaint.DrawFocusRectangle(e.Graphics, ClientRectangle);
        }
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
        button.BackColor = primary ? Accent : Selected;
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
        nvidia.Checked ? (NvidiaInstalled ? "release" : null) :
        (diagnostics.Checked ? new[] { "diag" } : new[] { "release", "diag" }).FirstOrDefault(p => File.Exists(Path.Combine(repo, "target", p, "victory.exe")));

    private void RefreshStatus()
    {
        waterLight.Enabled = nvidia.Checked;
        waterWave64.Enabled = !nvidia.Checked;
        unifiedTuning.Enabled = nvidia.Checked && !memory.Checked;
        string? game = GamePath();
        string? gameDirectory = game is null ? null : Path.GetDirectoryName(game);
        trainer.Enabled = nvidia.Checked && gameDirectory is not null
            && File.Exists(Path.Combine(gameDirectory, "shvi", "shvi_core.dll"))
            && File.Exists(Path.Combine(gameDirectory, "shvi", "plugins", "trainer.asi"))
            && File.Exists(Path.Combine(gameDirectory, "shvi", "natives", "1794606036.json"));
        trainerStatus.Text = !nvidia.Checked ? "Select Use NVIDIA runner on Home."
            : gameDirectory is null ? "Select your Xbox game folder on Home."
            : trainer.Enabled ? "Press F4 in offline story mode."
            : "Install ScriptHook VI with the NVIDIA build script, then reopen the launcher.";
        shadowBoost.Enabled = !nvidia.Checked && !rays.Checked && !original.Checked;
        dlss.Enabled = nvidia.Checked && !original.Checked;
        frameGeneration.Enabled = dlss.Enabled && dlss.SelectedIndex > 0;
        bool ready = repo is not null && BuildProfile() is not null && GamePath() is not null;
        play.Enabled = ready && !launchPending;
        if (launchPending)
            return;
        status.Text = repo is null ? "Place Omega Launcher inside the Victory Omega checkout."
            : BuildProfile() is null ? diagnostics.Checked ? "Build the diagnostics executable to use the FPS HUD." : "Build Victory Omega before launching."
            : GamePath() is null ? "Select the folder with the game executable."
            : nvidia.Checked ? "NVIDIA runner ready. Press F1 in game."
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
            var selection = CurrentSettings() with { Folder = Path.GetDirectoryName(exe) };
            SaveSettings(selection);
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
            if (selection.Nvidia)
            {
                start.ArgumentList.Add("-Nvidia");
                if (selection.WaterLight) start.ArgumentList.Add("-WaterLight");
                if (selection.UnifiedTuning && unifiedTuning.Enabled) start.ArgumentList.Add("-UnifiedTuning");
                if (selection.Trainer && trainer.Enabled) start.ArgumentList.Add("-Trainer");
                start.ArgumentList.Add("-Dlss");
                start.ArgumentList.Add(selection.Dlss!);
                start.ArgumentList.Add("-FrameGeneration");
                start.ArgumentList.Add(selection.FrameGeneration.ToString(System.Globalization.CultureInfo.InvariantCulture));
            }
            else if (selection.WaterWave64)
                start.ArgumentList.Add("-WaterWave64");
            if (selection.RayTracing)
                start.ArgumentList.Add("-RayTracing");
            if (selection.OriginalGraphics) start.ArgumentList.Add("-OriginalGraphics");
            if (selection.ShadowBoost && shadowBoost.Enabled) start.ArgumentList.Add("-ShadowBoost");
            if (selection.LowMemory) start.ArgumentList.Add("-LowMemory");
            if (selection.Diagnostics) start.ArgumentList.Add("-PerformanceLog");
            if (selection.AudioTrace) start.ArgumentList.Add("-AudioTrace");
            if (selection.FrameLimit is not null)
            {
                start.ArgumentList.Add("-FrameLimit");
                start.ArgumentList.Add(selection.FrameLimit);
            }
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
            using var stdout = new FileStream(Path.Combine(StateDir, "game-stdout.log"), FileMode.Create, FileAccess.Write, FileShare.Read);
            using var stderr = new FileStream(Path.Combine(StateDir, "game-stderr.log"), FileMode.Create, FileAccess.Write, FileShare.Read);
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
        catch (UnauthorizedAccessException) { return new(); }
        catch (JsonException) { return new(); }
    }

    private Settings CurrentSettings() => new(
        folder.Text.Trim(), profile.SelectedIndex == 1, rays.Checked,
        original.Checked, memory.Checked, diagnostics.Checked, shadowBoost.Checked,
        FrameLimits[frameLimit.SelectedIndex], audioTrace.Checked, nvidia.Checked,
        DlssModes[dlss.SelectedIndex], FrameMultipliers[frameGeneration.SelectedIndex],
        waterLight.Checked, unifiedTuning.Checked, trainer.Checked, waterWave64.Checked);

    private void SaveSettings(Settings selection)
    {
        Directory.CreateDirectory(StateDir);
        var pending = Path.Combine(StateDir, $"settings.{Guid.NewGuid():N}.tmp");
        try
        {
            File.WriteAllText(pending, JsonSerializer.Serialize(selection));
            File.Move(pending, StateFile, overwrite: true);
            lastSaved = selection;
        }
        finally
        {
            if (File.Exists(pending))
                File.Delete(pending);
        }
    }

    protected override void OnFormClosing(FormClosingEventArgs e)
    {
        base.OnFormClosing(e);
        if (e.Cancel)
            return;

        var selection = CurrentSettings();
        if (selection == lastSaved)
            return;

        try
        {
            SaveSettings(selection);
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException)
        {
            MessageBox.Show(this, $"Could not save launcher settings. {error.Message}",
                Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
        }
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
            logo.Dispose();
        base.Dispose(disposing);
    }

    private sealed record Settings(string? Folder = null, bool SeriesX = false, bool RayTracing = false,
        bool OriginalGraphics = false, bool LowMemory = false, bool Diagnostics = false, bool ShadowBoost = false,
        string? FrameLimit = null, bool AudioTrace = false, bool Nvidia = false, string? Dlss = null,
        int FrameGeneration = 0, bool WaterLight = false, bool UnifiedTuning = false, bool Trainer = false,
        bool WaterWave64 = false);
}
