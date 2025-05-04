# IO_Uring Detector

A Rust-based tool for detecting and monitoring processes using io_uring for I/O operations. This project helps identify potential security risks by detecting processes that utilize io_uring, which can be used to bypass traditional system call monitoring.

## Security Context and Motivation

The io_uring interface in Linux, while providing efficient asynchronous I/O operations, has been identified as a potential security risk. According to recent research by ARMO Security, io_uring can be exploited to bypass traditional system call monitoring used by many security tools. This creates a blind spot in Linux runtime security, as malicious processes can perform I/O operations without triggering standard system call monitoring.

Key findings from the research:
- io_uring allows processes to perform I/O operations without using traditional system calls
- Many security tools relying on system call monitoring are "blind" to io_uring-based operations
- This vulnerability affects major security solutions including Falco, Tetragon, and Microsoft Defender
- The issue is particularly relevant in cloud-native environments where Linux is widely used

This project aims to help identify and monitor processes using io_uring, providing visibility into potential security risks and helping organizations maintain better security posture in their Linux environments.

[Reference: ARMO Security Research on io_uring](https://www.armosec.io/blog/io_uring-rootkit-bypasses-linux-security/)

## Project Structure

The project consists of two main components:

```
.
├── io_uring_detector/          # Main detector binary
│   ├── src/
│   │   └── main.rs            # Detector implementation
│   ├── Cargo.toml             # Rust dependencies and configuration
│   └── build.sh               # Build script for the detector
│
└── io_uring_test/             # Test binary
    ├── src/
    │   └── main.rs            # Test implementation
    ├── Cargo.toml             # Rust dependencies and configuration
    └── build.sh               # Build script for the test binary
```

### Component Descriptions

1. **io_uring_detector**
   - `main.rs`: Implements the core detection logic for processes using io_uring
   - `build.sh`: Script to build a statically linked binary for Linux
   - `Cargo.toml`: Defines dependencies and build configuration

2. **io_uring_test**
   - `main.rs`: Creates test processes that use io_uring for I/O operations
   - `build.sh`: Script to build a statically linked binary for Linux
   - `Cargo.toml`: Defines dependencies and build configuration

## Building the Binaries

### Prerequisites

- Rust toolchain (latest stable)
- Cross-compilation toolchain for Linux
- Docker (for building statically linked binaries)

### Building Statically Linked Binaries

1. **Build the Detector**:
   ```bash
   cd io_uring_detector
   ./build.sh
   ```
   This will create a statically linked binary at `target/x86_64-unknown-linux-musl/release/io_uring_detector`

2. **Build the Test Binary**:
   ```bash
   cd io_uring_test
   ./build.sh
   ```
   This will create a statically linked binary at `target/x86_64-unknown-linux-musl/release/io_uring_test`

The build scripts use Docker to create statically linked binaries that can run on any Linux system without dependencies.

## Testing

### Running the Test Binary

1. Start the test binary:
   ```bash
   ./io_uring_test/target/x86_64-unknown-linux-musl/release/io_uring_test
   ```
   This will create processes that use io_uring for I/O operations.

### Running the Detector

1. In a separate terminal, run the detector:
   ```bash
   ./io_uring_detector/target/x86_64-unknown-linux-musl/release/io_uring_detector
   ```
   The detector will scan for processes using io_uring and display their PIDs.

### Expected Output

When running both binaries:

1. The test binary will create processes that use io_uring
2. The detector will output something like:
   ```
   Scanning for io_uring processes...
   Found process using io_uring: PID 1234
   Found process using io_uring: PID 1235
   ```

## Troubleshooting

If you encounter issues:

1. Ensure you have the required build dependencies installed
2. Check that Docker is running if using the build scripts
3. Verify that the binaries have execute permissions
4. Make sure you're running the binaries on a Linux system with io_uring support

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 