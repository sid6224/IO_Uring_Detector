#!/usr/bin/env bash

set -euo pipefail

# Configuration
BINARY_NAME="io_uring_test"
DEFAULT_TARGET="x86_64-unknown-linux-musl"
TARGET=$DEFAULT_TARGET
BUILD_PATH="target/${TARGET}/release/${BINARY_NAME}"
STRIP_BINARY=true
RUN_AFTER_BUILD=false
USE_SUDO=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Show help message
show_help() {
    echo -e "${GREEN}io_uring_test Build Script${NC}"
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --help          Show this help message"
    echo "  --no-strip      Do not strip debug symbols from the binary"
    echo "  --arch ARCH     Build for specific architecture (default: x86_64)"
    echo "                  Supported architectures:"
    echo "                    - x86_64"
    echo "                    - aarch64"
    echo "                    - arm"
    echo "                    - armv7"
    echo "                    - riscv64"
    echo "                    - powerpc64"
    echo "                    - s390x"
    echo ""
    echo "This script builds a static binary that tests io_uring functionality on Linux systems."
    echo "The binary will be created at: ${BUILD_PATH}"
    exit 0
}

# Get target for architecture
get_target_for_arch() {
    case "$1" in
        "x86_64")
            echo "x86_64-unknown-linux-musl"
            ;;
        "aarch64")
            echo "aarch64-unknown-linux-musl"
            ;;
        "arm")
            echo "arm-unknown-linux-musleabi"
            ;;
        "armv7")
            echo "armv7-unknown-linux-musleabihf"
            ;;
        "riscv64")
            echo "riscv64gc-unknown-linux-musl"
            ;;
        "powerpc64")
            echo "powerpc64-unknown-linux-musl"
            ;;
        "s390x")
            echo "s390x-unknown-linux-musl"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Install required dependencies
install_dependencies() {
    log_info "Checking and installing required dependencies..."
    
    # Check if Homebrew is installed
    if ! command -v brew >/dev/null 2>&1; then
        log_info "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    
    # Install Rust if not present
    if ! command -v rustup >/dev/null 2>&1; then
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Install musl-cross for cross-compilation
    if ! command -v x86_64-linux-musl-gcc >/dev/null 2>&1; then
        log_info "Installing musl-cross..."
        brew install FiloSottile/musl-cross/musl-cross
    fi
    
    # Add musl target to Rust
    if ! rustup target list | grep -q "${TARGET} (installed)"; then
        log_info "Adding musl target to Rust..."
        rustup target add "${TARGET}"
    fi
}

# Check for required tools
check_requirements() {
    local missing_tools=()

    # Check for Rust toolchain
    if ! command -v cargo >/dev/null 2>&1; then
        missing_tools+=("Rust toolchain")
    fi

    # Check for musl target
    if ! rustup target list | grep -q "${TARGET} (installed)"; then
        missing_tools+=("Rust musl target")
    fi

    # Check for cross-compiler
    local cross_compiler
    case "$TARGET" in
        x86_64-unknown-linux-musl)
            cross_compiler="x86_64-linux-musl-gcc"
            ;;
        aarch64-unknown-linux-musl)
            cross_compiler="aarch64-linux-musl-gcc"
            ;;
        arm-unknown-linux-musleabi)
            cross_compiler="arm-linux-musleabi-gcc"
            ;;
        armv7-unknown-linux-musleabihf)
            cross_compiler="arm-linux-musleabihf-gcc"
            ;;
        *)
            cross_compiler=""
            ;;
    esac

    if [ -n "$cross_compiler" ] && ! command -v "$cross_compiler" >/dev/null 2>&1; then
        missing_tools+=("$cross_compiler")
    fi

    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_info "Installing missing tools..."
        install_dependencies
    fi
}

# Create .cargo/config.toml for cross-compilation
setup_cargo_config() {
    mkdir -p .cargo
    local cross_compiler
    case "$TARGET" in
        x86_64-unknown-linux-musl)
            cross_compiler="x86_64-linux-musl-gcc"
            ;;
        aarch64-unknown-linux-musl)
            cross_compiler="aarch64-linux-musl-gcc"
            ;;
        arm-unknown-linux-musleabi)
            cross_compiler="arm-linux-musleabi-gcc"
            ;;
        armv7-unknown-linux-musleabihf)
            cross_compiler="arm-linux-musleabihf-gcc"
            ;;
        *)
            log_error "Unsupported target: $TARGET"
            exit 1
            ;;
    esac

    cat > .cargo/config.toml << EOF
[target.${TARGET}]
linker = "${cross_compiler}"
rustflags = ["-C", "target-feature=+crt-static"]
EOF
}

# Build the binary
build_binary() {
    log_info "Building static ${BINARY_NAME} binary for ${TARGET}..."
    
    # Setup cargo config for cross-compilation
    setup_cargo_config
    
    if ! cargo build --release --target "${TARGET}"; then
        log_error "Build failed"
        exit 1
    fi

    if [ "$STRIP_BINARY" = true ]; then
        log_info "Stripping debug symbols..."
        local strip_tool
        case "$TARGET" in
            x86_64-unknown-linux-musl)
                strip_tool="x86_64-linux-musl-strip"
                ;;
            aarch64-unknown-linux-musl)
                strip_tool="aarch64-linux-musl-strip"
                ;;
            arm-unknown-linux-musleabi)
                strip_tool="arm-linux-musleabi-strip"
                ;;
            armv7-unknown-linux-musleabihf)
                strip_tool="arm-linux-musleabihf-strip"
                ;;
            *)
                strip_tool="strip"
                ;;
        esac

        if ! $strip_tool "${BUILD_PATH}"; then
            log_warn "Failed to strip binary, continuing anyway..."
        fi
    fi

    # Check binary size
    local size=$(stat -f %z "${BUILD_PATH}")
    log_info "Binary size: $(numfmt --to=iec-i --suffix=B --format="%.2f" "${size}")"
}

# Main execution
main() {
    check_requirements
    build_binary
    
    log_info "Build completed successfully"
    log_info "Binary location: ${BUILD_PATH}"
    log_info "The binary is now ready to be copied to any Linux system and run independently"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help)
            show_help
            ;;
        --no-strip)
            STRIP_BINARY=false
            shift
            ;;
        --arch)
            if [ -z "${2:-}" ]; then
                log_error "Architecture not specified"
                show_help
            fi
            TARGET=$(get_target_for_arch "$2")
            if [ -z "$TARGET" ]; then
                log_error "Unsupported architecture: $2"
                show_help
            fi
            BUILD_PATH="target/${TARGET}/release/${BINARY_NAME}"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            log_error "Use --help to see available options"
            exit 1
            ;;
    esac
done

main 