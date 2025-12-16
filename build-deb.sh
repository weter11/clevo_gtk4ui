#!/bin/bash
set -e

echo "=========================================="
echo "TUXEDO Control Center - DEB Package Builder"
echo "=========================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run from project root directory"
    echo "   (Directory containing Cargo.toml)"
    exit 1
fi

if [ ! -d "debian" ]; then
    echo "❌ Error: debian/ directory not found"
    echo "   Please create debian packaging files first"
    exit 1
fi

# Check for required tools
echo "🔍 Checking build dependencies..."
MISSING_DEPS=""

command -v cargo >/dev/null 2>&1 || MISSING_DEPS="$MISSING_DEPS cargo"
command -v dpkg-buildpackage >/dev/null 2>&1 || MISSING_DEPS="$MISSING_DEPS devscripts"
pkg-config --exists gtk4 2>/dev/null || MISSING_DEPS="$MISSING_DEPS libgtk-4-dev"
pkg-config --exists libadwaita-1 2>/dev/null || MISSING_DEPS="$MISSING_DEPS libadwaita-1-dev"

if [ -n "$MISSING_DEPS" ]; then
    echo "❌ Missing dependencies:$MISSING_DEPS"
    echo ""
    echo "Installing missing dependencies..."
    sudo apt-get update
    sudo apt-get install -y build-essential debhelper cargo rustc \
        libgtk-4-dev libadwaita-1-dev pkg-config devscripts
fi

echo "✅ All dependencies available"
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean
rm -f ../tuxedo-control-center_*.deb
rm -f ../tuxedo-control-center_*.buildinfo
rm -f ../tuxedo-control-center_*.changes

# Build the package
echo "📦 Building Debian package..."
echo ""
dpkg-buildpackage -us -uc -b

# Check if build was successful
if [ $? -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "✅ BUILD SUCCESSFUL!"
    echo "=========================================="
    echo ""
    echo "📦 Package created:"
    ls -lh ../tuxedo-control-center_*.deb 2>/dev/null || echo "   (Package file not found)"
    echo ""
    echo "📋 To install, run:"
    echo "   sudo dpkg -i ../tuxedo-control-center_*.deb"
    echo "   sudo apt-get install -f"
    echo ""
    echo "🔍 To verify installation:"
    echo "   systemctl status tuxedo-daemon"
    echo "   tuxedo-control-center"
    echo ""
    echo "🗑️  To uninstall:"
    echo "   sudo apt-get remove tuxedo-control-center"
    echo ""
else
    echo ""
    echo "=========================================="
    echo "❌ BUILD FAILED"
    echo "=========================================="
    echo ""
    echo "Check the error messages above for details."
    exit 1
fi