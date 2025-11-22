# Ancestral Vision Tree

A 3D family tree visualization engine with bioluminescent aesthetics. Built with Rust + WebAssembly + WebGL2.

## Features

- **Organic Tree Growth**: Parametric algorithm creates natural-looking branch structures
- **Bioluminescent Visuals**: Ethereal glow effects with custom shaders
- **Biography-Driven Appearance**: Longer biographies = more prominent, vibrant branches
- **Firefly Particles**: Ambient particles attracted to luminous branches
- **Interactive**: Hover over branches to see person information
- **YAML Input**: Define family trees in simple YAML format

## Quick Start

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack`
- A web server (Python's `http.server` works fine)

### Build & Run

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build the WASM module
wasm-pack build --target web

# Serve the project
python3 -m http.server 8080

# Open http://localhost:8080/www/ in your browser
```

## YAML Family Format

```yaml
family:
  name: "Family Name"
  root: "root-person-id"

people:
  - id: "root-person-id"
    name: "Person Name"
    biography: |
      A detailed biography of this person.
      Longer biographies create more prominent branches.
    birth_year: 1900
    death_year: 1980
    children:
      - "child-id-1"
      - "child-id-2"

  - id: "child-id-1"
    name: "First Child"
    biography: "Short bio = subtle branch"
    children: []
```

## Controls

- **Drag**: Orbit camera around the tree
- **Scroll**: Zoom in/out
- **Shift+Drag**: Pan camera
- **Hover**: View person information

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Web Interface                       │
│  (HTML Canvas + Event Handlers + Info Tooltips)     │
├─────────────────────────────────────────────────────┤
│                 JavaScript Bridge                    │
│  (WASM bindings, Input handling, DOM manipulation)  │
├─────────────────────────────────────────────────────┤
│                  WASM Module (Rust)                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  Renderer   │  │   Tree      │  │   Input     │ │
│  │  Pipeline   │  │   Engine    │  │   System    │ │
│  └─────────────┘  └─────────────┘  └─────────────┘ │
├─────────────────────────────────────────────────────┤
│                    WebGL2 API                        │
└─────────────────────────────────────────────────────┘
```

## Visual Style

The visualization draws inspiration from:
- Bioluminescent organisms (deep-sea creatures, fireflies)
- Abstract data art
- Natural tree growth patterns

Key visual elements:
- **Glow intensity** based on biography length
- **Color vibrancy** from life story richness
- **Branch thickness** proportional to prominence
- **Ethereal atmosphere** with vignette and bloom

## Development

```bash
# Run tests
cargo test

# Build for development (faster)
wasm-pack build --target web --dev

# Build for release (optimized)
wasm-pack build --target web --release
```

## License

MIT
