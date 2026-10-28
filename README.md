<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=Raidlords%20Bot&fontAlign=50&fontAlignY=36&fontSize=56&desc=Village%20%7C%20Army%20%7C%20Raid%20%7C%20Quests%20%7C%20Champion%20%7C%20War&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Auto+Village+Build+%26+Resource+Collect;Auto+Army+Train+%7C+Forge+%7C+Research;Auto+Raid+%7C+Revenge+%7C+Battle+Sim;Auto+Quests+%7C+Daily+%7C+Pass+%7C+Events;Auto+Champion+Arena+%26+Duel;Auto+War+%7C+CWL+%7C+Treasury+Withdraw;Wallet+Auth+%7C+Proxy+Support"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-Raidlords-111111"/>
  <img alt="wallet" src="https://img.shields.io/badge/Auth-EVM%20Wallet-111111"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>Raidlords Bot</b> is a full-cycle automation client for the Raidlords village game.<br/>
  It authenticates via an EVM wallet private key, caches the session, and on every cycle builds and upgrades the village, collects resources, trains troops, runs the forge, conducts research, targets and executes raids (with a built-in battle simulator), takes revenges, completes quests, the daily board, season pass, and events, runs champion arena and duels, participates in War and CWL, manages the treasury, and sleeps until the next cycle all with proxy support.<br/>
  Built and distributed by <b>Yuurisandesu</b>.
</p>

</div>

---

## Table of Contents

- [Requirements](#requirements)
- [Install Rust](#install-rust)
- [Installation](#installation)
- [Configuration](#configuration)
- [First-Time Setup](#first-time-setup)
- [Running the Bot](#running-the-bot)
- [Run Modes](#run-modes)
- [Download Prebuilt Binary](#download-prebuilt-binary)
- [Features](#features)
- [Troubleshooting](#troubleshooting)
- [File Structure](#file-structure)
- [Disclaimer](#disclaimer)

---

## Requirements

- Rust `1.70+` (includes `cargo`, only needed if building from source)
- Git
- An EVM wallet private key (the account that owns the village)

---

## Install Rust

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:

```bash
rustc --version
```

### Windows

Download and run the installer from https://rustup.rs, then reopen the terminal and verify:

```cmd
rustc --version
```

### Termux (Android)

```bash
pkg update && pkg install proot-distro
proot-distro install ubuntu
proot-distro login ubuntu
```

Then inside Ubuntu:

```bash
apt update && apt install -y curl git build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## Installation

**Clone the repository:**

```bash
git clone https://github.com/Yuurisan-N1/Raidlords-Bor.git
cd Raidlords-Bor
```

**Build the binary:**

```bash
cargo build --release
# or
./run.sh build
```

---

## Configuration

### 1. Wallet Key (.env)

Copy the example file and fill in your EVM wallet private key:

```bash
cp .env.example .env
```

Edit `.env`:

```
PRIVATEKEY=0x<your 64 hex character private key>
```

Keep this file private and never share it:

```bash
chmod 600 .env
```

### 2. Proxy (proxy.txt)

Fill `proxy.txt` with proxies, one per line (optional, leave empty to run without proxy):

```
http://user:pass@host:port
socks5://user:pass@host:port
```

### 3. Bot Settings (config.json)

`sleep_seconds` controls how many seconds the bot waits between cycles. The default is `3600` (one hour).

```json
{
  "settings": {
    "sleep_seconds": 3600
  }
}
```

---

## First-Time Setup

### 1. Set a village name

Before paying the entry fee, give the village a name. This only needs to be done once:

```bash
./run.sh name "Your Village Name"
```

### 2. Check the entry fee

```bash
./run.sh entry
```

The bot will print the fee in USD and ETH, the destination address, the chain ID, and the exact amount in wei.

### 3. Send the payment

Transfer the displayed amount from your wallet to the address shown. Then submit the transaction hash:

```bash
./run.sh paid 0x<transaction hash>
```

### 4. Verify the setup

Run a self-test to confirm all required files are in place before starting the bot:

```bash
./run.sh selftest
```

Then do a dry-run probe to see what the first cycle would do without executing anything:

```bash
./run.sh probe
```

---

## Running the Bot

Make `run.sh` executable first:

```bash
chmod +x run.sh
```

Then choose a mode:

```bash
./run.sh run        # build if needed and run in foreground, loops indefinitely
./run.sh once       # build if needed and run one cycle then exit
./run.sh bg         # build if needed and run in background, logs saved to raidlords-bot.log
./run.sh screen     # build if needed and run in a detached screen session
./run.sh stop       # stop the background or screen session
./run.sh probe      # read game state and show the planned cycle without executing it
./run.sh selftest   # offline checks, no network
./run.sh build      # build only, do not run
```

For background mode, tail the log anytime:

```bash
tail -f raidlords-bot.log
```

For screen mode, attach anytime:

```bash
screen -r raidlords-bot
```

Press `Ctrl+C` once to stop the bot cleanly when running in foreground.

### Using make (Linux / macOS)

```bash
make release    # optimized release build
make start      # build release and run immediately
make run        # build debug and run
make check      # check for errors without building
make fmt        # format the code
make clean      # remove all build artifacts
make size       # show release binary size
```

### Manual run

```bash
# Linux / macOS / Termux
./target/release/raidlords-bot

# Windows
.\target\release\raidlords-bot.exe
```

---

## Run Modes

| Mode | Command | Description |
|---|---|---|
| run | `./run.sh run` | Full cycle loop: village, army, raids, quests, war, sleep, repeat |
| once | `./run.sh once` | Same as run but exits after one cycle |
| bg | `./run.sh bg` | Same as run but in background with log file |
| screen | `./run.sh screen` | Same as run in a detached screen session |
| probe | `./run.sh probe` | Read game state and print the planned steps without claiming |
| selftest | `./run.sh selftest` | Offline checks: verifies all required files are present |
| entry | `./run.sh entry` | Show the entry fee quote (amount, address, chain, wei) |
| name | `./run.sh name <name>` | Set the village name before paying the entry fee |
| paid | `./run.sh paid <hash>` | Submit the entry payment transaction hash |
| withdraw | `./run.sh withdraw <amount>` | Withdraw USD from the treasury |
| renew | `./run.sh renew` | Force a fresh session sign-in and print its expiry |
| stop | `./run.sh stop` | Stop the background or screen session |
| build | `./run.sh build` | Build only |

---

## Download Prebuilt Binary

Prebuilt binaries are automatically compiled on every push to main via GitHub Actions across 3 platforms.

Download the latest binaries from the Actions page:
https://github.com/Yuurisan-N1/Raidlords-Bor/actions/workflows/build.yml

Open the latest successful run and scroll to the Artifacts section.

| Artifact | Platform |
|---|---|
| `raidlords-bot-linux-x86_64` | Linux x86_64 |
| `raidlords-bot-windows-x86_64` | Windows x86_64 |
| `raidlords-bot-macos-aarch64` | macOS Apple Silicon |

**Linux / macOS after downloading:**

```bash
chmod +x raidlords-bot-linux-x86_64
./raidlords-bot-linux-x86_64
```

**Windows:**

```bash
.\raidlords-bot-windows-x86_64.exe
```

Place the binary in the same folder as `.env`, `proxy.txt`, `config.json`, and `assets/sim.js` before running.

---

## Features

### Wallet Authentication
The bot derives the player address from the EVM private key in `.env` and signs each request using the Ethereum personal sign method. The session token is cached in `session.json` after the first sign-in. On subsequent runs the cached token is reused. If the token expires, a fresh sign-in happens automatically. The private key is never printed to logs.

### Entry Gate
The game requires a one-time on-chain entry payment. The bot checks the gate on every startup and stops cleanly if the entry is unpaid, printing the amount, destination address, chain ID, and wei value. After the transfer, submit the transaction hash with `./run.sh paid <hash>` to unlock the village.

### Auto Village
Each cycle the bot collects gold and resources from all collectors, builds and upgrades buildings up to the configured cap, upgrades walls, removes obstacles, and upgrades the citadel when possible.

### Auto Army
Troops are trained automatically to keep the army ready for raids. The forge is collected each cycle. Hero and equipment upgrades are queued. Research is advanced when resources allow.

### Auto Quests
Daily quests, the quest board, the season pass, and active events are all completed each cycle. Each claimed reward is logged individually.

### Auto Champion
The champion is entered into arena matches and duels each cycle. Gear upgrades are applied when available.

### Auto Chest
Chests are opened automatically when they are ready.

### Auto Store
The in-game store is read each cycle and items are purchased according to the bot's logic.

### Auto War and CWL
The bot checks the war state and submits attacks when a war is active. Clan War League (CWL) attacks are also handled each cycle up to the configured daily cap.

### Treasury and Withdraw
The treasury balance is read each cycle. Use `./run.sh withdraw <amount>` to withdraw USD from the treasury directly.

### Session Cache
After sign-in the session is saved to `session.json`. The cached token is reused on every run. Forced renewal is available with `./run.sh renew`.

### Proxy Support
Proxies are loaded from `proxy.txt`. HTTP and SOCKS5 are both supported. Proxy credentials are masked in log output. Running without a proxy is fully supported.

### Battle Simulator
`assets/sim.js` is the game's own simulation script, loaded and executed locally via the embedded `boa_engine` JavaScript runtime. The sim result guides which targets are attacked and how troops are deployed.

### Auto Countdown
After each cycle completes, the bot displays a live `HH:MM:SS` countdown until the next cycle starts, with a small random jitter to avoid fixed-interval patterns.

---

## Troubleshooting

| Message | Cause | Fix |
|---|---|---|
| `.env holds no usable private key` | `.env` is missing or the key is not 64 hex chars | Fill `.env` with `PRIVATEKEY=0x` followed by 64 hex characters |
| `The village entry is not paid yet` | Entry fee has not been submitted | Run `./run.sh entry` then `./run.sh paid <hash>` |
| `The entry needs a name first` | Village name was never set | Run `./run.sh name <your name>` |
| `The account is banned` | The wallet address is banned on the server | Contact support |
| `The cycle stopped because the server answered 401` | Session expired and renewal failed | Check the private key and run `./run.sh renew` |
| `sim.js is missing` | `assets/sim.js` was not included | Re-clone the repository or restore the file from the release |
| `The server is rate limiting` | Too many requests in a short time | Increase `sleep_seconds` in `config.json` or add a proxy |
| `The game moved to build X` | Game was updated, raids paused | Wait for a bot update |

---

## File Structure

```text
Raidlords-Bor/
├── .github/
│   └── workflows/
│       └── build.yml                    # CI: 3-platform matrix build
├── src/
│   ├── main.rs                          # Entry point, Ctrl+C handler, mode dispatch
│   ├── logika.rs                        # AI planner: action graph, sleep logic
│   ├── army/
│   │   ├── equip.rs                     # Equipment upgrade
│   │   ├── forge.rs                     # Forge collect
│   │   ├── hero.rs                      # Hero upgrade
│   │   ├── research.rs                  # Research queue
│   │   ├── train.rs                     # Troop training
│   │   └── mod.rs
│   ├── champ/
│   │   ├── arena.rs                     # Champion arena
│   │   ├── duel.rs                      # Champion duel
│   │   ├── gear.rs                      # Champion gear
│   │   └── mod.rs
│   ├── chest/
│   │   ├── chest.rs                     # Chest open
│   │   └── mod.rs
│   ├── cli/
│   │   ├── dispatch.rs                  # Mode selector
│   │   ├── probe.rs                     # Probe: read without executing
│   │   ├── selftest.rs                  # Offline file checks
│   │   ├── verify.rs                    # selftest alias
│   │   └── mod.rs
│   ├── entry/
│   │   ├── entry.rs                     # Entry gate: check, quote, submit, set name
│   │   └── mod.rs
│   ├── net/
│   │   ├── client.rs                    # HTTP client
│   │   ├── device.rs                    # Device fingerprint
│   │   ├── edge.rs                      # API edge (RPC caller)
│   │   ├── pace.rs                      # Request pacing and jitter
│   │   ├── proxy.rs                     # Proxy loader and masker
│   │   └── mod.rs
│   ├── quests/
│   │   ├── daily.rs                     # Daily quest board
│   │   ├── event.rs                     # Events and season
│   │   ├── pass.rs                      # Season pass
│   │   ├── quest.rs                     # Quest list and claim
│   │   └── mod.rs
│   ├── raid/
│   │   ├── board.rs                     # Raid board fetch
│   │   ├── finish.rs                    # Finish raid early
│   │   ├── revenge.rs                   # Revenge attacks
│   │   ├── siege.rs                     # Battle simulator wrapper
│   │   ├── target.rs                    # Target scan and selection
│   │   └── mod.rs
│   ├── run/
│   │   ├── cycle.rs                     # Main cycle loop and state refresh
│   │   ├── plan.rs                      # Action executor
│   │   ├── session.rs                   # Sign-in and session renewal
│   │   └── mod.rs
│   ├── state/
│   │   ├── clock.rs                     # Timer helpers
│   │   ├── katalog.rs                   # Game catalog parser
│   │   ├── snapshot.rs                  # Full game state snapshot
│   │   ├── store.rs                     # State store
│   │   └── mod.rs
│   ├── store/
│   │   ├── market.rs                    # Market items
│   │   ├── store.rs                     # Store fetch and buy
│   │   └── mod.rs
│   ├── support/
│   │   ├── banner.rs                    # ASCII banner
│   │   ├── codec.rs                     # Hex encode/decode helpers
│   │   ├── error.rs                     # Error types
│   │   ├── log.rs                       # Log init, colored format, countdown
│   │   ├── cfg/                         # Config loader and schema
│   │   ├── consts/                      # App and network constants
│   │   └── mod.rs
│   ├── treasury/
│   │   ├── sell.rs                      # Treasury sell
│   │   ├── withdraw.rs                  # Treasury withdraw
│   │   └── mod.rs
│   ├── village/
│   │   ├── build.rs                     # Building upgrades
│   │   ├── citadel.rs                   # Citadel upgrade
│   │   ├── collect.rs                   # Resource collectors
│   │   ├── obstacle.rs                  # Obstacle removal
│   │   ├── walls.rs                     # Wall upgrades
│   │   └── mod.rs
│   ├── wallet/
│   │   ├── address.rs                   # Address derivation and keccak
│   │   ├── nonce.rs                     # Nonce management
│   │   ├── sign.rs                      # Ethereum personal sign
│   │   ├── token.rs                     # Token handling
│   │   └── mod.rs
│   └── war/
│       ├── cwl.rs                       # Clan War League attacks
│       ├── war.rs                       # War state and attacks
│       └── mod.rs
├── assets/
│   ├── icon.ico                         # Windows binary icon
│   ├── icon.png                         # App icon
│   └── sim.js                           # Battle simulator (game-supplied JS, run via boa_engine)
├── .env.example                         # Private key template
├── .gitignore
├── build.rs                             # Build script (embeds icon into Windows binary)
├── Cargo.toml                           # Project manifest and dependencies
├── Cargo.lock
├── Makefile                             # make targets: build, run, release, start, clean, size
├── run.sh                               # Run helper: all modes in one script
├── config.json                          # Sleep duration setting
├── proxy.txt                            # Proxy list (optional)
├── session.json                         # Session token cache (auto-generated, gitignored)
├── device.json                          # Device fingerprint cache (auto-generated, gitignored)
└── LICENSE                              # GNU AGPL v3.0
```

---

## Disclaimer

This tool is built for educational and technical exploration purposes. Use it wisely and at your own responsibility.

---

<div align="center">
<img width="100%" alt="footer" src="https://capsule-render.vercel.app/api?type=waving&height=120&section=footer"/>
</div>