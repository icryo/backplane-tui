# Backplane TUI

A terminal UI for managing Docker containers, built with Rust and [Ratatui](https://github.com/ratatui/ratatui).

## Features

- **Container Management** - Start, stop, restart, and remove containers
- **Live Stats** - CPU and memory usage with inline progress bars
- **Multiple Views** - Switch between Stats, Network, and Details views
- **Log Viewer** - Full-screen container log viewing with scrolling
- **Container Creation** - Create new containers with image picker
- **Exec Shell** - Shell into running containers (`/bin/bash`, `/bin/sh`, etc.)
- **Fuzzy Filter** - Quick container search
- **System Stats** - CPU, Memory, Disk, and GPU/VRAM usage in header
- **Catppuccin Theme** - Dark mode friendly color scheme

## Installation

```bash
# Build from source
cargo build --release

# Install to /bin (optional)
sudo cp target/release/backplane-tui /bin/jv
```

## Keybindings

### List View
| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate containers |
| `←` `→` | Switch view (Stats/Network/Details) |
| `/` | Filter containers |
| `Enter` `l` | View logs |
| `i` | Container info modal |
| `e` | Exec into container |
| `s` | Start container |
| `x` | Stop container |
| `R` | Restart container |
| `d` | Delete container |
| `n` | New container |
| `r` | Refresh |
| `?` | Help |
| `q` | Quit |

### Logs View
| Key | Action |
|-----|--------|
| `↑` `↓` | Scroll |
| `g` `G` | Top / Bottom |
| `Esc` | Back to list |

## Views

Toggle with `←` `→` arrows:

- **Stats** - Name, Type, Port, CPU bar, MEM bar
- **Network** - Name, RX/TX rates, Total RX/TX
- **Details** - Name, Image, Container ID, Uptime

## Requirements

- Docker daemon running locally
- Terminal with Unicode support

## License

MIT
