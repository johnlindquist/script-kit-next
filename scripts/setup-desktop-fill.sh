#!/bin/bash
# Install only the optional, pinned filename provider. Does not grant macOS
# permissions, launch the app/helper, change PATH, or replace an existing build.
set -euo pipefail
if [[ $# -ne 0 ]]; then
  printf 'Usage: [SK_PATH=/absolute/kit] bash scripts/setup-desktop-fill.sh\nInstalls the optional pinned macOS helper. No permissions are granted.\n'
  [[ $# == 1 && ( $1 == --help || $1 == -h ) ]] && exit 0
  exit 2
fi
umask 077
[[ $(uname -s) == Darwin ]] || { echo 'Requires macOS.' >&2; exit 2; }
case $(uname -m) in
  arm64) arch=aarch64; digest=7bee385196d7b478de80cf05ca4304aaa38f0eba593b3406ad3c247c1773be19;;
  x86_64) arch=x86_64; digest=8edda6df17eafc042395502a823dbb8415bc8ac14835a6731e4d8becc9cb4121;;
  *) echo 'Unsupported architecture.' >&2; exit 2;;
esac
kit=${SK_PATH:-"$HOME/.scriptkit"}
[[ $kit == /* && -d $kit ]] || { echo 'SK_PATH must name an existing absolute kit directory.' >&2; exit 2; }
parent="$kit/tools/agent-desktop"; dest="$parent/1.0.2"
[[ ! -L "$kit/tools" && ! -L "$parent" ]] || { echo 'Refusing symlinked tool directories.' >&2; exit 1; }
mkdir -p "$parent"
mkdir "$parent/.install-lock" || { echo 'Installation busy; not changing existing files.' >&2; exit 1; }
work=''; trap 'if [[ -n $work ]]; then rm -rf -- "$work"; fi; rmdir "$parent/.install-lock"' EXIT
work=$(mktemp -d "$parent/.stage.XXXXXXXX"); mkdir "$work/bin"
url="https://github.com/vercel-labs/agent-desktop/releases/download/v1.0.2/agent-desktop-v1.0.2-${arch}-apple-darwin.tar.gz"
curl --proto '=https' --proto-redir '=https' --fail --location --silent --show-error --max-time 90 "$url" -o "$work/release.tgz"
printf '%s  %s\n' "$digest" "$work/release.tgz" | shasum -a 256 -c - >/dev/null
for bin in agent-desktop agent-desktop-macos-helper; do
    tar -xOf "$work/release.tgz" "$bin" > "$work/bin/$bin"; chmod 700 "$work/bin/$bin"
done
if [[ -e $dest || -L $dest ]]; then
    [[ -d $dest && ! -L $dest && ! -L $dest/agent-desktop && ! -L $dest/agent-desktop-macos-helper && -x $dest/agent-desktop && -x $dest/agent-desktop-macos-helper ]] && diff -qr "$work/bin" "$dest" >/dev/null || { echo 'An unrecognized installation exists. Nothing replaced.' >&2; exit 1; }
else
    mv "$work/bin" "$dest"
fi
printf 'Verified agent-desktop 1.0.2: %s/agent-desktop\n' "$dest"
