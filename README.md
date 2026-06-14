# ROM Builder

A desktop tool for assembling, inspecting, and exporting ROM images for
programmable memory ICs (EPROM / EEPROM / NOR flash), aimed at retro-computing
projects. Take individual ROM dumps, lay them out into banks on a target chip,
and export a single binary ready to program.

Built in Rust with [`eframe`/`egui`](https://github.com/emilk/egui).

## Features

- **Bank layout** — divide a target IC into linear banks of a chosen ROM size
  (4k–512k) and place ROMs into them.
- **Multi-ROM per bank** — drop a ROM smaller than a bank and fill the leftover
  space with more ROMs as indexed sub-slots (`0`, `0.1`, `0.2`, …).
- **Target IC catalog** — common UV EPROMs (2716–27C080), Winbond reusable
  EEPROM-types (W27Cxxx), parallel EEPROMs (28C/AT28C), and NOR flash (SST39SF,
  Am29F, AT29C); pick by size or from a filterable IC list.
- **Bank map** — color-coded grid (unused / filled / duplicated) with a legend
  and pagination.
- **Duplicate detection** — banks/slots sharing a checksum are highlighted.
- **Hex preview** — paged hex view of any populated bank.
- **Name suggestions** — propose names from printable strings inside a ROM.
- **Add ROM handling** — exact-size ROMs drop straight in; larger ones can be
  trimmed or spread across banks; smaller ones keep the free remainder.
- **Safety prompts** — confirmations (with Enter/Esc shortcuts) before clipping
  to a smaller chip, splitting/consolidating on a layout change, removing banks,
  or quitting with unsaved changes.
- **Projects** — save/restore the full editing state as a `.romproj` file.
- **Export** — write the chip-sized `.bin` plus a `.metadata.txt` summary.

## Build & run

Requires a recent stable Rust toolchain.

```sh
cargo run            # build and launch
cargo build          # debug build  -> build/debug/rom-builder.exe
cargo test           # run unit tests
```

(Build output goes under `build/` via `.cargo/config.toml`.)

## Usage

The app starts with only **New Project**, **Open Project**, and **About**
available; everything else turns on once a project is active.

1. **New Project** (Ctrl+N), then pick a **ROM Layout** size and a **Target IC**.
2. **Add ROM** (or double-click a bank) to place ROM files into banks.
3. Manage banks in the list: rename, suggest names, replace, extract, remove.
4. **Save** (Ctrl+S) / **Save As** the project, and **Export Binary** (Ctrl+E)
   to produce the file for your programmer.

Keyboard shortcuts: **Ctrl+N/O/S/E** for New/Open/Save/Export; **Enter/Esc** to
confirm/cancel dialogs.

## Project files (`.romproj`)

A `.romproj` is a zip archive containing each populated bank's binary
(`banks/bank_NNN.bin`) and a `project.json` describing the chip, layout, pad
byte, UI settings, and per-bank slot layout (names, lengths, source paths). This
preserves sub-slot boundaries and names across sessions.

## Exported output

**Export Binary** writes a raw, chip-sized `.bin` (banks placed linearly, the
rest padded) and a `.metadata.txt` sidecar with per-bank offsets/checksums and a
machine-readable slot layout (used to reconstruct sub-slots if the `.bin` is
re-opened next to its sidecar).

## Notes & limitations

- Direct device programming is not implemented — export the binary and use your
  programmer.
- Re-dividing the layout to a different size preserves the bytes but not the
  per-bank names/sub-slots; prefer saving a project to keep those.

## License & credits

- Licensed under the MIT License.
- Author: **ArleyJR**.
- Built with eframe/egui (dual-licensed MIT or Apache-2.0); file dialogs via
  `rfd`; project files via `zip` and `serde_json`.
- Application icon: Puzzle icons created by Freepik — Flaticon
  (<https://www.flaticon.com/free-icons/puzzle>).
