#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
python3 - <<'PY'
import os, pathlib, re, subprocess, sys, time
text = pathlib.Path('rust-toolchain.toml').read_text()
channel = re.findall(r'(?m)^\s*channel\s*=\s*"([A-Za-z0-9._-]+)"\s*$', text)
profile = re.findall(r'(?m)^\s*profile\s*=\s*"([A-Za-z0-9._-]+)"\s*$', text)
components = re.findall(r'(?m)^\s*components\s*=\s*\[([^\]]*)\]\s*$', text)
if len(channel) != 1 or len(profile) != 1 or len(components) != 1:
    raise SystemExit('Unsupported root toolchain shape; refusing another version authority')
names = re.findall(r'"([A-Za-z0-9._-]+)"', components[0])
if re.sub(r'"[A-Za-z0-9._-]+"', '', components[0]).replace(',', '').strip() or not names:
    raise SystemExit('Invalid root toolchain component list')
pin = channel[0]
def current():
    # An installed exact toolchain needs no network. Don't auto-install in this probe.
    env = dict(os.environ, RUSTUP_AUTO_INSTALL='0')
    v = subprocess.run(['rustup', 'run', pin, 'rustc', '-vV'], capture_output=True, text=True, env=env)
    if v.returncode or f'release: {pin}\n' not in v.stdout:
        return None
    host = re.search(r'^host: ([^\s]+)$', v.stdout, re.M)
    if not host:
        return None
    c = subprocess.run(['rustup', 'component', 'list', '--installed', '--toolchain', pin], capture_output=True, text=True, env=env)
    installed = set(c.stdout.splitlines())
    if c.returncode or not all(n in installed or f'{n}-{host[1]}' in installed for n in names):
        return None
    return v.stdout

version = current()
if not version:
    args = ['rustup', 'toolchain', 'install', pin, '--profile', profile[0], '--component', ','.join(names)]
    for attempt in range(3):
        fetched = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        print(fetched.stdout, end='', file=sys.stderr)
        if fetched.returncode == 0:
            break
        if attempt == 2 or not re.search(r'dns error|failed to lookup address|timed out|connection reset|HTTP (502|503|504)', fetched.stdout, re.I):
            raise SystemExit(fetched.returncode)
        time.sleep(2 ** (attempt + 1))
    version = current()
if not version:
    raise SystemExit('Installed compiler or components differ from the root toolchain')
print(version, end='')
PY
