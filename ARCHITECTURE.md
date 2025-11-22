# Ancestral Vision Tree - Architecture & Plan

## Vision
A 3D family tree visualization engine that renders organic, bioluminescent trees where each branch represents a family member. The tree grows from a sapling, with visual prominence (glow, color, thickness) determined by biography length.

## Technical Stack

### Core Engine (Rust + WASM)
- **Rust** for performance-critical algorithms and memory safety
- **wasm-bindgen** for JavaScript interoperability
- **WebGL2** for 3D rendering (direct bindings, no frameworks)
- **serde + serde_yaml** for YAML parsing

### Visual Approach
- Custom shaders for bioluminescence effect
- Bloom post-processing for ethereal glow
- Particle system for fireflies
- Smooth mesh interpolation for organic branch flow

## Architecture Layers

```
┌─────────────────────────────────────────────────────┐
│                  Web Interface                       │
│  (HTML Canvas + Event Handlers + Info Tooltips)     │
├─────────────────────────────────────────────────────┤
│                 JavaScript Bridge                    │
│  (WASM bindings, Input handling, DOM manipulation)  │
├─────────────────────────────────────────────────────┤
│                  WASM Module                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  Renderer   │  │   Tree      │  │   Input     │ │
│  │  Pipeline   │  │   Engine    │  │   System    │ │
│  └─────────────┘  └─────────────┘  └─────────────┘ │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │   Mesh      │  │  Particle   │  │   Shader    │ │
│  │ Generator   │  │   System    │  │  Manager    │ │
│  └─────────────┘  └─────────────┘  └─────────────┘ │
├─────────────────────────────────────────────────────┤
│                    WebGL2 API                        │
└─────────────────────────────────────────────────────┘
```

## Data Flow

```
YAML Family Data
       │
       ▼
┌──────────────────┐
│   Parse YAML     │
│  Build FamilyTree│
└──────────────────┘
       │
       ▼
┌──────────────────┐
│  Tree Growth     │
│   Algorithm      │ ◄── Biography length affects growth params
└──────────────────┘
       │
       ▼
┌──────────────────┐
│  Mesh Generator  │
│  (Organic curves)│
└──────────────────┘
       │
       ▼
┌──────────────────┐
│  WebGL Renderer  │
│  + Shaders       │
└──────────────────┘
       │
       ▼
    Canvas Output
```

## Module Specifications

### 1. Data Structures (`src/data/`)

```rust
struct Person {
    id: String,
    name: String,
    biography: String,
    birth_year: Option<i32>,
    death_year: Option<i32>,
    children: Vec<String>,  // IDs of children
}

struct FamilyTree {
    people: HashMap<String, Person>,
    root: String,  // ID of root ancestor
}

struct VisualParams {
    glow_intensity: f32,     // 0.0 - 1.0, from bio length
    color_vibrancy: f32,     // Saturation boost
    branch_thickness: f32,   // Relative thickness
    luminance: f32,          // Bioluminescence strength
}
```

### 2. Tree Growth Algorithm (`src/growth/`)

**Parametric Binary Tree Growth:**
- Each person = one branch segment + potential split point
- Children create binary splits (left/right for two children)
- Single child continues straight with slight curve
- Multiple children (>2) create multi-way splits with spacing

**Growth Parameters (affected by biography):**
- `branch_length`: Base length modified by bio length
- `branch_thickness`: Thicker = more prominent
- `curvature`: Organic randomness
- `glow_radius`: Bioluminescent aura size

### 3. Mesh Generation (`src/mesh/`)

**Organic Branch Segments:**
- Cylindrical base with radial variation
- Catmull-Rom splines for smooth curves
- Smooth interpolation at branch joints
- Bark-like displacement for realism

**Key Functions:**
- `generate_branch_segment(start, end, radius, params) -> Mesh`
- `create_branch_joint(parent, children, radii) -> Mesh`
- `merge_meshes(meshes) -> Mesh`

### 4. Shader System (`src/shaders/`)

**Vertex Shader:**
- Standard MVP transformation
- Pass UV, normal, and custom attributes (glow, luminance)

**Fragment Shader (Bioluminescence):**
- Base color from person-specific hue
- Inner glow based on luminance parameter
- Fresnel effect for edge glow
- Subsurface scattering approximation

**Post-Processing:**
- Bloom pass for ethereal glow
- Color grading for atmosphere

### 5. Particle System (`src/particles/`)

**Firefly System:**
- Point sprites with animated glow
- Random wandering paths (Perlin noise)
- Attracted slightly to high-luminance branches
- Fade in/out lifecycle

### 6. Interaction System (`src/interaction/`)

**Hover Detection:**
- Ray casting from mouse position
- Sphere/cylinder intersection tests per branch
- Highlight hovered segment
- Trigger info panel display

## Visual Testing Strategy

**Critical for an AI to evaluate aesthetics:**

### Screenshot Testing Harness
1. **Puppeteer-based capture**: Automated browser screenshots
2. **Reference image comparison**: Detect visual regressions
3. **HTML Report Generation**: Side-by-side comparisons I can view

### Test Scenarios
- Single person (sapling)
- Two generations (one split)
- Multi-generation tree (complex branching)
- Varied biography lengths (test visual differentiation)
- Hover state visualization

### Visual Checkpoints
After each major feature, generate screenshots:
```
/test-output/
  ├── 01-basic-branch.png
  ├── 02-branch-joint.png
  ├── 03-simple-tree.png
  ├── 04-bioluminescence.png
  ├── 05-full-tree-with-particles.png
  └── visual-report.html
```

## TDD Approach

### Unit Tests (Rust)
1. **Data parsing**: YAML to FamilyTree conversion
2. **Tree algorithms**: Growth calculations, parameter derivation
3. **Mesh generation**: Vertex/index buffer correctness
4. **Math utilities**: Vector operations, spline evaluation

### Integration Tests
1. **WASM binding**: JS <-> Rust communication
2. **Render pipeline**: Shader compilation, draw calls
3. **Full flow**: YAML input → rendered output

### Visual Tests
1. **Snapshot tests**: Compare rendered output to references
2. **Parameter sweep**: Generate gallery of variations

## File Structure

```
ancestral-vision-tree/
├── Cargo.toml
├── src/
│   ├── lib.rs              # WASM entry point
│   ├── data/
│   │   ├── mod.rs
│   │   ├── person.rs
│   │   └── family_tree.rs
│   ├── growth/
│   │   ├── mod.rs
│   │   └── algorithm.rs
│   ├── mesh/
│   │   ├── mod.rs
│   │   ├── branch.rs
│   │   ├── joint.rs
│   │   └── spline.rs
│   ├── render/
│   │   ├── mod.rs
│   │   ├── webgl.rs
│   │   ├── shaders.rs
│   │   └── pipeline.rs
│   ├── particles/
│   │   ├── mod.rs
│   │   └── fireflies.rs
│   ├── interaction/
│   │   ├── mod.rs
│   │   └── picking.rs
│   └── math/
│       ├── mod.rs
│       ├── vec3.rs
│       └── matrix.rs
├── www/
│   ├── index.html
│   ├── style.css
│   └── bootstrap.js
├── shaders/
│   ├── tree.vert
│   ├── tree.frag
│   ├── particle.vert
│   ├── particle.frag
│   └── bloom.frag
├── test-data/
│   └── sample-family.yaml
├── tests/
│   ├── data_tests.rs
│   ├── mesh_tests.rs
│   └── visual/
│       └── snapshot_test.js
└── test-output/
    └── (generated screenshots)
```

## YAML Input Format

```yaml
family:
  name: "The Ancestors"
  root: "great-grandparent"

people:
  - id: "great-grandparent"
    name: "Elder Oakhart"
    biography: |
      A long and storied life spanning nearly a century...
      (Longer text = more prominent branch)
    birth_year: 1890
    death_year: 1985
    children:
      - "grandparent-1"
      - "grandparent-2"

  - id: "grandparent-1"
    name: "Willow Oakhart"
    biography: "Brief life summary."
    children:
      - "parent-1"
```

## Implementation Order

### Phase 1: Foundation
1. Project setup (Cargo, wasm-pack, web scaffold)
2. Math utilities with tests
3. Data structures with YAML parsing

### Phase 2: Core Algorithm
4. Tree growth algorithm
5. Mesh generation (branches)
6. Joint smoothing

### Phase 3: Rendering
7. WebGL context setup
8. Basic shader pipeline
9. First visual test (simple branch)

### Phase 4: Aesthetics
10. Bioluminescent shaders
11. Parameter-driven visuals
12. Firefly particles
13. Post-processing bloom

### Phase 5: Interaction
14. Mouse picking
15. Info panel display
16. Hover highlighting

### Phase 6: Polish
17. Visual refinement
18. Performance optimization
19. Documentation

## Success Criteria

1. **Functional**: YAML in → 3D tree visualization out
2. **Organic**: Smooth, natural-looking branch flow
3. **Parametric**: Biography length visibly affects appearance
4. **Interactive**: Hover shows person info
5. **Aesthetic**: Ethereal, bioluminescent atmosphere
6. **Tested**: Comprehensive unit + visual tests
