# bin2uf2 Web

This tool provides a browser-based pipeline for turning FPGA bitstream `.bin` files into UF2 packages that can be dropped straight onto ChipFoundry ChipDiscover boards during NYDesign workshops. It stitches together a Rust-to-WASM converter core with a SvelteKit front end so participants can stay focused on iterating hardware instead of babysitting CLI invocations.

> **Disclaimer:** This project was entirely vibe coded. Expect rough edges, keep a backup of your bitstreams, and send PRs if you find any issues.

## Features
- Converts raw bitstream binaries into UF2 images with ChipDiscover-friendly metadata.
- Generates randomized flash offsets inside the selected storage slot to avoid clobbering neighbouring payloads.
- Exposes auto-clock and slot selection options
- Runs completely in the browser via WebAssembly—no installs required for workshop attendees.

## Project Layout
- `frontend/` – SvelteKit UI, Tailwind styling, and WASM bindings.
- `converter/` – Rust crate compiled to WebAssembly using `wasm-bindgen`.
- `reference_bitstream_to_uf2.py` – Original Python reference script kept for parity checks and debugging.

## Getting Started
### Prerequisites
- Node.js 20+ (or Bun 1.1+) for the SvelteKit app.
- Rust toolchain with the `wasm32-unknown-unknown` target installed (`rustup target add wasm32-unknown-unknown`).

### Install & Run the UI
```bash
cd frontend
npm install   # or: bun install
npm run dev   # launches Vite dev server
```
The development server hot-reloads changes. Use `npm run build` for a production bundle.

### Rebuilding the WebAssembly Converter
If you touch anything under `converter/src/`, regenerate the WASM bindings:
```bash
# from repo root
wasm-pack build converter \
  --target web \
  --out-dir frontend/src/lib/wasm \
  --out-name converter
```
This refreshes `converter.js`, `converter_bg.wasm`, and the associated TypeScript definitions consumed by the frontend.

## Recommended Workshop Flow
1. Open the hosted app (or run it locally with the commands above).
2. Drop your ChipDiscover `.bin` bitstream, pick a slot (1–4), and optionally set an auto-clock rate.
3. Download the generated UF2 and drag it onto the ChipDiscover mass storage device.
4. Celebrate when the board reboots into your new bitstream—then iterate again.

## Troubleshooting
- **Conversion errors about slot size**: The bitstream is larger than the reserved space for that slot—trim the design or pick another slot.
- **Auto-clock validation failures**: Stick to the documented 10 Hz–60 MHz range; the firmware enforces it.
- **Browser complains about WASM**: Rebuild the converter with `wasm-pack` and ensure the refreshed artifacts are committed.

Enjoy!
