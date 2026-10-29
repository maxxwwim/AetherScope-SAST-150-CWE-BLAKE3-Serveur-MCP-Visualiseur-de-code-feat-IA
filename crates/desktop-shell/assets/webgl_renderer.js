// =========================================================================
// AETHERSCOPE V2 — WEBGL 2 RENDERER & 120 FPS COMPOSITOR (webgl_renderer.js)
// Cellular Membranes, Photons, Radar, Path2D Batching & 5 Affiliation Filters
// =========================================================================

// FPS Meter
let frameCount = 0, lastFpsTime = performance.now();
function trackFps(now) {
  frameCount++;
  if (now - lastFpsTime >= 500) {
    const fps = Math.round((frameCount * 1000) / (now - lastFpsTime));
    const el = document.getElementById('fps-counter');
    if (el) el.innerText = `${fps} FPS WebGL`;
    frameCount = 0;
    lastFpsTime = now;
  }
}

// --- Canvas 2D Font Stacks (Canvas does not resolve CSS var(--font-*)) ---
const FONT_UI = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif';
const FONT_MONO = 'ui-monospace, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace';

// --- Organic Biological Cellular Bubble Renderer (12-Lobe Quadratic Spline + Bilayer + Nucleus) ---
const _bubbleLabelCache = new Map();
const _cellPtsX = new Float32Array(12);
const _cellPtsY = new Float32Array(12);

function updateHydroHudProgress(folder, currentIdx, total) {
  const hud = document.getElementById('hydro-loading-hud');
  if (hud && currentIdx >= total) {
    hud.classList.add('hidden');
  }
}

// Traces a 100% smooth C1-continuous closed quadratic Bezier biological membrane (zero sharp vertices)
function traceCellularContour(ctx, radius, now, phase, lobeProfile, waveScale = 1.0, phaseShift = 0.0) {
  const N = 12;
  const t = now * 0.0018 + phase + phaseShift;
  for (let i = 0; i < N; i++) {
    const angle = (i / N) * Math.PI * 2;
    const lobeFactor = lobeProfile ? lobeProfile[i] : 1.0;
    // Multi-harmonic amoeboid pseudopod respiration (2-lobe elongation + 3-lobe & 5-lobe cellular ripple)
    const bioWave = (
      Math.sin(angle * 2.0 + t * 0.9) * 0.042 +
      Math.cos(angle * 3.0 - t * 1.15 + phase) * 0.034 +
      Math.sin(angle * 5.0 + t * 1.45) * 0.016
    ) * waveScale;
    const r = radius * (lobeFactor + bioWave);
    _cellPtsX[i] = Math.cos(angle) * r;
    _cellPtsY[i] = Math.sin(angle) * r;
  }

  ctx.beginPath();
  const startMidX = (_cellPtsX[N - 1] + _cellPtsX[0]) * 0.5;
  const startMidY = (_cellPtsY[N - 1] + _cellPtsY[0]) * 0.5;
  ctx.moveTo(startMidX, startMidY);
  for (let i = 0; i < N; i++) {
    const next = (i + 1) % N;
    const midX = (_cellPtsX[i] + _cellPtsX[next]) * 0.5;
    const midY = (_cellPtsY[i] + _cellPtsY[next]) * 0.5;
    ctx.quadraticCurveTo(_cellPtsX[i], _cellPtsY[i], midX, midY);
  }
  ctx.closePath();
}

function drawOrganicFluidBubble(ctx, fx, fy, bubbleR, now, phase, rR, rG, rB, isFolderSel, c, lodLevel, folder, count, isSubmerged = false, lobeProfile = null) {
  if (state.hiddenModules && state.hiddenModules.has(folder)) return;
  // Living harmonic cellular buoyancy float
  const floatY = Math.sin(now * 0.0015 + phase) * 2.6;
  const floatX = Math.cos(now * 0.0011 + phase * 0.85) * 1.8;

  ctx.save();
  ctx.translate(fx + floatX, fy + floatY);

  const colBase = `rgba(${rR}, ${rG}, ${rB}`;
  const lightR = Math.min(255, rR + 90);
  const lightG = Math.min(255, rG + 90);
  const lightB = Math.min(255, rB + 110);
  const colLight = `rgba(${lightR}, ${lightG}, ${lightB}`;

  let cachedLbl = _bubbleLabelCache.get(folder);
  if (!cachedLbl || cachedLbl.count !== count) {
    const shortName = typeof formatFolderLabel === 'function' ? formatFolderLabel(folder) : folder;
    cachedLbl = { shortName, count };
    _bubbleLabelCache.set(folder, cachedLbl);
  }

  // 1. OUTER ECTOPLASM HALO & EXOMEMBRANE (Smooth 12-Lobe Quadratic Spline)
  traceCellularContour(ctx, bubbleR, now, phase, lobeProfile, 1.0, 0.0);
  const topCrestY = _cellPtsY[9] || -bubbleR; // sector 9 is angle 3*PI/2 (top crest)

  const fillAlpha = isSubmerged ? 0.04 : (isFolderSel ? 0.16 : 0.075);
  const rimAlpha = isSubmerged ? 0.22 : (isFolderSel ? 0.82 : 0.48);
  ctx.fillStyle = `${colBase}, ${fillAlpha})`;
  ctx.fill();

  // Outer Lipid Bilayer Stroke
  ctx.lineWidth = Math.min(4.0, (isFolderSel ? 2.8 : 1.75) / c.zoom);
  ctx.strokeStyle = isFolderSel ? '#38bdf8' : `${colLight}, ${rimAlpha})`;
  ctx.stroke();

  // 2. INNER PHOSPHOLIPID CORTEX (Endomembrane breathing in counter-phase)
  traceCellularContour(ctx, bubbleR * 0.90, now, phase, lobeProfile, 0.85, 2.2);
  ctx.fillStyle = `${colBase}, ${isFolderSel ? 0.06 : 0.032})`;
  ctx.fill();
  ctx.lineWidth = Math.min(2.2, 1.0 / c.zoom);
  ctx.strokeStyle = isFolderSel ? 'rgba(56, 189, 248, 0.42)' : `${colLight}, 0.22)`;
  ctx.setLineDash([7 / c.zoom, 5 / c.zoom]);
  ctx.stroke();
  ctx.setLineDash([]);

  // 3. CELLULAR NUCLEUS & CYTOSKELETON FILAMENTS (Visible in LoD 1/2 or when selected)
  if (lodLevel <= 2 || isFolderSel) {
    // Cytoskeleton organic filaments radiating from nucleus to outer lobes
    ctx.beginPath();
    for (let k = 0; k < 12; k += 2) {
      const tx = _cellPtsX[k] * 0.88;
      const ty = _cellPtsY[k] * 0.88;
      const cpx = tx * 0.45 - ty * 0.12 * Math.sin(now * 0.0015 + k + phase);
      const cpy = ty * 0.45 + tx * 0.12 * Math.cos(now * 0.0015 + k + phase);
      ctx.moveTo(0, 0);
      ctx.quadraticCurveTo(cpx, cpy, tx, ty);
    }
    ctx.strokeStyle = `${colLight}, ${isFolderSel ? 0.20 : 0.09})`;
    ctx.lineWidth = Math.min(1.6, 0.85 / c.zoom);
    ctx.stroke();

    // Central Cellular Nucleus (Noyau Cellulaire)
    const nucR = Math.max(14, bubbleR * 0.21);
    traceCellularContour(ctx, nucR, now, phase * 1.3, null, 1.25, 1.1);
    ctx.fillStyle = `${colBase}, ${isFolderSel ? 0.22 : 0.12})`;
    ctx.fill();
    ctx.strokeStyle = isFolderSel ? 'rgba(56, 189, 248, 0.55)' : `${colLight}, 0.30)`;
    ctx.lineWidth = Math.min(1.8, 1.0 / c.zoom);
    ctx.stroke();

    // Cytoplasmic Vacuoles / Organelles (3 drifting micro-vesicles)
    ctx.beginPath();
    for (let v = 0; v < 3; v++) {
      const vAng = phase + v * 2.094 + now * 0.0004 * (v % 2 === 0 ? 1 : -1);
      const vDist = bubbleR * (0.48 + (v * 0.09));
      const vx = Math.cos(vAng) * vDist;
      const vy = Math.sin(vAng) * vDist;
      const vr = Math.max(4, bubbleR * (0.055 - v * 0.01));
      ctx.moveTo(vx + vr * 1.3, vy);
      ctx.ellipse(vx, vy, vr * 1.3, vr * 0.85, vAng + 0.5, 0, Math.PI * 2);
    }
    ctx.fillStyle = 'rgba(255, 255, 255, 0.07)';
    ctx.fill();
    ctx.strokeStyle = `${colLight}, 0.20)`;
    ctx.lineWidth = Math.min(1.4, 0.75 / c.zoom);
    ctx.stroke();
  }

  ctx.restore();

  const shortName = cachedLbl.shortName || (typeof formatFolderLabel === 'function' ? formatFolderLabel(folder) : folder);
  return {
    folder,
    shortName,
    count,
    x: fx + floatX,
    y: fy + floatY,
    topCrestY,
    bubbleR,
    isFolderSel,
    colBase,
    colLight
  };
}

// --- 4. High-Legibility Living Cellular Badges (Dedicated Top-Layer Pass) ---
// Renders folder cloud titles zoom-invariantly (sleek, compact 12-14px screen size) so they are crisp and never oversized
function drawFolderBadges(ctx, badges, c, lodLevel) {
  if (!badges || badges.length === 0) return;

  // By default (state.showFolderBadges === false), do NOT display folder names automatically across the galaxy.
  // Only render a folder badge when the folder is explicitly selected, hovered, or when the user enables showFolderBadges.
  const hoveredNodeFolder = state.hoveredNode ? (state.hoveredNode.folder || 'root') : null;
  const activeBadges = state.showFolderBadges === true
    ? badges
    : badges.filter(b => b.isFolderSel || b.folder === state.hoveredFolder || b.folder === hoveredNodeFolder);

  if (activeBadges.length === 0) return;

  // Refined screen font size: 12px - 14px depending on zoom for perfect legibility without being giant
  const screenFontSize = c.zoom < 0.4 ? 14 : (c.zoom < 0.9 ? 13 : 12);
  const fSize = screenFontSize / c.zoom;

  ctx.save();
  ctx.font = `700 ${fSize}px ${FONT_UI}`;
  ctx.textBaseline = 'middle';

  const padX = 8 / c.zoom;
  const padY = 4 / c.zoom;
  const badgeH = fSize + padY * 2;
  const borderRadius = 5 / c.zoom;
  const borderWidth = Math.max(1.0 / c.zoom, 1.2 / c.zoom);
  const tetherWidth = Math.max(1.0 / c.zoom, 1.3 / c.zoom);

  for (let i = 0; i < activeBadges.length; i++) {
    const b = activeBadges[i];
    if (state.hiddenModules && state.hiddenModules.has(b.folder)) continue;
    const folderDisplay = `📁 ${b.shortName}`;
    const countDisplay = `${b.count}`;

    const nameW = ctx.measureText(folderDisplay).width;
    const countW = ctx.measureText(countDisplay).width;

    const countPillW = countW + 10 / c.zoom;
    const badgeW = nameW + countPillW + padX * 2 + 6 / c.zoom;
    const badgeCenterY = b.topCrestY - badgeH * 0.5 - 10 / c.zoom;

    ctx.save();
    ctx.translate(b.x, b.y);

    // 1. Tether Link from top cellular lobe to badge
    ctx.beginPath();
    ctx.moveTo(0, b.topCrestY);
    ctx.lineTo(0, badgeCenterY + badgeH * 0.5);
    ctx.strokeStyle = b.isFolderSel ? '#38bdf8' : `${b.colLight}, 0.85)`;
    ctx.lineWidth = tetherWidth;
    ctx.stroke();

    // 2. Tether Anchor Dot on top lobe
    ctx.beginPath();
    ctx.arc(0, b.topCrestY, 2.5 / c.zoom, 0, Math.PI * 2);
    ctx.fillStyle = b.isFolderSel ? '#38bdf8' : `${b.colLight}, 0.95)`;
    ctx.fill();

    // 3. Render Badge Capsule
    ctx.translate(0, badgeCenterY);
    ctx.globalAlpha = (lodLevel === 3 && !b.isFolderSel) ? 0.92 : 1.0;

    // Crisp deep obsidian-glass backdrop
    ctx.beginPath();
    ctx.roundRect(-badgeW * 0.5, -badgeH * 0.5, badgeW, badgeH, borderRadius);
    ctx.fillStyle = b.isFolderSel ? 'rgba(12, 74, 110, 0.98)' : 'rgba(3, 7, 18, 0.96)';
    ctx.fill();

    // Glowing chromatic border matching folder palette
    ctx.strokeStyle = b.isFolderSel ? '#38bdf8' : `${b.colLight}, 0.85)`;
    ctx.lineWidth = borderWidth;
    ctx.shadowColor = b.isFolderSel ? 'rgba(56, 189, 248, 0.75)' : `${b.colLight}, 0.45)`;
    ctx.shadowBlur = 6 / c.zoom;
    ctx.stroke();
    ctx.shadowBlur = 0;

    // Folder Name Text (left-aligned)
    const leftX = -badgeW * 0.5 + padX;
    ctx.fillStyle = b.isFolderSel ? '#38bdf8' : '#ffffff';
    ctx.textAlign = 'left';
    ctx.shadowColor = 'rgba(0, 0, 0, 0.95)';
    ctx.shadowBlur = 3 / c.zoom;
    ctx.fillText(folderDisplay, leftX, 0);

    // Count Token Pill (right-aligned)
    const countPillX = badgeW * 0.5 - padX - countPillW;
    const countPillH = fSize + 2 / c.zoom;
    ctx.beginPath();
    ctx.roundRect(countPillX, -countPillH * 0.5, countPillW, countPillH, 3.5 / c.zoom);
    ctx.fillStyle = b.isFolderSel ? 'rgba(56, 189, 248, 0.35)' : `${b.colBase}, 0.42)`;
    ctx.fill();
    ctx.strokeStyle = b.isFolderSel ? '#38bdf8' : `${b.colLight}, 0.75)`;
    ctx.lineWidth = 1.0 / c.zoom;
    ctx.stroke();

    ctx.fillStyle = b.isFolderSel ? '#e0f2fe' : '#ffffff';
    ctx.textAlign = 'center';
    ctx.fillText(countDisplay, countPillX + countPillW * 0.5, 0);

    ctx.restore();
  }

  ctx.restore();
}

function smoothFlyTo(tx, ty, tz = null, customDuration = null) {
  if (!Number.isFinite(tx) || !Number.isFinite(ty)) return;
  const c = state.camera;
  const startX = Number.isFinite(c.x) ? c.x : 0;
  const startY = Number.isFinite(c.y) ? c.y : 0;
  const startZoom = Number.isFinite(c.zoom) && c.zoom > 0 ? c.zoom : 0.75;
  const finalZoom = (tz !== null && Number.isFinite(tz)) ? tz : (startZoom < 0.75 ? 0.95 : startZoom);

  const dist = Math.hypot(tx - startX, ty - startY);
  // Trajectoire cinématique naturelle : durée fluide et douce selon la distance
  const duration = customDuration || Math.max(900, Math.min(1850, 800 + dist * 0.45));

  // Courbe d'altitude : si la caméra voyage loin dans l'espace galactique,
  // elle prend une légère hauteur de survol en cloche (dézoom 12-22%) puis redescend délicatement
  const altitudeFactor = dist > 250 ? Math.min(0.20, (dist / 2400) * 0.20) : 0.0;

  state.cameraFlight = {
    active: true,
    startX,
    startY,
    startZoom,
    targetX: tx,
    targetY: ty,
    targetZoom: finalZoom,
    altitudeFactor,
    startTime: performance.now(),
    duration,
  };

  c.targetX = tx;
  c.targetY = ty;
  c.targetZoom = finalZoom;
  wakePhysics(120);
}
window.smoothFlyTo = smoothFlyTo;

// 120 FPS Main Loop with 3-Tier LoD Semantic Zoom
let instanceData = new Float32Array(0);
let lastRenderTime = 0;

function render(now) {
  requestAnimationFrame(render);
  const fpsLimit = parseInt(state.settings?.fpsLimit || '120', 10);
  if (fpsLimit > 0 && fpsLimit < 120) {
    const minDelta = 1000 / fpsLimit;
    if (now - lastRenderTime < minDelta) return;
  }
  lastRenderTime = now;

  try {
    trackFps(now);
    stepPhysics();

    // Camera NaN guard & smooth lerp
    const c = state.camera;
    if (!Number.isFinite(c.x)) c.x = 0;
    if (!Number.isFinite(c.y)) c.y = 0;
    if (!Number.isFinite(c.targetX)) c.targetX = 0;
    if (!Number.isFinite(c.targetY)) c.targetY = 0;
    if (!Number.isFinite(c.zoom) || c.zoom <= 0) c.zoom = 0.65;
    c.targetZoom = Math.max(0.04, Math.min(6.0, c.targetZoom));

    // Drone Flight Animation (Smooth cinematic camera flight for MCP & AI Navigation)
    if (state.cameraFlight && state.cameraFlight.active) {
      const flight = state.cameraFlight;
      const elapsed = now - flight.startTime;
      const t = Math.max(0, Math.min(1.0, elapsed / flight.duration));

      // Quintic Smootherstep (Perlin smoother curve : zéro à-coup au décollage et à l'atterrissage)
      const ease = t * t * t * (t * (t * 6 - 15) + 10);

      c.x = flight.startX + (flight.targetX - flight.startX) * ease;
      c.y = flight.startY + (flight.targetY - flight.startY) * ease;

      // Arc d'altitude galactique réaliste (prise d'altitude fluide en milieu de survol puis descente douce)
      const arc = Math.sin(t * Math.PI) * flight.altitudeFactor;
      c.zoom = (flight.startZoom + (flight.targetZoom - flight.startZoom) * ease) * (1.0 - arc);

      c.targetX = c.x;
      c.targetY = c.y;
      c.targetZoom = c.zoom;

      if (t >= 1.0) {
        c.x = flight.targetX;
        c.y = flight.targetY;
        c.zoom = flight.targetZoom;
        state.cameraFlight.active = false;
      }
    } else {
      // Déplacement standard fluide avec amortissement cinématique doux (0.14)
      c.x += (c.targetX - c.x) * 0.14;
      c.y += (c.targetY - c.y) * 0.14;
      c.zoom += (c.targetZoom - c.zoom) * 0.14;

      if (Math.abs(c.targetZoom - c.zoom) < 0.0004) c.zoom = c.targetZoom;
      if (Math.abs(c.targetX - c.x) < 0.04) c.x = c.targetX;
      if (Math.abs(c.targetY - c.y) < 0.04) c.y = c.targetY;
    }

    // 3-Tier LoD Semantic Zoom Hierarchy
    let lodLevel = 3;
    if (c.zoom < 0.42) lodLevel = 1;
    else if (c.zoom < 0.95) lodLevel = 2;

    // Sync Topbar Zoom Buttons active state only when LoD level changes
    if (lodLevel !== state._lastDomLod) {
      state._lastDomLod = lodLevel;
      const b1 = document.getElementById('btn-zoom-1');
      const b2 = document.getElementById('btn-zoom-2');
      const b3 = document.getElementById('btn-zoom-3');
      if (b1) b1.classList.toggle('active', lodLevel === 1);
      if (b2) b2.classList.toggle('active', lodLevel === 2);
      if (b3) b3.classList.toggle('active', lodLevel === 3);

      const lodEl = document.getElementById('lod-indicator');
      if (lodEl) {
        if (lodLevel === 1) lodEl.innerText = '🔭 Zoom 1 • Macro Dossiers';
        else if (lodLevel === 2) lodEl.innerText = '🪐 Zoom 2 • Entrée Dossiers';
        else lodEl.innerText = '🔬 Zoom 3 • Microcosme Appels';
      }
    }

    const nodes = state.data.nodes || [];
    const edges = state.data.edges || [];
    const nodeMap = state.nodeMap || (state.nodeMap = new Map(nodes.map(n => [n.id, n])));

    const selId = state.selectedNode ? state.selectedNode.id : null;
    const hovId = state.hoveredNode ? state.hoveredNode.id : null;
    const isAnimatedMode = state.deadCodeMode || (state.aiTelemetry && state.aiTelemetry.target);
    const isSpawning = !!(state.spawnAnimationEndTime && now < state.spawnAnimationEndTime);
    const isDirty = state.instanceDataDirty 
                 || !state.physicsSleeping 
                 || isAnimatedMode
                 || isSpawning
                 || (selId !== state.lastRenderedSelId)
                 || (hovId !== state.lastRenderedHovId)
                 || (lodLevel !== state.lastRenderedLod)
                 || (instanceData.length !== nodes.length * 9);

    if (isDirty) {
      state.lastRenderedSelId = selId;
      state.lastRenderedHovId = hovId;
      state.lastRenderedLod = lodLevel;
      state.instanceDataDirty = false;

      // Update Node Instances
      if (instanceData.length !== nodes.length * 9) {
        instanceData = new Float32Array(nodes.length * 9);
      }

      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i];
        const off = i * 9;
        instanceData[off] = n.x;
        instanceData[off + 1] = n.y;

        const isModuleHidden = state.hiddenModules && state.hiddenModules.has(n.folder || 'root');
        const isVisible = state.activeLayers.has(n.layer) && !isModuleHidden;
        const isSelected = state.selectedNode && state.selectedNode.id === n.id;
        const isHovered = state.hoveredNode && state.hoveredNode.id === n.id;
        const isBeacon = state.beaconNode && state.beaconNode.id === n.id;

        const isFunc = n.kind === 'function' || n.kind === 'method' || n.symbol_type === 'function' || n.stereotype === '«Function»' || n.stereotype === '«Method»';

        // Radii hierarchy: Functions are mini-points, files and classes are planetary orbs
        let baseR;
        if (isFunc) {
          baseR = 3.6; // mini point pour fonction!
        } else if (n.is_file) {
          baseR = 12.0; // planete fichier
        } else if (n.is_class) {
          baseR = 13.5; // cristal classe
        } else if (n.stereotype === '«Form»' || n.is_form) {
          baseR = 11.5; // balise formulaire
        } else if (n.layer === 'docs' || n.symbol_type === 'document' || (n.file_path && n.file_path.endsWith('.md'))) {
          baseR = 11.0; // parchemin doc
        } else {
          baseR = 8.5;
        }

        if (!isVisible) baseR = 0.0;
        else if (isSelected || isBeacon) baseR *= 1.6;
        else if (isHovered) baseR *= 1.35;

        // Zoom 1 & Zoom 2: ALL nodes carry the folder color
        // Zoom 3: Functional layer colors + Form (#f97316) & API (#14b8a6) accents
        let col;
        if (lodLevel === 1 || lodLevel === 2) {
          col = getFolderColor(n.folder);
        } else {
          if (n.stereotype === '«Form»' || n.is_form) {
            col = [0.99, 0.52, 0.15, 1.0]; // energetic orange
          } else if (n.stereotype === '«Endpoint»' || (n.id && n.id.startsWith('api::'))) {
            col = [0.10, 0.82, 0.72, 1.0]; // vibrant teal
          } else if (n.is_class) {
            col = [0.99, 0.82, 0.25, 1.0]; // celestial solar gold
          } else if (isFunc) {
            col = [0.42, 0.88, 1.0, 1.0]; // sparkling starlight cyan spark
          } else {
            col = LAYER_COLORS[n.layer] || [0.38, 0.88, 0.98, 1.0];
          }
        }
        if (isBeacon) col = [0.99, 0.85, 0.20, 1.0];

        // Mode X-Ray Focus Tunnel
        if (state.xrayMode && state.xrayTunnel) {
          const t = state.xrayTunnel;
          if (n.id === t.centerId) {
            baseR *= 1.85;
            col = [1.0, 1.0, 1.0, 1.0]; // Hyper-bright supernova core
          } else if (t.upstream.has(n.id)) {
            baseR *= 1.40;
            col = [0.22, 0.74, 0.97, 1.0]; // Electric Cyan #38bdf8 for callers
          } else if (t.downstream.has(n.id)) {
            baseR *= 1.40;
            col = [0.96, 0.62, 0.07, 1.0]; // Imperial Amber Gold #f59e0b for dependencies
          } else {
            baseR *= 0.50;
            col = [col[0] * 0.35, col[1] * 0.35, col[2] * 0.35, 0.06]; // Ghost star (6% alpha)
          }
        }

        // Mode Dead Code & Dépendances Orphelines
        if (state.deadCodeMode) {
          if (state.deadCodeSet.has(n.id)) {
            const pulse = 1.35 + 0.35 * Math.sin(now * 0.007);
            baseR *= pulse;
            col = [0.95, 0.22, 0.22, 1.0]; // Pulsing Crimson Red #ef4444
          } else {
            baseR *= 0.45;
            col = [col[0] * 0.30, col[1] * 0.30, col[2] * 0.30, 0.06]; // Ghost star
          }
        }

        // Mode Architecture & Health Heatmap
        if (state.heatmapMode) {
          const hm = state.heatmapScores?.get(n.id);
          if (hm) {
            col = getHeatmapColor(hm.score);
            baseR *= (0.75 + hm.score * 1.35); // Hotspots scale up massively
          }
        }

        // Mode Traçabilité Pipeline Données (Form Submit ➔ Net HTTP ➔ WS Temps Réel ➔ DB)
        if (state.dataPipelineMode) {
          const isPipelineNode = n.layer === 'database' || n.stereotype === '«Form»' || n.is_form || n.stereotype === '«Endpoint»' || (n.id && n.id.startsWith('api::')) || (n._incidentEdges && n._incidentEdges.some(e => {
            const r = e.relation;
            return r === 'submits' || r === 'network' || r === 'realtime' || r === 'realtimews' || r === 'realtime_ws' || r === 'queries';
          }));
          if (isPipelineNode) {
            baseR *= 1.35;
          } else {
            baseR *= 0.45;
            col = [col[0] * 0.35, col[1] * 0.35, col[2] * 0.35, 0.10];
          }
        }

        // Node Security Violation Alert Pulse
        if (n.has_security_violation) {
          const pulse = 1.25 + 0.28 * Math.sin(now * 0.009);
          baseR *= pulse;
          col = [0.99, 0.15, 0.25, 1.0]; // Blinking Red Alert Beacon
        }

        // Active AI Telemetry Target Beacon Glow
        if (state.aiTelemetry && state.aiTelemetry.target) {
          const tLower = state.aiTelemetry.target.toLowerCase();
          const isAiTarget = n.name.toLowerCase() === tLower || 
                             n.id.toLowerCase() === tLower || 
                             (n.file_path && n.file_path.toLowerCase().includes(tLower));
          if (isAiTarget) {
            baseR *= 1.55;
            col = [0.85, 0.52, 1.0, 1.0]; // Radiant Neon Violet #c084fc
          }
        }

        instanceData[off + 2] = baseR;
        instanceData[off + 3] = col[0];
        instanceData[off + 4] = col[1];
        instanceData[off + 5] = col[2];
        instanceData[off + 6] = col[3];
        instanceData[off + 7] = isFunc ? 0.0 : (n.is_file ? 1.0 : (n.is_class ? 2.0 : 3.0));
        instanceData[off + 8] = n.birth_time || 0.0;
      }

      gl.bindBuffer(gl.ARRAY_BUFFER, instanceVbo);
      gl.bufferData(gl.ARRAY_BUFFER, instanceData, gl.DYNAMIC_DRAW);
    }

    // 1. WebGL 2 GPU Render (1 Draw Call for all nodes!)
    gl.clearColor(0.012, 0.018, 0.038, 1.0); // Abyssal midnight cosmic void
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    gl.useProgram(program);
    gl.uniform2f(uRes, canvas.width, canvas.height);
    gl.uniform2f(uCam, c.x, c.y);
    gl.uniform1f(uZoom, c.zoom);
    gl.uniform1f(uTime, now);

    gl.bindVertexArray(vao);
    gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, nodes.length);
    gl.bindVertexArray(null);

    // 2. Lines Overlay, Labels, Parallax Starfield & Folder Halos
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, lineCanvas.width, lineCanvas.height);

    // Draw Living Celestial Parallax Starfield & Deep Space Nebulae
    if (state.showStarfield !== false) {
      ctx.save();
      const sfW = lineCanvas.width, sfH = lineCanvas.height;

      // Deep Space Nebula Glows (2 radiant celestial dust pockets)
      const neb1X = ((sfW * 0.25 - c.x * 0.04) % sfW + sfW) % sfW;
      const neb1Y = ((sfH * 0.35 - c.y * 0.04) % sfH + sfH) % sfH;
      const grad1 = ctx.createRadialGradient(neb1X, neb1Y, 0, neb1X, neb1Y, sfW * 0.45);
      grad1.addColorStop(0, 'rgba(99, 102, 241, 0.07)');
      grad1.addColorStop(0.5, 'rgba(56, 189, 248, 0.035)');
      grad1.addColorStop(1, 'rgba(0, 0, 0, 0)');
      ctx.fillStyle = grad1;
      ctx.fillRect(0, 0, sfW, sfH);

      const neb2X = ((sfW * 0.75 - c.x * 0.06) % sfW + sfW) % sfW;
      const neb2Y = ((sfH * 0.70 - c.y * 0.06) % sfH + sfH) % sfH;
      const grad2 = ctx.createRadialGradient(neb2X, neb2Y, 0, neb2X, neb2Y, sfW * 0.40);
      grad2.addColorStop(0, 'rgba(168, 85, 247, 0.06)');
      grad2.addColorStop(0.5, 'rgba(236, 72, 153, 0.025)');
      grad2.addColorStop(1, 'rgba(0, 0, 0, 0)');
      ctx.fillStyle = grad2;
      ctx.fillRect(0, 0, sfW, sfH);

      // Cosmic Constellation & Parallax Starfield
      for (let i = 0; i < starfield.length; i++) {
        const s = starfield[i];
        const driftX = Math.sin(now * 0.0008 + s.twinklePhase) * 4.0;
        const driftY = Math.cos(now * 0.0009 + s.twinklePhase * 1.3) * 3.0;
        let sx = ((s.x + driftX - c.x * s.z) * 0.7 + sfW / 2) % sfW;
        if (sx < 0) sx += sfW;
        let sy = ((s.y + driftY - c.y * s.z) * 0.7 + sfH / 2) % sfH;
        if (sy < 0) sy += sfH;

        // Twinkle pulse
        const tw = Math.sin(now * s.twinkleSpeed + s.twinklePhase);
        const alpha = Math.max(0.15, Math.min(0.95, s.baseAlpha * (0.7 + 0.3 * tw)));

        ctx.beginPath();
        ctx.arc(sx, sy, s.r, 0, Math.PI * 2);
        ctx.fillStyle = s.color.replace('ALPHA', alpha.toFixed(2));
        ctx.fill();

        // Corona & 4-point diffraction cross for bright stellar beacons
        if (s.hasCorona && alpha > 0.45) {
          ctx.beginPath();
          ctx.arc(sx, sy, s.r * 2.8, 0, Math.PI * 2);
          ctx.fillStyle = s.color.replace('ALPHA', (alpha * 0.22).toFixed(2));
          ctx.fill();

          ctx.strokeStyle = s.color.replace('ALPHA', (alpha * 0.40).toFixed(2));
          ctx.lineWidth = 1;
          const spike = s.r * 3.5;
          ctx.beginPath();
          ctx.moveTo(sx - spike, sy); ctx.lineTo(sx + spike, sy);
          ctx.moveTo(sx, sy - spike); ctx.lineTo(sx, sy + spike);
          ctx.stroke();
        }
      }
      ctx.restore();
    }

    ctx.save();
    ctx.translate(lineCanvas.width / 2 - c.x * c.zoom, lineCanvas.height / 2 - c.y * c.zoom);
    ctx.scale(c.zoom, c.zoom);

    // Viewport bounds in world coordinates for Frustum Culling
    const halfW = (lineCanvas.width / 2) / c.zoom;
    const halfH = (lineCanvas.height / 2) / c.zoom;
    const vLeft = c.x - halfW - 80;
    const vRight = c.x + halfW + 80;
    const vTop = c.y - halfH - 80;
    const vBottom = c.y + halfH + 80;

    // 3D CYBERNETIC GROUND PLATEAU
    if (state.showGrid !== false) {
      const gridSize = 140;
      const majorStep = 5;
      const startGridX = Math.floor(vLeft / gridSize) * gridSize;
      const endGridX = Math.ceil(vRight / gridSize) * gridSize;
      const startGridY = Math.floor(vTop / gridSize) * gridSize;
      const endGridY = Math.ceil(vBottom / gridSize) * gridSize;

      // Primary Grid Lines
      ctx.beginPath();
      for (let gx = startGridX; gx <= endGridX; gx += gridSize) {
        if (gx % (gridSize * majorStep) === 0) continue;
        ctx.moveTo(gx, vTop);
        ctx.lineTo(gx, vBottom);
      }
      for (let gy = startGridY; gy <= endGridY; gy += gridSize) {
        if (gy % (gridSize * majorStep) === 0) continue;
        ctx.moveTo(vLeft, gy);
        ctx.lineTo(vRight, gy);
      }
      ctx.strokeStyle = 'rgba(56, 189, 248, 0.055)';
      ctx.lineWidth = 1.0 / c.zoom;
      ctx.stroke();

      // Major Grid Lines & Coordinates
      ctx.beginPath();
      for (let gx = startGridX; gx <= endGridX; gx += gridSize) {
        if (gx % (gridSize * majorStep) !== 0) continue;
        ctx.moveTo(gx, vTop);
        ctx.lineTo(gx, vBottom);
      }
      for (let gy = startGridY; gy <= endGridY; gy += gridSize) {
        if (gy % (gridSize * majorStep) !== 0) continue;
        ctx.moveTo(vLeft, gy);
        ctx.lineTo(vRight, gy);
      }
      ctx.strokeStyle = 'rgba(56, 189, 248, 0.14)';
      ctx.lineWidth = 1.8 / c.zoom;
      ctx.stroke();

      // Intersection Crosshairs on Major Grid Intersections
      ctx.beginPath();
      const crossSize = 8 / c.zoom;
      for (let gx = startGridX; gx <= endGridX; gx += gridSize * majorStep) {
        for (let gy = startGridY; gy <= endGridY; gy += gridSize * majorStep) {
          ctx.moveTo(gx - crossSize, gy); ctx.lineTo(gx + crossSize, gy);
          ctx.moveTo(gx, gy - crossSize); ctx.lineTo(gx, gy + crossSize);
        }
      }
      ctx.strokeStyle = 'rgba(125, 211, 252, 0.35)';
      ctx.lineWidth = 1.2 / c.zoom;
      ctx.stroke();

      // Origin Coordinate Compass (0,0 Plateau Anchor)
      if (vLeft <= 0 && vRight >= 0 && vTop <= 0 && vBottom >= 0) {
        ctx.save();
        ctx.beginPath();
        ctx.arc(0, 0, 16 / c.zoom, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(56, 189, 248, 0.15)';
        ctx.fill();
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 1.8 / c.zoom;
        ctx.stroke();
        ctx.restore();
      }
    }

    // Compute folder clusters for organic water bubbles
    let folderStats = state.cachedFolderStats;
    if (!folderStats || !state.physicsSleeping || state.instanceDataDirty) {
      folderStats = updateFolderClusters();
    }

    // 3D Plateau Ground Drop Shadows
    if (!state.xrayMode && folderStats && state.showShadows !== false) {
      ctx.save();
      ctx.beginPath();
      folderStats.forEach((st) => {
        const cx = st.cx !== undefined ? st.cx : (st.sumX / st.count);
        const cy = st.cy !== undefined ? st.cy : (st.sumY / st.count);
        const bubbleR = st.computedRadius || Math.max(75, Math.sqrt(st.count) * 42);
        if (cx + bubbleR * 1.5 < vLeft || cx - bubbleR * 1.5 > vRight || cy + bubbleR * 1.5 < vTop || cy - bubbleR * 1.5 > vBottom) return;
        const shadowY = cy + bubbleR * 0.46 + 24;
        const shadowRx = bubbleR * 0.85;
        const shadowRy = bubbleR * 0.22;
        ctx.moveTo(cx + 6 + shadowRx, shadowY);
        ctx.ellipse(cx + 6, shadowY, shadowRx, shadowRy, 0, 0, Math.PI * 2);
      });
      ctx.fillStyle = 'rgba(2, 6, 23, 0.45)';
      ctx.fill();
      ctx.restore();
    }

    // Fast Semantic README Impact Zones
    const docImpactZones = [];
    const selNode = state.selectedNode;
    const hovNode = state.hoveredNode;
    const rawZones = state.cachedDocImpactZones || [];
    const isMassive = nodes.length > 500;

    for (let zi = 0; zi < rawZones.length; zi++) {
      const zone = rawZones[zi];
      const isDocActive = (selNode && selNode.id === zone.doc.id) || 
                          (hovNode && hovNode.id === zone.doc.id) || 
                          (state.selectedFolder && state.selectedFolder === zone.folder);
      const isTargetActive = (selNode && zone.allAssociated.some(an => an.id === selNode.id)) ||
                             (hovNode && zone.allAssociated.some(an => an.id === hovNode.id));
      const isHighlighted = isDocActive || isTargetActive;

      if (isMassive && !isHighlighted && lodLevel > 2) continue;

      let sumX = zone.doc.x, sumY = zone.doc.y, count = 1;
      let minX = zone.doc.x, maxX = zone.doc.x, minY = zone.doc.y, maxY = zone.doc.y;
      const assoc = zone.allAssociated;

      for (let ai = 0; ai < assoc.length; ai++) {
        const an = assoc[ai];
        sumX += an.x; sumY += an.y; count++;
        if (an.x < minX) minX = an.x; if (an.x > maxX) maxX = an.x;
        if (an.y < minY) minY = an.y; if (an.y > maxY) maxY = an.y;
      }

      const cx = sumX / count;
      const cy = sumY / count;
      const span = Math.hypot(maxX - minX, maxY - minY);
      const radius = Math.max(95, span * 0.58 + 40);

      docImpactZones.push({
        doc: zone.doc,
        cx, cy, radius,
        keywordMatchCount: zone.keywordMatchCount,
        totalCodeNodes: zone.totalCodeNodes,
        coverageRatio: zone.coverageRatio,
        isHighlighted,
        allAssociated: zone.allAssociated
      });

      if (isMassive && docImpactZones.length >= 25 && !isHighlighted) break;
    }

    // Draw Semantic README Impact Zones
    if (docImpactZones.length > 0) {
      docImpactZones.forEach(zone => {
        if (state.xrayMode && state.xrayTunnel) {
          const hasTunnelNode = zone.allAssociated.some(an => state.xrayTunnel.all.has(an.id));
          if (!hasTunnelNode && zone.doc.id !== state.xrayTunnel.centerId) return;
        }
        const { cx, cy, radius, isHighlighted, keywordMatchCount, coverageRatio, doc } = zone;
        if (cx + radius < vLeft || cx - radius > vRight || cy + radius < vTop || cy - radius > vBottom) return;

        ctx.beginPath();
        ctx.fillStyle = isHighlighted
          ? 'rgba(16, 185, 129, 0.14)'
          : (coverageRatio >= 0.4 ? 'rgba(16, 185, 129, 0.045)' : 'rgba(245, 158, 11, 0.045)');
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.fill();

        ctx.beginPath();
        ctx.strokeStyle = isHighlighted ? '#38bdf8' : (coverageRatio >= 0.4 ? 'rgba(16, 185, 129, 0.35)' : 'rgba(245, 158, 11, 0.30)');
        ctx.lineWidth = (isHighlighted ? 2.0 : 1.0) / c.zoom;
        ctx.setLineDash(isHighlighted ? [8 / c.zoom, 4 / c.zoom] : [4 / c.zoom, 4 / c.zoom]);
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.stroke();
        ctx.setLineDash([]);

        if (isHighlighted) {
          const badgeY = cy - radius + 14 / c.zoom;
          ctx.font = `700 ${11 / c.zoom}px ${FONT_MONO}`;
          const pct = Math.round(coverageRatio * 100);
          const badgeText = `📖 ${doc.name} • ${keywordMatchCount} mots-clés (${pct}% spec)`;
          const btw = ctx.measureText(badgeText).width;
          const bpad = 5 / c.zoom;

          ctx.fillStyle = 'rgba(14, 116, 144, 0.95)';
          ctx.strokeStyle = '#38bdf8';
          ctx.lineWidth = 1.0 / c.zoom;
          ctx.beginPath();
          ctx.roundRect(cx - btw / 2 - bpad, badgeY - 7 / c.zoom - bpad, btw + bpad * 2, 14 / c.zoom + bpad * 2, 4 / c.zoom);
          ctx.fill();
          ctx.stroke();

          ctx.fillStyle = '#ffffff';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(badgeText, cx, badgeY);
        }
      });
    }

    // Render Active Expanding Water Ripples
    if (state.waterRipples && state.waterRipples.length > 0) {
      const survivingRipples = [];
      for (let i = 0; i < state.waterRipples.length; i++) {
        const rip = state.waterRipples[i];
        const age = now - rip.startTime;
        if (age < rip.duration) {
          survivingRipples.push(rip);
          const prog = age / rip.duration;
          const ripR = rip.baseRadius * (0.8 + prog * 0.9);
          const ripAlpha = (1.0 - prog) * 0.45;
          ctx.beginPath();
          ctx.arc(rip.x, rip.y, ripR, 0, Math.PI * 2);
          ctx.strokeStyle = `rgba(${rip.r}, ${rip.g}, ${rip.b}, ${ripAlpha})`;
          ctx.lineWidth = Math.min(5.0, (2.2 * (1.0 - prog)) / c.zoom);
          ctx.stroke();
        }
      }
      state.waterRipples = survivingRipples;
    }

    // Organic Aquatic Folder Bubbles (Membranes & Cytoskeletons in Background Layer)
    const pendingFolderBadges = [];
    if (!state.xrayMode && folderStats && state.showFolderMembranes !== false) {
      folderStats.forEach((st, folder) => {
        let birthInfo = state.bubbleBirthMap ? state.bubbleBirthMap.get(folder) : null;
        if (!birthInfo) {
          birthInfo = { birthTime: now, poppedSound: true, phase: Math.sin(st.count * 12.3) * 3, folderIndex: 0, totalFolders: 1 };
          if (state.bubbleBirthMap) state.bubbleBirthMap.set(folder, birthInfo);
        }
        const phase = birthInfo.phase;
        const birthTime = birthInfo.birthTime || 0;

        const cx = st.cx !== undefined ? st.cx : (st.sumX / st.count);
        const cy = st.cy !== undefined ? st.cy : (st.sumY / st.count);
        const baseRadius = st.computedRadius || Math.max(75, Math.sqrt(st.count) * 42);
        if (cx + baseRadius * 1.5 < vLeft || cx - baseRadius * 1.5 > vRight || cy + baseRadius * 1.5 < vTop || cy - baseRadius * 1.5 > vBottom) return;

        const birthAge = now - birthTime;
        if (birthAge <= 0) return;
        const birthDuration = 520;
        let scale = 1.0;
        if (birthAge < birthDuration) {
          const p = Math.min(1.0, birthAge / birthDuration);
          scale = 1.0 - Math.pow(1.0 - p, 3.0);
        }

        const bubbleR = baseRadius * scale;
        if (bubbleR <= 2.0) return;

        const isFolderSel = state.selectedFolder === folder;
        const colRgba = getFolderColor(folder);
        const rR = Math.round(colRgba[0] * 255);
        const rG = Math.round(colRgba[1] * 255);
        const rB = Math.round(colRgba[2] * 255);

        const badgeData = drawOrganicFluidBubble(ctx, cx, cy, bubbleR, now, phase, rR, rG, rB, isFolderSel, c, lodLevel, folder, st.count, false, st.lobeProfile);
        if (badgeData) {
          pendingFolderBadges.push(badgeData);
        }
      });
    }

    // =========================================================================
    // 10K+ EDGES HYPER-FLUIDITY ENGINE WITH 5 AFFILIATION FILTERS
    // =========================================================================
    const highlightActive = !!(selId || hovId);
    const highlightDimFactor = highlightActive ? 0.35 : 1.0;
    const isLargeRepo = nodes.length > 450 || edges.length > 1200;
    const isUltraMassiveEdges = edges.length > 2200;

    // 1. Draw Inter-Cellular Synaptic Arteries (Disabled by default - only show listed typed edges)
    const activePhotons = [];
    if (state.showFolderArteries === true && !state.xrayMode && folderStats && state.folderBundles && state.folderBundles.length > 0) {
      const pArteries = new Path2D();
      let hasArteries = false;
      const bundles = state.folderBundles;
      for (let bi = 0; bi < bundles.length; bi++) {
        const b = bundles[bi];
        const stA = folderStats.get(b.folderA);
        const stB = folderStats.get(b.folderB);
        if (!stA || !stB) continue;
        const cxA = stA.cx !== undefined ? stA.cx : (stA.sumX / stA.count);
        const cyA = stA.cy !== undefined ? stA.cy : (stA.sumY / stA.count);
        const cxB = stB.cx !== undefined ? stB.cx : (stB.sumX / stB.count);
        const cyB = stB.cy !== undefined ? stB.cy : (stB.sumY / stB.count);
        if ((cxA < vLeft && cxB < vLeft) || (cxA > vRight && cxB > vRight) ||
            (cyA < vTop && cyB < vTop) || (cyA > vBottom && cyB > vBottom)) {
          continue;
        }
        const mx = (cxA + cxB) * 0.5 - (cyB - cyA) * 0.10;
        const my = (cyA + cyB) * 0.5 + (cxB - cxA) * 0.10;
        pArteries.moveTo(cxA, cyA);
        pArteries.quadraticCurveTo(mx, my, cxB, cyB);
        hasArteries = true;

        if (activePhotons.length < 30) {
          activePhotons.push({
            s: { x: cxA, y: cyA },
            t: { x: cxB, y: cyB },
            rel: b.submitsCount > 0 ? 'submits' : (b.importsCount > 0 ? 'imports' : 'calls'),
            isConn: state.selectedFolder === b.folderA || state.selectedFolder === b.folderB,
            idx: bi
          });
        }
      }
      if (hasArteries) {
        ctx.save();
        ctx.globalAlpha = (lodLevel === 1 ? 0.55 : (lodLevel === 2 ? 0.38 : 0.22)) * highlightDimFactor;
        ctx.strokeStyle = 'rgba(56, 189, 248, 0.78)';
        ctx.lineWidth = Math.min(6.0, Math.max(1.5, 2.4 / c.zoom));
        ctx.stroke(pArteries);
        ctx.restore();
      }
    }

    // 2. Build or Reuse Static Path2D Batch Cache with 6 Affiliation Filters
    const layersKey = state.activeLayers ? state.activeLayers.size : 7;
    const modulesKey = state.hiddenModules ? state.hiddenModules.size : 0;
    const relsKey = state.activeRelations ? state.activeRelations.size : 8;
    const filterBitsKey = state.filterBits !== undefined ? state.filterBits : 63;
    const xrayKey = (state.xrayMode && state.xrayTunnel) ? state.xrayTunnel.centerId : '0';
    const pipelineKey = state.dataPipelineMode ? '1' : '0';
    const cacheKey = `${lodLevel}|${edges.length}|${layersKey}|${modulesKey}|${relsKey}|${filterBitsKey}|${xrayKey}|${pipelineKey}`;

    let batch = (!isSpawning && state.physicsSleeping && state.cachedEdgePaths && state.cachedEdgePaths.key === cacheKey)
      ? state.cachedEdgePaths
      : null;

    if (!batch) {
      const pContains = new Path2D();
      const pCalls = new Path2D();
      const pInterModule = new Path2D();
      const pImports = new Path2D();
      const pSubmits = new Path2D();
      const pNetwork = new Path2D();
      const pRealtime = new Path2D();
      const pQueries = new Path2D();
      const pViolation = new Path2D();
      const pRenders = new Path2D();
      const pInherits = new Path2D();
      const pDocs = new Path2D();
      const pBinds = new Path2D();
      const pXrayUp = new Path2D();
      const pXrayDown = new Path2D();
      const samplePhotons = [];

      let hasContains = false, countCalls = 0, hasInterModule = false, hasImports = false;
      let hasSubmits = false, hasNetwork = false, hasRealtime = false, hasQueries = false, hasViolation = false;
      let hasRenders = false, hasInherits = false, hasDocs = false, hasBinds = false;
      let hasXrayUp = false, hasXrayDown = false;

      const edgeStride = (!state.physicsSleeping && edges.length > 5000) ? Math.ceil(edges.length / 4000) : 1;

      for (let i = 0; i < edges.length; i += edgeStride) {
        const e = edges[i];
        const s = e._s || (e._s = nodeMap.get(e.source));
        const t = e._t || (e._t = nodeMap.get(e.target));
        if (!s || !t) continue;
        if (!state.activeLayers.has(s.layer) || !state.activeLayers.has(t.layer)) continue;
        if (state.hiddenModules && (state.hiddenModules.has(s.folder || 'root') || state.hiddenModules.has(t.folder || 'root'))) continue;
        if (now < (s.birth_time || 0) || now < (t.birth_time || 0)) continue;

        const rawRel = e.relation || 'calls';
        const isDbQuery = rawRel === 'queries' || t.layer === 'database' || s.layer === 'database' || (t.folder && (t.folder === 'data-nodes' || t.folder.includes('db'))) || (s.folder && (s.folder === 'data-nodes' || s.folder.includes('db')));

        // 6 Affiliation Filters check:
        // Bit 1: PHYSICAL, Bit 2: NETWORK_RPC, Bit 4: REALTIME_WS, Bit 8: SEMANTIC, Bit 16: SECURITY_VIOLATES, Bit 32: DATABASE
        let filterBit = e.filter_bits;
        if (filterBit === undefined) {
          if (e.relation === 'security_violation' || e.relation === 'security_violates' || e.relation === 'violates' || e.relation === 'violation') filterBit = 16;
          else if (e.relation === 'realtime_ws' || e.relation === 'realtime' || e.relation === 'realtimews') filterBit = 4;
          else if (isDbQuery) filterBit = 32;
          else if (e.relation === 'submits' || e.relation === 'network') filterBit = 2;
          else if (e.relation === 'contains') filterBit = 1;
          else filterBit = 8;
        }

        if (state.filterBits !== undefined && (filterBit & state.filterBits) === 0) {
          continue;
        }

        const normRel = 
          (rawRel === 'realtime_ws' || rawRel === 'realtimews') ? 'realtime' :
          (rawRel === 'security_violation' || rawRel === 'security_violates' || rawRel === 'violation') ? 'violates' :
          rawRel;

        if (state.activeRelations) {
          const isAllowed = 
            state.activeRelations.has(rawRel) ||
            state.activeRelations.has(normRel) ||
            (isDbQuery && state.activeRelations.has('queries')) ||
            (normRel === 'realtime' && (state.activeRelations.has('realtime') || state.activeRelations.has('realtimews') || state.activeRelations.has('realtime_ws'))) ||
            (normRel === 'violates' && (state.activeRelations.has('violates') || state.activeRelations.has('violation') || state.activeRelations.has('security_violation')));
          if (!isAllowed) continue;
        }

        if (state.xrayMode && state.xrayTunnel) {
          const tunnel = state.xrayTunnel;
          if (tunnel.edges.has(i)) {
            const isUp = tunnel.upstream.has(e.source) || e.target === tunnel.centerId;
            if (isUp) {
              pXrayUp.moveTo(s.x, s.y); pXrayUp.lineTo(t.x, t.y); hasXrayUp = true;
              if (samplePhotons.length < 35) samplePhotons.push({ s, t, rel: 'xray_up', isConn: true, idx: i });
            } else {
              pXrayDown.moveTo(s.x, s.y); pXrayDown.lineTo(t.x, t.y); hasXrayDown = true;
              if (samplePhotons.length < 35) samplePhotons.push({ s, t, rel: 'xray_down', isConn: true, idx: i });
            }
          }
          continue;
        }

        const isCrossFolder = e._isCrossFolder !== undefined
          ? e._isCrossFolder
          : (s.folder && t.folder && s.folder !== t.folder);

        // Helper to add line with optional subtle curve if cross-folder
        const appendLine = (path) => {
          if (isCrossFolder && normRel !== 'contains' && normRel !== 'documents') {
            const mx = (s.x + t.x) * 0.5 - (t.y - s.y) * 0.08;
            const my = (s.y + t.y) * 0.5 + (t.x - s.x) * 0.08;
            path.moveTo(s.x, s.y);
            path.quadraticCurveTo(mx, my, t.x, t.y);
          } else {
            path.moveTo(s.x, s.y);
            path.lineTo(t.x, t.y);
          }
        };

        if (filterBit === 16 || normRel === 'violates') {
          appendLine(pViolation);
          hasViolation = true;
          if (samplePhotons.length < 30) samplePhotons.push({ s, t, rel: 'violation', isConn: true, idx: i });
        } else if (filterBit === 4 || normRel === 'realtime') {
          appendLine(pRealtime);
          hasRealtime = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'realtime', isConn: false, idx: i });
        } else if (rawRel === 'submits') {
          appendLine(pSubmits);
          hasSubmits = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'submits', isConn: false, idx: i });
        } else if (rawRel === 'network') {
          appendLine(pNetwork);
          hasNetwork = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'network', isConn: false, idx: i });
        } else if (isDbQuery) {
          appendLine(pQueries);
          hasQueries = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'queries', isConn: false, idx: i });
        } else if (rawRel === 'imports') {
          appendLine(pImports);
          hasImports = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'imports', isConn: false, idx: i });
        } else if (rawRel === 'renders') {
          appendLine(pRenders);
          hasRenders = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'renders', isConn: false, idx: i });
        } else if (rawRel === 'inherits') {
          appendLine(pInherits);
          hasInherits = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'inherits', isConn: false, idx: i });
        } else if (rawRel === 'binds') {
          appendLine(pBinds);
          hasBinds = true;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'binds', isConn: false, idx: i });
        } else if (rawRel === 'documents') {
          pDocs.moveTo(s.x, s.y);
          pDocs.lineTo(t.x, t.y);
          hasDocs = true;
        } else if (rawRel === 'contains') {
          pContains.moveTo(s.x, s.y);
          pContains.lineTo(t.x, t.y);
          hasContains = true;
        } else {
          appendLine(pCalls);
          countCalls++;
          if (samplePhotons.length < 25) samplePhotons.push({ s, t, rel: 'calls', isConn: false, idx: i });
        }
      }

      batch = {
        key: cacheKey,
        pContains, pCalls, pInterModule, pImports, pSubmits, pNetwork, pRealtime, pQueries, pViolation, pRenders, pInherits, pDocs, pBinds, pXrayUp, pXrayDown,
        hasContains, countCalls, hasInterModule, hasImports, hasSubmits, hasNetwork, hasRealtime, hasQueries, hasViolation, hasRenders, hasInherits, hasDocs, hasBinds, hasXrayUp, hasXrayDown,
        samplePhotons
      };
      if (state.physicsSleeping) {
        state.cachedEdgePaths = batch;
      }
    }

    const isPipelineActive = state.dataPipelineMode === true;
    const nonPipelineDim = isPipelineActive ? 0.06 : 1.0;

    // Merge sample photons from cached batch (or pipeline mode)
    if (state.showPhotons === true || isPipelineActive) {
      for (let pi = 0; pi < batch.samplePhotons.length && activePhotons.length < (isPipelineActive ? 75 : 45); pi++) {
        const ph = batch.samplePhotons[pi];
        if (isPipelineActive) {
          if (ph.rel === 'submits' || ph.rel === 'network' || ph.rel === 'realtime' || ph.rel === 'queries') {
            activePhotons.push(ph);
          }
        } else {
          activePhotons.push(ph);
        }
      }
    }

    const isMacro = c.zoom < 0.38;
    const microCallAlpha = (isMacro ? Math.max(0.35, Math.min(0.85, c.zoom * 2.2)) : (lodLevel === 2 ? 0.85 : 1.0)) * highlightDimFactor * nonPipelineDim;
    const highwayAlpha = (lodLevel === 1 ? 0.90 : (lodLevel === 2 ? 0.95 : 1.0)) * highlightDimFactor;
    const nonPipelineHighwayAlpha = highwayAlpha * nonPipelineDim;

    // Draw Batch 1: Contains (file -> symbols)
    if (batch.hasContains) {
      ctx.save();
      ctx.globalAlpha = Math.max(0.18, Math.min(0.55, c.zoom * 1.5)) * highlightDimFactor * nonPipelineDim;
      ctx.strokeStyle = 'rgba(203, 213, 225, 0.55)';
      ctx.lineWidth = Math.min(2.5, Math.max(0.7, 0.95 / c.zoom));
      if (!isUltraMassiveEdges) ctx.setLineDash([3 / c.zoom, 3 / c.zoom]);
      ctx.stroke(batch.pContains);
      ctx.restore();
    }

    // Draw Batch 2: Calls (function calls)
    if (batch.countCalls > 0) {
      ctx.save();
      ctx.globalAlpha = microCallAlpha;
      ctx.strokeStyle = 'rgba(56, 189, 248, 0.85)';
      ctx.lineWidth = Math.min(3.5, Math.max(0.9, 1.35 / c.zoom));
      ctx.stroke(batch.pCalls);
      ctx.restore();
    }

    // Draw Batch 2b: Inter-Module Synaptic Bridges (if populated)
    if (batch.hasInterModule) {
      ctx.save();
      ctx.globalAlpha = Math.min(1.0, nonPipelineHighwayAlpha * 1.1);
      ctx.strokeStyle = 'rgba(56, 189, 248, 0.92)';
      ctx.lineWidth = Math.min(4.5, Math.max(1.2, 1.85 / c.zoom));
      ctx.stroke(batch.pInterModule);
      ctx.restore();
    }

    // Draw Batch 3: Imports
    if (batch.hasImports) {
      ctx.save();
      ctx.globalAlpha = nonPipelineHighwayAlpha;
      ctx.strokeStyle = 'rgba(192, 132, 252, 0.95)';
      ctx.lineWidth = Math.min(4.8, Math.max(1.2, 1.75 / c.zoom));
      if (!isUltraMassiveEdges) ctx.setLineDash([7 / c.zoom, 4 / c.zoom]);
      ctx.stroke(batch.pImports);
      ctx.restore();
    }

    // Draw Batch 4a: Pipeline Step 1 — Submits (Vivid Coral-Orange #ff6b00)
    if (batch.hasSubmits) {
      ctx.save();
      ctx.globalAlpha = (isPipelineActive ? 1.0 : highwayAlpha) * highlightDimFactor;
      ctx.strokeStyle = '#ff6b00';
      ctx.lineWidth = Math.min(6.5, Math.max(1.5, (isPipelineActive ? 3.0 : 2.2) / c.zoom));
      if (isPipelineActive) {
        ctx.shadowColor = '#ff6b00';
        ctx.shadowBlur = 12;
      }
      ctx.stroke(batch.pSubmits);
      ctx.restore();
    }

    // Draw Batch 4b: Pipeline Step 2 — Network HTTP/REST (Electric Sky Blue #00c8ff)
    if (batch.hasNetwork) {
      ctx.save();
      ctx.globalAlpha = (isPipelineActive ? 1.0 : highwayAlpha) * highlightDimFactor;
      ctx.strokeStyle = '#00c8ff';
      ctx.lineWidth = Math.min(6.5, Math.max(1.5, (isPipelineActive ? 3.0 : 2.2) / c.zoom));
      if (isPipelineActive) {
        ctx.shadowColor = '#00c8ff';
        ctx.shadowBlur = 12;
      }
      ctx.stroke(batch.pNetwork);
      ctx.restore();
    }

    // Draw Batch 4c: Pipeline Step 3 — Realtime WS (Cyan glowing laser #00f0ff)
    if (batch.hasRealtime) {
      ctx.save();
      ctx.globalAlpha = (isPipelineActive ? 1.0 : highwayAlpha) * highlightDimFactor;
      ctx.strokeStyle = '#00f0ff';
      ctx.lineWidth = Math.min(7.0, Math.max(1.6, (isPipelineActive ? 3.4 : 2.4) / c.zoom));
      ctx.shadowColor = '#00f0ff';
      ctx.shadowBlur = isPipelineActive ? 16 : 8;
      ctx.stroke(batch.pRealtime);
      ctx.shadowBlur = 0;
      ctx.restore();
    }

    // Draw Batch 4d: Pipeline Step 4 — Database Queries & Persistence (Cyber Emerald #10b981)
    if (batch.hasQueries) {
      ctx.save();
      ctx.globalAlpha = (isPipelineActive ? 1.0 : highwayAlpha) * highlightDimFactor;
      ctx.strokeStyle = '#10b981';
      ctx.lineWidth = Math.min(7.0, Math.max(1.6, (isPipelineActive ? 3.4 : 2.4) / c.zoom));
      ctx.shadowColor = '#10b981';
      ctx.shadowBlur = isPipelineActive ? 16 : 8;
      ctx.stroke(batch.pQueries);
      ctx.shadowBlur = 0;
      ctx.restore();
    }

    // Draw Batch 4e: Security Violates (Pulsing / Blinking Red laser)
    if (batch.hasViolation) {
      ctx.save();
      const pulseAlpha = (Math.sin(now * 0.012) * 0.35 + 0.65) * highlightDimFactor;
      ctx.globalAlpha = pulseAlpha;
      ctx.strokeStyle = '#f43f5e';
      ctx.lineWidth = Math.min(6.5, Math.max(2.0, 3.2 / c.zoom));
      ctx.shadowColor = '#ef4444';
      ctx.shadowBlur = 14;
      ctx.setLineDash([8 / c.zoom, 4 / c.zoom]);
      ctx.lineDashOffset = -(now * 0.035) / c.zoom;
      ctx.stroke(batch.pViolation);
      ctx.setLineDash([]);
      ctx.shadowBlur = 0;
      ctx.restore();
    }

    // Draw Batch 5: Renders
    if (batch.hasRenders) {
      ctx.save();
      ctx.globalAlpha = nonPipelineHighwayAlpha;
      ctx.strokeStyle = '#ec4899';
      ctx.lineWidth = Math.min(4.5, Math.max(1.1, 1.6 / c.zoom));
      ctx.stroke(batch.pRenders);
      ctx.restore();
    }

    // Draw Batch 6: Inherits
    if (batch.hasInherits) {
      ctx.save();
      ctx.globalAlpha = nonPipelineHighwayAlpha;
      ctx.strokeStyle = '#f59e0b';
      ctx.lineWidth = Math.min(4.5, Math.max(1.1, 1.75 / c.zoom));
      ctx.stroke(batch.pInherits);
      ctx.restore();
    }

    // Draw Batch 7: Documents
    if (batch.hasDocs) {
      ctx.save();
      ctx.globalAlpha = nonPipelineHighwayAlpha * 0.75;
      ctx.strokeStyle = 'rgba(16, 185, 129, 0.85)';
      ctx.lineWidth = 1.1 / c.zoom;
      if (!isUltraMassiveEdges) ctx.setLineDash([3 / c.zoom, 3 / c.zoom]);
      ctx.stroke(batch.pDocs);
      ctx.restore();
    }

    // Draw Batch 8: Binds (Contract Interfaces)
    if (batch.hasBinds) {
      ctx.save();
      ctx.globalAlpha = nonPipelineHighwayAlpha;
      ctx.strokeStyle = '#818cf8';
      ctx.lineWidth = Math.min(4.5, Math.max(1.1, 1.8 / c.zoom));
      if (!isUltraMassiveEdges) ctx.setLineDash([5 / c.zoom, 3 / c.zoom]);
      ctx.stroke(batch.pBinds);
      ctx.restore();
    }

    // 3. Draw O(1) Highlighted Batch (Hovered / Selected Node Connections via _incidentEdges)
    const focusNode = state.hoveredNode || state.selectedNode;
    if (focusNode && focusNode._incidentEdges && focusNode._incidentEdges.length > 0) {
      const inc = focusNode._incidentEdges;
      const maxHighlight = Math.min(inc.length, 250);
      for (let j = 0; j < maxHighlight; j++) {
        const e = inc[j];
        const s = e._s;
        const t = e._t;
        if (!s || !t) continue;
        ctx.beginPath();
        const eRel = e.relation || 'calls';
        if (eRel === 'security_violation' || eRel === 'security_violates' || eRel === 'violates' || eRel === 'violation' || e.filter_bits === 16) {
          ctx.strokeStyle = '#f87171';
          ctx.lineWidth = 3.2 / c.zoom;
        } else if (eRel === 'realtime_ws' || eRel === 'realtime' || eRel === 'realtimews' || e.filter_bits === 4) {
          ctx.strokeStyle = '#00f0ff';
          ctx.lineWidth = 2.8 / c.zoom;
        } else if (eRel === 'submits') {
          ctx.strokeStyle = '#ff6b00';
          ctx.lineWidth = 2.8 / c.zoom;
        } else if (eRel === 'network') {
          ctx.strokeStyle = '#00c8ff';
          ctx.lineWidth = 2.8 / c.zoom;
        } else if (eRel === 'queries' || e.filter_bits === 32) {
          ctx.strokeStyle = '#10b981';
          ctx.lineWidth = 2.8 / c.zoom;
        } else if (eRel === 'imports') {
          ctx.strokeStyle = '#e9d5ff';
          ctx.lineWidth = 2.5 / c.zoom;
        } else if (eRel === 'renders') {
          ctx.strokeStyle = '#f472b6';
          ctx.lineWidth = 2.6 / c.zoom;
        } else if (eRel === 'inherits') {
          ctx.strokeStyle = '#fbbf24';
          ctx.lineWidth = 2.6 / c.zoom;
        } else if (eRel === 'binds') {
          ctx.strokeStyle = '#a5b4fc';
          ctx.lineWidth = 2.6 / c.zoom;
        } else if (eRel === 'documents') {
          ctx.strokeStyle = '#6ee7b7';
          ctx.lineWidth = 2.2 / c.zoom;
        } else if (eRel === 'contains') {
          ctx.strokeStyle = 'rgba(226, 232, 240, 0.85)';
          ctx.lineWidth = 2.0 / c.zoom;
        } else {
          ctx.strokeStyle = '#38bdf8';
          ctx.lineWidth = 2.6 / c.zoom;
        }
        ctx.moveTo(s.x, s.y);
        ctx.lineTo(t.x, t.y);
        ctx.stroke();
        if (state.showPhotons === true && activePhotons.length < 55) {
          activePhotons.push({ s, t, rel: eRel, isConn: true, idx: j });
        }
      }
    }

    // Draw X-Ray Batches
    if (batch.hasXrayUp) {
      ctx.save();
      ctx.strokeStyle = '#38bdf8';
      ctx.lineWidth = Math.max(2.4, 3.0 / c.zoom);
      ctx.stroke(batch.pXrayUp);
      ctx.restore();
    }

    if (batch.hasXrayDown) {
      ctx.save();
      ctx.strokeStyle = '#f59e0b';
      ctx.lineWidth = Math.max(2.4, 3.0 / c.zoom);
      ctx.stroke(batch.pXrayDown);
      ctx.restore();
    }

    ctx.setLineDash([]);

    // Draw Animated Laser Photons
    if ((state.showPhotons === true || isPipelineActive) && activePhotons.length > 0) {
      for (let j = 0; j < activePhotons.length; j++) {
        const ph = activePhotons[j];
        const pulseT = ((now * 0.0016 + ph.idx * 0.22) % 1.0);
        const px = ph.s.x + (ph.t.x - ph.s.x) * pulseT;
        const py = ph.s.y + (ph.t.y - ph.s.y) * pulseT;

        if (ph.rel === 'violation' || ph.rel === 'security_violation') ctx.fillStyle = '#ef4444';
        else if (ph.rel === 'realtime' || ph.rel === 'realtime_ws') ctx.fillStyle = '#00f0ff';
        else if (ph.rel === 'submits') ctx.fillStyle = '#ff6b00';
        else if (ph.rel === 'network') ctx.fillStyle = '#00c8ff';
        else if (ph.rel === 'queries') ctx.fillStyle = '#10b981';
        else if (ph.rel === 'xray_up') ctx.fillStyle = '#38bdf8';
        else if (ph.rel === 'xray_down') ctx.fillStyle = '#fbbf24';
        else if (ph.rel === 'documents') ctx.fillStyle = '#10b981';
        else if (ph.rel === 'imports') ctx.fillStyle = '#c084fc';
        else ctx.fillStyle = '#38bdf8';

        const isPipePh = ph.rel === 'submits' || ph.rel === 'network' || ph.rel === 'realtime' || ph.rel === 'queries';
        const pRadius = (isPipePh && isPipelineActive) ? (3.5 / c.zoom) : ((ph.rel === 'xray_up' || ph.rel === 'xray_down' || ph.rel === 'violation') ? 3.8 / c.zoom : ((ph.isConn ? 3.2 : 2.0) / c.zoom));
        ctx.beginPath();
        ctx.arc(px, py, pRadius, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    // 2b. Tainted Data Laser Beams & Floating CWE Data Badges
    const violationEdges = state.violationEdges || edges.filter(e => e.relation === 'violates' || e.filter_bits === 16 || e.cve_id || (e.relation && e.relation.includes('violat')));
    if (violationEdges.length > 0 && (((state.filterBits !== undefined ? state.filterBits : 31) & 16) !== 0 || c.zoom >= 0.35 || state.selectedAnomalyNodeId)) {
      ctx.save();
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      const cveScreenFontSize = 10;
      const badgeFontSize = cveScreenFontSize / c.zoom;
      ctx.font = `800 ${badgeFontSize}px ${FONT_MONO}`;

      violationEdges.forEach((e, vIdx) => {
        const s = e._s || nodeMap.get(e.source);
        const t = e._t || nodeMap.get(e.target);
        if (!s || !t) return;

        const mx = (s.x + t.x) / 2;
        const my = (s.y + t.y) / 2;

        if (mx < vLeft || mx > vRight || my < vTop || my > vBottom) return;

        const cveId = e.cve_id || 'CWE-SAST';
        const ruleId = e.rule_id || '';
        const taintedData = e.transmitted_data || 'payload';
        const taintedType = e.transmitted_type || 'Untyped';
        let labelText = `⚠️ [${cveId}] ${taintedData}: ${taintedType} ➔ ${t.name || 'target'}`;
        if (ruleId === 'DEFI-ASYMMETRIC-ROUNDING' || cveId.includes('ASYMMETRIC_ROUNDING_RISK')) {
          labelText = `☣️ [CWE-682] ASYMMETRIC_ROUNDING_RISK ➔ ${t.name || 'target'}`;
        } else if (ruleId === 'DEFI-STALE-CACHE-AFTER-CALL' || cveId.includes('STALE_CACHE_TOCTOU')) {
          labelText = `☣️ [CWE-362] STALE_CACHE_TOCTOU ➔ ${t.name || 'target'}`;
        } else if (ruleId === 'EVC-HOOK-BYPASS' || cveId.includes('EVC_HOOK_BYPASS_RISK')) {
          labelText = `☣️ [CWE-841] EVC_HOOK_BYPASS_RISK ➔ ${t.name || 'target'}`;
        } else if (ruleId === 'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER' || cveId.includes('ORACLE_MIN_MAX_CIRCUIT_BREAKER')) {
          labelText = `☣️ [LOGIC-CRITICAL] ORACLE_MIN_MAX_CIRCUIT_BREAKER ➔ ${t.name || 'target'}`;
        } else if (ruleId === 'DEFI-ORACLE-L2-SEQUENCER-RACE' || cveId.includes('L2_SEQUENCER_ORACLE_RACE')) {
          labelText = `☣️ [LOGIC-CRITICAL] L2_SEQUENCER_ORACLE_RACE ➔ ${t.name || 'target'}`;
        } else if (ruleId === 'DEFI-TWAP-SHORT-WINDOW' || cveId.includes('TWAP_SHORT_WINDOW')) {
          labelText = `☣️ [LOGIC-CRITICAL] TWAP_SHORT_WINDOW ➔ ${t.name || 'target'}`;
        }

        const tw = ctx.measureText(labelText).width;
        const padX = 7 / c.zoom;
        const padY = 4 / c.zoom;
        const badgeH = badgeFontSize + padY * 2;
        const badgeW = tw + padX * 2;

        const pulseVal = Math.sin(now * 0.008 + vIdx) * 0.2 + 0.8;
        ctx.fillStyle = `rgba(127, 29, 29, ${0.94 * pulseVal})`;
        ctx.strokeStyle = '#f87171';
        ctx.lineWidth = 1.3 / c.zoom;

        ctx.beginPath();
        ctx.roundRect(mx - badgeW / 2, my - badgeH / 2, badgeW, badgeH, 4 / c.zoom);
        ctx.fill();
        ctx.stroke();

        ctx.fillStyle = '#ffffff';
        ctx.fillText(labelText, mx, my);
      });
      ctx.restore();
    }

    // 3. Search Beacon Concentric Rings Illumination
    if (state.beaconNode) {
      const bn = state.beaconNode;
      const dt = (now - state.beaconStart) * 0.001;
      for (let k = 0; k < 3; k++) {
        const r = ((dt * 1.8 + k * 0.5) % 1.5) * 65 / c.zoom;
        const op = Math.max(0, 1.0 - (r / (65 / c.zoom)));
        ctx.beginPath();
        ctx.strokeStyle = `rgba(245, 158, 11, ${op})`;
        ctx.lineWidth = 2.2 / c.zoom;
        ctx.arc(bn.x, bn.y, r, 0, Math.PI * 2);
        ctx.stroke();
      }
    }

    // 3b. AI Hologram Telemetry Quantum Scan Rings & HUD Marker
    if (state.aiTelemetry && state.aiTelemetry.target) {
      const tLower = state.aiTelemetry.target.toLowerCase();
      const aiNode = nodes.find(n => 
        n.name.toLowerCase() === tLower || 
        n.id.toLowerCase() === tLower || 
        (n.file_path && n.file_path.toLowerCase().includes(tLower))
      );

      if (aiNode) {
        const dt = (now * 0.0012) % 2.0;
        for (let k = 0; k < 3; k++) {
          const r = (((dt * 1.5 + k * 0.6) % 2.0) * 45 + 16) / c.zoom;
          const op = Math.max(0, 1.0 - (r / (65 / c.zoom)));
          ctx.beginPath();
          ctx.strokeStyle = k % 2 === 0 ? `rgba(192, 132, 252, ${op * 0.95})` : `rgba(56, 189, 248, ${op * 0.95})`;
          ctx.lineWidth = 2.4 / c.zoom;
          ctx.setLineDash([4 / c.zoom, 4 / c.zoom]);
          ctx.arc(aiNode.x, aiNode.y, r, 0, Math.PI * 2);
          ctx.stroke();
          ctx.setLineDash([]);
        }

        // Rotating tactical crosshair
        const angle = now * 0.002;
        const chR = 34 / c.zoom;
        ctx.beginPath();
        ctx.strokeStyle = '#c084fc';
        ctx.lineWidth = 1.8 / c.zoom;
        ctx.arc(aiNode.x, aiNode.y, chR, angle, angle + Math.PI * 0.5);
        ctx.arc(aiNode.x, aiNode.y, chR, angle + Math.PI, angle + Math.PI * 1.5);
        ctx.stroke();

        // AI Badge floating above node
        const badgeY = aiNode.y - (aiNode.is_file ? 24 : 20) / c.zoom;
        ctx.font = `800 ${11 / c.zoom}px ${FONT_MONO}`;
        const aiText = `🤖 IA: ${state.aiTelemetry.action || 'Exploration'}`;
        const atw = ctx.measureText(aiText).width;
        const apad = 5 / c.zoom;
        ctx.fillStyle = 'rgba(88, 28, 135, 0.95)';
        ctx.strokeStyle = '#c084fc';
        ctx.lineWidth = 1.2 / c.zoom;
        ctx.beginPath();
        ctx.roundRect(aiNode.x - atw / 2 - apad, badgeY - 6 / c.zoom - apad, atw + apad * 2, 13 / c.zoom + apad * 2, 4 / c.zoom);
        ctx.fill();
        ctx.stroke();
        ctx.fillStyle = '#f5d0fe';
        ctx.fillText(aiText, aiNode.x, badgeY);
      }
    }

    // 3c. AI Navigation Trail ("Fil d'Ariane IA")
    if (state.aiTrail && state.aiTrail.length > 1) {
      ctx.save();
      ctx.beginPath();
      for (let tIdx = 0; tIdx < state.aiTrail.length; tIdx++) {
        const pt = state.aiTrail[tIdx];
        if (tIdx === 0) ctx.moveTo(pt.x, pt.y);
        else ctx.lineTo(pt.x, pt.y);
      }
      ctx.strokeStyle = '#c084fc';
      ctx.lineWidth = Math.max(1.8, 2.6 / c.zoom);
      ctx.setLineDash([8 / c.zoom, 5 / c.zoom]);
      ctx.lineDashOffset = -(now * 0.02) / c.zoom;
      ctx.shadowColor = '#c084fc';
      ctx.shadowBlur = 10;
      ctx.stroke();
      ctx.shadowBlur = 0;
      ctx.setLineDash([]);

      for (let tIdx = 0; tIdx < state.aiTrail.length; tIdx++) {
        const pt = state.aiTrail[tIdx];
        const isCurrent = tIdx === state.aiTrail.length - 1;
        const r = (isCurrent ? 5.5 : 3.8) / c.zoom;

        ctx.beginPath();
        ctx.fillStyle = isCurrent ? '#38bdf8' : 'rgba(192, 132, 252, 0.75)';
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 1.2 / c.zoom;
        ctx.arc(pt.x, pt.y, r, 0, Math.PI * 2);
        ctx.fill();
        ctx.stroke();

        if (c.zoom >= 0.45) {
          ctx.font = `800 ${8 / c.zoom}px ${FONT_MONO}`;
          ctx.fillStyle = '#f8fafc';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(`${tIdx + 1}`, pt.x, pt.y - (r + 7 / c.zoom));
        }
      }
      ctx.restore();
    }

    // 3d. AI Supernova Creation Shockwaves ("Naissance Stellaire")
    if (state.aiSupernova) {
      const sn = state.aiSupernova;
      const elapsed = now - sn.startTime;
      if (elapsed > sn.duration) {
        state.aiSupernova = null;
      } else {
        const progress = elapsed / sn.duration;
        const maxRadius = 180 / c.zoom;

        ctx.save();
        for (let sw = 0; sw < 4; sw++) {
          const swProgress = (progress + sw * 0.22) % 1.0;
          const r = (swProgress * maxRadius) + (14 / c.zoom);
          const alpha = Math.max(0, (1.0 - swProgress) * 0.9);

          ctx.beginPath();
          ctx.strokeStyle = sw % 2 === 0 ? `rgba(16, 185, 129, ${alpha})` : `rgba(245, 158, 11, ${alpha})`;
          ctx.lineWidth = Math.max(1.5, (4.0 * (1.0 - swProgress)) / c.zoom);
          ctx.arc(sn.x, sn.y, r, 0, Math.PI * 2);
          ctx.stroke();
        }

        const rayCount = 10;
        const rayAngle = now * 0.0015;
        const rayLen = (60 + Math.sin(now * 0.01) * 20) / c.zoom;
        ctx.strokeStyle = `rgba(56, 189, 248, ${Math.max(0, (1.0 - progress) * 0.85)})`;
        ctx.lineWidth = 2.0 / c.zoom;
        for (let ray = 0; ray < rayCount; ray++) {
          const a = rayAngle + (ray * (Math.PI * 2 / rayCount));
          ctx.beginPath();
          ctx.moveTo(sn.x, sn.y);
          ctx.lineTo(sn.x + Math.cos(a) * rayLen, sn.y + Math.sin(a) * rayLen);
          ctx.stroke();
        }

        const bannerY = sn.y - (45 / c.zoom);
        ctx.font = `800 ${12 / c.zoom}px ${FONT_UI}`;
        const snText = `✨ NAISSANCE STELLAIRE : ${sn.name}`;
        const snTw = ctx.measureText(snText).width;
        const snPad = 8 / c.zoom;

        ctx.fillStyle = 'rgba(6, 78, 59, 0.95)';
        ctx.strokeStyle = '#10b981';
        ctx.lineWidth = 1.6 / c.zoom;
        ctx.beginPath();
        ctx.roundRect(sn.x - snTw / 2 - snPad, bannerY - 8 / c.zoom - snPad, snTw + snPad * 2, 16 / c.zoom + snPad * 2, 6 / c.zoom);
        ctx.fill();
        ctx.stroke();

        ctx.fillStyle = '#6ee7b7';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(snText, sn.x, bannerY);
        ctx.restore();
      }
    }

    // 3e. CVE / Security Violation Red Alert Pulsars & Notched Beacon Rings
    if (((state.filterBits !== undefined ? state.filterBits : 31) & 16) !== 0 || state.selectedAnomalyNodeId) {
      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i];
        if (!n.has_security_violation) continue;
        if (n.x < vLeft || n.x > vRight || n.y < vTop || n.y > vBottom) continue;

        const alertR = (n.is_file ? 24 : 16) / c.zoom;
        const pulse = 1.0 + 0.18 * Math.sin(now * 0.01 + i);

        ctx.save();
        // Outer notched rotating warning ring
        const rot = now * 0.002 + i * 0.5;
        ctx.beginPath();
        ctx.strokeStyle = 'rgba(239, 68, 68, 0.9)';
        ctx.lineWidth = 2.0 / c.zoom;
        ctx.setLineDash([6 / c.zoom, 4 / c.zoom]);
        ctx.lineDashOffset = (now * 0.02) / c.zoom;
        ctx.arc(n.x, n.y, alertR * pulse, 0, Math.PI * 2);
        ctx.stroke();

        // Rotating caution brackets
        ctx.setLineDash([]);
        ctx.beginPath();
        ctx.strokeStyle = '#f43f5e';
        ctx.lineWidth = 2.2 / c.zoom;
        ctx.arc(n.x, n.y, (alertR + 6 / c.zoom) * pulse, rot, rot + Math.PI * 0.4);
        ctx.arc(n.x, n.y, (alertR + 6 / c.zoom) * pulse, rot + Math.PI, rot + Math.PI * 1.4);
        ctx.stroke();

        // Warning mini-glyph over the bubble
        if (c.zoom >= 0.35) {
          ctx.font = `900 ${10 / c.zoom}px ${FONT_MONO}`;
          ctx.fillStyle = '#ef4444';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText('🚨', n.x, n.y - alertR - 6 / c.zoom);
        }
        ctx.restore();
      }
    }

    // 4. Crisp Node Labels & Selection Target Ring
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const nodeScreenFontSize = Math.max(10, Math.min(13, c.zoom >= 1.0 ? 12 : 11));
    const fontSize = nodeScreenFontSize / c.zoom;
    const hideDense = state.settings?.hideBubbleLabelsOver20 !== false;
    const bubbleThreshold = state.settings?.clusterLabelThreshold ?? 20;
    const maxLabelsPerFrame = lodLevel === 3 ? 220 : (lodLevel === 2 ? 110 : 40);
    let drawnLabelsCount = 0;

    for (let i = 0; i < nodes.length; i++) {
      const n = nodes[i];
      if (!state.activeLayers.has(n.layer)) continue;

      if (n.x < vLeft || n.x > vRight || n.y < vTop || n.y > vBottom) continue;

      if (state.xrayMode && state.xrayTunnel) {
        if (!state.xrayTunnel.all.has(n.id)) {
          continue;
        }
      }

      if (state.deadCodeMode) {
        if (!state.deadCodeSet.has(n.id)) {
          continue;
        }
      }

      // Progressive spawn: don't show label or rings until node has emerged
      if (now < (n.birth_time || 0) + 120) continue;

      const isXrayCenter = state.xrayMode && state.xrayTunnel && n.id === state.xrayTunnel.centerId;
      const isXrayUp = state.xrayMode && state.xrayTunnel && state.xrayTunnel.upstream.has(n.id);
      const isXrayDown = state.xrayMode && state.xrayTunnel && state.xrayTunnel.downstream.has(n.id);

      const isSelected = selId === n.id;
      const isHovered = hovId === n.id;
      const isBeacon = !!(state.beaconNode && state.beaconNode.id === n.id);
      const isAiTarget = !!(state.aiTelemetryNode && state.aiTelemetryNode.id === n.id);
      const isPriorityLabel = isSelected || isHovered || isBeacon || isAiTarget || isXrayCenter;

      if (!isPriorityLabel && drawnLabelsCount >= maxLabelsPerFrame) continue;

      const folderCount = (folderStats && folderStats.get(n.folder || 'root')?.count) || 0;
      const isDenseBubble = hideDense && (folderCount > bubbleThreshold);
      const allowFileLabel = (lodLevel >= 2 || nodes.length <= 45) && 
                             (!isDenseBubble || isPriorityLabel);

      const isHmHotspot = !!(state.heatmapMode && state.heatmapScores && state.heatmapScores.get && ((state.heatmapScores.get(n.id)?.score || 0) > 0.45));
      const shouldShowLabel = state.deadCodeMode 
        || isHmHotspot 
        || isPriorityLabel
        || (state.xrayMode && state.xrayTunnel) 
        || (allowFileLabel && n.is_file) 
        || (lodLevel === 3 && !isDenseBubble && (n.is_class || n.stereotype === '«Form»' || n.is_form));

      if (isXrayCenter) {
        ctx.beginPath();
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 2.4 / c.zoom;
        ctx.setLineDash([5 / c.zoom, 3 / c.zoom]);
        ctx.arc(n.x, n.y, 25 / c.zoom, 0, Math.PI * 2);
        ctx.stroke();
        ctx.setLineDash([]);
      } else if (isSelected) {
        ctx.beginPath();
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 1.8 / c.zoom;
        ctx.setLineDash([4 / c.zoom, 3 / c.zoom]);
        ctx.arc(n.x, n.y, 22 / c.zoom, 0, Math.PI * 2);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      if (shouldShowLabel) {
        drawnLabelsCount++;
        const maxLen = state.settings?.maxLabelLength || 18;
        const displayName = truncateNodeName(n.name, maxLen);
        let text = displayName;
        if (state.deadCodeMode) {
          text = `💀 [Orphelin] ${displayName}`;
        } else if (state.heatmapMode) {
          const hm = state.heatmapScores?.get(n.id);
          const pct = hm ? Math.round(hm.score * 100) : 0;
          text = `🌡️ [${pct}%] ${displayName}`;
        } else if (state.xrayMode && state.xrayTunnel) {
          if (isXrayCenter) text = `⭐ ${displayName} (Focus)`;
          else if (isXrayUp) text = `▲ Amont: ${displayName}`;
          else if (isXrayDown) text = `▼ Aval: ${displayName}`;
        } else if (lodLevel === 3) {
          if (n.stereotype === '«Form»' || n.is_form) text = `📝 ${displayName}`;
          else if (n.stereotype === '«Endpoint»' || (n.id && n.id.startsWith('api::'))) text = `🌐 ${displayName}`;
          else if (n.is_file) text = `📄 ${displayName}`;
          else if (n.is_class) text = `🏛️ ${displayName}`;
          else if (n.layer === 'docs' || n.symbol_type === 'document') text = `📖 ${displayName}`;
          else if (n.symbol_type === 'function') text = `⚡ ${displayName}`;
        } else if (n.is_file) {
          text = `📄 ${displayName}`;
        }

        ctx.font = `600 ${fontSize}px ${FONT_UI}`;
        if (n._cachedText !== text) {
          n._cachedText = text;
          n._cachedBaseW = ctx.measureText(text).width / fontSize;
        }
        const tw = (n._cachedBaseW || 6) * fontSize;
        const padX = 6 / c.zoom, padY = 3 / c.zoom;
        const labelY = n.y + (n.is_file ? 14 : 12) / c.zoom + fontSize / 2;

        if (state.deadCodeMode) {
          ctx.fillStyle = 'rgba(153, 27, 27, 0.96)';
          ctx.strokeStyle = '#ef4444';
        } else if (state.heatmapMode) {
          const hm = state.heatmapScores?.get(n.id);
          const score = hm ? hm.score : 0;
          if (score > 0.65) {
            ctx.fillStyle = 'rgba(153, 27, 27, 0.96)';
            ctx.strokeStyle = '#ef4444';
          } else if (score > 0.35) {
            ctx.fillStyle = 'rgba(180, 83, 9, 0.96)';
            ctx.strokeStyle = '#f59e0b';
          } else {
            ctx.fillStyle = 'rgba(6, 95, 70, 0.96)';
            ctx.strokeStyle = '#10b981';
          }
        } else if (state.xrayMode && state.xrayTunnel) {
          if (isXrayCenter) {
            ctx.fillStyle = 'rgba(14, 116, 144, 0.98)';
            ctx.strokeStyle = '#ffffff';
          } else if (isXrayUp) {
            ctx.fillStyle = 'rgba(3, 105, 161, 0.94)';
            ctx.strokeStyle = '#38bdf8';
          } else {
            ctx.fillStyle = 'rgba(180, 83, 9, 0.94)';
            ctx.strokeStyle = '#f59e0b';
          }
        } else {
          ctx.fillStyle = isSelected ? 'rgba(14, 116, 144, 0.92)' : (isBeacon ? 'rgba(180, 83, 9, 0.95)' : 'rgba(2, 6, 23, 0.82)');
          ctx.strokeStyle = 'transparent';
        }

        ctx.beginPath();
        ctx.roundRect(n.x - tw / 2 - padX, labelY - fontSize / 2 - padY, tw + padX * 2, fontSize + padY * 2, 4 / c.zoom);
        ctx.fill();
        if (state.deadCodeMode || state.heatmapMode || (state.xrayMode && state.xrayTunnel)) {
          ctx.lineWidth = 1.0 / c.zoom;
          ctx.stroke();
        }

        ctx.fillStyle = (state.xrayMode && state.xrayTunnel) || isSelected || isBeacon ? '#ffffff' : (n.is_file ? '#e2e8f0' : '#cbd5e1');
        ctx.fillText(text, n.x, labelY);
      }
    }

    // 5. Dedicated Foreground Pass for High-Visibility Folder Cloud Badges
    drawFolderBadges(ctx, pendingFolderBadges, c, lodLevel);

    ctx.restore();
    renderRadar();
  } catch (renderErr) {
    console.error('AetherScope render frame error:', renderErr);
  }
}

// --- Orbital Radar Minimap Renderer ---
const radarCanvas = document.getElementById('radar-canvas');
const rCtx = radarCanvas?.getContext('2d');
let radarFrameCount = 0;

function renderRadar(force = false) {
  if (!rCtx || !radarCanvas) return;
  const nodes = state.data.nodes || [];
  if (nodes.length === 0) return;

  radarFrameCount++;
  if (!force && nodes.length > 600 && (radarFrameCount % 6 !== 0)) {
    const zEl = document.getElementById('radar-zoom-level');
    if (zEl) zEl.innerText = `${state.camera.zoom.toFixed(2)}x`;
    return;
  }

  const w = radarCanvas.width, h = radarCanvas.height;
  rCtx.clearRect(0, 0, w, h);

  let minX = 0, maxX = 0, minY = 0, maxY = 0;
  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    if (n.x < minX) minX = n.x; if (n.x > maxX) maxX = n.x;
    if (n.y < minY) minY = n.y; if (n.y > maxY) maxY = n.y;
  }
  const span = Math.max(maxX - minX, maxY - minY, 800);
  const scale = (w * 0.74) / span;
  const gCenterX = (minX + maxX) / 2;
  const gCenterY = (minY + maxY) / 2;
  const cx = w / 2, cy = h / 2;

  // Faint grid
  rCtx.strokeStyle = 'rgba(56, 189, 248, 0.15)';
  rCtx.lineWidth = 0.8;
  rCtx.beginPath();
  rCtx.arc(cx, cy, 35, 0, Math.PI * 2);
  rCtx.arc(cx, cy, 65, 0, Math.PI * 2);
  rCtx.moveTo(cx, 0); rCtx.lineTo(cx, h);
  rCtx.moveTo(0, cy); rCtx.lineTo(w, cy);
  rCtx.stroke();

  // Nodes
  const isMassive = nodes.length > 1200;
  const step = isMassive ? 2 : 1;
  for (let i = 0; i < nodes.length; i += step) {
    const n = nodes[i];
    if (!state.activeLayers.has(n.layer) || (state.hiddenModules && state.hiddenModules.has(n.folder || 'root'))) continue;
    const rx = cx + (n.x - gCenterX) * scale;
    const ry = cy + (n.y - gCenterY) * scale;
    rCtx.fillStyle = n.id === (state.selectedNode && state.selectedNode.id) ? '#38bdf8' : 'rgba(148, 163, 184, 0.6)';
    rCtx.fillRect(rx - 1, ry - 1, 2, 2);
  }

  // Camera viewport box
  const cam = state.camera;
  const vw = Math.min(w * 2.5, (window.innerWidth / cam.zoom) * scale);
  const vh = Math.min(h * 2.5, (window.innerHeight / cam.zoom) * scale);
  const vx = cx + (cam.x - gCenterX) * scale - vw / 2;
  const vy = cy + (cam.y - gCenterY) * scale - vh / 2;

  rCtx.strokeStyle = '#38bdf8';
  rCtx.lineWidth = 1.2;
  rCtx.strokeRect(vx, vy, vw, vh);

  const zEl = document.getElementById('radar-zoom-level');
  if (zEl) zEl.innerText = `${cam.zoom.toFixed(2)}x`;
}

radarCanvas?.addEventListener('click', (e) => {
  const rect = radarCanvas.getBoundingClientRect();
  const clickX = e.clientX - rect.left - radarCanvas.width / 2;
  const clickY = e.clientY - rect.top - radarCanvas.height / 2;
  const nodes = state.data.nodes || [];
  let minX = 0, maxX = 0, minY = 0, maxY = 0;
  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    if (n.x < minX) minX = n.x; if (n.x > maxX) maxX = n.x;
    if (n.y < minY) minY = n.y; if (n.y > maxY) maxY = n.y;
  }
  const span = Math.max(maxX - minX, maxY - minY, 800);
  const scale = (radarCanvas.width * 0.74) / span;
  const gCenterX = (minX + maxX) / 2;
  const gCenterY = (minY + maxY) / 2;
  state.camera.targetX = gCenterX + clickX / scale;
  state.camera.targetY = gCenterY + clickY / scale;
  AudioSys.playTone(440, 'sine', 0.15, 0.03);
});

// Screen to World Converter
function screenToWorld(sx, sy) {
  const c = state.camera;
  const wx = (sx - window.innerWidth / 2) / c.zoom + c.x;
  const wy = (sy - window.innerHeight / 2) / c.zoom + c.y;
  return { x: wx, y: wy };
}

// Find node under mouse
function findNodeAt(sx, sy) {
  const w = screenToWorld(sx, sy);
  const nodes = state.data.nodes || [];
  let best = null, bestDist = 22 / state.camera.zoom;
  const wx = w.x, wy = w.y;
  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    const dx = Math.abs(n.x - wx);
    if (dx >= bestDist) continue;
    const dy = Math.abs(n.y - wy);
    if (dy >= bestDist) continue;
    if (!state.activeLayers.has(n.layer) || (state.hiddenModules && state.hiddenModules.has(n.folder || 'root'))) continue;
    const d = Math.hypot(dx, dy);
    if (d < bestDist) {
      best = n;
      bestDist = d;
    }
  }
  return best;
}

// Find folder cluster under mouse in LoD 1, 2, 3
function findFolderAt(sx, sy) {
  if (!state.folderClusters) return null;
  const w = screenToWorld(sx, sy);
  const c = state.camera;
  let best = null, bestDist = Infinity;
  state.folderClusters.forEach((st, folder) => {
    if (state.hiddenModules && state.hiddenModules.has(folder)) return;
    const cx = st.cx !== undefined ? st.cx : (st.sumX / st.count);
    const cy = st.cy !== undefined ? st.cy : (st.sumY / st.count);
    const radius = st.computedRadius || Math.max(80, Math.sqrt(st.count) * 45);
    const d = Math.hypot(w.x - cx, w.y - cy);
    
    // 1. Inside membrane circle
    if (d <= radius && d < bestDist) {
      best = { folder, cx, cy, count: st.count, nodes: st.nodes, isBadge: false };
      bestDist = d;
    } else if (state.showFolderBadges === true || state.selectedFolder === folder || state.hoveredFolder === folder || (state.hoveredNode && (state.hoveredNode.folder || 'root') === folder)) {
      // 2. Inside attached top badge area (compact 12-14px) when visible
      const zoomFactor = Math.max(0.04, c ? c.zoom : 1);
      const screenFontSize = zoomFactor < 0.4 ? 14 : (zoomFactor < 0.9 ? 13 : 12);
      const badgeH = (screenFontSize + 8) / zoomFactor;
      const badgeY = cy - radius - badgeH * 0.5 - 10 / zoomFactor;
      const badgeBoxHalfW = (screenFontSize * 5.0) / zoomFactor;
      const badgeBoxHalfH = badgeH * 0.6;
      if (Math.abs(w.x - cx) <= badgeBoxHalfW && Math.abs(w.y - badgeY) <= badgeBoxHalfH && d < bestDist) {
        best = { folder, cx, cy, count: st.count, nodes: st.nodes, isBadge: true };
        bestDist = d;
      }
    }
  });
  return best;
}

// Multi-Target 3D Dragging & Plateau Navigation
var dragMode = null; // 'node' | 'folder' | 'plateau'
var draggedNode = null;
var dragNodeStart = { x: 0, y: 0 };
var draggedFolder = null;
var dragFolderNodes = null;
var dragStart = { x: 0, y: 0 }, mouseDownPos = { x: 0, y: 0 }, dragMouseStart = { x: 0, y: 0 };

window.render = render;
window.renderRadar = renderRadar;
window.screenToWorld = screenToWorld;
window.findNodeAt = findNodeAt;
window.findFolderAt = findFolderAt;
