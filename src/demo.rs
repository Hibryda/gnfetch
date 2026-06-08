//! `--demo`: render a gallery of example cards with dummy data, each labelled
//! with the flags that produce it — a visual tour of gnfetch's capabilities.
//!
//! Each example uses a different fictional distro (shuffled per run) so the
//! gallery also shows off the range of logos and auto-themes.

use crate::cli::Mode;
use crate::config::{BackgroundMode, Fit, Layout, LogoKind};
use crate::model::{CpuInfo, DiskInfo, PackageCount, SystemInfo, UsageInfo};
use crate::render::{self, CardSettings, FontChoice, Generic, ImageSource, Theme};
use std::process::ExitCode;
use std::time::Duration;

const RESET: &str = "\x1b[0m";

/// Render the demo gallery to stdout.
pub fn run() -> ExitCode {
    intro();
    for example in examples() {
        header(example.title, example.flags);
        let renderer = render::select_renderer(example.mode, example.settings);
        if let Err(err) = renderer.render(&example.info) {
            eprintln!("gnfetch: demo render failed: {err}");
        }
        println!();
    }
    println!(
        "\x1b[2m  Tip: run any of the flags above on your own system. `gnfetch --help` lists them all.{RESET}\n"
    );
    ExitCode::SUCCESS
}

fn intro() {
    println!(
        "\n\x1b[1;36m  gnfetch — capability tour\x1b[0m  \x1b[2m(fictional systems; your real output uses this machine)\x1b[0m\n"
    );
}

fn header(title: &str, flags: &str) {
    println!("\x1b[1;36m  ▌ {title}{RESET}");
    println!("\x1b[2m    {flags}{RESET}");
}

struct Example {
    title: &'static str,
    flags: &'static str,
    mode: Mode,
    info: SystemInfo,
    settings: CardSettings,
}

type Tweak = Box<dyn Fn(&mut CardSettings, &SystemInfo)>;

fn examples() -> Vec<Example> {
    let mut pool = systems();
    shuffle(&mut pool);

    let specs: Vec<(&str, &str, Mode, Tweak)> = vec![
        ("Default card", "gnfetch", Mode::Image, Box::new(|_, _| {})),
        (
            "Neofetch layout (logo + info)",
            "gnfetch --layout neofetch",
            Mode::Image,
            Box::new(|s, _| s.layout = Layout::Neofetch),
        ),
        (
            "Columns layout (wide & short)",
            "gnfetch --layout columns",
            Mode::Image,
            Box::new(|s, _| s.layout = Layout::Columns),
        ),
        (
            "Strip layout (status bar)",
            "gnfetch --layout strip",
            Mode::Image,
            Box::new(|s, _| s.layout = Layout::Strip),
        ),
        (
            "Compact layout",
            "gnfetch --layout compact",
            Mode::Image,
            Box::new(|s, _| s.layout = Layout::Compact),
        ),
        (
            "Aesthetic theme",
            "gnfetch --layout neofetch --theme dracula",
            Mode::Image,
            Box::new(|s, _| {
                s.layout = Layout::Neofetch;
                s.theme = Theme::by_name("dracula").unwrap();
            }),
        ),
        (
            "Keep distro brand with another theme",
            "gnfetch --layout neofetch --theme tokyonight --brand",
            Mode::Image,
            Box::new(|s, sys| {
                s.layout = Layout::Neofetch;
                s.theme = Theme::by_name("tokyonight").unwrap();
                s.brand = Theme::for_distro(sys.distro_id.as_deref()).map(|t| t.accent);
            }),
        ),
        (
            "Light theme",
            "gnfetch --light",
            Mode::Image,
            Box::new(|s, _| s.theme = s.theme.light()),
        ),
        (
            "Fancy heading font",
            "gnfetch --layout neofetch --sans lobster",
            Mode::Image,
            Box::new(|s, _| {
                s.layout = Layout::Neofetch;
                s.heading_font = FontChoice {
                    name: Some("lobster".into()),
                    generic: Generic::Sans,
                };
            }),
        ),
        (
            "ASCII-art logo",
            "gnfetch --layout neofetch --logo ascii",
            Mode::Image,
            Box::new(|s, _| {
                s.layout = Layout::Neofetch;
                s.logo = LogoKind::Ascii;
            }),
        ),
        (
            "Gradient background (any angle)",
            "gnfetch --background linear-30",
            Mode::Image,
            Box::new(|s, _| s.background = BackgroundMode::Linear(30.0)),
        ),
        (
            "Bundled CC0 image background",
            "gnfetch --background-image carina  (also: a file or https URL; --list-backgrounds)",
            Mode::Image,
            Box::new(|s, _| {
                s.background = BackgroundMode::Image;
                s.background_image = Some(ImageSource::Bundled("carina".into()));
                s.background_fit = Fit::Fill;
            }),
        ),
        (
            "Classic ANSI output (themed)",
            "gnfetch --mode ansi --theme gruvbox",
            Mode::Ansi,
            Box::new(|s, _| s.theme = Theme::by_name("gruvbox").unwrap()),
        ),
    ];

    specs
        .into_iter()
        .enumerate()
        .map(|(i, (title, flags, mode, tweak))| {
            let info = pool[i % pool.len()].clone();
            let mut settings = base(&info);
            tweak(&mut settings, &info);
            Example {
                title,
                flags,
                mode,
                info,
                settings,
            }
        })
        .collect()
}

/// Base settings: auto theme for the example's distro, drawn logo, bundled fonts
/// (so the gallery renders fast without scanning system fonts).
fn base(info: &SystemInfo) -> CardSettings {
    CardSettings {
        layout: Layout::Card,
        theme: Theme::for_distro(info.distro_id.as_deref()).unwrap_or_else(Theme::default_theme),
        brand: None,
        fields: None,
        heading_font: FontChoice {
            name: Some("poppins".into()),
            generic: Generic::Sans,
        },
        body_font: FontChoice {
            name: Some("dejavu-mono".into()),
            generic: Generic::Mono,
        },
        logo: LogoKind::Drawn,
        logo_image: None,
        background: BackgroundMode::Solid,
        background_image: None,
        background_fit: Fit::Fill,
        darken_image: true,
    }
}

/// In-place Fisher–Yates shuffle seeded from the clock (xorshift64; no deps).
fn shuffle<T>(v: &mut [T]) {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9e37_79b9_7f4a_7c15)
        | 1;
    let mut rng = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for i in (1..v.len()).rev() {
        let j = (rng() % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
}

/// A fictional system for screenshots / testing (`--mock`). `id` selects a distro
/// from the pool by os-release id (e.g. `arch`); `None` or an unknown id returns
/// the first entry. Carries no real machine data; the username is a plausible
/// placeholder per distro so screenshots show varied mocked users.
pub fn mock_system(id: Option<&str>) -> SystemInfo {
    let pool = systems();
    let mut sys = id
        .and_then(|want| pool.iter().find(|s| s.distro_id.as_deref() == Some(want)))
        .cloned()
        .unwrap_or_else(|| pool[0].clone());
    sys.user = Some(mock_user(sys.distro_id.as_deref()).to_string());
    sys
}

/// A plausible placeholder username per distro (keeps `--mock` screenshots varied).
fn mock_user(id: Option<&str>) -> &'static str {
    match id {
        Some("arch") => "alex",
        Some("fedora") => "max",
        Some("ubuntu") => "jo",
        Some("gentoo") => "ada",
        Some("nixos") => "kai",
        Some("manjaro") => "remy",
        Some("opensuse") => "lena",
        Some("popos") => "theo",
        Some("void") => "nova",
        _ => "sam",
    }
}

/// A pool of fictional systems (distinct distros) for the gallery.
fn systems() -> Vec<SystemInfo> {
    vec![
        mk(
            "workstation",
            "Debian GNU/Linux 13 (trixie)",
            "debian",
            "6.12.9-amd64",
            ("dpkg", 2843),
            "GNOME",
            "Wayland",
            "Intel Core i9-14900K",
            32,
            5.8,
            "NVIDIA GeForce RTX 4080",
            "bash",
            (18.2, 64.0),
            (212.0, 931.0, "ext4"),
            (88.0, 465.0, "ext4"),
            (52, 14),
        ),
        mk(
            "rigarch",
            "Arch Linux",
            "arch",
            "6.12.0-arch1-1",
            ("pacman", 1287),
            "Hyprland",
            "Wayland",
            "AMD Ryzen 9 7950X",
            32,
            4.5,
            "AMD Radeon RX 7900 XTX",
            "zsh",
            (12.4, 32.0),
            (118.0, 500.0, "btrfs"),
            (642.0, 2000.0, "btrfs"),
            (3, 25),
        ),
        mk(
            "fedorabox",
            "Fedora Linux 41 (Workstation)",
            "fedora",
            "6.13.4-200.fc41",
            ("rpm", 2104),
            "GNOME",
            "Wayland",
            "Intel Core Ultra 7 155H",
            22,
            4.8,
            "Intel Arc Graphics",
            "fish",
            (9.8, 16.0),
            (61.0, 256.0, "btrfs"),
            (0.0, 0.0, ""),
            (7, 41),
        ),
        mk(
            "ubuntudev",
            "Ubuntu 24.04.1 LTS",
            "ubuntu",
            "6.8.0-45-generic",
            ("dpkg", 1976),
            "GNOME",
            "X11",
            "AMD Ryzen 7 7840U",
            16,
            5.1,
            "AMD Radeon 780M",
            "bash",
            (6.1, 16.0),
            (45.0, 238.0, "ext4"),
            (0.0, 0.0, ""),
            (29, 3),
        ),
        mk(
            "portage",
            "Gentoo Linux",
            "gentoo",
            "6.12.8-gentoo",
            ("emerge", 1043),
            "KDE Plasma",
            "Wayland",
            "AMD Ryzen 9 5950X",
            32,
            4.9,
            "NVIDIA GeForce RTX 3070",
            "zsh",
            (21.7, 64.0),
            (180.0, 1000.0, "xfs"),
            (430.0, 1800.0, "zfs"),
            (96, 12),
        ),
        mk(
            "nixbox",
            "NixOS 24.11 (Vicuna)",
            "nixos",
            "6.12.6",
            ("nix", 1622),
            "sway",
            "Wayland",
            "Apple M-series (vm)",
            10,
            3.5,
            "llvmpipe (software)",
            "fish",
            (5.4, 16.0),
            (38.0, 200.0, "ext4"),
            (0.0, 0.0, ""),
            (1, 8),
        ),
        mk(
            "plasma",
            "Manjaro Linux",
            "manjaro",
            "6.12.4-1-MANJARO",
            ("pacman", 1498),
            "KDE Plasma",
            "X11",
            "Intel Core i7-12700H",
            20,
            4.7,
            "NVIDIA GeForce RTX 3060",
            "bash",
            (10.9, 32.0),
            (96.0, 476.0, "ext4"),
            (0.0, 0.0, ""),
            (14, 52),
        ),
        mk(
            "tumbleweed",
            "openSUSE Tumbleweed",
            "opensuse",
            "6.13.1-1-default",
            ("zypper", 2531),
            "KDE Plasma",
            "Wayland",
            "AMD Ryzen 5 7600",
            12,
            5.1,
            "AMD Radeon RX 6600",
            "zsh",
            (7.3, 32.0),
            (72.0, 500.0, "btrfs"),
            (0.0, 0.0, ""),
            (40, 6),
        ),
        mk(
            "popdev",
            "Pop!_OS 22.04 LTS",
            "pop",
            "6.9.3-76060903-generic",
            ("dpkg", 2188),
            "COSMIC",
            "Wayland",
            "Intel Core i5-13600K",
            20,
            5.1,
            "NVIDIA GeForce RTX 4060",
            "bash",
            (8.0, 32.0),
            (130.0, 465.0, "ext4"),
            (0.0, 0.0, ""),
            (5, 33),
        ),
        mk(
            "voidrig",
            "Void Linux",
            "void",
            "6.12.7_1",
            ("xbps", 712),
            "dwm",
            "X11",
            "Intel Core i5-10400",
            12,
            4.3,
            "Intel UHD Graphics 630",
            "dash",
            (4.2, 16.0),
            (52.0, 240.0, "ext4"),
            (0.0, 0.0, ""),
            (61, 18),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn mk(
    host: &str,
    distro: &str,
    id: &str,
    kernel: &str,
    pkg: (&str, usize),
    de: &str,
    wm: &str,
    cpu: &str,
    cores: usize,
    ghz: f64,
    gpu: &str,
    shell: &str,
    mem_g: (f64, f64),
    root: (f64, f64, &str),
    home: (f64, f64, &str),
    up: (u64, u64),
) -> SystemInfo {
    let gib = |g: f64| (g * 1024.0 * 1024.0 * 1024.0) as u64;
    let mut disks = vec![DiskInfo {
        mount: "/".to_string(),
        used: gib(root.0),
        total: gib(root.1),
        fs: root.2.to_string(),
    }];
    if home.1 > 0.0 {
        disks.push(DiskInfo {
            mount: "/home".to_string(),
            used: gib(home.0),
            total: gib(home.1),
            fs: home.2.to_string(),
        });
    }
    SystemInfo {
        user: Some("demo".to_string()),
        hostname: Some(host.to_string()),
        distro: Some(distro.to_string()),
        distro_id: Some(id.to_string()),
        kernel: Some(kernel.to_string()),
        uptime: Some(Duration::from_secs(up.0 * 3600 + up.1 * 60)),
        cpu: Some(CpuInfo {
            brand: cpu.to_string(),
            cores,
            freq_mhz: (ghz * 1000.0) as u64,
        }),
        gpus: vec![gpu.to_string()],
        memory: Some(UsageInfo {
            used: gib(mem_g.0),
            total: gib(mem_g.1),
        }),
        swap: Some(UsageInfo {
            used: gib(mem_g.1 * 0.02),
            total: gib(mem_g.1 / 4.0),
        }),
        disks,
        packages: vec![PackageCount {
            manager: pkg.0.to_string(),
            count: pkg.1,
        }],
        shell: Some(shell.to_string()),
        de: Some(de.to_string()),
        wm: Some(wm.to_string()),
    }
}
