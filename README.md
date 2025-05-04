# IO_Uring Detector

A cross-platform tool to detect io_uring usage on Linux systems. This tool checks if the Linux kernel supports io_uring (requires kernel 5.1+) and identifies any processes using it.

## Features

- Detects io_uring support on Linux systems
- Works across multiple architectures (x86_64, ARM, ARM64, etc.)
- Identifies processes using io_uring
- Distinguishes between on-disk and in-memory processes
- Provides detailed process information
- Single static binary - no dependencies required

## Requirements

- Linux kernel 5.1 or later
- Any architecture supported by io_uring (x86_64, ARM, ARM64, etc.)

## Building

The project uses Rust and can be built for multiple architectures using the provided build script:

```bash
# Build for default architecture (x86_64)
./build.sh

# Build for specific architecture
./build.sh --arch aarch64  # For ARM64
./build.sh --arch arm      # For 32-bit ARM
./build.sh --arch armv7    # For ARMv7
```

Supported architectures:
- x86_64 (default)
- aarch64 (ARM64)
- arm (32-bit ARM)
- armv7 (ARMv7)
- riscv64 (RISC-V 64-bit)
- powerpc64 (PowerPC 64-bit)
- s390x (IBM System z)

## Usage

After building, copy the binary to your Linux system and run:

```bash
chmod +x io_uring_detector
./io_uring_detector
```

The tool will:
1. Display system architecture and kernel version
2. Check for io_uring support
3. Show available io_uring features if supported
4. List any processes using io_uring
5. Provide detailed information about each process

## Output Example

```
Checking system information...
  Architecture: x86_64
  Kernel Version: Linux 5.15.0
  Node Name: localhost
  Version: #1 SMP

io_uring is supported on this system!
Reported io_uring feature flags:
  - IORING_FEAT_SINGLE_MMAP
  - IORING_FEAT_NODROP
  - IORING_FEAT_SUBMIT_STABLE

Checking for processes using io_uring...
Process using io_uring:
  PID: 1234
  Name: nginx
  Executable: /usr/sbin/nginx
  Command line: nginx -g daemon off;
  Status: Running in memory
  Virtual Memory: 123456 kB
  Resident Memory: 78901 kB
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 