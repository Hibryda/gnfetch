---
title: "Documentation"
role: part
parent: null
order: 1
description: "gnfetch documentation index"
---

# gnfetch Documentation

Documentation for **gnfetch** — a Rust neofetch/fastfetch alternative that renders system
information as both ASCII/ANSI terminal output and a rich graphical "system visiting card"
(Kitty graphics protocol, iTerm2 inline images, Sixel).

This directory is the single source of truth for the project. As features land, the relevant
docs here are created or updated.

## Planned sections

- **Architecture** — the Collectors → `SystemInfo` → Renderers pipeline and terminal capability detection
- **Collectors** — per-domain system info modules (os, kernel, cpu, gpu, memory, disk, packages, shell, de/wm)
- **Renderers** — the ANSI renderer and the graphical card renderer
- **Graphics protocols** — Kitty / iTerm2 / Sixel detection and emission, with fallback behavior
- **Configuration** — CLI flags and config file format

> This directory is maintained automatically. When features are added or changed, corresponding documentation is updated.
