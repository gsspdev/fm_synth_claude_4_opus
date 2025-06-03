
# FM Synthesizer - Setup and Build Guide

## Project Structure

```
fm_synth/
├── Cargo.toml
├── src/
│   ├── main.rs      # The Rust code (fm_synth_rust artifact)
│   └── lib.rs       # Same code as main.rs for WASM build
├── index.html       # Web interface (fm_synth_web artifact)
├── build.sh         # Build script for WASM
└── README.md        # This file
```

## Prerequisites

1. **Rust** - Install from https://rustup.rs/
2. **wasm-pack** - Install with: `cargo install wasm-pack`
3. **Basic HTTP server** - Python or Node.js for local testing

## Setting Up the Project

### 1. Create the project directory

```bash
mkdir fm_synth
cd fm_synth
cargo init
```

### 2. Update Cargo.toml

```toml
[package]
name = "fm_synth"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "fm_synth"
path = "src/main.rs"

[dependencies]
anyhow = "1.0"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
cpal = "0.15"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"

[dependencies.web-sys]
version = "0.3"
features = [
  "AudioContext",
  "OscillatorNode", 
  "GainNode",
  "AudioDestinationNode",
  "AudioParam",
  "Window",
]
```

### 3. Create the source files

- Copy the Rust code to both `src/main.rs` and `src/lib.rs`
- Copy the HTML code to `index.html` in the project root

### 4. Create build script (build.sh)

```bash
#!/bin/bash

echo "Building WebAssembly module..."
wasm-pack build --target web --out-dir pkg

echo "Build complete! Files in ./pkg/"
echo "To run the web version:"
echo "  1. Start a local server: python3 -m http.server 8000"
echo "  2. Open http://localhost:8000 in your browser"
```

Make it executable:
```bash
chmod +x build.sh
```

## Building and Running

### Native Desktop Version

```bash
# Build
cargo build --release

# Run
cargo run --release
```

### WebAssembly Version

```bash
# Build WASM
./build.sh

# Serve locally (Python)
python3 -m http.server 8000

# Or with Node.js
npx http-server -p 8000

# Open in browser
# http://localhost:8000
```

## Using the CLI

### Desktop Commands

- `list presets` - Show all 12 available sound presets
- `list melodies` - Show all 10 available melodies  
- `play <preset> <melody>` - Play a melody with a specific preset
  - Example: `play bell twinkle`
  - Example: `play 1 3` (using numbers)
- `demo` - Play all presets with a scale
- `help` - Show command list
- `quit` - Exit the program

### Web Commands

Same as desktop, plus:
- `clear` - Clear the terminal display

## Available Presets

1. **Bell** - Bright, metallic bell sound
2. **Bass** - Deep bass synth
3. **Electric Piano** - Classic FM electric piano
4. **Brass** - Synthetic brass sound
5. **Organ** - Church/Hammond organ style
6. **Synth Lead** - Sharp lead synthesizer
7. **Marimba** - Wooden xylophone sound
8. **Strings** - Soft string pad
9. **Flute** - Simple flute tone
10. **Metallic** - Harsh metallic sound
11. **Glockenspiel** - Light bell sound
12. **Wood Block** - Percussive wood sound

## Available Melodies

1. **Twinkle Twinkle** - Classic children's song
2. **Happy Birthday** - Birthday celebration tune
3. **Ode to Joy** - Beethoven's famous melody
4. **Mary Had a Little Lamb** - Simple nursery rhyme
5. **Chromatic Scale** - All semitones ascending
6. **Major Arpeggio** - C major broken chord
7. **Minor Pentatonic** - Blues/rock scale
8. **Jazz Lick** - Short jazz phrase
9. **Bach Invention** - Classical melody fragment
10. **Synth Demo** - Arpeggio demonstration

## Technical Details

### FM Synthesis Parameters

- **Carrier Frequency**: Main pitch of the sound
- **Modulator Frequency**: Affects timbre/harmonics
- **Modulation Index**: Brightness/complexity (0-12)
- **Amplitude**: Volume level (0.0-1.0)

### ADSR Envelope

- **Attack**: 10ms (fast attack)
- **Decay**: 100ms 
- **Sustain**: 70% level
- **Release**: 500ms

## Troubleshooting

### Desktop Issues

- **No audio device**: Ensure your system has audio output enabled
- **Compilation errors**: Update cpal with `cargo update`
- **Performance**: Reduce buffer size in audio config if needed

### Web Issues

- **No sound**: Click anywhere on the page first (browser security)
- **Module not loading**: Check browser console for errors
- **CORS errors**: Use a proper HTTP server, not file:// protocol

## Extending the Synthesizer

### Adding New Presets

Edit the `get_presets()` function:

```rust
("Your Preset", FMParams {
    carrier_freq: 440.0,
    modulator_freq: 880.0,  // Try ratios like 2:1, 3:2, etc
    modulation_index: 3.0,  // 0.5-12, higher = brighter
    amplitude: 0.3,
})
```

### Adding New Melodies

Edit the `get_melodies()` function:

```rust
("Your Melody", vec![
    ("C4", 500),  // Note, Duration in ms
    ("E4", 500),
    ("G4", 1000),
])
```

## License

This project is provided as an educational example for FM synthesis in Rust.
