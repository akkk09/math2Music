# MathMusic

MathMusic is a browser-based mathematical DAW built around one rule: the mathematical representation is the source of truth.

This is a Rust workspace compiled to WebAssembly for the browser. The first vertical slice includes:

- mathematical expression parsing with implicit multiplication;
- editable function state;
- interactive graph rendering;
- Web Audio playback;
- a first MathTrack/timeline surface;
- GitHub Actions CI;
- GitHub Pages deployment.

The long-term architecture follows the project roadmap: expression → function graph → transform/compose/evolve → sampling → audio → DAW timeline → mixing/export.

## Development

Install the WASM target, then run:

    rustup target add wasm32-unknown-unknown
    cargo test --workspace
    cargo check -p mathmusic-web --target wasm32-unknown-unknown

The online build compiles the Rust web crate to WASM, generates browser bindings with wasm-bindgen, and deploys the static site to GitHub Pages.

## Architecture

The mathematical core is independent of the browser. The web crate owns DOM, Canvas 2D, and Web Audio integration. The next audio step is a sampled AudioBuffer renderer so expensive mathematical work stays out of the realtime callback.

## License

MIT
