#!/bin/bash
set -euo pipefail

# Marty Plugin Installer
# Usage: curl -sSL https://raw.githubusercontent.com/codyspate/marty/main/scripts/install-plugin.sh | bash -s -- [plugin-name] [version]

PLUGIN_NAME="${1:-}"
VERSION="${2:-latest}"
REPO="codyspate/marty"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1" >&2
}

# Show usage
usage() {
    cat << EOF
Marty Plugin Installer

Usage: $0 [plugin-name] [version]

Available plugins:
  cargo      - Cargo workspace detection plugin  
  pnpm       - PNPM workspace detection plugin
  typescript - TypeScript project detection plugin

Arguments:
  plugin-name    Plugin to install (required)
  version        Plugin version to install (default: latest)

Examples:
  $0 cargo
  $0 pnpm latest
  $0 typescript v0.2.0

Environment Variables:
  MARTY_PLUGINS_DIR    Custom plugin installation directory (default: ~/.marty/plugins)
EOF
}

# Validate arguments
if [[ -z "$PLUGIN_NAME" ]]; then
    error "Plugin name is required"
    usage
    exit 1
fi

if [[ ! "$PLUGIN_NAME" =~ ^(cargo|pnpm|typescript)$ ]]; then
    error "Invalid plugin name: $PLUGIN_NAME"
    error "Available plugins: cargo, pnpm, typescript"
    exit 1
fi

# Detect OS and architecture
detect_platform() {
    local os arch
    
    # Detect OS
    case "$(uname -s)" in
        Darwin*)
            os="apple-darwin"
            ;;
        Linux*)
            os="unknown-linux-gnu"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="pc-windows-msvc"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${arch}-${os}"
}

# Get latest version from GitHub releases
get_latest_version() {
    local plugin_name="$1"
    info "Fetching latest version for marty-plugin-${plugin_name}..."
    
    local api_url="https://api.github.com/repos/${REPO}/releases"
    local version
    
    version=$(curl -sSL "$api_url" | \
        grep -o "\"tag_name\": \"marty-plugin-${plugin_name}-v[^\"]*\"" | \
        head -n1 | \
        cut -d'"' -f4 | \
        sed "s/marty-plugin-${plugin_name}-//")
    
    if [[ -z "$version" ]]; then
        error "Could not find any releases for marty-plugin-${plugin_name}"
        exit 1
    fi
    
    echo "$version"
}

# Download and install plugin
install_plugin() {
    local plugin_name="$1"
    local version="$2"
    local platform="$3"
    
    # Determine version to install
    if [[ "$version" == "latest" ]]; then
        version=$(get_latest_version "$plugin_name")
        info "Latest version: $version"
    fi
    
    # Remove 'v' prefix if present
    version=${version#v}
    
    # Set up directories
    local plugins_dir="${MARTY_PLUGINS_DIR:-$HOME/.marty/plugins}"
    mkdir -p "$plugins_dir"
    
    info "Installing marty-plugin-${plugin_name} v${version} for ${platform}..."
    
    # Determine file extension and archive format
    local ext archive_ext
    case "$platform" in
        *windows*)
            ext="dll"
            archive_ext="zip"
            ;;
        *darwin*)
            ext="dylib"
            archive_ext="tar.gz"
            ;;
        *linux*)
            ext="so"
            archive_ext="tar.gz"
            ;;
    esac
    
    # Construct download URL
    local tag="marty-plugin-${plugin_name}-v${version}"
    local filename="marty-plugin-${plugin_name}-${version}-${platform}.${archive_ext}"
    local download_url="https://github.com/${REPO}/releases/download/${tag}/${filename}"
    
    info "Downloading from: $download_url"
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" EXIT
    
    # Download the archive
    if ! curl -sSL -o "${temp_dir}/${filename}" "$download_url"; then
        error "Failed to download plugin from: $download_url"
        error "Please check that the version exists and try again"
        exit 1
    fi
    
    # Extract the archive
    cd "$temp_dir"
    case "$archive_ext" in
        tar.gz)
            tar -xzf "$filename"
            ;;
        zip)
            unzip -q "$filename"
            ;;
    esac
    
    # Find the plugin file
    local plugin_file
    if [[ "$ext" == "dll" ]]; then
        plugin_file="marty_plugin_${plugin_name}.dll"
    else
        plugin_file="libmarty_plugin_${plugin_name}.${ext}"
    fi
    
    # Locate the extracted plugin
    local extracted_plugin
    extracted_plugin=$(find . -name "$plugin_file" -type f | head -n1)
    
    if [[ -z "$extracted_plugin" ]]; then
        error "Could not find plugin file '$plugin_file' in downloaded archive"
        exit 1
    fi
    
    # Copy to plugins directory
    local target_path="${plugins_dir}/${plugin_file}"
    cp "$extracted_plugin" "$target_path"
    
    # Make executable on Unix systems
    if [[ "$ext" != "dll" ]]; then
        chmod +x "$target_path"
    fi
    
    success "Successfully installed marty-plugin-${plugin_name} v${version}"
    success "Plugin location: $target_path"
    info "The plugin will be automatically discovered by Marty on next run"
}

# Main installation process
main() {
    info "Marty Plugin Installer"
    info "Installing plugin: $PLUGIN_NAME"
    
    local platform
    platform=$(detect_platform)
    info "Detected platform: $platform"
    
    install_plugin "$PLUGIN_NAME" "$VERSION" "$platform"
    
    cat << EOF

${GREEN}Installation complete!${NC}

Next steps:
1. Run 'marty list' to verify the plugin is loaded
2. The plugin will automatically detect projects in your workspace

For more information, visit: https://github.com/codyspate/marty
EOF
}

# Run main function
main "$@"