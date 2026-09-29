MathMusic is split into a pure Rust mathematical core and a browser adapter.

The core parses expressions, evaluates them, and samples them. The web crate owns DOM, Canvas 2D, and Web Audio integration.

The mathematical representation remains the source of truth. The next audio step is a sampled AudioBuffer renderer, which will keep expensive mathematical work out of the realtime callback.

The long-term architecture follows the project roadmap: expression -> function graph -> transform/compose/evolve -> sampling -> audio -> DAW timeline -> mixing/export.
