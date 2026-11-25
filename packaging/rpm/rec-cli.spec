Name:           rec-cli
Version:        1.1.0
Release:        1%{?dist}
Summary:        Smart terminal clip recorder CLI for Zsh, Bash, and Fish
License:        MIT
URL:            https://github.com/0xEleven/rec-cli
Source0:        %{name}-%{version}.tar.gz
BuildArch:      noarch

Requires:       bash
Requires:       python3
Requires:       asciinema
Requires:       ffmpeg
Requires:       util-linux

%description
rec-cli is a smart terminal clip recorder for Linux supporting Zsh, Bash, and
Fish. It records real input+output sessions in whitelisted terminals (Kitty and
Tabby by default), skips passive OpenVPN/log panes, renders crisp 1080p 2x-speed
MP4 clips capped at 5 minutes each, and names every clip after the primary tool
or process you ran.

%prep
%setup -q

%install
install -Dm755 bin/rec-cli    %{buildroot}%{_bindir}/rec-cli
install -Dm755 bin/cast2mp4   %{buildroot}%{_bindir}/cast2mp4
install -Dm755 bin/casts2mp4  %{buildroot}%{_bindir}/casts2mp4
install -Dm644 shell/rec.zsh  %{buildroot}%{_datadir}/rec-cli/shell/rec.zsh
install -Dm644 shell/rec.bash %{buildroot}%{_datadir}/rec-cli/shell/rec.bash
install -Dm644 shell/rec.fish %{buildroot}%{_datadir}/rec-cli/shell/rec.fish
install -Dm644 starship/rec-module.toml %{buildroot}%{_datadir}/rec-cli/starship/rec-module.toml

%post
echo ""
echo "rec-cli installed! Enable it in your shell:"
echo '  Zsh  (~/.zshrc):                   eval "$(rec-cli init zsh)"'
echo '  Bash (~/.bashrc):                  eval "$(rec-cli init bash)"'
echo '  Fish (~/.config/fish/config.fish): rec-cli init fish | source'
echo ""
echo "Then run: rec doctor"
echo ""

%files
%license LICENSE
%doc README.md
%{_bindir}/rec-cli
%{_bindir}/cast2mp4
%{_bindir}/casts2mp4
%{_datadir}/rec-cli/

%changelog
* Wed Nov 25 2026 0xEleven <0xEleven@users.noreply.github.com> - 1.0.0-1
- Initial release with Zsh, Bash, and Fish support
