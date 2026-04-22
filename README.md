<<<<<<< HEAD
still technically just a text editor.
=======
# Gravity Editor

> A modern, fast text editor built with Rust and Iced, featuring an integrated terminal, intuitive file browser, and powerful customization.

![Version](https://img.shields.io/badge/version-0.0.4-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Overview

Gravity is a native desktop text editor designed for developers and power users who need a lightweight yet feature-rich editing environment. Built entirely in Rust using the [Iced GUI framework](https://iced.rs/), Gravity combines performance with an elegant interface.

Whether you're writing code, documentation, or configuration files, Gravity provides a seamless editing experience with modern features like multi-tab support, file tree navigation, integrated terminal, and customizable themes.

## Features

- **Multi-Tab Editing** - Switch between multiple open files effortlessly
- **File Tree Browser** - Expandable directory tree for easy project navigation
- **Integrated Terminal** - Built-in terminal panel with full shell integration
- **Resizable Panels** - Drag-and-drop sidebar and terminal resizing
- **Syntax Highlighting** - Code highlighting powered by Iced's syntax highlighter
- **Customizable Themes** - Full color configuration via JSON settings
- **Font Support** - Nerd Font and custom font support (Inter, JetBrains Mono)
- **High Performance** - Native Rust application with minimal overhead
- **Keyboard Shortcuts** - Intuitive shortcuts for common operations
- **macOS Ready** - Native macOS application support with custom icons

## Tech Stack

| Component | Technology |
|-----------|-----------|
| **Language** | Rust 2024 Edition |
| **GUI Framework** | Iced 0.14.0 |
| **Terminal Integration** | iced_term 0.8.0 |
| **Serialization** | serde + serde_json |
| **Async Runtime** | tokio |
| **File Picker** | rfd 0.15 |

## Requirements

- **Rust:** 1.70 or later
- **OS:** macOS, Linux, Windows
- **Memory:** 100MB minimum

## Installation

### From Source

1. **Clone the repository**
   ```bash
   git clone https://github.com/ExpoInf/Gravity.git
   cd Gravity
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run Gravity**
   ```bash
   cargo run --release
   ```

The compiled binary will be in `target/release/Gravity`.

### macOS Installation

For a native macOS application bundle:

```bash
cargo install cargo-bundle
cargo bundle --release
```

The `.app` will be created in `target/release/bundle/osx/`.

## Getting Started

### Opening a Project

1. Launch Gravity Editor
2. The file browser appears on the left sidebar
3. Click the folder icon to browse and open directories
4. Click any file to open it in a new tab

### Using the Editor

- **Create/Edit** - Click in the editor and start typing
- **Switch Tabs** - Click on file tabs at the top
- **Close Tabs** - Click the ✕ button on any tab
- **Resize Panels** - Drag the vertical divider between sidebar and editor, or horizontal divider above the terminal

### Using the Integrated Terminal

- The terminal occupies the bottom panel of the editor
- Interact with it as you would any shell
- Resize by dragging the terminal divider upward

## Configuration

Gravity stores configuration in your home directory:

```
~/.config/gravity/settings.json
```

### Default Configuration

On first launch, a default config file is created with these settings:

```json
{
  "accent_r": 40,
  "accent_b": 40,
  "accent_g": 40,
  "main_r": 30,
  "main_b": 30,
  "main_g": 30,
  "text_r": 255,
  "text_b": 255,
  "text_g": 255,
  "scheme": "rust",
  "last_browsed": "~/"
}
```

### Customizing Colors

Edit `~/.config/gravity/settings.json` to customize:

- **accent_r/g/b** - Accent color (RGB 0-255)
- **main_r/g/b** - Main background color (RGB 0-255)
- **text_r/g/b** - Text color (RGB 0-255)
- **scheme** - Syntax highlighting scheme (e.g., "rust")
- **last_browsed** - Default directory to open

Example custom theme:

```json
{
  "accent_r": 100,
  "accent_b": 200,
  "accent_g": 150,
  "main_r": 20,
  "main_b": 20,
  "main_g": 20,
  "text_r": 220,
  "text_b": 220,
  "text_g": 220,
  "scheme": "rust",
  "last_browsed": "~/projects"
}
```

Changes take effect on next launch.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Cmd/Ctrl + S** | Save current file |
| **Cmd/Ctrl + T** | Expand dynamic island (experimental) |

Additional standard shortcuts:
- **Tab** - Indent
- **Shift + Tab** - Unindent
- **Cmd/Ctrl + C** - Copy (in editor context)
- **Cmd/Ctrl + V** - Paste (in editor context)

## Project Structure

```
Gravity/
├── src/
│   ├── main.rs           # Main application logic, UI components
│   └── config_lib.rs     # Configuration loading/saving
├── fonts/                # Custom fonts (Nerd Font, Inter)
├── archpackaging/        # Arch Linux packaging files
├── Cargo.toml           # Project manifest
├── Cargo.lock           # Dependency lock file
├── LICENSE              # MIT License
└── README.md            # This file
```

## Architecture

### Core Components

**Project State** - Main application struct holding:
- Text editor content and state
- File tree structure
- Open file tabs
- Terminal state
- Panel dimensions and resize states

**File Tree** - Recursive file node structure enabling:
- Lazy directory expansion
- File/folder distinction with Nerd Font icons
- Shallow directory scanning for performance

**Terminal Integration** - Uses `iced_term` for:
- Native shell integration
- ANSI color support
- Full terminal interactivity

**Async Operations** - Tokio-powered async for:
- File I/O operations
- Terminal command handling

## Development

### Building in Debug Mode

```bash
cargo build
cargo run
```

### Running Tests

```bash
cargo test
```

### Code Style

This project follows Rust standard conventions. Format your code before committing:

```bash
cargo fmt
cargo clippy
```

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- Code compiles without warnings
- Tests pass
- Commits are descriptive

## Roadmap

Planned features for future releases:

- [ ] Plugin system
- [ ] Extended syntax highlighting schemes
- [ ] Minimap support
- [ ] Search and replace functionality
- [ ] Git integration UI
- [ ] Custom keybinding configuration
- [ ] Linux and Windows native packaging

## Known Limitations

- Currently optimized for macOS (Linux/Windows support in progress)
- Limited syntax highlighting schemes
- No plugin system yet
- Terminal resizing animation can be CPU-intensive on older systems

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

Copyright © 2026 ExpoInf

## Acknowledgments

- [Iced](https://iced.rs/) - Elm-inspired GUI framework
- [iced_term](https://github.com/exa-labs/iced_term) - Terminal emulator for Iced
- [Inter Font](https://rsms.me/inter/) - Beautiful open-source typeface
- [JetBrains Mono](https://www.jetbrains.com/lp/mono/) - Code font with Nerd Font support

## Support

For issues, feature requests, or questions:
1. Check existing [Issues](https://github.com/ExpoInf/Gravity/issues)
2. Create a new issue with a clear description
3. Include your OS version and Gravity version

---

Made with ❤️ in Rust
>>>>>>> bbe1711 (Security fixes)
