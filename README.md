# easy-vial

Run a script from each key of a USB macro pad on Linux.

easy-vial is a small Tauri app with two parts:

- **Window**: a 3×3 grid of Slots. Learn a key by pressing it on the pad, pick a script, test it.
- **Daemon** (`easy-vial --daemon`): runs in the background, grabs the pad exclusively (its keys stop typing into other apps) and runs the matching script on each key press. It keeps running after you close the window and picks up config changes live.

Defaults to the "Pabile P9v2 Macro Pad"; pick any other input device from the dropdown.

## Install

Download the latest `.deb` or AppImage from [Releases](https://github.com/mmvergara/easy-vial/releases).

```sh
sudo apt install ./easy-vial_*_amd64.deb
```

## Setup

1. **Access**: if the pad shows "No access", click **Grant access**. It installs a udev rule (`/etc/udev/rules.d/70-easy-vial.rules`) through a password prompt so the pad is readable without sudo.
2. **Daemon**: click **Start daemon**, and turn on **Start on login** to add it to `~/.config/autostart`.
3. **Slots**: for each key, click **Learn** and press the key on the pad, then **Script** to pick a script. Non-executable scripts run with `bash` (or let easy-vial `chmod +x` them).

Config: `~/.config/easy-vial/config.json`. Daemon log: `~/.cache/easy-vial/daemon.log`.

## Develop

Needs Node, pnpm, Rust and the [Tauri Linux prerequisites](https://tauri.app/start/prerequisites/#linux).

```sh
make install   # pnpm install
make dev       # run the app (closes any running window; the daemon keeps running)
make build     # bump patch version, build the .deb and install it locally
make release   # bump patch version, commit, tag, push; CI publishes the release
```

## License

MIT
