// Bootstrap the WASM module and set up the application

import init, { AncestralVisionTree } from '../pkg/ancestral_vision_tree.js';

// Sample family data
const SAMPLE_FAMILY_YAML = `
family:
  name: "The Oakhart Legacy"
  root: "elder-oakhart"

people:
  - id: "elder-oakhart"
    name: "Elder Oakhart"
    biography: |
      The founding ancestor of the Oakhart lineage, Elder Oakhart was a visionary
      who planted the first trees of what would become the Great Forest. Born in
      a time of turmoil, they dedicated their life to cultivating not just trees,
      but a philosophy of interconnection between all living things.

      Their journals, passed down through generations, speak of dreams where trees
      and people shared consciousness, where roots connected not just soil but souls.
      Elder lived for nearly a century, watching their children and grandchildren
      grow like the saplings they so lovingly tended.

      "To plant a tree," Elder wrote, "is to believe in tomorrow. To raise a child
      is to believe in eternity."
    birth_year: 1820
    death_year: 1915
    children:
      - "willow-oakhart"
      - "cedar-oakhart"

  - id: "willow-oakhart"
    name: "Willow Oakhart"
    biography: |
      Named for the graceful trees that lined the riverside, Willow inherited
      their parent's love for nature but channeled it into art. They became
      renowned for paintings that seemed to capture the very essence of life
      itself - viewers swore they could feel the wind in the leaves.

      Willow's studio was always filled with the scent of pine and turpentine,
      and children from the village would gather at the windows to watch the
      magic unfold on canvas.
    birth_year: 1855
    death_year: 1932
    children:
      - "ash-oakhart"
      - "maple-oakhart"

  - id: "cedar-oakhart"
    name: "Cedar Oakhart"
    biography: |
      The adventurer of the family, Cedar traveled to distant lands, collecting
      seeds and stories in equal measure. Their letters home were treasured,
      read aloud by firelight to eager ears.
    birth_year: 1858
    death_year: 1920
    children:
      - "birch-oakhart"

  - id: "ash-oakhart"
    name: "Ash Oakhart"
    biography: |
      A botanist who dedicated their life to understanding the secret language
      of plants. Ash's research papers, though dense and technical, contained
      passages of such beauty that they were sometimes mistaken for poetry.

      "The forest does not speak in words," Ash once wrote, "but in the subtle
      chemistry of connection. Every root whisper, every fungal thread, carries
      messages older than human memory."

      Their greenhouse laboratory became a pilgrimage site for scientists and
      mystics alike, all seeking to understand the mysteries Ash spent a
      lifetime unraveling.
    birth_year: 1882
    death_year: 1968
    children:
      - "rowan-oakhart"
      - "holly-oakhart"

  - id: "maple-oakhart"
    name: "Maple Oakhart"
    biography: |
      The keeper of family traditions, Maple was the one who remembered
      every birthday, every anniversary, every story worth telling twice.
    birth_year: 1885
    death_year: 1970
    children:
      - "ivy-oakhart"

  - id: "birch-oakhart"
    name: "Birch Oakhart"
    biography: Brief but brilliant, like the white bark catching sunlight.
    birth_year: 1890
    death_year: 1945
    children: []

  - id: "rowan-oakhart"
    name: "Rowan Oakhart"
    biography: |
      Named for the protective tree of myth, Rowan became a healer of both
      body and spirit. Their clinic in the valley was never closed - there
      was always a light burning for those who needed it.

      Rowan's approach combined ancient herbal knowledge passed down through
      the Oakhart line with modern medical science. They were known to say
      that the best medicine was often simply being present with someone in
      their suffering.
    birth_year: 1915
    death_year: 1998
    children:
      - "hazel-oakhart"
      - "laurel-oakhart"

  - id: "holly-oakhart"
    name: "Holly Oakhart"
    biography: Thorny exterior, heart of gold.
    birth_year: 1918
    death_year: 2005
    children: []

  - id: "ivy-oakhart"
    name: "Ivy Oakhart"
    biography: |
      Like their namesake, Ivy was known for persistence and the ability to
      find light in the darkest places. A teacher who inspired three
      generations of students.
    birth_year: 1920
    death_year: 2010
    children:
      - "fern-oakhart"

  - id: "hazel-oakhart"
    name: "Hazel Oakhart"
    biography: |
      The family historian, Hazel spent decades compiling the Oakhart archives.
      Every photograph, every letter, every pressed flower found its place in
      their meticulous collection. It is thanks to Hazel that this family tree
      can be visualized at all.

      "We are not just individuals," Hazel would say, adjusting their spectacles
      as they pored over another faded document. "We are a story being written
      across time, each generation adding their chapter."
    birth_year: 1945
    children:
      - "sage-oakhart"
      - "moss-oakhart"

  - id: "laurel-oakhart"
    name: "Laurel Oakhart"
    biography: Crowned with achievement.
    birth_year: 1948
    children: []

  - id: "fern-oakhart"
    name: "Fern Oakhart"
    biography: |
      A gardener who believed that every plant had a personality. Their garden
      was a conversation, not a collection.
    birth_year: 1950
    children: []

  - id: "sage-oakhart"
    name: "Sage Oakhart"
    biography: |
      True to their name, Sage became a philosopher and writer, pondering the
      deep questions of existence while sitting beneath the very trees their
      ancestors planted. Their books on ecological consciousness have inspired
      a new generation of environmental thinkers.
    birth_year: 1975
    children: []

  - id: "moss-oakhart"
    name: "Moss Oakhart"
    biography: Quiet, patient, everywhere at once if you know where to look.
    birth_year: 1978
    children: []
`;

class Application {
    constructor() {
        this.engine = null;
        this.canvas = null;
        this.isInitialized = false;
        this.isDragging = false;
        this.isShiftDown = false;
        this.lastMouseX = 0;
        this.lastMouseY = 0;
    }

    async init() {
        try {
            // Initialize WASM
            await init();

            // Set up canvas
            this.canvas = document.getElementById('canvas');
            this.resizeCanvas();

            // Create engine
            this.engine = new AncestralVisionTree(this.canvas);

            // Load sample family
            this.engine.load_family(SAMPLE_FAMILY_YAML);

            // Set up event listeners
            this.setupEventListeners();

            // Hide loading indicator
            document.getElementById('loading').classList.add('hidden');

            this.isInitialized = true;

            // Start render loop
            this.renderLoop();

        } catch (error) {
            console.error('Failed to initialize:', error);
            document.getElementById('loading').innerHTML = `
                <p style="color: #ff6b6b;">Failed to initialize: ${error.message}</p>
                <p style="color: #888; font-size: 0.9em; margin-top: 10px;">
                    Make sure WebGL2 is supported in your browser.
                </p>
            `;
        }
    }

    resizeCanvas() {
        const dpr = window.devicePixelRatio || 1;
        const rect = this.canvas.getBoundingClientRect();
        this.canvas.width = rect.width * dpr;
        this.canvas.height = rect.height * dpr;

        if (this.engine) {
            this.engine.resize(this.canvas.width, this.canvas.height);
        }
    }

    setupEventListeners() {
        // Window resize
        window.addEventListener('resize', () => this.resizeCanvas());

        // Mouse events for camera control
        this.canvas.addEventListener('mousedown', (e) => {
            this.isDragging = true;
            this.lastMouseX = e.clientX;
            this.lastMouseY = e.clientY;
        });

        window.addEventListener('mouseup', () => {
            this.isDragging = false;
        });

        window.addEventListener('mousemove', (e) => {
            if (this.isDragging) {
                const dx = e.clientX - this.lastMouseX;
                const dy = e.clientY - this.lastMouseY;

                if (this.isShiftDown) {
                    this.engine.pan(dx, dy);
                } else {
                    this.engine.orbit(dx, dy);
                }

                this.lastMouseX = e.clientX;
                this.lastMouseY = e.clientY;
            } else {
                // Hover detection
                const rect = this.canvas.getBoundingClientRect();
                const dpr = window.devicePixelRatio || 1;
                const x = (e.clientX - rect.left) * dpr;
                const y = (e.clientY - rect.top) * dpr;

                const hoveredId = this.engine.on_mouse_move(x, y);
                this.updateInfoPanel(hoveredId);
            }
        });

        // Scroll for zoom
        this.canvas.addEventListener('wheel', (e) => {
            e.preventDefault();
            this.engine.zoom(e.deltaY * 0.01);
        }, { passive: false });

        // Keyboard
        window.addEventListener('keydown', (e) => {
            if (e.key === 'Shift') this.isShiftDown = true;
        });

        window.addEventListener('keyup', (e) => {
            if (e.key === 'Shift') this.isShiftDown = false;
        });

        // Custom family button
        document.getElementById('custom-family-btn').addEventListener('click', () => {
            document.getElementById('yaml-input-container').classList.remove('hidden');
        });

        document.getElementById('close-yaml-btn').addEventListener('click', () => {
            document.getElementById('yaml-input-container').classList.add('hidden');
        });

        document.getElementById('load-yaml-btn').addEventListener('click', () => {
            const yaml = document.getElementById('yaml-input').value;
            if (yaml.trim()) {
                try {
                    this.engine.load_family(yaml);
                    document.getElementById('yaml-input-container').classList.add('hidden');
                } catch (error) {
                    alert('Failed to load family: ' + error.message);
                }
            }
        });
    }

    updateInfoPanel(personId) {
        const panel = document.getElementById('info-panel');

        if (!personId) {
            panel.classList.add('hidden');
            return;
        }

        const infoJson = this.engine.get_person_info(personId);
        if (!infoJson) {
            panel.classList.add('hidden');
            return;
        }

        try {
            const info = JSON.parse(infoJson);

            document.getElementById('person-name').textContent = info.name;
            document.getElementById('person-lifespan').textContent = info.lifespan;
            document.getElementById('person-bio').textContent = info.biography;

            panel.classList.remove('hidden');
        } catch (e) {
            console.error('Failed to parse person info:', e);
        }
    }

    renderLoop() {
        if (!this.isInitialized) return;

        const dt = 1 / 60; // Fixed timestep
        this.engine.render(dt);

        requestAnimationFrame(() => this.renderLoop());
    }
}

// Start the application
const app = new Application();
app.init();
