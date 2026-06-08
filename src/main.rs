//! gnfetch — a neofetch/fastfetch alternative.
//!
//! Pipeline: collect system info into a [`model::SystemInfo`], resolve settings
//! (config file overridden by CLI flags), then either run an informational flag
//! (--probe / --list-*) , export the card (--save), or select a renderer and
//! render to stdout.

mod cli;
mod collectors;
mod config;
mod demo;
mod model;
mod render;

use clap::Parser;
use cli::Cli;
use config::{BackgroundMode, Config, Layout};
use render::{CardSettings, FontChoice, Generic, ImageSource, Theme};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Rust sets SIGPIPE to SIG_IGN at startup, which turns a closed output pipe
    // (e.g. `gnfetch | head`) into write errors / `println!` panics. Restore the
    // default so we terminate quietly like any other Unix tool.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();

    if cli.demo {
        return demo::run();
    }
    if cli.list_themes {
        println!("auto    (match the detected distro, else default)");
        for name in Theme::names() {
            println!("{name}");
        }
        println!("# distro themes:");
        for name in Theme::distro_names() {
            println!("{name}");
        }
        return ExitCode::SUCCESS;
    }
    if cli.list_fields {
        for key in render::FIELD_KEYS {
            println!("{key}");
        }
        return ExitCode::SUCCESS;
    }
    if cli.list_fonts {
        println!("# bundled fonts (use with --sans/--serif/--mono):");
        for name in render::bundled_names() {
            println!("{name}");
        }
        println!("# any installed system font is also settable by family name");
        return ExitCode::SUCCESS;
    }
    if cli.list_backgrounds {
        println!("# bundled background images (use with --background-image):");
        for name in render::background_names() {
            println!("{name}");
        }
        println!("# --background-image also accepts a file path or an https:// URL");
        return ExitCode::SUCCESS;
    }
    if cli.list_layouts {
        for layout in [
            Layout::Card,
            Layout::Neofetch,
            Layout::Columns,
            Layout::Strip,
            Layout::Compact,
        ] {
            println!("{}", layout_name(layout));
        }
        return ExitCode::SUCCESS;
    }

    let mut info = collectors::collect_all();
    if let Some(distro) = &cli.distro {
        info.distro_id = Some(distro.clone());
    }
    let settings = resolve_settings(&cli, info.distro_id.as_deref());

    // `--probe` reports terminal detection and sizing, then exits.
    if cli.probe {
        println!("{}", render::describe_plan(&info, &settings));
        return ExitCode::SUCCESS;
    }

    // `--save` exports the card to a file instead of rendering.
    if let Some(path) = cli.save.as_deref() {
        return match render::save(&info, &settings, path, cli.width) {
            Ok(()) => {
                println!("gnfetch: wrote card to {}", path.display());
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("gnfetch: failed to write {}: {err}", path.display());
                ExitCode::FAILURE
            }
        };
    }

    let renderer = render::select_renderer(cli.mode, settings);
    match renderer.render(&info) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("gnfetch: failed to render: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Merge config-file defaults with CLI overrides into resolved [`CardSettings`].
fn resolve_settings(cli: &Cli, distro_id: Option<&str>) -> CardSettings {
    let cfg = Config::load();

    let layout = cli.layout.unwrap_or(cfg.layout);
    let theme_name = cli.theme.clone().unwrap_or(cfg.theme);
    // "auto" picks a distro-branded theme, falling back to the generic default.
    let mut theme = if theme_name.eq_ignore_ascii_case("auto") {
        Theme::for_distro(distro_id).unwrap_or_else(Theme::default_theme)
    } else {
        Theme::by_name(&theme_name).unwrap_or_else(|| {
            eprintln!("gnfetch: unknown theme '{theme_name}'; using default.");
            Theme::default_theme()
        })
    };

    if cli.light || cfg.light {
        theme = theme.light();
    }

    if let Some(hex) = cli.accent.clone().or(cfg.accent) {
        match render::parse_hex(&hex) {
            Some(color) => theme = theme.with_accent(color),
            None => eprintln!("gnfetch: invalid accent '{hex}' (expected #rrggbb); ignoring."),
        }
    }

    // `--brand` keeps the distro brand color for the logo + title.
    let brand = if cli.brand || cfg.brand {
        Theme::for_distro(distro_id).map(|t| t.accent)
    } else {
        None
    };

    // Heading font: CLI --serif > CLI --sans > config serif > config sans > system sans.
    // (A flag with no value yields Some("") meaning "system default of that kind".)
    let heading_font = if let Some(v) = cli.serif.clone() {
        FontChoice {
            name: nonempty(v),
            generic: Generic::Serif,
        }
    } else if let Some(v) = cli.sans.clone() {
        FontChoice {
            name: nonempty(v),
            generic: Generic::Sans,
        }
    } else if let Some(v) = cfg.serif {
        FontChoice {
            name: Some(v),
            generic: Generic::Serif,
        }
    } else if let Some(v) = cfg.sans {
        FontChoice {
            name: Some(v),
            generic: Generic::Sans,
        }
    } else {
        FontChoice::system_default(Generic::Sans)
    };
    // Body font: CLI --mono > config mono > system monospace.
    let body_font = if let Some(v) = cli.mono.clone() {
        FontChoice {
            name: nonempty(v),
            generic: Generic::Mono,
        }
    } else if let Some(v) = cfg.mono {
        FontChoice {
            name: Some(v),
            generic: Generic::Mono,
        }
    } else {
        FontChoice::system_default(Generic::Mono)
    };

    // A background image source implies `--background image` unless a background
    // mode was set explicitly.
    let background_spec = cli.background_image.clone().or(cfg.background_image);
    let background = match cli.background.clone().or(cfg.background) {
        Some(spec) => BackgroundMode::parse(&spec).unwrap_or_else(|| {
            eprintln!("gnfetch: unknown background '{spec}'; using solid.");
            BackgroundMode::Solid
        }),
        None if background_spec.is_some() => BackgroundMode::Image,
        None => BackgroundMode::Solid,
    };
    // Classify the spec (URL / bundled / file) once, here, rather than at render.
    let background_image = background_spec.map(|s| ImageSource::classify(&s));

    CardSettings {
        layout,
        theme,
        brand,
        fields: cli.fields.clone().or(cfg.fields),
        heading_font,
        body_font,
        logo: cli.logo.unwrap_or(cfg.logo),
        logo_image: cli.logo_image.clone().or(cfg.logo_image),
        background,
        background_image,
        background_fit: cli.background_fit.unwrap_or(cfg.background_fit),
        darken_image: !(cli.no_darken || cfg.no_darken),
    }
}

/// `None` for empty/whitespace (= use the system default of the slot's category).
fn nonempty(s: String) -> Option<String> {
    Some(s).filter(|v| !v.trim().is_empty())
}

fn layout_name(layout: Layout) -> &'static str {
    match layout {
        Layout::Card => "card",
        Layout::Neofetch => "neofetch",
        Layout::Columns => "columns",
        Layout::Strip => "strip",
        Layout::Compact => "compact",
    }
}
