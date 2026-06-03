# 🖨️ ESC/POS Printer Emulator

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey.svg)](https://github.com/digambar9354/escpos-printer-emulator)

> **A cross-platform virtual ESC/POS thermal receipt printer emulator, built in Rust.**
> Turn any computer into a virtual receipt printer to **test, preview and debug POS applications** — no physical thermal printer required.

Point your POS / receipt software at this emulator over the network (TCP port `9100`), and watch every receipt render **live** on screen — text, formatting, barcodes-as-bitmaps, logos and raster images included.

<img width="1920" height="1080" alt="ESC/POS Printer Emulator – receipt preview" src="https://github.com/user-attachments/assets/709335cd-79b9-40fd-ab51-7027f6ee0405" />
<img width="1920" height="1080" alt="ESC/POS Printer Emulator – command log" src="https://github.com/user-attachments/assets/c02db29b-53ca-49e1-b145-6b7cb31e4fc1" />

---

## ✨ Features

- 🌐 **Network printer over TCP/IP** — listens on `0.0.0.0:9100` (the standard RAW / JetDirect / AppSocket port), reachable from **other PCs, tablets and phones on the same LAN**.
- 🧾 **Live receipt preview** — receipts render on a realistic paper sheet in real time as data arrives.
- 🔍 **Zoom (0.5×–3×)** — scale the preview (text *and* images) to inspect fine detail.
- ⬆️ **Newest-on-top** — each print job (split at every paper cut) is shown as its own sheet, latest first.
- 📋 **Command log** — colour-coded, filterable, with optional raw-hex view for low-level debugging.
- 🖼️ **Image support** — renders `GS v 0` raster bitmaps (logos, QR/barcodes printed as images).
- 🌍 **Code page support** — handles non-Latin output (e.g. Arabic) via ESC/POS code-page commands.
- 🪟 **Native look** — uses the system UI font (Segoe UI on Windows) with a clean, compact, IDE-style dark theme.
- 🖨️ **One-click OS printer install** — register the emulator as a Windows or Linux (CUPS) printer.

## 📏 Supported Paper Widths

| Width | Characters | Dots | Use Case |
|-------|------------|------|----------|
| **50mm** | 48 chars | 384 dots | Small receipts, tickets |
| **78mm** | 72 chars | 576 dots | Standard receipts |
| **80mm** | 80 chars | 640 dots | Large receipts, invoices |

---

## 🚀 Quick Start

### Prerequisites
- **Rust 1.70+** — [Install Rust](https://rustup.rs/) (`winget install Rustlang.Rustup` on Windows)
- **Windows 10/11** or **Linux** with CUPS
- **Administrator privileges** (only needed to install the OS printer)

### Installation

1. **Clone your fork**
   ```bash
   git clone https://github.com/digambar9354/escpos-printer-emulator.git
   cd escpos-printer-emulator
   ```

2. **Build**
   ```bash
   cargo build --release
   ```

3. **Run**
   ```bash
   cargo run --release
   ```
   The GUI opens and the server starts listening on `0.0.0.0:9100`.

4. **(Optional) Install as a system printer**
   - Open the **Settings** tab → click **🖨️ Install Windows Printer** (or **🐧 Install Linux Printer**).
   - Requires administrator/root privileges.
   - The printer appears in Windows **Devices and Printers** as `ESC_POS_Virtual_Printer`.

---

## 🌐 Connecting a POS app over the network

This emulator behaves exactly like a real **network thermal printer**: it accepts **raw ESC/POS bytes over TCP port 9100**. Most POS apps (Loyverse, etc.) support this directly.

### 1. Find your PC's LAN IP
```powershell
# Windows
Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.IPAddress -notlike '127.*'}
```
```bash
# Linux
hostname -I
```

### 2. Open the firewall (Windows, run as Administrator)
```powershell
New-NetFirewallRule -DisplayName "ESC/POS Emulator 9100" -Direction Inbound -Protocol TCP -LocalPort 9100 -Action Allow
```

### 3. Add a network printer in your POS app
Use a **Raw / Socket / Ethernet (JetDirect / AppSocket)** printer:

```
IP:   <your-PC-LAN-IP>
Port: 9100
```

> **⚠️ Use "Raw / Socket", not IPP.** Port 9100 is the *raw* printing port. Android's built-in
> *Settings → Add printer by IP* uses **IPP** only and will **not** work with this emulator.
> Configure the printer **inside your POS app** (e.g. Loyverse → *Settings → Printers → Ethernet*),
> choosing a generic **ESC/POS** model — not through the OS "add printer by IP" dialog.

### Quick reachability test
```powershell
Test-NetConnection -ComputerName <your-PC-LAN-IP> -Port 9100   # TcpTestSucceeded : True = OK
```

---

## 🧾 Supported ESC/POS Commands

| Command | Description | Example |
|---------|-------------|---------|
| `ESC @` | Initialize printer | `\x1B@` |
| `ESC M n` | Select font | `\x1BM0` (Font A) |
| `ESC a n` | Justification | `\x1Ba1` (Center) |
| `ESC E` | Emphasis (Bold) | `\x1BE` |
| `ESC - n` | Underline | `\x1B-1` |
| `ESC 4` | Italic | `\x1B4` |
| `ESC 3 n` | Line height | `\x1B324` |
| `ESC ! n` | Font size | `\x1B!16` |
| `ESC t n` | Select code page | `\x1Bt\x16` |
| `GS v 0` | Raster bit image | (logos / images) |
| `GS V` / `ESC m` | Cut paper | `\x1Bm` |

---

## 🛠️ Development

### Project Structure
```
escpos-printer-emulator/
├── src/
│   ├── main.rs              # Entry point, theme & font setup
│   ├── lib.rs               # Library exports
│   ├── escpos/              # ESC/POS command handling
│   │   ├── commands.rs      # Command definitions
│   │   ├── parser.rs        # Command parsing
│   │   └── printer.rs       # Printer state & receipt buffer
│   ├── emulator/            # Core emulator state
│   │   └── mod.rs
│   ├── networking/          # TCP server (port 9100)
│   │   └── server.rs
│   └── gui/                 # egui interface
│       ├── app.rs           # App shell, header, status bar, tabs
│       ├── receipt_viewer.rs# Receipt preview (zoom, newest-on-top)
│       ├── command_log.rs   # Command monitor
│       └── settings_panel.rs# Printer install / network tools
├── Cargo.toml
└── README.md
```

### Building & testing
```bash
cargo build            # Development build
cargo build --release  # Optimized build
cargo run --release    # Run
cargo test             # Tests
cargo check            # Type-check
```

### Tech stack
- **eframe / egui** — immediate-mode GUI
- **tokio** — async runtime & TCP networking
- **image** — bitmap decoding/rendering
- **serde** — serialization
- **tracing** — structured logging

---

## 🔌 About port 9100

`9100` is the de-facto standard **RAW / JetDirect / AppSocket** TCP port for thermal printers.
The emulator listens on `0.0.0.0:9100`, meaning *all* network interfaces — so it is reachable
from `127.0.0.1` (local), your LAN IP, and a hotspot, all at once.

- **Change the port?** Edit the bind address in [`src/networking/server.rs`](src/networking/server.rs)
  (`TcpListener::bind("0.0.0.0:9100")`) and the firewall rule below. Ports below `1024` need
  admin/root; `9100` does not.
- **Only one program can listen on a port at a time.** If another print service already holds
  `9100`, the emulator will fail to start with an "address in use" error — see the platform notes.

---

## 🪟 Windows notes & common issues

**Open the firewall (Administrator PowerShell):**
```powershell
New-NetFirewallRule -DisplayName "ESC/POS Emulator 9100" -Direction Inbound -Protocol TCP -LocalPort 9100 -Action Allow
```

**Check what's using port 9100:**
```powershell
Get-NetTCPConnection -LocalPort 9100 -ErrorAction SilentlyContinue | Select-Object State, OwningProcess
Get-Process -Id (Get-NetTCPConnection -LocalPort 9100).OwningProcess
```

| Issue | Fix |
|-------|-----|
| `cargo: command not found` | Install Rust: `winget install Rustlang.Rustup`, then **reopen** the terminal. |
| `Address already in use (os error 10048)` | Another app holds port 9100 (often the **Print Spooler** or a vendor print service). Stop it, or change the emulator's port. |
| Phone/POS can't reach the PC | Firewall rule missing, or you're on different networks. Verify with `Test-NetConnection -ComputerName <IP> -Port 9100`. |
| "Install Windows Printer" fails | Must run the app **as Administrator**. The installer uses `Add-PrinterPort` / `Add-Printer`. |
| Unrelated "Virtual Printer (Server) – No connection to workstation module" | That's a **remote-desktop / terminal-server** printer redirection error from other software — **not** this emulator. |

---

## 🐧 Ubuntu / Linux notes & common issues

**Install Rust and GUI build dependencies (egui needs X11/Wayland + GL libs):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Debian/Ubuntu build deps for eframe/egui:
sudo apt update
sudo apt install -y build-essential pkg-config \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev \
  libgl1-mesa-dev libxkbcommon-dev libwayland-dev libssl-dev
```

**Open the firewall (UFW):**
```bash
sudo ufw allow 9100/tcp
```

**Check what's using port 9100:**
```bash
sudo ss -ltnp 'sport = :9100'      # or: sudo lsof -i :9100
```

**Install as a CUPS printer (from the Settings tab, or manually):**
```bash
sudo lpadmin -p ESC_POS_Linux_Printer -E -v socket://127.0.0.1:9100 -m 'Generic Text-Only Printer'
sudo lpadmin -d ESC_POS_Linux_Printer
```

| Issue | Fix |
|-------|-----|
| Build fails: missing `xcb`, `GL`, `wayland` headers | Install the GUI build deps listed above. |
| Window doesn't open over SSH / headless | egui needs a display. Use a desktop session, or `WAYLAND_DISPLAY` / `DISPLAY` set correctly. |
| `Address already in use (os error 98)` | Port 9100 is taken: `sudo ss -ltnp 'sport = :9100'`, stop that process or change the port. |
| `Permission denied` binding the port | Only happens for ports < 1024; `9100` is fine without root. |
| CUPS print does nothing | Ensure `cups` is running (`sudo systemctl status cups`) and the device URI is `socket://127.0.0.1:9100`. |
| Blurry text / wrong scaling on HiDPI | Set `WINIT_X11_SCALE_FACTOR=1.5` (or your factor) before launching. |

---

## 🩺 General troubleshooting

| Symptom | Likely cause / fix |
|---------|--------------------|
| Phone/POS can't connect | Same Wi-Fi? Firewall rule added? Some guest/corporate Wi-Fi blocks device-to-device traffic ("client isolation") — use the PC's Mobile Hotspot as a fallback. |
| "Printer offline / not connected" in the POS app | Many apps poll printer **status** before printing. This emulator does not yet answer real-time status queries (`DLE EOT`). See *Roadmap*. |
| Garbage / HTTP text appears as a receipt | The client is sending **IPP** (HTTP) instead of raw ESC/POS. Switch the printer to a **Raw / Socket** connection, not IPP. |
| Receipt doesn't update | Fixed — the preview now auto-refreshes; ensure you're on the latest build. |
| Receipts merge into one sheet | Jobs split at the **paper-cut** command. A POS app that doesn't send a cut will appear as a single continuous receipt. |

---

## 🗺️ Roadmap

- [ ] Respond to ESC/POS real-time status queries (`DLE EOT n`) so POS apps see the printer as **online**.
- [ ] Export receipts as PNG / PDF.
- [ ] Barcode (`GS k`) and QR (`GS ( k`) native rendering.
- [ ] Configurable listen address/port from the Settings tab.

---

## 🤝 Contributing

Issues and pull requests are welcome. Please run `cargo fmt` and `cargo clippy` before submitting.

## 🙏 Acknowledgements

This project is a fork of and builds upon
[Garletz/escpos-virtual-printer-emulator](https://github.com/Garletz/escpos-virtual-printer-emulator).
Thanks to the original author and contributors.

## 📄 License

Licensed under the **MIT License** — see [LICENSE](LICENSE) for details.
