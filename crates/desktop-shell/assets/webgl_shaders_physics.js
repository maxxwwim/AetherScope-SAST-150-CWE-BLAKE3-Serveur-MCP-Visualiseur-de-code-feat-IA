// =========================================================================
// AETHERSCOPE V2 — WEBGL 2 SHADERS & PHYSICS ENGINE (webgl_shaders_physics.js)
// 120 FPS Instanced SDF Celestial Renderer & Cybernetic Line Canvas
// =========================================================================

// --- WebGL 2 GPU Instanced Engine ---
const canvas = document.getElementById('aether-canvas');
const gl = canvas.getContext('webgl2', { alpha: false, antialias: true, powerPreference: 'high-performance', preserveDrawingBuffer: true });

function resize() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  gl.viewport(0, 0, canvas.width, canvas.height);
}
window.addEventListener('resize', resize);
resize();

// 2D Instant Renderer Shaders with Crypto Space Star / Lunar Moon / Pulsar SDF Halos
const vsSource = `#version 300 es
  in vec2 a_quad_pos;
  in vec2 a_node_pos;
  in float a_radius;
  in vec4 a_color;
  in float a_is_file;
  in float a_birth_time;

  uniform vec2 u_resolution;
  uniform vec2 u_camera_pos;
  uniform float u_zoom;
  uniform float u_time;

  out vec2 v_uv;
  out vec4 v_color;
  out float v_is_file;
  out float v_scale;
  out float v_spawn_progress;

  void main() {
    v_uv = a_quad_pos;
    v_color = a_color;
    v_is_file = a_is_file;

    float dt = u_time - a_birth_time;
    if (dt <= 0.0) {
      gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
      v_scale = 0.0;
      v_spawn_progress = 0.0;
      return;
    }

    // 300ms (0.3s) organic breathing birth curve
    float spawn_dur = 300.0;
    float progress = clamp(dt / spawn_dur, 0.0, 1.0);
    v_spawn_progress = progress;

    // Organic breath easing: soft cubic expansion with gentle elastic overshoot
    float ease_out = 1.0 - pow(1.0 - progress, 3.0);
    float breath_overshoot = sin(progress * 3.14159) * 0.20 * (1.0 - progress);
    float scale = clamp(ease_out + breath_overshoot, 0.0, 1.30);
    v_scale = scale;

    // Adaptive zoom factor so nodes remain prominently visible and radiant at ALL zoom levels
    float zoom_factor = clamp(1.0 / sqrt(clamp(u_zoom, 0.08, 3.5)), 0.85, 2.5);
    float actual_r = a_radius * scale * 2.8 * zoom_factor;

    // Organic aquatic buoyant drift (continuous 120 FPS living fluid motion)
    float buoy_x = sin(u_time * 0.0016 + a_node_pos.x * 0.012 + a_birth_time * 0.001) * 3.5;
    float buoy_y = cos(u_time * 0.0012 + a_node_pos.y * 0.012 + a_birth_time * 0.001) * 4.2;
    vec2 dynamic_pos = a_node_pos + vec2(buoy_x, buoy_y);

    vec2 world_pos = dynamic_pos + a_quad_pos * actual_r;
    vec2 screen_pos = (world_pos - u_camera_pos) * u_zoom + u_resolution * 0.5;
    vec2 clip_pos = (screen_pos / u_resolution) * 2.0 - 1.0;
    clip_pos.y = -clip_pos.y;

    gl_Position = vec4(clip_pos, 0.0, 1.0);
  }
`;

const fsSource = `#version 300 es
  precision highp float;
  in vec2 v_uv;
  in vec4 v_color;
  in float v_is_file;
  in float v_scale;
  in float v_spawn_progress;

  out vec4 fragColor;

  void main() {
    if (v_spawn_progress <= 0.001) discard;
    float r = length(v_uv); // 0.0 at center, 1.0 at perimeter
    if (r > 1.0) discard;

    vec3 col = v_color.rgb;
    vec3 final_rgb = col;
    float alpha = 1.0;

    if (v_is_file < 0.5) {
      // --- MINI POINT POUR FONCTION : Étincelle stellaire ultra-lumineuse, noyau blanc pur ---
      float core = 1.0 - smoothstep(0.0, 0.32, r);
      float halo = 1.0 - smoothstep(0.18, 0.95, r);
      final_rgb = mix(col * 1.35, vec3(1.0, 1.0, 1.0), core * 0.90) + col * halo * 0.55;
      alpha = clamp(core * 0.98 + halo * 0.85, 0.0, 1.0);
    } else if (v_is_file < 1.5) {
      // --- PLANÈTE FICHIER : Orbe céleste avec anneau de Saturne vibrant ---
      float body = 1.0 - smoothstep(0.44, 0.56, r);
      float rim = 1.0 - smoothstep(0.0, 0.08, abs(r - 0.52));
      float core = 1.0 - smoothstep(0.0, 0.22, r);
      float outer_glow = (1.0 - smoothstep(0.50, 0.96, r)) * 0.45;
      float ring = (1.0 - smoothstep(0.0, 0.05, abs(r - 0.78))) * 0.85;

      final_rgb = mix(col * 0.95, vec3(1.0), core * 0.70);
      final_rgb += vec3(1.0) * rim * 0.95 + col * rim * 0.70;
      final_rgb += col * outer_glow * 0.85;
      final_rgb += mix(col, vec3(0.9, 0.96, 1.0), 0.65) * ring;
      alpha = clamp(body * 0.95 + rim * 0.90 + outer_glow * 0.60 + ring, 0.0, 1.0);
    } else if (v_is_file < 2.5) {
      // --- CLASSE : Cristal céleste avec double halo concentrique ---
      float body = 1.0 - smoothstep(0.48, 0.58, r);
      float rim = 1.0 - smoothstep(0.0, 0.07, abs(r - 0.52));
      float inner_halo = 1.0 - smoothstep(0.0, 0.06, abs(r - 0.32));
      float core = 1.0 - smoothstep(0.0, 0.18, r);
      float outer_glow = (1.0 - smoothstep(0.52, 0.95, r)) * 0.50;

      final_rgb = mix(col, vec3(1.0), core * 0.80);
      final_rgb += vec3(1.0) * rim * 0.95 + col * inner_halo * 0.80;
      final_rgb += col * outer_glow * 0.80;
      alpha = clamp(body * 0.95 + rim + inner_halo * 0.70 + outer_glow * 0.50, 0.0, 1.0);
    } else {
      // --- FORM / ENDPOINT : Pulsar / Balise énergétique hyper-contrastée ---
      float body = 1.0 - smoothstep(0.46, 0.56, r);
      float rim = 1.0 - smoothstep(0.0, 0.09, abs(r - 0.50));
      float core = 1.0 - smoothstep(0.0, 0.25, r);
      float outer_glow = (1.0 - smoothstep(0.50, 0.98, r)) * 0.60;

      final_rgb = mix(col, vec3(1.0, 0.95, 0.80), core * 0.75);
      final_rgb += vec3(1.0) * rim * 0.95 + col * outer_glow * 0.90;
      alpha = clamp(body * 0.95 + rim * 0.90 + outer_glow * 0.70, 0.0, 1.0);
    }

    if (v_color.a < 0.2) {
      final_rgb *= (v_color.a * 2.5 + 0.08);
    }
    float spawn_alpha = clamp(v_spawn_progress * 1.6, 0.0, 1.0);
    fragColor = vec4(final_rgb, alpha * v_color.a * spawn_alpha);
  }
`;

function compileShader(type, src) {
  const s = gl.createShader(type);
  gl.shaderSource(s, src);
  gl.compileShader(s);
  return s;
}
const program = gl.createProgram();
gl.attachShader(program, compileShader(gl.VERTEX_SHADER, vsSource));
gl.attachShader(program, compileShader(gl.FRAGMENT_SHADER, fsSource));
gl.linkProgram(program);

// Quad geometry
const quadVbo = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, quadVbo);
gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
  -1, -1,  1, -1, -1,  1,  1,  1
]), gl.STATIC_DRAW);

// Dynamic Instanced VBO
const instanceVbo = gl.createBuffer();
const vao = gl.createVertexArray();
gl.bindVertexArray(vao);

gl.bindBuffer(gl.ARRAY_BUFFER, quadVbo);
const aQuadPos = gl.getAttribLocation(program, 'a_quad_pos');
gl.enableVertexAttribArray(aQuadPos);
gl.vertexAttribPointer(aQuadPos, 2, gl.FLOAT, false, 0, 0);

gl.bindBuffer(gl.ARRAY_BUFFER, instanceVbo);
const stride = 9 * 4;
const aNodePos = gl.getAttribLocation(program, 'a_node_pos');
gl.enableVertexAttribArray(aNodePos);
gl.vertexAttribPointer(aNodePos, 2, gl.FLOAT, false, stride, 0);
gl.vertexAttribDivisor(aNodePos, 1);

const aRadius = gl.getAttribLocation(program, 'a_radius');
gl.enableVertexAttribArray(aRadius);
gl.vertexAttribPointer(aRadius, 1, gl.FLOAT, false, stride, 2 * 4);
gl.vertexAttribDivisor(aRadius, 1);

const aColor = gl.getAttribLocation(program, 'a_color');
gl.enableVertexAttribArray(aColor);
gl.vertexAttribPointer(aColor, 4, gl.FLOAT, false, stride, 3 * 4);
gl.vertexAttribDivisor(aColor, 1);

const aIsFile = gl.getAttribLocation(program, 'a_is_file');
gl.enableVertexAttribArray(aIsFile);
gl.vertexAttribPointer(aIsFile, 1, gl.FLOAT, false, stride, 7 * 4);
gl.vertexAttribDivisor(aIsFile, 1);

const aBirthTime = gl.getAttribLocation(program, 'a_birth_time');
gl.enableVertexAttribArray(aBirthTime);
gl.vertexAttribPointer(aBirthTime, 1, gl.FLOAT, false, stride, 8 * 4);
gl.vertexAttribDivisor(aBirthTime, 1);

gl.bindVertexArray(null);

// Uniform locations
const uRes = gl.getUniformLocation(program, 'u_resolution');
const uCam = gl.getUniformLocation(program, 'u_camera_pos');
const uZoom = gl.getUniformLocation(program, 'u_zoom');
const uTime = gl.getUniformLocation(program, 'u_time');

// Canvas 2D overlay for lines, labels & Parallax Starfield
const lineCanvas = document.createElement('canvas');
lineCanvas.id = 'line-canvas';
lineCanvas.style.cssText = 'position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none !important;z-index:2;';
document.body.appendChild(lineCanvas);
const ctx = lineCanvas.getContext('2d', { alpha: true });

function resizeLines() {
  lineCanvas.width = window.innerWidth;
  lineCanvas.height = window.innerHeight;
}
window.addEventListener('resize', resizeLines);
resizeLines();

// 650 Rich Celestial Parallax Stars & Navigational Beacons
const starfield = [];
const starPalettes = [
  'rgba(255, 255, 255, ALPHA)',   // 55% Diamond White / Silver
  'rgba(255, 255, 255, ALPHA)',
  'rgba(56, 189, 248, ALPHA)',    // 20% Electric Ice Blue
  'rgba(192, 132, 252, ALPHA)',   // 15% Celestial Lavender
  'rgba(253, 224, 71, ALPHA)'     // 10% Warm Topaz Gold
];

for (let i = 0; i < 650; i++) {
  const isMajor = i < 40; // Top 40 major stellar beacons with diffraction spike
  const isMedium = !isMajor && i < 180;
  const colIdx = Math.floor(Math.random() * starPalettes.length);
  
  starfield.push({
    x: (Math.random() - 0.5) * 6000,
    y: (Math.random() - 0.5) * 6000,
    z: 0.08 + Math.random() * 0.55,
    r: isMajor ? (2.2 + Math.random() * 1.6) : (isMedium ? (1.2 + Math.random() * 0.9) : (0.5 + Math.random() * 0.6)),
    baseAlpha: isMajor ? (0.85 + Math.random() * 0.15) : (isMedium ? (0.65 + Math.random() * 0.25) : (0.35 + Math.random() * 0.35)),
    twinkleSpeed: 0.001 + Math.random() * 0.003,
    twinklePhase: Math.random() * Math.PI * 2,
    color: starPalettes[colIdx],
    hasCorona: isMajor
  });
}

// Dynamic 3D Folder Clusters with Cohesive Cellular Membrane Lobes & Real-Time Function Following
function updateFolderClusters() {
  const nodes = state.data?.nodes || [];
  const folderStats = new Map();
  if (!state.folderAnchors) state.folderAnchors = new Map();

  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    if (!state.activeLayers.has(n.layer)) continue;
    const f = n.folder || 'root';
    if (state.hiddenModules && state.hiddenModules.has(f)) continue;
    let st = folderStats.get(f);
    if (!st) {
      st = { sumX: 0, sumY: 0, count: 0, minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity, nodes: [] };
      folderStats.set(f, st);
    }
    st.sumX += n.x;
    st.sumY += n.y;
    st.count++;
    st.nodes.push(n);
    if (n.x < st.minX) st.minX = n.x;
    if (n.x > st.maxX) st.maxX = n.x;
    if (n.y < st.minY) st.minY = n.y;
    if (n.y > st.maxY) st.maxY = n.y;
  }
  const zoom = state.camera ? state.camera.zoom : 1;
  const comfortPadding = Math.max(48, Math.min(88, 54 / Math.sqrt(zoom || 1)));
  const SECTORS = 12;
  const TWO_PI = Math.PI * 2;

  folderStats.forEach((st, folder) => {
    // Persistent stable anchor center: moving a bubble inside never shifts the cloud or badge!
    let anchor = state.folderAnchors ? state.folderAnchors.get(folder) : null;
    if (!anchor || !Number.isFinite(anchor.x) || !Number.isFinite(anchor.y)) {
      anchor = { x: st.sumX / st.count, y: st.sumY / st.count };
      if (state.folderAnchors) state.folderAnchors.set(folder, anchor);
    }
    const cx = anchor.x;
    const cy = anchor.y;
    st.cx = cx;
    st.cy = cy;
    st.sumX = cx * st.count;
    st.sumY = cy * st.count;

    let maxDist = 0;
    let sumDist = 0;
    const minBaseR = Math.max(110, Math.sqrt(st.count) * 58);
    const rawSectors = new Float32Array(SECTORS);

    for (let ni = 0; ni < st.nodes.length; ni++) {
      const nd = st.nodes[ni];
      const dx = nd.x - cx;
      const dy = nd.y - cy;
      const d = Math.hypot(dx, dy);
      sumDist += d;
      if (d > maxDist) maxDist = d;
      let ang = Math.atan2(dy, dx);
      if (ang < 0) ang += TWO_PI;
      const sIdx = Math.floor((ang / TWO_PI) * SECTORS) % SECTORS;
      if (d > rawSectors[sIdx]) rawSectors[sIdx] = d;
      // Bleed reach into adjacent sectors so lobes are wide enough to wrap the node
      const prevIdx = (sIdx + SECTORS - 1) % SECTORS;
      const nextIdx = (sIdx + 1) % SECTORS;
      if (d * 0.90 > rawSectors[prevIdx]) rawSectors[prevIdx] = d * 0.90;
      if (d * 0.90 > rawSectors[nextIdx]) rawSectors[nextIdx] = d * 0.90;
    }

    // Mean-weighted base radius so dragging 1 function stretches its local pseudopod lobe
    // rather than inflating the entire cell uniformly in all directions!
    const meanDist = sumDist / Math.max(1, st.count);
    const coreR = Math.max(meanDist * 1.55 + comfortPadding, minBaseR);
    const baseR = Math.max(coreR, maxDist * 0.75 + comfortPadding);
    st.computedRadius = Math.max(baseR, maxDist + comfortPadding * 0.75);
    st.naturalRadius = coreR;

    // Compute normalized cellular lobe profile (amoeboid shape molding around internal organs/nodes)
    const lobes = new Float32Array(SECTORS);
    for (let s = 0; s < SECTORS; s++) {
      const neededR = rawSectors[s] > 0 ? (rawSectors[s] + comfortPadding) : (baseR * 0.86);
      const ratio = neededR / st.computedRadius;
      lobes[s] = Math.max(0.76, Math.min(1.42, ratio));
    }
    // 2-pass biological laplacian smoothing across neighboring lobes for plump cellular curvature
    const smoothed = new Float32Array(SECTORS);
    for (let pass = 0; pass < 2; pass++) {
      const src = pass === 0 ? lobes : smoothed;
      const dst = pass === 0 ? smoothed : lobes;
      for (let s = 0; s < SECTORS; s++) {
        const prev = src[(s + SECTORS - 1) % SECTORS];
        const next = src[(s + 1) % SECTORS];
        dst[s] = prev * 0.25 + src[s] * 0.50 + next * 0.25;
      }
    }
    st.lobeProfile = lobes;
  });
  state.cachedFolderStats = folderStats;
  state.folderClusters = folderStats;
  return folderStats;
}

// Locks a node's current coordinates into its active 4D layout anchor so it never snaps back after dragging
function pinNodeToActiveLayout(n) {
  if (!n || !n.layouts) return;
  const d = state.depth;
  if (d === 1 && n.layouts.obsidian) { n.layouts.obsidian.x = n.x; n.layouts.obsidian.y = n.y; }
  else if (d === 2 && n.layouts.system) { n.layouts.system.x = n.x; n.layouts.system.y = n.y; }
  else if (d === 3 && n.layouts.flow) { n.layouts.flow.x = n.x; n.layouts.flow.y = n.y; }
  else if (d === 4 && n.layouts.classes) { n.layouts.classes.x = n.x; n.layouts.classes.y = n.y; }
}
window.pinNodeToActiveLayout = pinNodeToActiveLayout;

// High-Performance Smooth Physics Step with Auto-Sleep & Adaptive Damping
function stepPhysics() {
  if (!state.physicsRunning || !state.data.nodes || state.data.nodes.length === 0) return;
  if (state.physicsSleeping) return;

  const nodes = state.data.nodes;
  const edges = state.data.edges || [];
  const nodeMap = state.nodeMap || (state.nodeMap = new Map(nodes.map(n => [n.id, n])));
  const d = state.depth;
  const isHuge = nodes.length > 1000;
  const activeDragId = state.draggedNode ? state.draggedNode.id : null;
  let totalSpeed = 0;

  // 1. Gentle pull towards active 4D topological layout target (soft so bubbles can live and breathe inside cell)
  const pull = isHuge ? 0.045 : 0.026;
  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    if (activeDragId && n.id === activeDragId) continue;
    let target = null;
    if (n.layouts) {
      if (d === 1 && n.layouts.obsidian) target = n.layouts.obsidian;
      else if (d === 2 && n.layouts.system) target = n.layouts.system;
      else if (d === 3 && n.layouts.flow) target = n.layouts.flow;
      else if (d === 4 && n.layouts.classes) target = n.layouts.classes;
    }
    if (target) {
      const dx = target.x - n.x;
      const dy = target.y - n.y;
      n.vx = (n.vx || 0) + dx * pull;
      n.vy = (n.vy || 0) + dy * pull;
    }
  }

  // 2. Hooke Spring Attraction on active edges (limit edge springs on huge graphs to avoid CPU starvation)
  const maxEdges = isHuge ? Math.min(edges.length, 2500) : edges.length;
  for (let i = 0; i < maxEdges; i++) {
    const e = edges[i];
    const s = e._s || nodeMap.get(e.source);
    const t = e._t || nodeMap.get(e.target);
    if (!s || !t) continue;
    if (!state.activeLayers.has(s.layer) || !state.activeLayers.has(t.layer)) continue;
    if (activeDragId && (s.id === activeDragId || t.id === activeDragId)) continue;

    const dx = t.x - s.x;
    const dy = t.y - s.y;
    const dist = Math.hypot(dx, dy) || 1;
    const restLen = e.relation === 'contains' ? 65 : 160;
    const force = Math.max(-4.5, Math.min(4.5, (dist - restLen) * 0.012));
    const fx = (dx / dist) * force;
    const fy = (dy / dist) * force;
    s.vx = (s.vx || 0) + fx;
    s.vy = (s.vy || 0) + fy;
    t.vx = (t.vx || 0) - fx;
    t.vy = (t.vy || 0) - fy;
  }

  // 2b. Organic Cellular Membrane Confinement, Cytoplasmic Gravity & Bubble-to-Bubble Repulsion
  updateFolderClusters();
  if (state.folderClusters) {
    // A) Intra-Cellular Organelle Dynamics (gravity to nucleus + soft anti-overlap spacing)
    state.folderClusters.forEach((st, folder) => {
      if (!st.nodes || st.nodes.length <= 1) return;
      const cx = st.cx !== undefined ? st.cx : (st.sumX / st.count);
      const cy = st.cy !== undefined ? st.cy : (st.sumY / st.count);
      const maxR = st.computedRadius || Math.max(100, Math.sqrt(st.count) * 58);
      const safeR = maxR * 0.85;

      // Intra-cellular particle repulsion (strided O(K) cap on dense folders for 120 FPS)
      const count = st.nodes.length;
      const innerStep = count > 48 ? Math.max(1, Math.floor(count / 28)) : 1;
      for (let i = 0; i < count; i++) {
        const na = st.nodes[i];
        for (let j = i + 1; j < count; j += innerStep) {
          const nb = st.nodes[j];
          const bdx = nb.x - na.x;
          const bdy = nb.y - na.y;
          const bdist2 = bdx * bdx + bdy * bdy;
          const minD = 55; // comfortable padding between bubbles
          if (bdist2 < minD * minD && bdist2 > 0.01) {
            const bdist = Math.sqrt(bdist2);
            const repulse = Math.min(3.5, (minD - bdist) * 0.05);
            const pfx = (bdx / bdist) * repulse;
            const pfy = (bdy / bdist) * repulse;
            if (!activeDragId || na.id !== activeDragId) {
              na.vx = (na.vx || 0) - pfx;
              na.vy = (na.vy || 0) - pfy;
            }
            if (!activeDragId || nb.id !== activeDragId) {
              nb.vx = (nb.vx || 0) + pfx;
              nb.vy = (nb.vy || 0) + pfy;
            }
          }
        }
      }

      // Micro-gravity towards folder nucleus & soft membrane perimeter retention
      for (let ni = 0; ni < st.nodes.length; ni++) {
        const n = st.nodes[ni];
        if (activeDragId && n.id === activeDragId) continue;

        const dx = n.x - cx;
        const dy = n.y - cy;
        const dist = Math.hypot(dx, dy) || 1;

        // Gentle cohesive gravity towards stable folder center
        const cohesion = 0.0025;
        n.vx = (n.vx || 0) + (cx - n.x) * cohesion;
        n.vy = (n.vy || 0) + (cy - n.y) * cohesion;

        // Elastic Membrane Surface Tension (repels inward if approaching perimeter)
        if (dist > safeR) {
          const overflow = dist - safeR;
          const pushBack = Math.min(5.0, overflow * 0.07);
          n.vx -= (dx / dist) * pushBack;
          n.vy -= (dy / dist) * pushBack;

          const hardLimit = maxR * 0.94;
          if (dist > hardLimit) {
            n.x = cx + (dx / dist) * hardLimit;
            n.y = cy + (dy / dist) * hardLimit;
            n.vx *= 0.5;
            n.vy *= 0.5;
          }
        }
      }
    });

    // B) Inter-Cluster Repulsion (accumulated per cluster in O(C^2) then applied in single O(N) pass)
    const clusterList = Array.from(state.folderClusters.values());
    for (let i = 0; i < clusterList.length; i++) {
      clusterList[i]._cfx = 0;
      clusterList[i]._cfy = 0;
    }
    for (let i = 0; i < clusterList.length; i++) {
      const cA = clusterList[i];
      if (!cA.nodes || cA.count === 0) continue;
      const cxA = cA.cx !== undefined ? cA.cx : (cA.sumX / cA.count);
      const cyA = cA.cy !== undefined ? cA.cy : (cA.sumY / cA.count);
      const rA = cA.computedRadius || 120;

      for (let j = i + 1; j < clusterList.length; j++) {
        const cB = clusterList[j];
        if (!cB.nodes || cB.count === 0) continue;
        const cxB = cB.cx !== undefined ? cB.cx : (cB.sumX / cB.count);
        const cyB = cB.cy !== undefined ? cB.cy : (cB.sumY / cB.count);
        const rB = cB.computedRadius || 120;

        const cdx = cxB - cxA;
        const cdy = cyB - cyA;
        const cdist = Math.hypot(cdx, cdy) || 1;
        const safeClusterDist = rA + rB + 110; // wide celestial corridor between folders

        if (cdist < safeClusterDist) {
          const overlap = safeClusterDist - cdist;
          const repulse = Math.min(6.5, overlap * 0.05);
          const rfx = (cdx / cdist) * repulse * 0.6;
          const rfy = (cdy / cdist) * repulse * 0.6;
          cA._cfx -= rfx;
          cA._cfy -= rfy;
          cB._cfx += rfx;
          cB._cfy += rfy;
        }
      }
    }
    for (let i = 0; i < clusterList.length; i++) {
      const c = clusterList[i];
      if (!c.nodes || (c._cfx === 0 && c._cfy === 0)) continue;
      const fx = c._cfx, fy = c._cfy;
      for (let k = 0; k < c.nodes.length; k++) {
        const nd = c.nodes[k];
        if (activeDragId && nd.id === activeDragId) continue;
        nd.vx = (nd.vx || 0) + fx;
        nd.vy = (nd.vy || 0) + fy;
      }
    }
  }

  // 3. Serene Cosmic Integration & Friction Damping
  const maxSpeed = 3.6;
  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    if (activeDragId && n.id === activeDragId) {
      n.vx = 0;
      n.vy = 0;
      continue;
    }
    n.vx = (n.vx || 0) * 0.82;
    n.vy = (n.vy || 0) * 0.82;
    const sp = Math.hypot(n.vx, n.vy);
    if (sp > maxSpeed) {
      n.vx = (n.vx / sp) * maxSpeed;
      n.vy = (n.vy / sp) * maxSpeed;
    }
    n.x += n.vx;
    n.y += n.vy;
    totalSpeed += sp;
  }

  // Auto-sleep physics when nodes have reached relaxation to maintain solid 120 FPS
  if (state.physicsCooldown !== undefined) {
    state.physicsCooldown--;
    if (!activeDragId && (state.physicsCooldown <= 0 || (isHuge ? totalSpeed < 0.80 : totalSpeed < 0.20))) {
      state.physicsSleeping = true;
      state.cachedEdgePaths = null; // trigger 1 clean static Path2D bake
    }
  }
}

window.stepPhysics = stepPhysics;
window.updateFolderClusters = updateFolderClusters;
