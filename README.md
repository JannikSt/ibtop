# ibtop

Real-time terminal monitor for InfiniBand networks - htop for high-speed interconnects

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Installation

### From Release Binary

```bash
# Download the latest release
wget https://github.com/JannikSt/ibtop/releases/download/v0.1.0/ibtop-linux-amd64
chmod +x ibtop-linux-amd64
sudo mv ibtop-linux-amd64 /usr/local/bin/ibtop
```

### From Source

```bash
# Clone the repository
git clone https://github.com/JannikSt/ibtop.git
cd ibtop

# Build and install
cargo build --release
sudo cp target/release/ibtop /usr/local/bin/
```

## Usage

```bash
# Monitor all InfiniBand adapters
ibtop
```

### Controls

- `q` or `ESC` - Quit

## Requirements

- Linux system with InfiniBand sysfs (`/sys/class/infiniband/`)
- Terminal with color support
- Rust 1.70+ (for building from source)

## Compatibility

- **Mellanox mlx5** - Fully tested, counters use 32-bit word units
- **Mellanox mlx4** - Expected to work (same counter format)
- **Intel/Cornelis** - May require adjustments for hw_counters path
- **Other IB adapters** - Should work if they follow standard sysfs layout

## Building from Source

```bash
# Clone the repository
git clone https://github.com/JannikSt/ibtop.git
cd ibtop

# Build debug version
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test
```
## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
