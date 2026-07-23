#!/usr/bin/env sh
set -eu

APP_NAME="nailsnake"
DISPLAY_NAME="NailSnake"
VERSION="1.0.0"
REPO_URL="https://github.com/voltsparx/NailSnake"
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$DIST_DIR/stage-linux"

say() {
    printf '%s\n' "$*"
}

ask_yes_no() {
    prompt=$1
    default=${2:-n}
    if [ "$default" = "y" ]; then
        suffix="[Y/n]"
    else
        suffix="[y/N]"
    fi
    printf '%s %s ' "$prompt" "$suffix"
    read answer
    answer=${answer:-$default}
    case "$answer" in
        y|Y|yes|YES) return 0 ;;
        *) return 1 ;;
    esac
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
        say "This action needs root privileges, but sudo was not found."
        return 1
    fi
}

detect_os() {
    if [ -r /etc/os-release ]; then
        . /etc/os-release
        OS_ID=${ID:-unknown}
        OS_LIKE=${ID_LIKE:-}
        OS_NAME=${PRETTY_NAME:-$OS_ID}
    else
        OS_ID=unknown
        OS_LIKE=
        OS_NAME=$(uname -s)
    fi
}

suggest_package() {
    case " $OS_ID $OS_LIKE " in
        *" arch "*|*" manjaro "*) printf '%s' "pkg.tar.zst" ;;
        *" debian "*|*" ubuntu "*|*" linuxmint "*|*" pop "*) printf '%s' "deb" ;;
        *" fedora "*|*" rhel "*|*" centos "*|*" suse "*|*" opensuse "*) printf '%s' "rpm" ;;
        *) printf '%s' "tar.gz" ;;
    esac
}

build_binary() {
    say "Building $DISPLAY_NAME release binary..."
    cd "$ROOT_DIR"
    cargo build --release --locked
}

prepare_stage() {
    rm -rf "$STAGE_DIR"
    mkdir -p "$STAGE_DIR/usr/bin" "$STAGE_DIR/usr/share/man/man1" \
        "$STAGE_DIR/usr/share/doc/$APP_NAME" "$STAGE_DIR/usr/share/licenses/$APP_NAME"
    cp "$ROOT_DIR/target/release/$APP_NAME" "$STAGE_DIR/usr/bin/$APP_NAME"
    cp "$ROOT_DIR/man/$APP_NAME.1" "$STAGE_DIR/usr/share/man/man1/$APP_NAME.1"
    cp "$ROOT_DIR/README.md" "$STAGE_DIR/usr/share/doc/$APP_NAME/README.md"
    cp "$ROOT_DIR/LICENSE" "$STAGE_DIR/usr/share/licenses/$APP_NAME/LICENSE"
}

build_arch_package() {
    if need_cmd makepkg; then
        cd "$ROOT_DIR"
        makepkg -f >&2
        package=$(find "$ROOT_DIR" -maxdepth 1 -name "$APP_NAME-$VERSION-*.pkg.tar.zst" | sort | tail -n 1)
        printf '%s' "$package"
    else
        say "makepkg is required for .pkg.tar.zst packages."
        return 1
    fi
}

build_deb_package() {
    need_cmd dpkg-deb || { say "dpkg-deb is required for .deb packages."; return 1; }
    prepare_stage
    mkdir -p "$STAGE_DIR/DEBIAN"
    installed_size=$(du -sk "$STAGE_DIR/usr" | awk '{print $1}')
    cat > "$STAGE_DIR/DEBIAN/control" <<EOF
Package: $APP_NAME
Version: $VERSION
Section: games
Priority: optional
Architecture: $(dpkg --print-architecture 2>/dev/null || printf 'amd64')
Maintainer: Voltsparx <voltsparx@gmail.com>
Installed-Size: $installed_size
Description: Cross-platform terminal Snake game written in Rust
 $DISPLAY_NAME is a full-screen terminal Snake game with persistent stats,
 rich terminal colors, and a manual page.
Homepage: $REPO_URL
EOF
    package="$DIST_DIR/${APP_NAME}_${VERSION}_$(dpkg --print-architecture 2>/dev/null || printf 'amd64').deb"
    dpkg-deb --build "$STAGE_DIR" "$package" >/dev/null
    printf '%s' "$package"
}

build_rpm_package() {
    need_cmd rpmbuild || { say "rpmbuild is required for .rpm packages."; return 1; }
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

build_tar_package() {
    prepare_stage
    package="$DIST_DIR/${APP_NAME}-${VERSION}-linux-$(uname -m).tar.gz"
    (cd "$STAGE_DIR" && tar -czf "$package" .)
    printf '%s' "$package"
}

install_package() {
    package=$1
    case "$package" in
        *.pkg.tar.zst) sudo_cmd pacman -U "$package" ;;
        *.deb) sudo_cmd dpkg -i "$package" ;;
        *.rpm)
            if need_cmd dnf; then sudo_cmd dnf install -y "$package"
            elif need_cmd zypper; then sudo_cmd zypper install -y "$package"
            elif need_cmd yum; then sudo_cmd yum install -y "$package"
            else sudo_cmd rpm -Uvh "$package"
            fi
            ;;
        *.tar.gz)
            sudo_cmd tar -xzf "$package" -C /
            if need_cmd mandb; then sudo_cmd mandb -q || true; fi
            ;;
        *) say "No installer rule for $package"; return 1 ;;
    esac
}

main() {
    mkdir -p "$DIST_DIR"
    detect_os
    suggested=$(suggest_package)

    say "$DISPLAY_NAME Linux installer"
    say "Detected OS: $OS_NAME"
    say "Recommended package: $suggested"
    say ""
    say "Choose package type:"
    say "  1) pkg.tar.zst  Arch/Manjaro/EndeavourOS"
    say "  2) deb          Debian/Ubuntu/Linux Mint/Pop!_OS"
    say "  3) rpm          Fedora/RHEL/CentOS/openSUSE"
    say "  4) tar.gz       Generic Linux fallback"
    printf 'Selection [recommended: %s]: ' "$suggested"
    read choice
    case "${choice:-$suggested}" in
        1|pkg|pkg.tar.zst|zst) kind="pkg.tar.zst" ;;
        2|deb) kind="deb" ;;
        3|rpm) kind="rpm" ;;
        4|tar|tar.gz) kind="tar.gz" ;;
        *) kind="$suggested" ;;
    esac

    build_binary
    case "$kind" in
        pkg.tar.zst) package=$(build_arch_package) ;;
        deb) package=$(build_deb_package) ;;
        rpm) package=$(build_rpm_package) ;;
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
