# Circle of Fifths

An interactive music theory tool for piano learners and music producers. Built with Rust/WebAssembly using the [Yew](https://yew.rs) framework, bundled by [Trunk](https://trunkrs.dev). Fully static — no backend, no ads.

## Features

- **Interactive circle of fifths** — click any key to explore relationships; adjacent and opposite keys highlighted
- **Key info panel** — key signature, scale notes, and all 7 diatonic chords with Roman numeral labels
- **Chord progression recommender** — curated progressions per key with mood/genre tags, borrowed chord labels, and favorites
- **Piano keyboard** — scrollable 3-octave keyboard with scale and chord highlighting color-coded by role (root/third/fifth)
- **Quiz mode** — flashcard-style questions on key signatures, relative minors, and scale notes
- **Audio playback** — hear scales, chords, and progressions via the Web Audio API
- **Dark/light theme** — persisted across sessions

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- `wasm32-unknown-unknown` target:
  ```sh
  rustup target add wasm32-unknown-unknown
  ```
- [Trunk](https://trunkrs.dev):
  ```sh
  cargo install trunk
  ```

## Running locally

```sh
trunk serve
```

Then open [http://localhost:8080](http://localhost:8080).

## Building for production

```sh
trunk build --release
```

Output is in the `dist/` directory — deploy it to any static host (GitHub Pages, Netlify, Cloudflare Pages, etc.).

## Running tests

Pure Rust tests (music theory, state reducer, storage, quiz logic):

```sh
cargo test
```

WASM tests (requires `wasm-pack` and a browser driver):

```sh
wasm-pack test --headless --firefox
```

## Project structure

```
src/
  music_theory/   # Pure functions: scale_notes, diatonic_chords, key_signature, etc.
  state/          # AppState, AppAction, app_reducer
  data/           # Static chord progression data for all 12 keys
  audio/          # AudioEngine wrapping Web Audio API
  storage/        # localStorage persistence (theme, mute, favorites, best scores)
  components/
    app.rs         # Root component — wires everything together
    circle_view.rs # SVG circle of fifths diagram
    key_info_panel.rs
    progression_panel.rs
    piano_panel.rs
    nav_bar.rs
    quiz_panel.rs
```

## Tech stack

| Layer | Technology |
|-------|-----------|
| Language | Rust → WASM via `wasm-bindgen` |
| UI framework | Yew 0.21 |
| Bundler | Trunk |
| Audio | Web Audio API via `web-sys` |
| Persistence | `localStorage` via `web-sys` |
| Testing | `proptest` (property-based) + `wasm-bindgen-test` |
