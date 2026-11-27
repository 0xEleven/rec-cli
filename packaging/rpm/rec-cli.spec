Name:           rec-cli
Version:        2.1.0
Release:        1%{?dist}
Summary:        Terminal clip recorder and native Rust desktop app for Linux
License:        MIT
URL:            https://github.com/0xEleven/rec-cli
Source0:        %{name}-%{version}.tar.gz

Requires:       bash
Requires:       python3
Requires:       asciinema
Requires:       ffmpeg
Requires:       util-linux

%description
rec-cli is a terminal clip recorder for Linux supporting Zsh, Bash, and Fish,
paired with a fast native Rust desktop app (rec-gui) to start, pause, snapshot,
bookmark, stop, and organize recorded terminal clips for demos, tutorials,
content creation, and proof-of-concepts.

%prep
%setup -q

%install
install -Dm755 bin/rec-cli   %{buildroot}%{_bindir}/rec-cli
install -Dm755 bin/rec-pty   %{buildroot}%{_bindir}/rec-pty
install -Dm755 bin/rec-gui   %{buildroot}%{_bindir}/rec-gui
install -Dm755 bin/cast2mp4  %{buildroot}%{_bindir}/cast2mp4
install -Dm755 bin/casts2mp4 %{buildroot}%{_bindir}/casts2mp4
install -Dm644 shell/rec.zsh  %{buildroot}%{_datadir}/rec-cli/shell/rec.zsh
install -Dm644 shell/rec.bash %{buildroot}%{_datadir}/rec-cli/shell/rec.bash
install -Dm644 shell/rec.fish %{buildroot}%{_datadir}/rec-cli/shell/rec.fish
install -Dm644 starship/rec-module.toml %{buildroot}%{_datadir}/rec-cli/starship/rec-module.toml
install -Dm644 packaging/desktop/rec-cli.desktop %{buildroot}%{_datadir}/applications/rec-cli.desktop
install -Dm644 assets/logo.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/rec-cli.svg

%post
echo ""
echo "rec-cli installed! Enable it in your shell:"
echo '  Zsh  (~/.zshrc):                   eval "$(rec-cli init zsh)"'
echo '  Bash (~/.bashrc):                  eval "$(rec-cli init bash)"'
echo '  Fish (~/.config/fish/config.fish): rec-cli init fish | source'
echo ""
echo "Then launch the desktop app with: rec gui"
echo ""

%files
%license LICENSE
%doc README.md
%{_bindir}/rec-cli
%{_bindir}/rec-pty
%{_bindir}/rec-gui
%{_bindir}/cast2mp4
%{_bindir}/casts2mp4
%{_datadir}/rec-cli/
%{_datadir}/applications/rec-cli.desktop
%{_datadir}/icons/hicolor/scalable/apps/rec-cli.svg

%changelog
* Fri Nov 27 2026 0xEleven <0xEleven@users.noreply.github.com> - 2.1.0-1
- Rebuilt desktop GUI in native Rust (rec-gui) and removed Recordly screen/editor bloat
* Wed Nov 25 2026 0xEleven <0xEleven@users.noreply.github.com> - 1.0.0-1
- Initial release with Zsh, Bash, and Fish support
