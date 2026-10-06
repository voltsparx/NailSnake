#!/usr/bin/env sh
set -eu

APP_NAME="nailsnake"
DISPLAY_NAME="NailSnake"
REPO_URL="https://github.com/voltsparx/NailSnake"
ROOT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
VERSION=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$DIST_DIR/stage-linux"

say() {
    printf '%s\n' "$*"
}

ask_yes_no() {
    prompt=$1
    default=${2:-n}
    if [ "$default" = "y" ]; then suffix="[Y/n]"; else suffix="[y/N]"; fi
    printf '%s %s ' "$prompt" "$suffix"
    read -r answer
    answer=${answer:-$default}
    case "$answer" in y|Y|yes|YES) return 0 ;; *) return 1 ;; esac
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1
}

sudo_cmd() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif need_cmd sudo; then
        sudo "$@"
    else
        say "Root privileges are required, but sudo was not found."
        say "Re-run as root or install sudo."
        return 1
    fi
}

detect_os() {
    OS_ID=unknown
    OS_NAME=$(uname -s)
    if [ -r /etc/os-release ]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        OS_ID=${ID:-unknown}
        OS_NAME=${PRETTY_NAME:-$OS_ID}
    fi
}

detect_package_manager() {
    if need_cmd apt-get; then PM="apt"; FORMAT="deb"
    elif need_cmd pacman; then PM="pacman"; FORMAT="pkg.tar.zst"
    elif need_cmd dnf; then PM="dnf"; FORMAT="rpm"
    elif need_cmd zypper; then PM="zypper"; FORMAT="rpm"
    elif need_cmd xbps-install; then PM="xbps"; FORMAT="xbps"
    elif need_cmd yum; then PM="yum"; FORMAT="rpm"
    elif need_cmd apk; then PM="apk"; FORMAT="tar.gz"
    else PM="unknown"; FORMAT="tar.gz"
    fi
}

missing_common_tools() {
    missing=""
    for cmd in cargo git tar gzip awk; do
        if ! need_cmd "$cmd"; then
            missing="$missing $cmd"
        fi
    done
    printf '%s' "$missing"
}

missing_format_tools() {
    kind=$1
    missing=""
    case "$kind" in
        pkg.tar.zst)
            need_cmd makepkg || missing="$missing makepkg"
            ;;
        deb)
            need_cmd dpkg-deb || missing="$missing dpkg-deb"
            need_cmd dpkg || missing="$missing dpkg"
            ;;
        rpm)
            need_cmd rpmbuild || missing="$missing rpmbuild"
            ;;
        xbps)
            need_cmd xbps-create || missing="$missing xbps-create"
            ;;
        tar.gz)
            ;;
    esac
    printf '%s' "$missing"
}

install_prerequisites() {
    kind=$1
    common=$(missing_common_tools)
    format_missing=$(missing_format_tools "$kind")
    missing="$common $format_missing"

    if [ -z "$(printf '%s' "$missing" | tr -d ' ')" ]; then
        say "Prerequisites: ok"
        return 0
    fi

    say "Missing prerequisites:$missing"
    ask_yes_no "Install missing prerequisites using $PM?" "y" || return 1

    case "$PM" in
        apt)
            sudo_cmd apt-get update
            sudo_cmd apt-get install -y build-essential cargo git tar gzip dpkg-dev
            ;;
        pacman)
            sudo_cmd pacman -Sy --needed base-devel rust cargo git tar gzip
            ;;
        dnf)
            sudo_cmd dnf install -y gcc gcc-c++ make cargo git tar gzip rpm-build
            ;;
        zypper)
            sudo_cmd zypper install -y gcc gcc-c++ make cargo git tar gzip rpm-build
            ;;
        yum)
            sudo_cmd yum install -y gcc gcc-c++ make cargo git tar gzip rpm-build
            ;;
        xbps)
            sudo_cmd xbps-install -Sy base-devel rust cargo git tar gzip xbps
            ;;
        apk)
            sudo_cmd apk add build-base cargo git tar gzip
            ;;
        unknown)
            say "No supported package manager was detected."
            say "Install Rust/Cargo, git, tar, gzip, and the package builder manually."
            return 1
            ;;
    esac
}

prepare_stage() {
    prefix=${1:-usr}
    rm -rf "$STAGE_DIR"
    mkdir -p "$STAGE_DIR/$prefix/bin" "$STAGE_DIR/$prefix/share/man/man1" \
        "$STAGE_DIR/$prefix/share/doc/$APP_NAME" "$STAGE_DIR/$prefix/share/licenses/$APP_NAME"
    cp "$ROOT_DIR/target/release/$APP_NAME" "$STAGE_DIR/$prefix/bin/$APP_NAME"
    cp "$ROOT_DIR/man/$APP_NAME.1" "$STAGE_DIR/$prefix/share/man/man1/$APP_NAME.1"
    cp "$ROOT_DIR/README.md" "$STAGE_DIR/$prefix/share/doc/$APP_NAME/README.md"
    cp "$ROOT_DIR/LICENSE" "$STAGE_DIR/$prefix/share/licenses/$APP_NAME/LICENSE"
}

build_binary() {
    say "Building $DISPLAY_NAME release binary..."
    cd "$ROOT_DIR"
    cargo build --release --locked
}

build_arch_package() {
    cd "$ROOT_DIR"
    makepkg -f >&2
    package=$(find "$ROOT_DIR" -maxdepth 1 -name "$APP_NAME-$VERSION-*.pkg.tar.zst" | sort | tail -n 1)
    printf '%s' "$package"
}

build_deb_package() {
    prepare_stage usr
    mkdir -p "$STAGE_DIR/DEBIAN"
    installed_size=$(du -sk "$STAGE_DIR/usr" | awk '{print $1}')
    arch=$(dpkg --print-architecture 2>/dev/null || printf 'amd64')
    cat > "$STAGE_DIR/DEBIAN/control" <<EOF
Package: $APP_NAME
Version: $VERSION
Section: games
Priority: optional
Architecture: $arch
Maintainer: Voltsparx <voltsparx@gmail.com>
Installed-Size: $installed_size
Description: Cross-platform terminal Snake game written in Rust
 $DISPLAY_NAME is a full-screen terminal Snake game with persistent stats,
 rich terminal colors, and a manual page.
Homepage: $REPO_URL
EOF
    package="$DIST_DIR/${APP_NAME}_${VERSION}_${arch}.deb"
    dpkg-deb --build "$STAGE_DIR" "$package" >/dev/null
    printf '%s' "$package"
}

build_rpm_package() {
    buildroot="$DIST_DIR/rpmbuild"
    rm -rf "$buildroot"
    mkdir -p "$buildroot/BUILD" "$buildroot/RPMS" "$buildroot/SOURCES" "$buildroot/SPECS" "$buildroot/SRPMS"
    tarball="$buildroot/SOURCES/$APP_NAME-$VERSION.tar.gz"
    git -C "$ROOT_DIR" archive --format=tar.gz --prefix="$APP_NAME-$VERSION/" HEAD > "$tarball"
    cat > "$buildroot/SPECS/$APP_NAME.spec" <<EOF
Name:           $APP_NAME
Version:        $VERSION
Release:        1%{?dist}
Summary:        Cross-platform terminal Snake game written in Rust
License:        MIT
URL:            $REPO_URL
Source0:        %{name}-%{version}.tar.gz
BuildRequires:  cargo

%description
$DISPLAY_NAME is a full-screen terminal Snake game with persistent stats,
rich terminal colors, and a manual page.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -Dm755 target/release/$APP_NAME %{buildroot}%{_bindir}/$APP_NAME
install -Dm644 man/$APP_NAME.1 %{buildroot}%{_mandir}/man1/$APP_NAME.1
install -Dm644 LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md

%files
%{_bindir}/$APP_NAME
%{_mandir}/man1/$APP_NAME.1*
%{_licensedir}/%{name}/LICENSE
%{_docdir}/%{name}/README.md
EOF
    rpmbuild --define "_topdir $buildroot" -bb "$buildroot/SPECS/$APP_NAME.spec" >&2
    package=$(find "$buildroot/RPMS" -name "*.rpm" | sort | tail -n 1)
    printf '%s' "$package"
}

build_xbps_package() {
    prepare_stage usr
    arch=$(uname -m)
    package="$DIST_DIR/${APP_NAME}-${VERSION}_1.${arch}.xbps"
    (
        cd "$DIST_DIR"
        xbps-create \
            -A "$arch" \
            -n "${APP_NAME}-${VERSION}_1" \
            -s "Cross-platform terminal Snake game written in Rust" \
            -l MIT \
            -H "$REPO_URL" \
            "$STAGE_DIR" >/dev/null
    )
    built=$(find "$DIST_DIR" -maxdepth 1 -name "${APP_NAME}-${VERSION}_1.*.xbps" | sort | tail -n 1)
    mv "$built" "$package"
    printf '%s' "$package"
}

build_tar_package() {
    prepare_stage usr/local
    package="$DIST_DIR/${APP_NAME}-${VERSION}-linux-$(uname -m).tar.gz"
    (cd "$STAGE_DIR" && tar -czf "$package" .)
    printf '%s' "$package"
}

install_package() {
    package=$1
    case "$package" in
        *.pkg.tar.zst) sudo_cmd pacman -U --needed "$package" ;;
        *.deb)
            if need_cmd apt-get; then sudo_cmd apt-get install -y "$package"
            else sudo_cmd dpkg -i "$package"
            fi
            ;;
        *.rpm)
            if need_cmd dnf; then sudo_cmd dnf install -y "$package"
            elif need_cmd zypper; then sudo_cmd zypper install -y "$package"
            elif need_cmd yum; then sudo_cmd yum install -y "$package"
            else sudo_cmd rpm -Uvh "$package"
            fi
            ;;
        *.xbps) sudo_cmd xbps-install -y --repository="$DIST_DIR" "$APP_NAME" ;;
        *.tar.gz)
            say "Generic tar install extracts under /usr/local and needs root."
            sudo_cmd tar -xzf "$package" -C /
            if need_cmd mandb; then sudo_cmd mandb -q || true; fi
            ;;
        *) say "No installer rule for $package"; return 1 ;;
    esac
}

choose_format() {
    say "Choose package type:"
    say "  1) $FORMAT        detected for $PM"
    say "  2) deb           apt/dpkg systems"
    say "  3) pkg.tar.zst   pacman/makepkg systems"
    say "  4) rpm           dnf/yum/zypper systems"
    say "  5) xbps          Void Linux"
    say "  6) tar.gz        generic fallback"
    printf 'Selection [recommended: %s]: ' "$FORMAT"
    read -r choice
    case "${choice:-$FORMAT}" in
        1) printf '%s' "$FORMAT" ;;
        2|deb) printf '%s' "deb" ;;
        3|pkg|pkg.tar.zst|zst) printf '%s' "pkg.tar.zst" ;;
        4|rpm) printf '%s' "rpm" ;;
        5|xbps) printf '%s' "xbps" ;;
        6|tar|tar.gz) printf '%s' "tar.gz" ;;
        *) printf '%s' "$FORMAT" ;;
    esac
}

main() {
    mkdir -p "$DIST_DIR"
    detect_os
    detect_package_manager

    say "$DISPLAY_NAME Linux installer"
    say "Detected OS: $OS_NAME"
    say "Detected package manager: $PM"
    say "Recommended package format: $FORMAT"
    say ""

    kind=$(choose_format)
    install_prerequisites "$kind"
    build_binary

    case "$kind" in
        pkg.tar.zst) package=$(build_arch_package) ;;
        deb) package=$(build_deb_package) ;;
        rpm) package=$(build_rpm_package) ;;
        xbps) package=$(build_xbps_package) ;;
        tar.gz) package=$(build_tar_package) ;;
        *) say "Unsupported package type: $kind"; exit 1 ;;
    esac

    say "Created package: $package"
    if ask_yes_no "Install it now?" "n"; then
        install_package "$package"
        say "$DISPLAY_NAME installed. Run: $APP_NAME"
    else
        say "Package left at: $package"
    fi
}

main "$@"
