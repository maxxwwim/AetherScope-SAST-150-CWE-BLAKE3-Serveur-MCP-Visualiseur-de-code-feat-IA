// =========================================================================
// AETHERSCOPE V2 — VISION MODES & AFFILIATION FILTERS (ui_vision_modes.js)
// X-Ray Focus, Dead Code Scanner, Heatmap, 5 Filter Toggles & HD Snapshot
// =========================================================================

// --- Mode X-Ray Focus Tunnel ---
function computeXrayTunnel(node) {
  if (!node) {
    state.xrayTunnel = null;
    return;
  }
  const nodeMap = state.nodeMap || new Map((state.data.nodes || []).map(n => [n.id, n]));
  const upstream = new Set();
  const downstream = new Set();
  const tunnelEdges = new Set();

  const upQueue = [node.id];
  const upVisited = new Set([node.id]);
  while (upQueue.length > 0) {
    const curr = upQueue.shift();
    const currNode = nodeMap.get(curr);
    const inc = (currNode && currNode._incidentEdges) || [];
    for (let i = 0; i < inc.length; i++) {
      const e = inc[i];
      if (e.target === curr) {
        if (e._idx !== undefined) tunnelEdges.add(e._idx);
        if (!upVisited.has(e.source)) {
          upVisited.add(e.source);
          upstream.add(e.source);
          upQueue.push(e.source);
        }
      }
    }
  }

  const downQueue = [node.id];
  const downVisited = new Set([node.id]);
  while (downQueue.length > 0) {
    const curr = downQueue.shift();
    const currNode = nodeMap.get(curr);
    const inc = (currNode && currNode._incidentEdges) || [];
    for (let i = 0; i < inc.length; i++) {
      const e = inc[i];
      if (e.source === curr) {
        if (e._idx !== undefined) tunnelEdges.add(e._idx);
        if (!downVisited.has(e.target)) {
          downVisited.add(e.target);
          downstream.add(e.target);
          downQueue.push(e.target);
        }
      }
    }
  }

  const all = new Set([node.id, ...upstream, ...downstream]);

  state.xrayTunnel = {
    centerId: node.id,
    centerNode: node,
    upstream,
    downstream,
    all,
    edges: tunnelEdges
  };
}

function updateXrayHud() {
  const hud = document.getElementById('xray-hud');
  const btn = document.getElementById('btn-xray');
  if (!hud) return;

  if (state.xrayMode && state.xrayTunnel) {
    hud.classList.remove('hidden');
    if (btn) btn.classList.add('active');
    const nodeNameEl = document.getElementById('xray-node-name');
    if (nodeNameEl) nodeNameEl.innerText = state.xrayTunnel.centerNode.name;
    const upEl = document.getElementById('xray-stat-up');
    if (upEl) upEl.innerText = `▲ ${state.xrayTunnel.upstream.size} Appelant(s)`;
    const downEl = document.getElementById('xray-stat-down');
    if (downEl) downEl.innerText = `▼ ${state.xrayTunnel.downstream.size} Dépendance(s)`;
    const totEl = document.getElementById('xray-stat-total');
    if (totEl) totEl.innerText = `⚡ ${state.xrayTunnel.edges.size} Flux`;
  } else {
    hud.classList.add('hidden');
    if (btn) btn.classList.remove('active');
  }
}

function toggleXrayMode() {
  if (!state.xrayMode) {
    if (!state.selectedNode) {
      showToast('⚠️ Cliquez d\'abord sur une bulle pour entrer dans le Tunnel X-Ray !');
      return;
    }
    if (state.deadCodeMode) toggleDeadCodeMode();
    if (state.heatmapMode) toggleHeatmapMode();
    state.xrayMode = true;
    computeXrayTunnel(state.selectedNode);
    updateXrayHud();
    AudioSys.xraySweep();
    showToast(`🔦 Tunnel X-Ray Focus activé : ${state.selectedNode.name}`);
  } else {
    state.xrayMode = false;
    state.xrayTunnel = null;
    updateXrayHud();
    AudioSys.depthSwitch();
    showToast('🌌 Mode X-Ray désactivé (Cosmos complet restauré)');
  }
}

// --- Mode Dead Code & Dépendances Orphelines ---
function computeDeadCode() {
  const nodes = state.data.nodes || [];
  const edges = state.data.edges || [];
  const incomingCount = new Map();
  nodes.forEach(n => incomingCount.set(n.id, 0));
  edges.forEach(e => {
    if (incomingCount.has(e.target)) {
      incomingCount.set(e.target, incomingCount.get(e.target) + 1);
    }
  });

  const deadSet = new Set();
  let totalLoc = 0;
  const orphanNodes = [];

  nodes.forEach(n => {
    const inc = incomingCount.get(n.id) || 0;
    if (inc === 0) {
      const nameLower = n.name.toLowerCase();
      const pathLower = (n.file_path || '').toLowerCase();
      const isEntryPoint = 
        nameLower.includes('main') ||
        nameLower.includes('index') ||
        nameLower.includes('app') ||
        nameLower.includes('server') ||
        nameLower.includes('root') ||
        pathLower.endsWith('config.js') ||
        pathLower.endsWith('config.ts') ||
        pathLower.endsWith('vite.config.js') ||
        pathLower.endsWith('package.json') ||
        pathLower.endsWith('cargo.toml') ||
        n.layer === 'docs' ||
        pathLower.endsWith('.md') ||
        n.stereotype === '«System»' ||
        n.stereotype === '«Dossier»';

      if (!isEntryPoint) {
        deadSet.add(n.id);
        const loc = (n.metadata && n.metadata.lines) || 15;
        totalLoc += loc;
        orphanNodes.push(n);
      }
    }
  });

  state.deadCodeSet = deadSet;
  return { deadSet, totalLoc, orphanNodes };
}

function updateDeadCodeHud() {
  const hud = document.getElementById('deadcode-hud');
  const btn = document.getElementById('btn-deadcode');
  if (!hud) return;

  if (state.deadCodeMode) {
    hud.classList.remove('hidden');
    if (btn) btn.classList.add('active');
    const { totalLoc, orphanNodes } = computeDeadCode();
    const cntEl = document.getElementById('deadcode-count');
    if (cntEl) cntEl.innerText = `${orphanNodes.length} Orphelin(s)`;
    const locEl = document.getElementById('deadcode-loc-saved');
    if (locEl) locEl.innerText = `~${totalLoc} LOC isolées`;

    const listEl = document.getElementById('deadcode-quick-list');
    if (listEl) {
      listEl.innerHTML = '';
      orphanNodes.slice(0, 10).forEach(n => {
        const pill = document.createElement('span');
        pill.className = 'deadcode-tag-pill';
        pill.innerText = n.name;
        pill.title = `Localiser ${n.name} (${n.file_path})`;
        pill.addEventListener('click', () => {
          selectNode(n);
          state.camera.targetX = n.x;
          state.camera.targetY = n.y;
          state.camera.targetZoom = 1.25;
        });
        listEl.appendChild(pill);
      });
    }
  } else {
    hud.classList.add('hidden');
    if (btn) btn.classList.remove('active');
  }
}

function toggleDeadCodeMode() {
  if (!state.deadCodeMode) {
    if (state.xrayMode) toggleXrayMode();
    if (state.heatmapMode) toggleHeatmapMode();
    state.deadCodeMode = true;
    computeDeadCode();
    updateDeadCodeHud();
    AudioSys.deadCodeScan();
    showToast(`💀 Scanner Code Mort activé : ${state.deadCodeSet.size} orphelins isolés`);
    AetherAPI.sendTelemetry('DeadCodeScanner', 'Audit', `Scan code mort : ${state.deadCodeSet.size} orphelins détectés`).catch(() => {});
  } else {
    state.deadCodeMode = false;
    state.deadCodeSet.clear();
    updateDeadCodeHud();
    AudioSys.depthSwitch();
    showToast('🌌 Mode Code Mort désactivé (Galaxie complète restaurée)');
  }
}

function generateDeadCodeCleanPrompt() {
  const orphans = (state.data.nodes || []).filter(n => state.deadCodeSet.has(n.id));
  const projName = state.data.project_name || 'Project';
  const origin = window.location.origin;
  return `# 💀 AUDIT CODE MORT & NETTOYAGE MODULAIRE
# Projet : ${projName}
# Cibles orphelines identifiées : ${orphans.length}

<contexte_audit>
L'analyseur de flux AetherScope a identifié les symboles suivants comme n'ayant aucun appelant ni import direct dans le projet :

${orphans.map(o => `- [${o.stereotype || 'Symbol'}] ${o.name} (${o.file_path}) [${(o.metadata && o.metadata.lines) || 0} LOC]`).join('\n')}
</contexte_audit>

<instructions_ia>
1. Déterminer avec certitude si ces symboles sont véritablement obsolètes ou appelés dynamiquement.
2. Pour chaque symbole véritablement mort : proposer une suppression propre sans casser de contrat.
3. Pour chaque symbole à conserver : proposer de le brancher avec un import explicite ou spécifier dans README.md.
</instructions_ia>

<aetherscope_api_and_research>
1. Documentation & Contrats Modulaires : Consulter impérativement AETHER_AI_RULES.md à la racine et les README.md des sous-dossiers concernés.
2. Télémétrie Spatiale Temps Réel : Émettre POST ${origin}/api/ai/telemetry à chaque symbole inspecté.
3. Exploration du Graphe : Consulter l'API locale AetherScope sur ${origin}.
</aetherscope_api_and_research>

<response_format_rules>
- Concision & Zéro Blabla.
- Diffs Chirurgicaux : Ne JAMAIS renvoyer des fichiers entiers.
- Respect Strict des Couches & Contrats.
</response_format_rules>`;
}

// --- Mode Architecture & Health Heatmap ---
function computeHeatmap() {
  const nodes = state.data.nodes || [];
  const edges = state.data.edges || [];
  const inDegree = new Map();
  const outDegree = new Map();
  nodes.forEach(n => {
    inDegree.set(n.id, 0);
    outDegree.set(n.id, 0);
  });
  edges.forEach(e => {
    if (outDegree.has(e.source)) outDegree.set(e.source, outDegree.get(e.source) + 1);
    if (inDegree.has(e.target)) inDegree.set(e.target, inDegree.get(e.target) + 1);
  });

  let maxRaw = 1;
  const list = [];
  nodes.forEach(n => {
    const inc = inDegree.get(n.id) || 0;
    const out = outDegree.get(n.id) || 0;
    const loc = (n.metadata && n.metadata.lines) || 15;
    const raw = (out * 2.4) + (inc * 1.5) + (loc / 25.0) + (n.is_class ? 3.5 : 0);
    if (raw > maxRaw) maxRaw = raw;
    list.push({ node: n, raw, inc, out, loc });
  });

  const scores = new Map();
  list.forEach(item => {
    const norm = Math.min(1.0, Math.max(0.0, item.raw / maxRaw));
    scores.set(item.node.id, {
      score: norm,
      raw: item.raw,
      inc: item.inc,
      out: item.out,
      loc: item.loc
    });
  });

  list.sort((a, b) => b.raw - a.raw);
  const topHotspots = list.slice(0, 5);
  const avgRaw = list.reduce((acc, x) => acc + x.raw, 0) / (list.length || 1);
  const healthScore = Math.max(45, Math.min(99, Math.round(100 - (avgRaw * 1.6))));

  state.heatmapScores = scores;
  return { scores, topHotspots, healthScore };
}

function updateHeatmapHud() {
  const hud = document.getElementById('heatmap-hud');
  const btn = document.getElementById('btn-heatmap');
  if (!hud) return;

  if (state.heatmapMode) {
    hud.classList.remove('hidden');
    if (btn) btn.classList.add('active');
    const { topHotspots, healthScore } = computeHeatmap();
    const scoreBadge = document.getElementById('heatmap-global-score');
    if (scoreBadge) {
      scoreBadge.innerText = `Santé : ${healthScore}/100`;
      scoreBadge.style.color = healthScore > 75 ? '#6ee7b7' : (healthScore > 55 ? '#fde68a' : '#fca5a5');
    }

    const box = document.getElementById('heatmap-hotspots');
    if (box) {
      box.innerHTML = '';
      topHotspots.forEach(h => {
        const pill = document.createElement('span');
        pill.className = 'heatmap-hotspot-pill';
        pill.innerText = `🔥 ${h.node.name}`;
        pill.title = `Hotspot: ${h.node.name} (Couplage: In ${h.inc}, Out ${h.out}, LOC ${h.loc})`;
        pill.addEventListener('click', () => {
          selectNode(h.node);
          state.camera.targetX = h.node.x;
          state.camera.targetY = h.node.y;
          state.camera.targetZoom = 1.25;
        });
        box.appendChild(pill);
      });
    }
  } else {
    hud.classList.add('hidden');
    if (btn) btn.classList.remove('active');
  }
}

function toggleHeatmapMode() {
  if (!state.heatmapMode) {
    if (state.xrayMode) toggleXrayMode();
    if (state.deadCodeMode) toggleDeadCodeMode();
    state.heatmapMode = true;
    computeHeatmap();
    updateHeatmapHud();
    AudioSys.heatmapPulse();
    showToast('🌡️ Heatmap de Santé & Complexité activée');
    AetherAPI.sendTelemetry('HeatmapEngine', 'Analyse', 'Analyse thermique de complexité').catch(() => {});
  } else {
    state.heatmapMode = false;
    state.heatmapScores.clear();
    updateHeatmapHud();
    AudioSys.depthSwitch();
    showToast('🌌 Mode Heatmap désactivé (Rendu cosmique standard)');
  }
}

function generateHeatmapArchitecturePrompt() {
  const { topHotspots, healthScore } = computeHeatmap();
  const projName = state.data.project_name || 'Project';
  const origin = window.location.origin;
  return `# 🌡️ AUDIT DE COUPLAGE & RÉDUCTION DE COMPLEXITÉ (HEATMAP)
# Projet : ${projName}
# Score de Santé Actuel : ${healthScore}/100

<top_hotspots_complexite>
L'analyseur de flux AetherScope a identifié les 5 composants les plus critiques en termes de couplage et de masse :

${topHotspots.map(h => `- ${h.node.name} (${h.node.file_path}) [${h.loc} LOC] : In=${h.inc} (appelants), Out=${h.out} (dépendances)`).join('\n')}
</top_hotspots_complexite>

<mission_refactorisation>
1. Examiner ces Hotspots et proposer une découpe modulaire suivant le principe de responsabilité unique (SRP).
2. Réduire le fan-out (nombre de dépendances externes) en déléguant des responsabilités vers des sous-services.
3. Rédiger ou mettre à jour la spécification README.md des modules touchés.
</mission_refactorisation>

<aetherscope_api_and_research>
1. Documentation & Contrats Modulaires : Consulter impérativement AETHER_AI_RULES.md à la racine et les README.md des dossiers des composants identifiés.
2. Télémétrie Spatiale Temps Réel : Émettre POST ${origin}/api/ai/telemetry à chaque hotspot inspecté.
3. Exploration du Graphe : Consulter l'API locale AetherScope sur ${origin}.
</aetherscope_api_and_research>

<response_format_rules>
- Concision & Zéro Blabla.
- Diffs Chirurgicaux : Ne JAMAIS renvoyer des fichiers entiers.
- Respect Strict des Couches & Contrats.
</response_format_rules>`;
}

// --- Dynamic Affiliation Counts & 5 Affiliation Filters Controller ---
// 1 = PHYSICAL, 2 = NETWORK_RPC, 4 = REALTIME_WS, 8 = SEMANTIC, 16 = SECURITY_VIOLATES
function computeAffiliationCounts() {
  const counts = { 1: 0, 2: 0, 4: 0, 8: 0, 16: 0 };
  const edges = (state.data && state.data.edges) || [];
  for (let i = 0; i < edges.length; i++) {
    const e = edges[i];
    let bit = e.filter_bits;
    if (bit === undefined) {
      const rel = e.relation;
      if (rel === 'security_violation' || rel === 'security_violates' || rel === 'violates' || rel === 'violation') bit = 16;
      else if (rel === 'realtime_ws' || rel === 'realtime' || rel === 'realtimews') bit = 4;
      else if (rel === 'submits' || rel === 'network') bit = 2;
      else if (rel === 'contains') bit = 1;
      else bit = 8;
    }
    if (counts[bit] !== undefined) {
      counts[bit]++;
    }
  }
  return counts;
}
window.computeAffiliationCounts = computeAffiliationCounts;

function renderAffiliationPills() {
  const box = document.getElementById('affiliation-filters-bar');
  if (!box) return;
  box.innerHTML = '';

  const bits = state.filterBits !== undefined ? state.filterBits : 31;
  const counts = computeAffiliationCounts();

  const affDefs = [
    { bit: 1, name: 'STRUCTURE', col: '#38bdf8', icon: '📦', title: 'Structure physique de confinement (contains)' },
    { bit: 2, name: 'RPC NET', col: '#00c8ff', icon: '🌐', title: 'Réseau & Endpoints RPC (submits / network)' },
    { bit: 4, name: 'REALTIME WS', col: '#00f0ff', icon: '⚡', title: 'Canaux WebSocket Temps Réel (Lasers Cyan)' },
    { bit: 8, name: 'SÉMANTIQUE', col: '#a855f7', icon: '🔗', title: 'Dépendances sémantiques (imports / calls / docs)' },
    { bit: 16, name: 'VIOLATIONS', col: '#ef4444', icon: '🚨', title: 'Violations de Contrats & Linter SAST (SECURITY_VIOLATES)' },
    { bit: 32, name: 'DATABASE', col: '#10b981', icon: '💾', title: 'Persistance SQL & Base de Données (queries)' }
  ];

  affDefs.forEach(def => {
    const pill = document.createElement('div');
    const active = (bits & def.bit) !== 0;
    const count = counts[def.bit] || 0;
    pill.className = `synth-pill ${active ? '' : 'disabled'}`;
    pill.setAttribute('data-bit', def.bit);
    pill.title = `${def.title} — ${active ? 'Actif' : 'Désactivé'}`;
    pill.style.borderColor = def.col;
    pill.style.color = def.col;
    pill.style.background = active ? `${def.col}22` : 'transparent';
    pill.innerHTML = `<span>${def.icon}</span> <span>${def.name} (${count})</span>`;

    pill.addEventListener('click', () => {
      toggleAffiliationFilter(def.bit, pill);
    });
    box.appendChild(pill);
  });
}
window.renderAffiliationPills = renderAffiliationPills;

function updateAffiliationUI() {
  const bits = state.filterBits !== undefined ? state.filterBits : 63;

  // 1. Update dynamic pills in #synthesis-card
  renderAffiliationPills();

  // 2. Update items in Affichage dropdown menu
  document.querySelectorAll('.aff-menu-item').forEach(item => {
    const bit = parseInt(item.getAttribute('data-aff-bit') || '0');
    if (bit) {
      const active = (bits & bit) !== 0;
      item.classList.toggle('active', active);
      const badge = item.querySelector('.aff-status-badge');
      if (badge) {
        badge.innerText = active ? 'ON' : 'OFF';
        badge.style.color = active ? '#38bdf8' : '#64748b';
      }
    }
  });
}
window.updateAffiliationUI = updateAffiliationUI;

function setAllAffiliationFilters(enableAll) {
  state.filterBits = enableAll ? 31 : 0;
  updateAffiliationUI();
  invalidateEdgePathCache();
  wakePhysics(60);
  AudioSys.playTone(enableAll ? 750 : 300, 'sine', 0.15, 0.05);
  showToast(enableAll ? '👁️ Tous les filtres d\'affiliation activés (31)' : '🚫 Tous les filtres d\'affiliation masqués (0)');
}
window.setAllAffiliationFilters = setAllAffiliationFilters;

function toggleAffiliationFilter(bit, pillEl) {
  if (state.filterBits === undefined) state.filterBits = 31;
  state.filterBits ^= bit;
  const isEnabled = (state.filterBits & bit) !== 0;
  updateAffiliationUI();
  invalidateEdgePathCache();
  wakePhysics(60);
  AudioSys.playTone(isEnabled ? 660 : 330, 'sine', 0.12, 0.04);
  const names = {
    1: 'Structure Physique (contains)',
    2: 'Réseau RPC (HTTP / submits)',
    4: 'Canaux Temps Réel (WebSocket)',
    8: 'Sémantique (imports / calls / docs)',
    16: 'Violations de Sécurité / Contrats'
  };
  showToast(`${isEnabled ? '👁️ Activé' : '🚫 Masqué'} : ${names[bit] || bit}`);
  if (bit === 16 && isEnabled) {
    if (typeof openCveInspectorTab === 'function') {
      openCveInspectorTab();
    }
  }
}
window.toggleAffiliationFilter = toggleAffiliationFilter;

// --- Display Settings Toggle Controller (Menu Affichage) ---
function syncFolderBadgesButtonUI() {
  const isEnabled = state.showFolderBadges === true;
  const lbl = document.getElementById('folder-badges-status-lbl');
  const btn = document.getElementById('btn-toggle-folder-badges');
  if (lbl) {
    lbl.innerText = isEnabled ? 'ON' : 'OFF';
    lbl.style.color = isEnabled ? '#38bdf8' : '#94a3b8';
  }
  if (btn) {
    btn.classList.toggle('active', isEnabled);
  }
}
window.syncFolderBadgesButtonUI = syncFolderBadgesButtonUI;

function toggleFolderBadgesButton() {
  toggleDisplaySetting('showFolderBadges', 'Noms des Dossiers Auto');
}
window.toggleFolderBadgesButton = toggleFolderBadgesButton;

function toggleDisplaySetting(key, name) {
  state[key] = state[key] === undefined ? false : !state[key];
  const isEnabled = state[key];
  const item = document.getElementById(`btn-menu-toggle-${key}`);
  if (item) {
    const badge = item.querySelector('.menu-toggle-badge');
    if (badge) {
      badge.innerText = isEnabled ? 'ON' : 'OFF';
      badge.style.color = isEnabled ? '#38bdf8' : '#64748b';
    }
    item.classList.toggle('active', isEnabled);
  }
  if (key === 'showFolderBadges') {
    syncFolderBadgesButtonUI();
  }
  state.instanceDataDirty = true;
  state.cachedEdgePaths = null;
  wakePhysics(30);
  AudioSys.playTone(isEnabled ? 600 : 300, 'sine', 0.10, 0.03);
  showToast(`${isEnabled ? '👁️ Affiché' : '🚫 Masqué'} : ${name}`);
}
window.toggleDisplaySetting = toggleDisplaySetting;

// --- Edge Typology Legend Engine (Dynamic Scanning & Full Visibility) ---
const RELATION_DEFINITIONS = {
  submits: { label: "1. Formulaire ➔ Soumission (`submits`)", glyph: "lg-submits", color: "#ff6b00", defaultOrder: 1 },
  network: { label: "2. Requêtes HTTP / REST (`network`)", glyph: "lg-network", color: "#00c8ff", defaultOrder: 2 },
  realtime: { label: "3. WebSocket Temps Réel (`realtime` / WS)", glyph: "lg-realtime", color: "#00f0ff", defaultOrder: 3 },
  realtimews: { label: "3. WebSocket Temps Réel (`realtime` / WS)", glyph: "lg-realtime", color: "#00f0ff", defaultOrder: 3 },
  queries: { label: "4. Persistance Base de Données (`queries` / DB)", glyph: "lg-queries", color: "#10b981", defaultOrder: 4 },
  calls: { label: "Appels de fonctions (`calls`)", glyph: "lg-call", color: "#38bdf8", defaultOrder: 5 },
  renders: { label: "Rendu composants UI (`renders`)", glyph: "lg-render", color: "#ec4899", defaultOrder: 6 },
  imports: { label: "Imports modules (`imports`)", glyph: "lg-import", color: "#c084fc", defaultOrder: 7 },
  contains: { label: "Fichier ➔ Symbole (`contains`)", glyph: "lg-contains", color: "#cbd5e1", defaultOrder: 8 },
  inherits: { label: "Héritage POO / Traits (`inherits`)", glyph: "lg-inherits", color: "#f59e0b", defaultOrder: 9 },
  violates: { label: "Violations Sécurité (`violates`)", glyph: "lg-violation", color: "#ef4444", defaultOrder: 10 },
  violation: { label: "Violations Sécurité (`violates`)", glyph: "lg-violation", color: "#ef4444", defaultOrder: 10 },
  security_violation: { label: "Violations Sécurité (`violates`)", glyph: "lg-violation", color: "#ef4444", defaultOrder: 10 },
  documents: { label: "Document ➔ Code (`documents`)", glyph: "lg-docs", color: "#34d399", defaultOrder: 11 },
  binds: { label: "Liaisons de Contrats (`binds`)", glyph: "lg-binds", color: "#818cf8", defaultOrder: 12 },
  inter_module: { label: "Ponts Transverses Inter-Modules", glyph: "lg-call", color: "#38bdf8", defaultOrder: 13 }
};

function renderEdgeTypologyLegend() {
  const container = document.getElementById('synth-legend-list') || document.querySelector('.synth-legend');
  if (!container) return;

  const edges = state.data?.edges || [];
  const relCounts = new Map();

  // Count occurrences of each relation in current dataset
  edges.forEach(e => {
    const raw = e.relation || 'calls';
    const norm = 
      (raw === 'realtime_ws' || raw === 'realtimews') ? 'realtime' :
      (raw === 'security_violation' || raw === 'security_violates' || raw === 'violation') ? 'violates' :
      raw;
    relCounts.set(norm, (relCounts.get(norm) || 0) + 1);
  });

  // Ensure standard relations exist in the map
  const standardKeys = ['submits', 'network', 'realtime', 'queries', 'calls', 'renders', 'imports', 'contains', 'inherits', 'violates', 'documents', 'binds'];
  standardKeys.forEach(k => {
    if (!relCounts.has(k)) {
      relCounts.set(k, 0);
    }
  });

  // Include any other custom relation present in dataset
  edges.forEach(e => {
    const r = e.relation;
    if (r && !relCounts.has(r)) {
      relCounts.set(r, (relCounts.get(r) || 0) + 1);
    }
  });

  let targetBox = document.getElementById('synth-legend-list');
  if (!targetBox) {
    targetBox = container;
  }
  targetBox.innerHTML = '';

  // Sort relations: known standard first by defaultOrder, then custom by count descending
  const sortedRels = Array.from(relCounts.entries()).sort((a, b) => {
    const orderA = RELATION_DEFINITIONS[a[0]]?.defaultOrder || 99;
    const orderB = RELATION_DEFINITIONS[b[0]]?.defaultOrder || 99;
    if (orderA !== orderB) return orderA - orderB;
    return b[1] - a[1];
  });

  sortedRels.forEach(([relKey, count]) => {
    const def = RELATION_DEFINITIONS[relKey] || {
      label: `Liaison \`${relKey}\``,
      glyph: 'lg-call',
      color: '#38bdf8'
    };

    const row = document.createElement('div');
    const isRelActive = state.activeRelations ? (
      state.activeRelations.has(relKey) ||
      (relKey === 'realtime' && (state.activeRelations.has('realtimews') || state.activeRelations.has('realtime_ws'))) ||
      (relKey === 'violates' && (state.activeRelations.has('violation') || state.activeRelations.has('security_violation')))
    ) : true;

    row.className = `legend-row ${isRelActive ? '' : 'disabled'}`;
    row.setAttribute('data-rel', relKey);

    row.innerHTML = `
      <div class="legend-row-left">
        <span class="line-glyph ${def.glyph}"></span>
        <span class="legend-row-text">${def.label}</span>
        <span class="legend-row-count">(${count})</span>
      </div>
      <span class="legend-toggle-badge">${isRelActive ? 'ON' : 'OFF'}</span>
    `;

    row.addEventListener('click', () => {
      toggleSingleRelation(relKey, row);
    });

    targetBox.appendChild(row);
  });

  // Also update burger badge
  const burgerBadge = document.getElementById('burger-count-badge');
  if (burgerBadge) {
    burgerBadge.innerText = `${edges.length} flux`;
  }

  // Update CVE Anomalies Panel
  renderCveAnomaliesPanel();
}
window.renderEdgeTypologyLegend = renderEdgeTypologyLegend;

// --- 🚨 CVE & SAST Anomalies Controller ---
let currentCveLangFilter = 'all';
let currentCveAnomaliesCache = [];

function getLanguageForAnomaly(filePath, ruleId) {
  const p = (filePath || '').toLowerCase();
  const r = (ruleId || '').toUpperCase();
  if (r.startsWith('RULE_') || r.includes('ARCH') || r.includes('BLAKE3') || r === 'SAST_DIRECT_ENV_ACCESS') return 'arch';
  if (p.endsWith('.php') || p.endsWith('.phtml') || p.endsWith('.php3') || p.endsWith('.php4') || p.endsWith('.php5') || p.endsWith('.php7') || p.endsWith('.phps') || r.includes('-PHP-') || r.includes('PHP')) return 'php';
  if (p.endsWith('.c') || p.endsWith('.cpp') || p.endsWith('.cc') || p.endsWith('.h') || p.endsWith('.hpp') || r.includes('-CPP-')) return 'cpp';
  if (p.endsWith('.sol')) return 'web3';
  if (p.endsWith('.rs')) return 'rust';
  if (p.endsWith('.py')) return 'python';
  if (p.endsWith('.js') || p.endsWith('.ts') || p.endsWith('.tsx') || p.endsWith('.jsx') || p.endsWith('.mjs') || p.endsWith('.cjs')) return 'node';
  if (r.includes('WEB3')) return 'web3';
  return 'arch';
}

function isWeb3Anomaly(a) {
  const r = (a.rule_id || '').toUpperCase();
  const p = (a.file_path || '').toLowerCase();
  return a.lang === 'web3' || r.includes('WEB3') || r.includes('DEFI') || r.includes('EVC') || p.includes('wallet') || p.includes('auth_controller') || p.endsWith('.sol');
}

function renderCveAnomaliesPanel(lang = null) {
  if (lang !== null) currentCveLangFilter = lang;
  const targetBox = document.getElementById('cve-anomalies-list');
  const badgeEl = document.getElementById('cve-total-badge');
  if (!targetBox) return;

  const nodes = state.data?.nodes || [];
  const edges = state.data?.edges || [];
  const anomalies = [];
  const seenKey = new Set();

  const activeRulesSet = (state.settings && Array.isArray(state.settings.activeCveRules))
    ? new Set(state.settings.activeCveRules)
    : null;

  // 1. Collect structured anomalies from node metadata
  nodes.forEach(n => {
    if (n.metadata && n.metadata.cve_anomalies && Array.isArray(n.metadata.cve_anomalies)) {
      n.metadata.cve_anomalies.forEach(a => {
        if (activeRulesSet && a.rule_id && !activeRulesSet.has(a.rule_id)) {
          return;
        }
        // Exclure les simples avertissements de budget de lignes du panneau CVE
        if (a.rule_id === 'RULE_MAX_LINES' || a.severity === 'WARNING') {
          return;
        }
        const key = `${a.file_path}:${a.line_number}:${a.rule_id}`;
        if (!seenKey.has(key)) {
          seenKey.add(key);
          anomalies.push({
            rule_id: a.rule_id,
            cve_id: a.cve_id || a.rule_id,
            severity: (a.severity || 'HIGH').toUpperCase(),
            line_number: a.line_number || 1,
            file_path: a.file_path || n.file_path,
            symbol_name: n.name,
            symbol_id: n.id,
            message: a.message,
            transmitted_data: a.transmitted_data,
            transmitted_type: a.transmitted_type,
            remediation: a.remediation,
            lang: getLanguageForAnomaly(a.file_path || n.file_path, a.rule_id)
          });
        }
      });
    }
  });

  // 2. Also collect from violation edges if any edge has cve_id or violation_details
  edges.forEach(e => {
    if (e.relation === 'violates' || e.filter_bits === 16 || e.cve_id) {
      const srcNode = nodes.find(n => n.id === e.source);
      const tgtNode = nodes.find(n => n.id === e.target);
      const filePath = (srcNode && srcNode.file_path) || (tgtNode && tgtNode.file_path) || 'source';
      const ruleId = (e.label || 'SECURITY_VIOLATION').split(':')[0].trim();
      if (activeRulesSet && ruleId !== 'SECURITY_VIOLATION' && !activeRulesSet.has(ruleId)) {
        return;
      }
      if (ruleId === 'RULE_MAX_LINES' || (e.severity && e.severity.toUpperCase() === 'WARNING')) {
        return;
      }
      const exactDup = anomalies.some(a =>
        a.file_path === filePath && (
          a.rule_id === ruleId ||
          a.cve_id === ruleId ||
          (e.cve_id && (a.cve_id === e.cve_id || a.rule_id === e.cve_id))
        )
      );
      if (exactDup) return;
      const key = `${e.source}:${e.target}:${ruleId}`;
      if (!seenKey.has(key)) {
        seenKey.add(key);
        anomalies.push({
          rule_id: ruleId,
          cve_id: e.cve_id || ruleId,
          severity: (e.severity || 'CRITICAL').toUpperCase(),
          line_number: (srcNode && srcNode.line_number) || 1,
          file_path: filePath,
          symbol_name: srcNode ? srcNode.name : e.source,
          symbol_id: e.source,
          message: e.violation_details || 'Violation de contrat ou anomalie de sécurité détectée',
          transmitted_data: e.transmitted_data || (tgtNode ? tgtNode.name : null),
          transmitted_type: e.transmitted_type || 'Illegal Boundary Crossing',
          remediation: 'Corriger la règle de contrat ou sécuriser la transmission de données.',
          lang: getLanguageForAnomaly(filePath, ruleId)
        });
      }
    }
  });

  // Sort by severity (CRITICAL, then HIGH, then others)
  const sevWeight = { 'CRITICAL': 3, 'HIGH': 2, 'MEDIUM': 1, 'WARNING': 1 };
  anomalies.sort((a, b) => (sevWeight[b.severity] || 0) - (sevWeight[a.severity] || 0));
  anomalies.forEach((a, i) => { a._cacheIdx = i; });
  currentCveAnomaliesCache = anomalies;
  window.currentCveAnomaliesCache = anomalies;

  // Update total badge
  const totalCount = anomalies.length;
  if (badgeEl) {
    badgeEl.innerText = `${totalCount} anomalie${totalCount > 1 ? 's' : ''}`;
    badgeEl.style.color = totalCount > 0 ? '#f87171' : '#10b981';
    badgeEl.style.borderColor = totalCount > 0 ? 'rgba(239,68,68,0.5)' : 'rgba(16,185,129,0.4)';
    badgeEl.style.background = totalCount > 0 ? 'rgba(239,68,68,0.18)' : 'rgba(16,185,129,0.12)';
  }

  const badgeLeftEl = document.getElementById('cve-total-badge-left');
  if (badgeLeftEl) {
    badgeLeftEl.innerText = `${totalCount} anomalie${totalCount > 1 ? 's' : ''} ➔`;
    badgeLeftEl.style.color = totalCount > 0 ? '#f87171' : '#10b981';
    badgeLeftEl.style.borderColor = totalCount > 0 ? 'rgba(239,68,68,0.5)' : 'rgba(16,185,129,0.4)';
    badgeLeftEl.style.background = totalCount > 0 ? 'rgba(239,68,68,0.22)' : 'rgba(16,185,129,0.12)';
  }

  const cveTabBtn = document.getElementById('tab-btn-cve');
  if (cveTabBtn) {
    cveTabBtn.classList.toggle('has-anomalies', totalCount > 0);
  }

  const topbarCveBadge = document.getElementById('topbar-cve-badge');
  if (topbarCveBadge) {
    topbarCveBadge.innerText = `${totalCount}`;
    topbarCveBadge.style.background = totalCount > 0 ? 'rgba(239,68,68,0.32)' : 'rgba(16,185,129,0.22)';
    topbarCveBadge.style.color = totalCount > 0 ? '#fecaca' : '#6ee7b7';
    topbarCveBadge.style.borderColor = totalCount > 0 ? 'rgba(248,113,113,0.55)' : 'rgba(16,185,129,0.45)';
  }

  const synthBadgeCve = document.getElementById('synth-badge-cve');
  if (synthBadgeCve) {
    synthBadgeCve.innerText = `${totalCount}`;
  }

  // Also update pill on 16 filter button
  const pill16 = document.querySelector('#affiliation-filters-bar [data-bit="16"]');
  if (pill16) {
    pill16.innerHTML = `<span>🚨</span> <span>VIOLATIONS (${totalCount})</span>`;
  }

  // Compute live counts per filter button & KPI strip
  const critCount = anomalies.filter(a => a.severity === 'CRITICAL').length;
  const highCount = anomalies.filter(a => a.severity === 'HIGH').length;
  const warnCount = anomalies.length - critCount - highCount;
  const uniqueFiles = new Set(anomalies.map(a => a.file_path)).size;

  const kpiStrip = document.getElementById('cve-kpi-strip');
  if (kpiStrip) {
    kpiStrip.innerHTML = `
      <span style="font-size:0.62rem; font-family:var(--font-mono); padding:2px 7px; border-radius:4px; background:rgba(239,68,68,0.2); border:1px solid rgba(239,68,68,0.45); color:#fca5a5;">🔴 ${critCount} CRITICAL</span>
      <span style="font-size:0.62rem; font-family:var(--font-mono); padding:2px 7px; border-radius:4px; background:rgba(249,115,22,0.18); border:1px solid rgba(249,115,22,0.4); color:#fdba74;">🟠 ${highCount} HIGH</span>
      ${warnCount > 0 ? `<span style="font-size:0.62rem; font-family:var(--font-mono); padding:2px 7px; border-radius:4px; background:rgba(234,179,8,0.16); border:1px solid rgba(234,179,8,0.35); color:#fde047;">🟡 ${warnCount} WARN</span>` : ''}
      <span style="font-size:0.62rem; font-family:var(--font-mono); padding:2px 7px; border-radius:4px; background:rgba(56,189,248,0.14); border:1px solid rgba(56,189,248,0.35); color:#7dd3fc;">📄 ${uniqueFiles} fichier${uniqueFiles > 1 ? 's' : ''}</span>
    `;
  }

  // Update filter button labels with live counts
  const filterLabels = {
    all: `[Tous (${anomalies.length})]`,
    critical: `[🔴 Critical (${critCount})]`,
    php: `[🐘 PHP (${anomalies.filter(a => a.lang === 'php').length})]`,
    rust: `[🦀 Rust (${anomalies.filter(a => a.lang === 'rust').length})]`,
    node: `[🟢 Node/TS (${anomalies.filter(a => a.lang === 'node').length})]`,
    web3: `[🌐 Web3 (${anomalies.filter(a => isWeb3Anomaly(a)).length})]`,
    python: `[🐍 Python (${anomalies.filter(a => a.lang === 'python').length})]`,
    cpp: `[⚙️ C/C++ (${anomalies.filter(a => a.lang === 'cpp').length})]`,
    arch: `[🏛️ Arch (${anomalies.filter(a => a.lang === 'arch').length})]`
  };
  document.querySelectorAll('#cve-lang-filters .cve-filter-btn').forEach(btn => {
    const key = btn.getAttribute('data-lang');
    if (key && filterLabels[key]) {
      btn.innerText = filterLabels[key];
    }
  });

  // Filter by selected category/severity
  let filtered = anomalies;
  if (currentCveLangFilter === 'critical') {
    filtered = anomalies.filter(a => a.severity === 'CRITICAL');
  } else if (currentCveLangFilter === 'web3') {
    filtered = anomalies.filter(a => isWeb3Anomaly(a));
  } else if (currentCveLangFilter !== 'all') {
    filtered = anomalies.filter(a => a.lang === currentCveLangFilter);
  }

  // Filter by instant search query if any
  const searchInput = document.getElementById('cve-quick-search-input');
  const query = (searchInput && searchInput.value || '').trim().toLowerCase();
  if (query) {
    filtered = filtered.filter(a =>
      (a.rule_id || '').toLowerCase().includes(query) ||
      (a.cve_id || '').toLowerCase().includes(query) ||
      (a.file_path || '').toLowerCase().includes(query) ||
      (a.symbol_name || '').toLowerCase().includes(query) ||
      (a.message || '').toLowerCase().includes(query) ||
      (a.transmitted_data || '').toLowerCase().includes(query)
    );
  }

  if (filtered.length === 0) {
    targetBox.innerHTML = `
      <div style="color:#64748b; font-size:0.65rem; font-family:var(--font-mono); padding:10px; text-align:center;">
        ✅ Zéro anomalie (${currentCveLangFilter.toUpperCase()}${query ? ` • "${query}"` : ''})
      </div>
    `;
    return;
  }

  const langIcons = {
    php: '🐘',
    rust: '🦀',
    python: '🐍',
    node: '🟢',
    web3: '🌐',
    cpp: '⚙️',
    arch: '🏛️'
  };

  // 1. Mettre à jour le panneau de diagnostics positifs en haut du menu de gauche
  renderSynthesisTopDiagnostics();

  const escapeFn = typeof escapeHtml === 'function' ? escapeHtml : (str => String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;'));

  targetBox.innerHTML = filtered.map((a) => {
    const cacheIdx = a._cacheIdx;
    const sevClass = a.severity.toLowerCase();
    const icon = isWeb3Anomaly(a) ? '🌐' : (langIcons[a.lang] || '🚨');
    const hasTainted = a.transmitted_data || a.transmitted_type;
    const expl = CVE_ATTACK_EXPLANATIONS[a.rule_id] || {
      attack: "Faiblesse de sécurité ou contournement de contrat détecté dans le flux d'exécution ou les données.",
      shield: a.remediation || "Appliquer les directives de sécurité et valider les invariants d'architecture."
    };

    let defiLaserBadge = '';
    let defiPatchBtn = '';
    if (a.rule_id === 'DEFI-ASYMMETRIC-ROUNDING' || (a.cve_id && a.cve_id.includes('ASYMMETRIC_ROUNDING_RISK'))) {
      defiLaserBadge = `<div class="defi-laser-badge" style="margin-top:4px;">☣️ [CWE-682] ASYMMETRIC_ROUNDING_RISK</div>`;
      defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="event.stopPropagation(); if (window.copyDefiPatchPrompt) window.copyDefiPatchPrompt('${escapeFn(a.file_path)}', ${a.line_number}, 'DEFI-ASYMMETRIC-ROUNDING');">🛡️ Patch Arrondi Asymétrique</button>`;
    } else if (a.rule_id === 'DEFI-STALE-CACHE-AFTER-CALL' || (a.cve_id && a.cve_id.includes('STALE_CACHE_TOCTOU'))) {
      defiLaserBadge = `<div class="defi-laser-badge" style="margin-top:4px;">☣️ [CWE-362] STALE_CACHE_TOCTOU</div>`;
      defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="event.stopPropagation(); if (window.copyDefiPatchPrompt) window.copyDefiPatchPrompt('${escapeFn(a.file_path)}', ${a.line_number}, 'DEFI-STALE-CACHE-AFTER-CALL');">🛡️ Invalidation Cache Post-Call</button>`;
    } else if (a.rule_id === 'EVC-HOOK-BYPASS' || (a.cve_id && a.cve_id.includes('EVC_HOOK_BYPASS_RISK'))) {
      defiLaserBadge = `<div class="defi-laser-badge" style="margin-top:4px;">☣️ [CWE-841] EVC_HOOK_BYPASS_RISK</div>`;
      defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="event.stopPropagation(); if (window.copyDefiPatchPrompt) window.copyDefiPatchPrompt('${escapeFn(a.file_path)}', ${a.line_number}, 'EVC-HOOK-BYPASS');">🛡️ Invariant EVC nonReentrant</button>`;
    } else if (a.rule_id === 'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER' || (a.cve_id && a.cve_id.includes('ORACLE_MIN_MAX_CIRCUIT_BREAKER'))) {
      defiLaserBadge = `<div class="defi-laser-badge" style="margin-top:4px;">☣️ [LOGIC-CRITICAL] ORACLE_MIN_MAX_CIRCUIT_BREAKER</div>`;
      defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="event.stopPropagation(); if (window.copyDefiPatchPrompt) window.copyDefiPatchPrompt('${escapeFn(a.file_path)}', ${a.line_number}, 'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER');">🛡️ Patch Circuit Breaker Min/Max</button>`;
    } else if (a.rule_id === 'DEFI-ORACLE-L2-SEQUENCER-RACE' || (a.cve_id && a.cve_id.includes('L2_SEQUENCER_ORACLE_RACE'))) {
      defiLaserBadge = `<div class="defi-laser-badge" style="margin-top:4px;">☣️ [LOGIC-CRITICAL] L2_SEQUENCER_ORACLE_RACE</div>`;
      defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="event.stopPropagation(); if (window.copyDefiPatchPrompt) window.copyDefiPatchPrompt('${escapeFn(a.file_path)}', ${a.line_number}, 'DEFI-ORACLE-L2-SEQUENCER-RACE');">🛡️ Patch Sequencer Grace Period</button>`;
    } else if (a.rule_id === 'DEFI-TWAP-SHORT-WINDOW' || (a.cve_id && a.cve_id.includes('TWAP_SHORT_WINDOW'))) {
      defiLaserBadge = `<div class="defi-laser-badge" style="margin-top:4px;">☣️ [LOGIC-CRITICAL] TWAP_SHORT_WINDOW</div>`;
      defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="event.stopPropagation(); if (window.copyDefiPatchPrompt) window.copyDefiPatchPrompt('${escapeFn(a.file_path)}', ${a.line_number}, 'DEFI-TWAP-SHORT-WINDOW');">🛡️ Patch TWAP Safe Window (>= 30m)</button>`;
    }

    return `
      <div class="cve-card ${sevClass}" onclick="teleportToCveAnomaly(${cacheIdx})">
        <div class="cve-card-header">
          <span class="cve-badge ${sevClass}">
            ${icon} [${a.severity}] ${escapeFn(a.cve_id)}
          </span>
          <span style="font-size:0.60rem; color:#94a3b8; font-family:var(--font-mono);">L${a.line_number}</span>
        </div>
        ${defiLaserBadge}
        <div class="cve-target-path" title="${escapeFn(a.file_path)}">
          📄 ${escapeFn(a.file_path)} ${a.symbol_name ? `• <code>${escapeFn(a.symbol_name)}</code>` : ''}
        </div>
        <div class="cve-msg-box" style="color:#f1f5f9; font-size:0.68rem; margin-top:3px; line-height:1.35;">
          ${escapeFn(a.message)}
        </div>
        <div style="font-size:0.65rem; color:#fee2e2; background:rgba(239,68,68,0.12); border-left:2px solid #ef4444; border-radius:4px; padding:4px 6px; margin-top:4px; line-height:1.35;">
          <strong>💥 Attaque potentielle :</strong> ${escapeFn(expl.attack)}
        </div>
        <div style="font-size:0.65rem; color:#d1fae5; background:rgba(16,185,129,0.10); border-left:2px solid #10b981; border-radius:4px; padding:4px 6px; margin-top:4px; line-height:1.35;">
          <strong>🛡️ Protection requise :</strong> ${escapeFn(expl.shield)}
        </div>
        ${hasTainted ? `
          <div class="cve-transmitted-box">
            <div><strong>📦 Donnée :</strong> <code>${escapeFn(a.transmitted_data || 'payload')}</code></div>
            <div><strong>🏷️ Type :</strong> <code>${escapeFn(a.transmitted_type || 'Untyped')}</code></div>
          </div>
        ` : ''}
        <div class="cve-card-actions" style="display:flex; gap:5px; flex-wrap:wrap;">
          <button class="cve-btn-prompt" style="flex:1; background:linear-gradient(135deg, rgba(245,158,11,0.28), rgba(239,68,68,0.28)); border-color:#fbbf24; color:#fef08a;" onclick="event.stopPropagation(); copyCvePocBountyPrompt(${cacheIdx})" title="Copier le Prompt Proof of Concept (PoC) Bug Bounty avec Scope auto (sans modifier le code)">
            🧪 Copier Prompt PoC Bounty
          </button>
          <button class="cve-btn-patch" style="border-color:rgba(56,189,248,0.5); color:#7dd3fc; background:rgba(56,189,248,0.14);" onclick="event.stopPropagation(); openBountyScopeModalForAnomaly(${cacheIdx})" title="Ouvrir cette CVE dans le préparateur de Scope & PoC Bug Bounty">
            🎯 Scope/PoC
          </button>
          <button class="cve-btn-patch" style="opacity:0.8;" onclick="event.stopPropagation(); copyCveSinglePatchPrompt(${cacheIdx})" title="Copier un prompt de patch chirurgical (si vous souhaitez corriger le code)">
            🛡️ Patch
          </button>
          ${defiPatchBtn}
        </div>
      </div>
    `;
  }).join('');
}
window.renderCveAnomaliesPanel = renderCveAnomaliesPanel;

// --- 🛡️ Catalogue Exhaustif des Explications d'Attaques & Protections par CVE ---
const CVE_ATTACK_EXPLANATIONS = {
  // 🦀 Rust & Réseau Bas-Niveau
  'CVE-RUST-DESER-BOMB-OOM': {
    attack: "Bombe d'allocation mémoire : un attaquant injecte un préfixe de taille binaire forgé de 8 octets (ex: 0xFFFFFFFFFFFFFFFF). L'allocateur Rust tente de pré-allouer des téraoctets et crashe immédiatement tout le serveur par un abort() OOM sans possibilité de reprise.",
    shield: "Plafonner strictement la désérialisation avec bincode::Options::with_limit(MAX_BYTES) ou Vec::with_capacity(claimed_len.min(MAX_CAP))."
  },
  'CVE-RUST-CANCEL-SAFETY-LOSS': {
    attack: "Perte silencieuse de flux TCP : dans tokio::select!, si une autre branche gagne, read_exact() ou write_all() est détruit en plein vol. Les octets déjà lus/écrits sur le socket sont définitivement perdus, corrompant les messages suivants.",
    shield: "Utiliser tokio_util::codec::Framed avec StreamExt::next(), dont le buffer BytesMut interne conserve l'état même en cas d'annulation."
  },
  'CVE-RUST-TIMING-SIDECHANNEL': {
    attack: "Attaque par canal auxiliaire temporel : l'opérateur == sur les secrets s'arrête au premier octet divergent (court-circuit SIMD memcmp). Un attaquant distant mesure les micro-latences réseau pour extraire les secrets, nonces ou HMAC octet par octet.",
    shield: "Comparer les secrets exclusivement en temps constant sans saut conditionnel via subtle::ConstantTimeEq::ct_eq()."
  },
  'CVE-NET-SLOWLORIS-KEEPALIVE-DOS': {
    attack: "Épuisement de descripteurs de fichiers (Slowloris) : un attaquant ouvre des centaines de connexions TCP/WebSocket et envoie 1 octet toutes les 30 secondes. Sans timeout, les descripteurs OS (EMFILE) saturent et les clients légitimes sont bloqués.",
    shield: "Envelopper le handshake dans tokio::time::timeout(Duration::from_secs(5), accept_async()) et poser un TimeoutLayer."
  },
  'CVE-NET-SSRF-DNS-REBIND': {
    attack: "SSRF & détournement DNS Rebinding : une requête sortante vers une cible dynamique peut viser 169.254.169.254 (métadonnées Cloud AWS/GCP) ou 127.0.0.1 via un DNS à TTL=0 pour exfiltrer les clés IAM et secrets du cluster.",
    shield: "Résoudre le DNS une seule fois, interdire toute IP privée/loopback/link-local et forcer redirect::Policy::none()."
  },
  'CVE-RUST-BLOCKING-IN-ASYNC': {
    attack: "Famine de l'exécuteur Tokio : un appel bloquant synchrone (std::fs, std::thread::sleep) dans async fn bloque le thread de travail OS. Avec N cœurs CPU, N requêtes concurrentes gèlent complètement le serveur (Deadlock Starvation).",
    shield: "Remplacer par tokio::fs / tokio::time::sleep ou déporter le travail bloquant sur tokio::task::spawn_blocking."
  },

  // 🌐 Web3 & Cryptographie
  'CVE-WEB3-REENTRANCY-UNCHECKED': {
    attack: "Attaque par réentrance (Reentrancy Drain) : un contrat malveillant externe ré-invoque la fonction de retrait avant que le solde ne soit décrémenté, permettant de siphonner tous les fonds du pool.",
    shield: "Appliquer l'ordre Checks-Effects-Interactions (déduire le solde avant l'appel externe) et poser un ReentrancyGuard."
  },
  'CVE-WEB3-SIG-MALLEABILITY': {
    attack: "Malléabilité de signature ECDSA : pour chaque signature (r, s), la signature (r, -s mod n) est également valide. Un attaquant modifie la signature en transit pour rejouer une transaction sans connaître la clé privée.",
    shield: "Imposer la normalisation canonique Low-S (rejeter tout s > secp256k1_n / 2) avant vérification."
  },
  'CVE-WEB3-UNVALIDATED-CHAINID': {
    attack: "Rejeu inter-chaînes (Cross-Chain Replay) : une signature signée pour un testnet ou une sidechain sans chain_id peut être réémise à l'identique sur le mainnet pour voler les tokens d'un utilisateur.",
    shield: "Lier la signature au chain_id et à l'adresse du contrat vérificateur dans le domaine EIP-712."
  },
  'CVE-WEB3-NONCE-RACE': {
    attack: "Course TOCTOU sur nonce : supprimer le nonce après l'émission de session permet à deux requêtes parallèles avec le même nonce de passer simultanément avant son invalidation.",
    shield: "Consommer et invalider atomiquement le nonce avant toute action (ex: GETDEL Redis ou HashMap::remove atomique)."
  },
  'CVE-WEB3-AUTH-BYPASS-MOCK': {
    attack: "Usurpation totale de wallet : un préfixe mock actif (0xmock) ou un ecrecover ne rejetant pas l'adresse 0x0 permet à n'importe qui de se connecter avec n'importe quelle adresse publique sans clé privée.",
    shield: "Supprimer tout mock en production, exiger une vraie signature secp256k1 et vérifier recovered != address(0)."
  },
  'CVE-WEB3-ORACLE-SPOT-MANIPULATION': {
    attack: "Manipulation d'oracle par Flash Loan : lire le cours directement sur les réserves spot d'une pool AMM (getReserves/slot0) permet à un attaquant d'emprunter 50M$ en un bloc, déséquilibrer la pool et siphonner le protocole.",
    shield: "Interdire la lecture spot sur AMM ; utiliser Chainlink (latestRoundData avec contrôle de fraîcheur) ou TWAP Uniswap v3."
  },
  'CVE-WEB3-TX-ORIGIN-PHISHING': {
    attack: "Hameçonnage et prise de contrôle : authentifier l'administrateur avec tx.origin permet à un contrat tiers malveillant d'invoquer le protocole au nom de l'admin lorsque celui-ci interagit avec un site piégé.",
    shield: "Remplacer systématiquement tx.origin par msg.sender ou OpenZeppelin _msgSender()."
  },
  'CVE-WEB3-UNCHECKED-TRANSFER-RETURN': {
    attack: "Échec silencieux sur tokens non-standard (USDT/BNB) : ces tokens ne renvoient pas de booléen, provoquant un revert systématique ou un échec silencieux créditant des tokens gratuits sans dépôt réel.",
    shield: "Utiliser impérativement la bibliothèque SafeERC20 d'OpenZeppelin (token.safeTransfer, token.safeTransferFrom)."
  },
  'CVE-WEB3-PERMIT-FRONT-RUNNING': {
    attack: "Griefing et DoS de signature dans le mempool : des bots sniffent la signature permit(v, r, s) et la soumettent avec un gas supérieur, faisant échouer la transaction légitime de l'utilisateur.",
    shield: "Envelopper token.permit() dans un try/catch pour continuer si l'allowance a déjà été validée."
  },
  'CVE-WEB3-HOOK-REENTRANCY': {
    attack: "Réentrance invisible par hook de token (ERC-777 / ERC-1363) : le callback tokensReceived() est exécuté automatiquement lors d'un simple transfer(), permettant de ré-entrer avant la mise à jour des états.",
    shield: "Appliquer ReentrancyGuard nonReentrant sur TOUTES les opérations de transferts de tokens, pas seulement sur l'ETH."
  },
  'CVE-VM-ACCOUNT-MISSING-OWNER': {
    attack: "Confusion de compte (modèle de comptes) : accepter un AccountInfo sans vérifier account.owner == program_id permet à l'attaquant de forger de faux comptes depuis un programme externe pour voler les coffres.",
    shield: "Vérifier formellement account.owner == expected_program_id ou contraindre avec un type de compte validé par le framework."
  },
  'CVE-WEB3-READONLY-REENTRANCY': {
    attack: "Réentrance en lecture seule sur Oracle LP : lire le get_virtual_price() de Curve ou les réserves Balancer dans une fonction view pendant un retrait en cours renvoie un cours faussé.",
    shield: "Vérifier le verrou de réentrance de la pool externe ou appeler une fonction d'état protégée avant de consommer le cours LP."
  },
  'CVE-WEB3-DIRTY-MEMORY-ASSEMBLY': {
    attack: "Bits hauts pollués en mémoire assembly : manipuler des bool/uint8/address en inline assembly (mload/mstore) sans masque binaire laisse des bits hauts non nettoyés, inversant les tests conditionnels.",
    shield: "Appliquer un masque binaire strict en assembly : and(val, 0xff) pour bool/uint8 ou and(val, 0xffffffffffffffffffffffffffffffffffffffff) pour address."
  },
  'CVE-WEB3-ERC4626-INFLATION-ATTACK': {
    attack: "Attaque par inflation de Vault ERC-4626 : le premier déposant mint 1 wei de part puis fait une donation directe massive, tronquant par arrondi à 0 parts les dépôts des utilisateurs suivants.",
    shield: "Implémenter les parts virtuelles OpenZeppelin v4.9+ via _decimalsOffset() ou brûler 1000 wei de dead shares à l'initialisation."
  },
  'CVE-WEB3-VAULT-ROUNDING-DIRECTION': {
    attack: "Direction d'arrondi inversée ERC-4626 : utilisation de divisions entières sans arrondi Floor sur les dépôts ou Ceil sur les rachats, permettant à des attaquants de voler des parts par micro-dépôts successifs.",
    shield: "Utiliser Math.mulDiv avec Math.Rounding.Floor sur les dépôts/conversions en shares, et Math.Rounding.Ceil sur les retraits."
  },
  'CVE-WEB3-STALE-ORACLE-HEARTBEAT': {
    attack: "Oracle de prix obsolète / gelé : appel à latestRoundData() sans contrôle de l'horodatage updatedAt, permettant d'exécuter des emprunts ou liquidations à des cours périmés lors d'une congestion réseau.",
    shield: "Valider la fraîcheur de la ronde : require(updatedAt > 0 && block.timestamp - updatedAt <= HEARTBEAT && answeredInRound >= roundId, 'Prix périmé')."
  },
  'CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR': {
    attack: "Callback flash loan ou hook sans contrôle d'accès : fonction de rappel publique/externe sans vérifier msg.sender == pool, permettant à un attaquant d'invoquer le callback directement et de siphonner les autorisations de tokens.",
    shield: "Ajouter une garde d'accès stricte : require(msg.sender == authorizedLenderPool, 'Invocateur de callback non autorisé')."
  },
  'CVE-WEB3-SIGNATURE-DEADLINE-MISSING': {
    attack: "Signature EIP-712 / Permit sans date limite : un validateur ou attaquant mempool stocke une signature utilisateur valide pour ne l'exécuter que plus tard dans des conditions de marché défavorables.",
    shield: "Inclure un paramètre deadline dans le digest signé et vérifier require(block.timestamp <= deadline, 'Signature expirée')."
  },
  'CVE-WEB3-HOOK-DEFERRED-CHECKS-DESYNC': {
    attack: "Désynchronisation EVC / ChecksDeferred dans HookTarget : un hook personnalisé lit le solde du Vault ou fait un appel externe sans vérifier si l'EVC a des vérifications différées en cours (deferLiquidityCheck). Un attaquant exploite un état transitoirement sous-collatéralisé.",
    shield: "Ajouter une invariant d'état EVC : require(!evc.isChecksDeferred(account) || evc.areChecksInProgress(), 'Vérifications différées en cours')."
  },
  'CVE-WEB3-DECIMAL-SCALING-TRUNCATION': {
    attack: "Troncature ou panique de normalisation décimale : assumption que tous les collatéraux ont 18 décimales (ex: 10**(18 - decimals)), provoquant un overflow sur les tokens > 18 décimales ou une division avant multiplication tronquant à 0 les collatéraux WBTC (8 dec) et USDC (6 dec).",
    shield: "Utiliser Math.mulDiv avec mise à l'échelle bidirectionnelle (handle both decimals <= 18 and decimals > 18) sans division prématurée."
  },
  'CVE-WEB3-FORK-REWARD-STAKING-DILUTION': {
    attack: "Dilution instantanée de récompenses par flash-loan : distribution ou harvest de rewards lors de deposit/withdraw sans actualisation préalable de l'accumulateur accRewardPerShare et de lastRewardBlock, permettant de siphonner le yield en 1 bloc.",
    shield: "Exécuter systématiquement updatePool() pour actualiser l'accumulateur et le block checkpoint AVANT toute mutation de balance ou distribution de récompenses."
  },
  'DEFI-ASYMMETRIC-ROUNDING': {
    attack: "Arrondi asymétrique favorable à l'attaquant : utiliser une division entière brute dans convertToShares/convertToAssets ou les taxes économiques permet à un attaquant de voler des micro-parts (Dust Inflation) ou d'évader les frais en tronquant les montants à 0.",
    shield: "Utiliser OpenZeppelin Math.mulDiv avec Math.Rounding.Floor sur les dépôts et Math.Rounding.Ceil sur les retraits, ou imposer un minimum de 1 wei / part (.max(1))."
  },
  'DEFI-STALE-CACHE-AFTER-CALL': {
    attack: "Course TOCTOU & État périmé post-appel externe : réutiliser un snapshot local de solde ou de cache (VaultCache, local_balance) après un appel externe (.call, HookTarget, .await) expose le contrat à des décisions prises sur un état obsolète après modification par l'appelé.",
    shield: "Invalider ou recharger impérativement le cache (ex: vaultCache = _loadVaultCache() ou local_balance = fetch()) immédiatement après tout appel externe."
  },
  'EVC-HOOK-BYPASS': {
    attack: "Contournement de collatéral & Réentrance de batch EVC : effectuer des mutations d'état ou transferts dans un hook/callback sans vérifier les contrôles différés (deferLiquidityCheck) permet à un attaquant de vider les collatéraux avant la validation de solvabilité globale.",
    shield: "Protéger par nonReentrant et vérifier formellement l'état de l'EVC via require(!evc.isChecksDeferred(account) || evc.areChecksInProgress(), 'ChecksDeferred active')."
  },
  'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER': {
    attack: "Plancher artificiel d'oracle lors d'un crash de token : Chainlink clamp les flux à minAnswer en cas d'effondrement extrême (ex: 0.10$ au lieu de 0.00001$). L'absence de contrôle de bornes permet d'emprunter tous les fonds du coffre avec un collatéral sans valeur.",
    shield: "Interroger les bornes minAnswer/maxAnswer de l'agrégateur et basculer sur un circuit breaker ou oracle de secours dès que le prix percute ces bornes."
  },
  'DEFI-ORACLE-L2-SEQUENCER-RACE': {
    attack: "Course à la liquidation après panne de séquenceur L2 (Arbitrum/Base/Optimism) : après une panne de séquenceur, les prix ont divergé hors-chaîne. Exécuter les liquidations immédiatement au premier bloc de reprise sans période de grâce liquide massivement des utilisateurs innocents.",
    shield: "Interroger le sequencerUptimeFeed et exiger formellement require(answer == 0 && block.timestamp - startedAt > GRACE_PERIOD_TIME, 'Période de grâce L2 active')."
  },
  'DEFI-TWAP-SHORT-WINDOW': {
    attack: "Manipulation d'oracle TWAP AMM par MEV multi-blocs : une fenêtre TWAP inférieure à 30 minutes (< 1800s) permet à des attaquants avec pots-de-vin aux builders d'altérer la moyenne géométrique des prix sur 2 ou 3 blocs consécutifs pour liquider un vault à bas coût.",
    shield: "Imposer un plancher strict twapWindow >= 1800 (au minimum 30 minutes) pour rendre la manipulation économique mathématiquement déficitaire."
  },

  // 🎮 MMO, Concurrence & Boucle de Jeu
  'CVE-RUST-ASYNC-MUTEX': {
    attack: "Deadlock asynchrone : conserver un garde std::sync::Mutex across .await empêche le thread Tokio d'être libéré quand la tâche suspend, gelant la boucle d'événements.",
    shield: "Utiliser tokio::sync::Mutex ou structurer le code pour libérer le MutexGuard avant le point .await."
  },
  'CVE-RUST-UNBOUNDED-CHANNEL': {
    attack: "Déni de service mémoire (OOM) : un canal unbounded_channel() sans limite de capacité permet à un client d'empiler des millions de messages en RAM sans aucune contre-pression (Backpressure).",
    shield: "Utiliser tokio::sync::mpsc::channel(CAPACITY) borné et bloquer ou rejeter les envois excédentaires."
  },
  'CVE-MMO-UNCHECKED-SPEEDHACK': {
    attack: "Téléportation & Speedhack : appliquer directement les coordonnées client (x, y) permet aux clients modifiés de traverser les murs et se déplacer à une vitesse illimitée.",
    shield: "Vérifier le delta de déplacement côté serveur et plafonner strictement avec clamp(-MAX_SPEED * dt, MAX_SPEED * dt)."
  },
  'CVE-MATH-UNCHECKED-BALANCE': {
    attack: "Underflow d'inventaire / solde : décrémenter un entier non signé (balance -= cost) sans contrôle provoque un dépassement vers 2^64-1 en release, créditant des milliards de monnaie au joueur.",
    shield: "Utiliser checked_sub(cost).ok_or(...) ou saturating_sub(cost) pour empêcher tout wrapping."
  },
  'CVE-MMO-IDOR-TRUSTED-USERID': {
    attack: "Usurpation d'identité joueur (IDOR) : faire confiance au userId fourni dans le JSON client permet à un joueur de modifier les coordonnées, l'inventaire ou bannir un autre joueur en changeant l'ID.",
    shield: "Résoudre impérativement l'identité du joueur depuis le jeton Bearer dans la session serveur authentifiée."
  },

  // 🗄️ SAST & Injections Fondamentales
  'SAST_RAW_SQL_INJECTION': {
    attack: "Injection SQL directe : concaténer des chaînes dans une requête permet à l'attaquant d'injecter du SQL pour lire, modifier ou détruire l'intégralité de la base de données.",
    shield: "Utiliser systématiquement des requêtes préparées avec des variables de liaison paramétrées (? ou $1)."
  },
  'SAST_HARDCODED_SECRET': {
    attack: "Fuite d'identifiants critiques : stocker une clé d'API ou clé privée en clair dans le code expose l'infrastructure en cas de fuite du repository.",
    shield: "Déporter tous les identifiants dans un gestionnaire de secrets ou des variables d'environnement chiffrées."
  },
  'CVE-DB-REDIS-UNAUTH-KEYS-DOS': {
    attack: "Déni de service O(N) Redis & Fuite de session : la commande KEYS * bloque le thread unique de Redis pendant plusieurs secondes et un socket RESP non protégé permet de lire toutes les sessions.",
    shield: "Exiger AUTH avant toute commande, restreindre l'écoute à localhost et remplacer KEYS par SCAN."
  },
  'CVE-RUST-UNSAFE-MEM': {
    attack: "Corruption de mémoire / RCE : les blocs unsafe et pointeurs bruts brisent la garantie de sûreté mémoire Rust, ouvrant à des corruptions de pile et exécutions de code arbitraire.",
    shield: "Éliminer les blocs unsafe et utiliser des structures sûres (Box, Arc, Mutex, NonNull validé)."
  },
  'CVE-RUST-PANIC-DOS': {
    attack: "Crash serveur par panic : appeler .unwrap() sur une entrée utilisateur fait paniquer le thread serveur lorsqu'une valeur nulle ou invalide est injectée.",
    shield: "Remplacer unwrap() par l'opérateur ? ou match pour renvoyer une erreur HTTP 400 propre."
  },
  'CVE-RUST-CMD-INJ': {
    attack: "Injection de commande OS : exécuter un binaire avec des paramètres dynamiques non filtrés permet à l'attaquant d'exécuter des scripts shell sur la machine hôte.",
    shield: "Valider les arguments contre une liste blanche stricte et ne jamais invoquer d'interpréteur shell (sh, bash, cmd)."
  },
  'CVE-PY-DESERIALIZE-RCE': {
    attack: "Exécution de code arbitraire à distance (RCE) : désérialiser avec pickle.loads() permet à l'attaquant de construire un objet exécutant du code OS dès la lecture.",
    shield: "Interdire pickle pour les flux distants et utiliser json.loads() ou des schémas Pydantic typés."
  },
  'CVE-PY-SHELL-INJ': {
    attack: "Injection shell Python : utiliser subprocess avec shell=True ou os.system() permet d'injecter des commandes additionnelles via des séparateurs (; ou &&).",
    shield: "Définir shell=False et transmettre la commande sous forme de liste d'arguments tokenisés [\"cmd\", arg]."
  },
  'CVE-PY-SSRF-UNVALIDATED': {
    attack: "SSRF sortante Python : envoyer des requêtes requests.get() vers des URL arbitraires permet de scanner le réseau local d'entreprise et d'accéder aux métadonnées cloud privées.",
    shield: "Valider l'URL contre une liste blanche explicite et bloquer les plages IP privées (10.0.0.0/8, 127.0.0.1, 169.254.169.254)."
  },
  'CVE-PY-UNTYPED-PAYLOAD': {
    attack: "Injection d'attributs non contrôlés : recevoir un dict brut dans une route API empêche la validation de type et permet d'injecter des champs de privilèges inattendus.",
    shield: "Typer strictement le corps de la requête avec un modèle Pydantic v2 validé."
  },
  'CVE-PY-PATH-TRAVERSAL-ARCHIVE': {
    attack: "Path Traversal & ZipSlip : extraire une archive ou ouvrir un fichier dynamique permet d'écrire des fichiers hors du dossier cible via des séquences ../../ (RCE potentielle).",
    shield: "Résoudre avec Path.resolve() et vérifier strictement target.is_relative_to(BASE_DIR)."
  },

  // 🏛️ Architecture & Intégrité BLAKE3
  'BLAKE3_TAMPERED': {
    attack: "Modification clandestine de contrat : le fichier README ou contrat de module a été altéré sans recalcul de son empreinte cryptographique dans aether.lock.toml.",
    shield: "Revalider et signer l'architecture avec l'empreinte BLAKE3 officielle."
  },
  'RULE_FORBIDDEN_IMPORT': {
    attack: "Franchissement illégal de frontière : un composant interne importe un module interdit (ex: accès direct à la DB depuis le client UI), brisant l'isolation.",
    shield: "Supprimer l'import direct et passer par les contrats d'API autorisés."
  },
  'RULE_MAX_LINES': {
    attack: "Complexité excessive & opacité : un fichier dépassant la limite de lignes augmente considérablement le risque d'angles morts de sécurité non audités.",
    shield: "Refactoriser et découper le module en sous-services modulaires respectant les contrats."
  },
  'SAST_DIRECT_ENV_ACCESS': {
    attack: "Fuite de configuration dans le domaine : lire process.env dans la logique métier pure crée un couplage caché avec l'environnement d'exécution de l'hôte.",
    shield: "Injecter les variables de configuration depuis le bootstrap ou la couche service."
  },

  // 🟢 Front-End PixiJS / Node.js / TypeScript
  'CVE-JS-EVENTLOOP-REDOS-JSON': {
    attack: "Blocage de l'Event-Loop Node.js : accumuler des streams HTTP sans limite de taille ou évaluer des regex vulnérables sature la mémoire heap V8 et gèle toutes les connexions.",
    shield: "Fixer un plafond strict sur les chunks (req.destroy() si > 64 KiB) et bannir les expressions régulières complexes."
  },
  'CVE-JS-XSS-DOM': {
    attack: "Cross-Site Scripting (XSS) : injecter du HTML brut via innerHTML permet d'exécuter des scripts dans le navigateur des utilisateurs pour voler leurs sessions et jetons.",
    shield: "Utiliser textContent, les éléments React/DOM sûrs ou assainir systématiquement avec DOMPurify."
  },
  'CVE-JS-PROTO-POLLUTION': {
    attack: "Pollution de Prototype JavaScript : modifier Object.prototype via un payload non validé injecte des propriétés globales pouvant mener à des contournements d'auth ou une RCE.",
    shield: "Valider strictement les charges utiles avec un schéma Zod et geler Object.prototype."
  },
  'CVE-JS-CMD-INJ': {
    attack: "Injection de commande shell Node.js : child_process.exec() invoque un shell système sensible aux métacaractères injectés par un attaquant.",
    shield: "Utiliser child_process.execFile() ou spawn() avec shell: false."
  },
  'CVE-JS-CODE-INJ': {
    attack: "Évaluation dynamique de code arbitraire : l'utilisation de eval() ou new Function() permet à l'attaquant d'exécuter du JavaScript arbitraire avec tous les droits de l'application.",
    shield: "Supprimer eval() et traiter les données sous forme de structures JSON sécurisées."
  },
  'CVE-CLIENT-UNSAFE-ASSETS': {
    attack: "Empoisonnement de ressources WebGL : charger des textures PixiJS/WebGL dynamiques sans validation d'origine permet à un attaquant de forcer le client à charger des payloads malveillants.",
    shield: "Vérifier que l'URL d'asset provient obligatoirement du CDN ou de la passerelle IPFS approuvée."
  },

  // ⚙️ C / C++ FFI
  'CVE-CPP-UNBOUNDED-COPY-UAF': {
    attack: "Dépassement de tampon mémoire C/C++ : copier des données avec strcpy, sprintf ou gets sans vérifier la taille écrase la pile ou le tas et permet le détournement du flux d'exécution (RCE).",
    shield: "Utiliser snprintf(buf, sizeof(buf), ...) et remplacer les pointeurs bruts par std::string ou std::span."
  },
  // ⚙️ Top 10 C++ Network & Protocol Hard-to-Find CWEs
  'CVE-CPP-NET-INTEGER-OVERFLOW-ALLOC': {
    attack: "Dépassement d'entier sur taille de trame réseau : additionner la taille de l'en-tête (len + sizeof(Header)) avant allocation sans garde overflow enveloppe l'entier (UINT32_MAX) allouant un buffer minuscule puis corrompt le heap lors du recv/memcpy.",
    shield: "Valider len < MAX_PACKET_SIZE - sizeof(Header) ou utiliser __builtin_add_overflow avant allocation."
  },
  'CVE-CPP-NET-TYPE-PUNNING-STRICT-ALIASING': {
    attack: "Violation de Strict Aliasing sur paquet réseau : convertir un tampon d'octets brut (char*) en pointeur de structure via reinterpret_cast provoque des optimisations destructrices du compilateur (-O3) et des crashs d'alignement SIGBUS sur ARM/x86.",
    shield: "Désérialiser via std::memcpy(&header, buf, sizeof(Header)) ou std::bit_cast<Header> (C++20)."
  },
  'CVE-CPP-NET-ASYNC-CALLBACK-UAF': {
    attack: "Use-After-Free en callback asynchrone : capturer [this] dans async_read ou async_write sans shared_from_this() déclenche un accès à un objet détruit si la connexion réseau est close avant la fin de l'opération d'I/O.",
    shield: "Hériter de std::enable_shared_from_this et capturer auto self = shared_from_this() ou weak_from_this() dans la lambda."
  },
  'CVE-CPP-NET-PARTIAL-READ-SHORT-FRAME': {
    attack: "Désynchronisation de trame streaming TCP : un appel unique recv() sans boucle d'accumulation suppose une trame complète ; la fragmentation TCP injecte des paquets partiels et entraîne la désynchronisation du protocole.",
    shield: "Boucler tant que la trame n'est pas complète (while (total < expected)) ou utiliser boost::asio::read."
  },
  'CVE-CPP-NET-SOCKET-FD-LEAK-EXCEPTION': {
    attack: "Fuite de descripteur socket sans RAII : allouer un int sock = socket() brut dans une fonction susceptible de lever une exception C++ ou d'échouer sans close() épuise la table des descripteurs de l'OS (EMFILE DoS).",
    shield: "Encapsuler le descripteur dans un guard RAII (UniqueFd / std::unique_ptr avec SocketCloser)."
  },
  'CVE-CPP-NET-HOST-BYTE-ORDER-CONFUSION': {
    attack: "Confusion d'endianness sur longueur réseau : lire un champ multi-octets (header.length) sans ntohs() ou ntohl() inverse l'ordre des octets sur machine little-endian, contournant les filtres de bornes.",
    shield: "Convertir systématiquement les champs d'en-tête avec ntohs() (16-bit) ou ntohl() (32-bit) avant tout test."
  },
  'CVE-CPP-NET-STRING-NULL-TERMINATION-OFF-BY-ONE': {
    attack: "Absence de terminateur nul sur chaîne reçue du réseau : passer un buffer recv() directement à strlen() ou std::string sans forcer buf[n] = '\0' provoque une fuite d'informations mémoires adjacentes (Heartbleed-like).",
    shield: "Allouer 1 octet de plus et forcer buf[bytes_received] = '\0' ou construire std::string(buf, bytes_received)."
  },
  'CVE-CPP-NET-FORMAT-STRING-LOGGING': {
    attack: "Format String non contrôlé sur log réseau : transmettre une chaîne issue du réseau directement à syslog() ou printf() sans format \"%s\" permet l'exécution de code arbitraire via %n et la fuite mémoire via %x.",
    shield: "Toujours utiliser un littéral de format fixe : printf(\"%s\", net_str) ou syslog(LOG_ERR, \"%s\", net_str)."
  },
  'CVE-CPP-NET-SOCKET-RACE-DOUBLE-CLOSE': {
    attack: "Course critique et double-close de socket multi-thread : fermer close(fd) sans verrou atomique exchange(-1) permet à un thread concurrent de fermer par erreur un nouveau descripteur réalloué à un autre client.",
    shield: "Utiliser un descripteur atomique std::atomic<int> avec m_fd.exchange(-1) ou un verrou std::lock_guard."
  },
  'CVE-CPP-NET-UNBOUNDED-RECURSION-TLV-PARSER': {
    attack: "Déni de service par récursion non bornée : un parseur réseau récursif (TLV/JSON/arbre) sans compteur de profondeur maximale consomme toute la pile du thread lors de l'envoi d'une trame imbriquée (SIGSEGV).",
    shield: "Transmettre un paramètre size_t depth = 0 et rejeter la trame si depth >= MAX_RECURSION_DEPTH."
  },

  // 🐍 Nouvelles Règles Python V2
  'CVE-PY-JINJA2-SSTI-INJECTION': {
    attack: "Injection de template côté serveur (Jinja2 SSTI) : formater des chaînes dynamiques (f-string ou +) dans render_template_string() ou jinja2.Template() permet à l'attaquant d'exécuter du code arbitraire en escaladant la hiérarchie d'objets MRO de Python (__subclasses__).",
    shield: "Transmettre les variables dans le dictionnaire de contexte render_template_string(TEMPLATE, user_val=val) sans altérer la chaîne de template."
  },
  'CVE-PY-YAML-UNSAFE-LOAD': {
    attack: "Désérialisation non sécurisée PyYAML : yaml.load() sans SafeLoader ou yaml.unsafe_load() instancie n'importe quel constructeur d'objet Python (ex: !!python/object/apply:os.system), octroyant une RCE instantanée.",
    shield: "Utiliser impérativement yaml.safe_load(data) ou yaml.load(data, Loader=yaml.SafeLoader)."
  },
  'CVE-PY-SQL-ALCHEMY-TEXT-INJECTION': {
    attack: "Injection SQL dans SQLAlchemy text() : concaténer des chaînes brutes dans sqlalchemy.text() contourne la protection ORM et permet à un attaquant d'injecter des requêtes destructrices (DROP/UPDATE/UNION).",
    shield: "Utiliser des paramètres liés : text('SELECT * FROM users WHERE id = :id').bindparams(id=val)."
  },
  'CVE-PY-JWT-NONE-ALGORITHM': {
    attack: "Contournement d'authentification JWT (alg: none) : décoder un jeton avec verify=False ou accepter l'algorithme none permet à un attaquant de modifier le payload (userId, admin: true) sans signer le token.",
    shield: "Exiger une clé secrète et contraindre explicitement les algorithmes : jwt.decode(token, SECRET, algorithms=['HS256'])."
  },
  'CVE-PY-PARAMIKO-MISSING-HOST-KEY': {
    attack: "Attaque Man-In-The-Middle SSH : configurer Paramiko avec AutoAddPolicy() accepte aveuglément toute clé d'hôte SSH inconnue, permettant à un routeur intermédiaire malveillant d'intercepter identifiants et commandes.",
    shield: "Utiliser paramiko.RejectPolicy() et charger les clés autorisées du système via client.load_system_host_keys()."
  },
  'CVE-PY-XML-XXE-ENTITY-EXPANSION': {
    attack: "Injection d'entités externes XML (XXE) : parser du XML non fiable avec ElementTree ou minidom sans protection résout les entités externes DTD, permettant de lire des fichiers locaux (/etc/passwd) ou de lancer un déni de service Billion Laughs.",
    shield: "Utiliser la bibliothèque defusedxml (defusedxml.ElementTree / defusedxml.minidom) ou désactiver resolve_entities."
  },

  // 🦀 Nouvelles Règles Rust V2
  'CVE-RUST-INTEGER-OVERFLOW-CAST': {
    attack: "Troncature silencieuse de conversion 'as' : convertir un entier 64-bit vers u32/u16 avec 'as' sans contrôle de bornes tronque silencieusement les bits de poids fort en release (ex: 2^32 + 100 devient 100), faussant la taille des paquets ou les soldes.",
    shield: "Remplacer les 'as' casts par u32::try_from(val).map_err(...) ou .try_into() avec gestion explicite d'erreur."
  },
  'CVE-RUST-TRANSMUTE-SIZE-MISMATCH': {
    attack: "Confusion de types par std::mem::transmute : transmuter arbitrairement des structures de données en mémoire unsafe sans assertion de taille et d'alignement déclenche un Comportement Indéfini (UB) et une corruption de mémoire.",
    shield: "Utiliser des abstractions sûres (bytemuck::cast, From/Into, TryFrom, ou conversion endianess from_le_bytes)."
  },
  'CVE-RUST-FORGOTTEN-MEM-LEAK': {
    attack: "Contournement RAII & Deadlock par std::mem::forget : oublier volontairement ou par erreur un verrou (MutexGuard) ou descripteur de fichier empêche le destructeur Drop de libérer la ressource, bloquant définitivement les threads suivants.",
    shield: "Laisser le destructeur RAII s'exécuter ou appeler explicitement drop(guard) au lieu de mem::forget."
  },
  'CVE-RUST-UNCHECKED-INDEX-SLICING': {
    attack: "Panique DoS par indexation directe de tranche : découper un buffer réseau brut avec packet[offset..offset+len] sans contrôle préalable fait paniquer le thread Tokio dès qu'un paquet tronqué est reçu.",
    shield: "Utiliser packet.get(offset..offset+len).ok_or(PacketError::Truncated) pour valider les bornes sans panique."
  },
  'CVE-RUST-THREAD-POISON-IGNORED': {
    attack: "Invariants d'état corrompus par Mutex empoisonné : avaler PoisonError avec into_inner() sans réparer les données partagées expose le serveur à des corruptions logiques en cascade après la panique d'un thread.",
    shield: "Traiter PoisonError comme une erreur fatale ou réinitialiser/valider formellement les invariants de la structure partagée."
  },
  'CVE-RUST-FFI-UNCHECKED-NULL-PTR': {
    attack: "Déréférencement FFI de pointeur nul : déréférencer un pointeur brut (*const T) reçu d'une bibliothèque native C sans test ptr.is_null() provoque un crash immédiat par segmentation fault (SIGSEGV).",
    shield: "Tester systématiquement ptr.is_null() ou convertir via NonNull::new(ptr).ok_or(FfiError::NullPointer)."
  },

  // 🟢 Nouvelles Règles Node.js / TypeScript V2
  'CVE-JS-VM-SANDBOX-ESCAPE': {
    attack: "Évasion de Sandbox Node.js vm : le module vm de Node.js n'est pas un bac à sable de sécurité. Le code exécuté peut remonter au constructeur Function global et invoquer child_process pour obtenir une RCE complète sur l'hôte.",
    shield: "Ne jamais utiliser le module vm pour du code non fiable ; préférer isolated-vm ou un runtime WebAssembly."
  },
  'CVE-JS-SSRF-DYNAMIC-AXIOS-FETCH': {
    attack: "SSRF sortant via requête HTTP dynamique : envoyer une requête axios/fetch vers une URL fournie par l'utilisateur permet à un attaquant de sonder le réseau interne et d'exfiltrer les métadonnées AWS/GCP (169.254.169.254).",
    shield: "Valider le domaine contre une liste blanche stricte et bloquer les plages IP privées et de métadonnées (127.0.0.1, 169.254.169.254)."
  },
  'CVE-JS-PATH-TRAVERSAL-SENDFILE': {
    attack: "Lecture arbitraire de fichiers système via sendFile : passer un chemin utilisateur non confiné à res.sendFile() ou fs.readFile() permet d'accéder à des fichiers système sensibles via des séquences ../.",
    shield: "Définir l'option { root: PUBLIC_DIR, dotfiles: 'deny' } dans res.sendFile() ou vérifier que path.resolve(p).startsWith(PUBLIC_DIR)."
  },
  'CVE-JS-REGEX-DOS-DYNAMIC-INPUT': {
    attack: "Déni de service ReDoS par RegExp dynamique : instancier new RegExp() directement avec des données utilisateurs permet d'injecter des expressions à rebroussement catastrophique qui gèlent l'Event-Loop Node.js.",
    shield: "Échapper systématiquement les caractères spéciaux avec lodash.escapeRegExp ou utiliser un moteur de regex linéaire (re2)."
  },
  'CVE-JS-CORS-WILDCARD-CREDENTIALS': {
    attack: "Vol de session par CORS permissif : configurer CORS avec credentials: true tout en reflétant l'Origin client (origin: true) permet à tout site malveillant de lire des données privées en utilisant les cookies de session des utilisateurs.",
    shield: "Définir une liste blanche stricte d'origines autorisées dans la configuration CORS au lieu de refléter la requête client."
  },
  'CVE-JS-UNHANDLED-PROMISE-CRASH': {
    attack: "Crash serveur par rejet de promesse non intercepté : une exception non gérée dans une route Express async provoque un UnhandledPromiseRejection qui tue instantanément le processus Node.js sous Node 15+.",
    shield: "Envelopper le code de route async dans un bloc try/catch appelant next(err), ou importer express-async-errors."
  },

  // 💎 4 Règles Web3 Next-Gen Ultra-Critiques & Inédites (EVM Cancun / Pectra / AA / Omnichain)
  'CVE-WEB3-EIP1153-TRANSIENT-COMPOSABILITY-LEAK': {
    attack: "Pollution de mémoire transitoire EIP-1153 (Dirty TSTORE) : contrairement à la pile d'appels, l'opcode tstore persiste jusqu'à la fin de la transaction entière. Omettre de remettre le slot à 0 (tstore(slot, 0)) avant la sortie permet à un second appel dans un multicall/batch/bundler d'hériter de l'autorisation transitoire ou du delta de Flash Accounting du premier appel.",
    shield: "Nettoyer impérativement chaque slot transitoire (tstore(slot, 0) ou delete transientVar) en sortie de fonction (pattern try/finally ou modifier post-hook)."
  },
  'CVE-WEB3-EIP7702-EOA-CODE-BYPASS-COLLISION': {
    attack: "Contournement EOA EIP-7702 & Collision de Slot-0 : vérifier msg.sender.code.length == 0 ou extcodesize == 0 pour bloquer les bots/flashloans est contourné par les constructeurs et par la délégation EOA EIP-7702 (0xef0100). De plus, un contrat délégué sans stockage à espace de noms ERC-7201 écrase le slot 0 lors d'une re-délégation.",
    shield: "Supprimer les gardes extcodesize/code.length == 0 et isoler tout stockage de compte délégué via le standard ERC-7201 (@custom:storage-location erc7201:...)."
  },
  'CVE-WEB3-ERC4337-PAYMASTER-GAS-SIPHON': {
    attack: "Siphonnage de trésorerie Paymaster ERC-4337 : si le Paymaster valide validatePaymasterUserOp sans borner maxCost/postOpGasLimit ou attend _postOp pour prélever les tokens ERC-20 via transferFrom, l'attaquant peut faire réverter l'exécution interne tout en révoquant son allowance, forçant le Paymaster à payer l'ETH de gaz à l'EntryPoint à fonds perdus.",
    shield: "Pré-séquestrer (pre-charge) le coût maximal en tokens ERC-20 dès validatePaymasterUserOp, rembourser le surplus dans _postOp et borner strictement maxCost."
  },
  'CVE-WEB3-OMNICHAIN-LZ-CHANNEL-BRICKING': {
    attack: "Destruction de fonds brûlés & Paralysie de canal Cross-Chain (_lzReceive / _ccipReceive) : lorsque les tokens sont brûlés sur la chaîne source, tout revert non intercepté dans _lzReceive (destinataire USDC blacklisté, hook qui revert, limite atteinte) bloque le canal ordonné ou détruit définitivement les fonds en transit.",
    shield: "Isoler l'exécution du message dans un sous-appel externe try/catch et stocker les payloads en échec dans failedMessages[guid] = keccak256(payload) pour rejeu manuel (Stored Payload Pattern)."
  },

  // 🏆 6 Masterclass Deep-Core & Zero-Day Class (EIP-150, ZK-SNARK BN254, Merkle 64B, CREATE2, Rust Pin, V8 Heap)
  'CVE-WEB3-EIP150-63-64-GAS-STIPEND-GRIEFING': {
    attack: "Sabotage silencieux EIP-150 (Règle des 63/64 de gaz) : l'EVM ne transmet que 63/64 du gaz restant à un sous-appel (.call / try-catch). Un relayer malveillant calibre le gasLimit pour que le sous-appel échoue par Out-Of-Gas, tandis que le contrat parent survit avec le 1/64 restant, consomme définitivement le nonce de la victime et empoche la récompense.",
    shield: "Vérifier impérativement gasleft() après ou avant le sous-appel : require(gasleft() > (targetGas * 64) / 63 + GAS_BUFFER)."
  },
  'CVE-WEB3-ZK-SNARK-FIELD-OVERFLOW-ALIASING': {
    attack: "Double-Spend (5x Spend) de preuve ZK-SNARK sur BN254 : le corps scalaire de la courbe BN254 vaut r ≈ 2^254 alors qu'un uint256 EVM monte à 2^256 - 1. Sans require(input < SNARK_SCALAR_FIELD), l'attaquant soumet nullifier + r, nullifier + 2r... (jusqu'à 5 fois) : le circuit ZK réduit modulo r et valide la même preuve, tandis que le mapping Solidity voit 5 clés distinctes !",
    shield: "Forcer require(input[i] < 21888242871839275222246405745257275088548364400416034343698204186575808495617) sur chaque entrée publique et nullifier."
  },
  'CVE-WEB3-MERKLE-SECOND-PREIMAGE-LEAF-NODE-COLLISION': {
    attack: "Falsification d'arbre de Merkle par seconde pré-image (64 octets) : un nœud interne de Merkle est le hash de deux enfants de 32 octets (64 octets). Si une feuille est hachée via un simple keccak256(abi.encode(addr, amount)) de 64 octets sans double-hash, tout nœud interne de l'arbre peut être soumis comme une feuille d'airdrop valide !",
    shield: "Double-hacher systématiquement les feuilles : bytes32 leaf = keccak256(bytes.concat(keccak256(abi.encode(account, amount))))."
  },
  'CVE-WEB3-CREATE2-METAMORPHIC-REINIT-RUG': {
    attack: "Hijack de déploiement déterministe CREATE2 & Rug Métamorphique : si le salt CREATE2 n'inclut pas msg.sender, un bot mempool ou cross-chain peut front-runner le déploiement à la même adresse. De plus, un initCode qui charge son bytecode depuis un contrat externe permet de redéployer un code différent sous le même initCodeHash.",
    shield: "Dériver cryptographiquement le salt avec l'appelant : bytes32 safeSalt = keccak256(abi.encode(msg.sender, userSalt))."
  },
  'CVE-RUST-PIN-STRUCTURAL-UNPIN-SOUNDNESS-UB': {
    attack: "Rupture de l'invariant de Pinning asynchrone (Self-Referential UAF) : projeter un Pin<&mut Self> via Pin::get_unchecked_mut() sans pin_project! permet de déplacer (mem::swap / replace) une Future auto-référentielle entre deux points .await, faisant pointer la machine à états vers une adresse mémoire désallouée.",
    shield: "Utiliser exclusivement #[pin_project] (pin-project-lite) ou tokio::pin!() et interdire Pin::get_unchecked_mut manuel."
  },
  'CVE-JS-V8-TYPEDARRAY-SHARED-AB-RACE-OOB': {
    attack: "Nodebleed (Fuite de Heap V8 via Buffer.allocUnsafe) & Race Condition TOCTOU sur SharedArrayBuffer : Buffer.allocUnsafe expose des fragments bruts de RAM V8 contenant clés JWT et secrets, et lire un SharedArrayBuffer sans Atomics.load permet à un Worker Thread de modifier la taille du paquet entre le test de bornes et la lecture.",
    shield: "Remplacer Buffer.allocUnsafe par Buffer.alloc(size) et utiliser Atomics.load() avant de copier dans un ArrayBuffer privé."
  },

  // ⛓️ 6 Règles L1 Blockchain & Consensus Distribué
  'CVE-L1-BORSH-UNBOUNDED-DESER-OOM': {
    attack: "[P2P & Consensus Flaws] Bombe OOM par désérialisation Borsh / décompression Zstd non bornée : un pair P2P malveillant envoie une trame contenant un préfixe de longueur u32 forgé (ex: Vec::with_capacity(4 Go)) ou une bombe zstd. L'allocateur Rust/Jemalloc tente d'allouer la mémoire avant la fin du parsing et tue instantanément le nœud validateur (OOM Crash).",
    shield: "Vérifier raw_bytes.len() <= MAX_NETWORK_FRAME_BYTES avant BorshDeserialize::try_from_slice et utiliser zstd::stream::read::Decoder borné par .take(MAX_DECOMPRESSED_SIZE)."
  },
  'CVE-L1-RUNTIME-GAS-PREPAY-BYPASS': {
    attack: "[Fee Payment Bypass & Contracts Execution Flows] Contournement du prépaiement de gaz dans une Host Function Wasm : si le runtime alloue un vecteur sur le tas ou lit la mémoire linéaire Wasm avant d'appeler gas_counter.burn_gas(), un contrat malveillant peut invoquer l'import en boucle avec des tailles géantes, provoquer un OOM validateur puis déclencher GuestPanic pour éviter de payer le gaz.",
    shield: "Facturer systématiquement gas_counter.burn_gas(base_fee + len * byte_fee)? et vérifier len <= max_register_size AVANT toute allocation mémoire hôte."
  },
  'CVE-L1-BALANCE-STORAGE-STAKING-DESYNC': {
    attack: "[Stealing/Loss of Funds & Balance Manipulation] Désynchronisation Solde / Storage Staking & Soustraction Saturante : utiliser saturating_sub sur un solde de compte absorbe silencieusement un découvert (création monétaire), et muter set_amount() / set_locked() sans check_storage_stake() permet d'occuper l'état RocksDB gratuitement ou de verrouiller des jetons inexistants.",
    shield: "Utiliser exclusivement checked_sub().ok_or(BalanceOverflowError)? et exécuter check_storage_stake(&account, &runtime_config) à la sortie de chaque action."
  },
  'CVE-L1-CONSENSUS-UNVERIFIED-WITNESS-CHUNK': {
    attack: "[Consensus Flaws] Empoisonnement de consensus par ChunkStateWitness / PartialEncodedChunk non authentifié : décoder un témoin d'état (Stateless Validation) ou l'insérer dans l'Orphan Pool avant de vérifier la signature Ed25519/BLS du producteur de chunk dans l'EpochManager permet à un nœud Byzantin de saturer le CPU et la RAM des validateurs.",
    shield: "Valider epoch_manager.verify_chunk_state_witness_signature(&witness)? en tout premier point d'entrée avant toute décompression du trie ou mise en cache."
  },
  'CVE-L1-CROSS-SHARD-RECEIPT-NONCE-REPLAY': {
    attack: "[Unauthorized Transaction & Transaction Manipulation] Rejeu de Meta-Transaction NEP-366 (DelegateAction) & Double-Exécution Cross-Shard : valider une DelegateAction sans imposer block_height <= max_block_height ni incrémenter atomiquement access_key.nonce avant l'émission du ActionReceipt permet à un relayer de rejouer l'autorisation sur plusieurs shards.",
    shield: "Vérifier block_height <= delegate_action.max_block_height, valider nonce > access_key.nonce et persister immédiatement le nouveau nonce dans le Trie avant de créer le Receipt."
  },
  'CVE-L1-CRYPTO-BLIND-SCALAR-MERKLE-MALLEABILITY': {
    attack: "[Cryptographic Flaws] Malléabilité de sous-groupe (Small-Subgroup / Cofactor Attack) & Décompression de point non vérifiée : utiliser decompress_unchecked() sur Curve25519/BLS12-381 ou accepter une signature Ed25519 non stricte permet d'injecter un point de torsion d'ordre 8, produisant plusieurs signatures/VRF valides pour le même bloc et faisant diverger l'état entre validateurs.",
    shield: "Interdire *_unchecked() sur les clés externes, utiliser verify_strict() pour Ed25519 et vérifier point.is_torsion_free() (appartenance au sous-groupe d'ordre premier)."
  },
  // 🔬 7 Règles Rust Ultra-Bas-Niveau (LLVM Backend, Unwind UB, MaybeUninit, GlobalAlloc, FFI ABI, Noalias, Mmap SIGBUS)
  'CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK': {
    attack: "Élimination de magasin mort par l'optimiseur LLVM (Dead Store Elimination) : effacer une clé privée ou un seed en mémoire avec fill(0) ou write_bytes(..., 0) juste avant son drop est supprimé silencieusement par LLVM en --release, laissant le secret intact dans la pile ou la heap.",
    shield: "Utiliser std::ptr::write_volatileaccompagnée de std::sync::atomic::compiler_fence(Ordering::SeqCst), ou implémenter zeroize::Zeroize / ZeroizeOnDrop."
  },
  'CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB': {
    attack: "Double-Free & Use-After-Free par déroulement de pile (Panic Unwind Exception Safety) : après ptr::read() ou set_len(), si un appel intermédiaire panique avant la restauration des invariants du conteneur, le destructeur Drop s'exécute pendant l'unwinding et libère une seconde fois la même mémoire.",
    shield: "Protéger la section critique par un Drop Guard RAII (ManuallyDrop / ScopeGuard) ou réduire set_len(0) AVANT l'opération susceptible de paniquer."
  },
  'CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB': {
    attack: "Comportement Indéfini (LLVM Poison UB) & Fuite d'octets de padding mémoire : MaybeUninit::uninit().assume_init() sur un type non-valide pour tout motif binaire déclenche un UB immédiat, et sérialiser une struct via from_raw_parts expose en réseau les octets d'alignement non initialisés de la pile.",
    shield: "Utiliser MaybeUninit::zeroed(), initialiser champ par champ via addr_of_mut!, ou dériver bytemuck::NoUninit / Pod."
  },
  'CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW': {
    attack: "Corruption de Heap via GlobalAlloc (Layout de taille 0 ou dépassement isize::MAX) : appeler std::alloc::alloc(layout) sans vérifier layout.size() > 0 est un UB direct en Rust, et calculer size * align sans checked_mul provoque un dépassement entier allouant un tampon minuscule.",
    shield: "Vérifier formellement layout.size() > 0 && layout.size() <= isize::MAX as usize et utiliser Layout::from_size_align(size, align)?."
  },
  'CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY': {
    attack: "Corruption d'ABI par panique traversant extern \"C\" (#[no_mangle]) : laisser une panique Rust remonter à travers une frontière FFI C/C++/JVM/Node.js sans catch_unwind ni extern \"C-unwind\" provoque un Comportement Indéfini immédiat dans le runtime appelant.",
    shield: "Envelopper le corps de toute fonction extern \"C\" #[no_mangle] dans std::panic::catch_unwind(AssertUnwindSafe(|| { ... }))."
  },
  'CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB': {
    attack: "Violation d'aliasing LLVM noalias via UnsafeCell::get() : déréférencer simultanément plusieurs &mut T depuis un même UnsafeCell sans synchronisation atomique viole la règle LLVM noalias et permet au compilateur de réordonner ou supprimer les écritures concurrentes.",
    shield: "Remplacer par Mutex<T>, RwLock<T>, Atomic*, ou manipuler uniquement des pointeurs bruts (*mut T) sans créer de référence &mut T partagée."
  },
  'CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS': {
    attack: "Crash matériel SIGBUS par troncature externe de fichier mappé (Mmap) : exposer un memmap2::Mmap sous forme de tranche &[u8] sur un fichier non verrouillé permet à un processus externe de tronquer le fichier, déclenchant un signal kernel SIGBUS fatal (non interceptable par catch_unwind) à la lecture.",
    shield: "Acquérir un verrou exclusif OS (flock / FileExt::lock_exclusive) avant Mmap::map et valider la taille avant chaque accès."
  },
  // 🐘 PHP & Web Security CWE Rules
  'CVE-PHP-SQL-INJECTION': {
    attack: "Injection SQL directe dans une requête non préparée : un attaquant injecte des clauses SQL (' OR '1'='1, UNION SELECT, DROP TABLE) pour court-circuiter l'authentification ou siphonner l'intégralité de la base de données.",
    shield: "Utiliser impérativement une requête préparée paramétrée ($pdo->prepare() avec execute([$param]) ou bindParam)."
  },
  'CVE-PHP-COMMAND-INJECTION': {
    attack: "Exécution de commandes système OS arbitraires : un attaquant injecte des séparateurs shell (; & | ` $) pour faire exécuter des commandes par le serveur web avec les privilèges de www-data.",
    shield: "Échapper chaque argument individuellement via escapeshellarg($arg) ou proscrire l'exécution shell au profit de fonctions natives PHP."
  },
  'CVE-PHP-EVAL-CODE-INJECTION': {
    attack: "Exécution de code PHP arbitraire à distance (RCE) via eval() ou create_function() : un attaquant soumet du code PHP brut qui est exécuté immédiatement par le moteur Zend.",
    shield: "Bannir tout appel à eval(), create_function() ou preg_replace('/.../e'). Utiliser des parseurs dédiés ou preg_replace_callback()."
  },
  'CVE-PHP-UNSERIALIZE-RCE': {
    attack: "Désérialisation non sécurisée d'objets (PHP Object Injection) : un attaquant fournit un flux d'objets sérialisés forgé déclenchant des méthodes magiques (__destruct, __wakeup) menant à un RCE par chaîne de gadgets POP.",
    shield: "Remplacer par json_decode($json, true) ou forcer unserialize($data, ['allowed_classes' => false])."
  },
  'CVE-PHP-FILE-INCLUSION-RFI-LFI': {
    attack: "Inclusion dynamique locale ou distante (LFI/RFI) : inclusion de fichiers arbitraires du serveur (/etc/passwd) ou de code externe via wrappers php://filter ou URLs distantes.",
    shield: "Restreindre strictement les fichiers inclus à une liste blanche statique fermée : in_array($module, ALLOWED_MODULES, true)."
  },
  'CVE-PHP-PATH-TRAVERSAL': {
    attack: "Traversée de répertoire (Path Traversal) : séquences ../../ permettant d'accéder, lire ou écraser des fichiers système en dehors de la racine web autorisée.",
    shield: "Appliquer basename($filename) pour éliminer les répertoires ou vérifier str_starts_with(realpath($path), realpath($base_dir))."
  },
  'CVE-PHP-XSS-UNESCAPED-OUTPUT': {
    attack: "Cross-Site Scripting (XSS) réfléchi ou stocké : injection de scripts JavaScript malveillants exécutés dans le navigateur des utilisateurs pour voler leurs cookies et jetons de session.",
    shield: "Neutraliser la sortie avec htmlspecialchars($str, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8')."
  },
  'CVE-PHP-SSRF-CURL-STREAM': {
    attack: "Server-Side Request Forgery (SSRF) : forcer le serveur à émettre des requêtes vers des services internes non exposés (127.0.0.1, réseau privé RFC 1918) ou métadonnées cloud (169.254.169.254).",
    shield: "Résoudre l'adresse IP de destination et valider avec filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE)."
  },
  'CVE-PHP-UNRESTRICTED-FILE-UPLOAD': {
    attack: "Téléversement de fichier non restreint : dépôt direct d'un webshell .php dans un dossier public exécutable, permettant la prise de contrôle totale du serveur.",
    shield: "Valider l'extension dans une liste blanche stricte (.jpg, .png) et générer un nom aléatoire sécurisé : bin2hex(random_bytes(16)) . '.' . $ext."
  },
  'CVE-PHP-XXE-LIBXML-INJECTION': {
    attack: "Injection d'entités externes XML (XXE) : inclusion d'entités SYSTEM menant à l'exfiltration de fichiers locaux ou au déni de service (Billion Laughs).",
    shield: "Désactiver les entités externes via libxml_disable_entity_loader(true) ou utiliser le flag LIBXML_NONET."
  },
  'CVE-PHP-SESSION-FIXATION-NO-REGEN': {
    attack: "Fixation de session : un attaquant force la victime à utiliser un identifiant de session connu d'avance pour s'approprier sa session une fois celle-ci authentifiée.",
    shield: "Appeler systématiquement session_regenerate_id(true) lors de l'authentification ou de l'élévation de privilèges."
  },
  'CVE-PHP-COOKIE-SENSITIVE-PLAINTEXT': {
    attack: "Émission de cookie sensible sans protection : cookie de session accessible via JavaScript (vol XSS) ou transmis en clair sur réseau non sécurisé.",
    shield: "Définir les attributs de sécurité : ['secure' => true, 'httponly' => true, 'samesite' => 'Strict']."
  },
  'CVE-PHP-FILE-LOCKING-TOCTOU': {
    attack: "Condition de course temporelle (TOCTOU) sur manipulation de fichier sensible : écriture concurrente menant à la corruption ou l'écrasement de données critiques.",
    shield: "Utiliser un verrouillage exclusif OS via flock($fp, LOCK_EX) ou le mode atomique fopen($path, 'x')."
  },
  'CVE-PHP-BROKEN-CRYPTO-MD5-SHA1': {
    attack: "Algorithme de hachage rapide cassé (MD5/SHA1) : vulnérable aux attaques par dictionnaire et collisions GPU permettant de retrouver les mots de passe en clair.",
    shield: "Employer l'API native password_hash($password, PASSWORD_ARGON2ID) ou PASSWORD_BCRYPT."
  },
  'CVE-PHP-HEADER-INJECTION-CRLF': {
    attack: "Injection CRLF et Open Redirect : caractères \r\n permettant de diviser la réponse HTTP (Response Splitting) ou de rediriger l'utilisateur vers un site de phishing.",
    shield: "Supprimer les caractères CRLF str_replace(['\r', '\n'], '', $url) et valider l'URL de redirection avec filter_var($url, FILTER_VALIDATE_URL)."
  },
  'CVE-PHP-VARIABLE-OVERWRITE-EXTRACT': {
    attack: "Écrasement arbitraire de variables (Variable Overwrite) : utilisation d'extract() sans EXTR_SKIP ou assignation dynamique $$ permettant d'écraser des variables internes critiques ($is_admin, $role).",
    shield: "Bannir extract() sur les entrées externes ou forcer le drapeau extract($_POST, EXTR_SKIP)."
  },
  'CVE-PHP-MAIL-PARAM-INJECTION': {
    attack: "Injection de paramètres Sendmail (RCE via mail()) : le 5ème argument de mail() non assaini permet d'injecter des drapeaux MTA (-X) pour déposer un webshell sur le serveur.",
    shield: "Échapper le 5ème paramètre via escapeshellcmd($extra) ou utiliser une bibliothèque SMTP moderne (Symfony Mailer)."
  },
  'CVE-PHP-TEMPLATE-INJECTION-SSTI': {
    attack: "Injection de modèle côté serveur (SSTI) : rendu dynamique de template Twig/Blade/Smarty avec une chaîne utilisateur permettant l'exécution de code arbitraire à distance (RCE).",
    shield: "Ne jamais compiler de code utilisateur en template ; transmettre les données en variables de contexte."
  },
  'CVE-PHP-DISABLED-SSL-VERIFY': {
    attack: "Désactivation du contrôle SSL/TLS (CURLOPT_SSL_VERIFYPEER à false) : expose toutes les requêtes réseau sortantes vers des APIs ou passerelles de paiement aux attaques Man-in-the-Middle (MitM).",
    shield: "Maintenir CURLOPT_SSL_VERIFYPEER à true et spécifier un bundle CA valide avec CURLOPT_CAINFO."
  },
  'CVE-PHP-TYPE-JUGGLING-AUTH': {
    attack: "Comparaison faible (Type Juggling) : comparaison non stricte (== ou strcmp == 0) sur un secret, mot de passe ou jeton permettant de contourner l'authentification (magic hashes 0e...).",
    shield: "Utiliser la comparaison à temps constant hash_equals($secret, $input) ou l'opérateur strict ===."
  },
  'CVE-PHP-WEAK-PRNG-SECRETS': {
    attack: "Générateur pseudo-aléatoire prévisible (rand, mt_rand, uniqid) pour des jetons sensibles : un attaquant peut reconstituer la graine et forger des tokens de réinitialisation de mot de passe.",
    shield: "Utiliser les fonctions CSPRNG sécurisées random_bytes(32) ou random_int()."
  },
  'CVE-PHP-LDAP-INJECTION': {
    attack: "Injection LDAP : concaténation d'entrées externes dans des filtres ldap_search() permettant le contournement d'authentification et l'exfiltration d'annuaires d'entreprise.",
    shield: "Échapper rigoureusement les filtres avec ldap_escape($input, null, LDAP_ESCAPE_FILTER)."
  },
  'CVE-PHP-NOSQL-INJECTION': {
    attack: "Injection NoSQL MongoDB : passage direct de tableaux associatifs superglobaux sans cast strict permettant l'injection d'opérateurs BSON ($ne, $gt) pour court-circuiter l'authentification.",
    shield: "Transtyper obligatoirement en chaîne scalaire : ['password' => (string)$_POST['password']]."
  },
  'CVE-PHP-ZIP-SLIP-EXTRACTION': {
    attack: "Décompression Zip Slip : extraction d'archive via ZipArchive::extractTo() sans validation de confinement des chemins relatifs (../../) écrasant des fichiers système hors cible.",
    shield: "Résoudre realpath() de chaque entrée et vérifier qu'elle commence strictement par le répertoire cible."
  },
  'CVE-PHP-EXPOSED-DEBUG-CONFIG': {
    attack: "Exposition de configuration de débogage : ini_set('display_errors', '1') ou phpinfo() révélant les variables d'environnement, secrets et chemins du serveur.",
    shield: "Forcer display_errors à 0 en production, journaliser via error_log() et supprimer tout appel à phpinfo()."
  }
};
window.CVE_ATTACK_EXPLANATIONS = CVE_ATTACK_EXPLANATIONS;

// --- 🚨 Rendu Léger & Épuré des Diagnostics en Haut du Menu de Gauche (#synthesis-card) ---
function renderSynthesisTopDiagnostics() {
  const container = document.getElementById('synth-top-diagnostics');
  if (!container) return;

  const anomalies = currentCveAnomaliesCache || [];
  if (anomalies.length === 0) {
    container.innerHTML = `
      <div class="synth-diag-healthy-pill">
        <span>🛡️</span>
        <span>Contrats & Sécurité : 0 vulnérabilité active • Code sain</span>
      </div>
    `;
    return;
  }

  const critCount = anomalies.filter(a => a.severity === 'CRITICAL').length;
  const highCount = anomalies.filter(a => a.severity === 'HIGH').length;

  container.innerHTML = `
    <div style="background:linear-gradient(135deg, rgba(239,68,68,0.16), rgba(15,23,42,0.92)); border:1px solid rgba(239,68,68,0.42); border-radius:8px; padding:8px 10px; margin-bottom:6px; display:flex; flex-direction:column; gap:6px;">
      <div style="display:flex; align-items:center; justify-content:space-between; gap:6px;">
        <span style="font-size:0.70rem; font-weight:800; color:#f87171; font-family:var(--font-mono); display:flex; align-items:center; gap:5px;">
          <span>🚨</span> ${anomalies.length} Anomalie(s) (${critCount} Crit • ${highCount} High)
        </span>
        <button class="synth-diag-btn" onclick="teleportToCveAnomaly(0)" title="Localiser la faille n°1 sur le graphe">🎯 Cibler #1</button>
      </div>
      <div style="display:flex; gap:5px;">
        <button class="aether-btn bounty-btn" onclick="openBountyScopeModal()" style="flex:1; font-size:0.64rem; padding:4px 6px;" title="Préparer le Prompt CVE avec In-Scope / Out-of-Scope">
          🎯 Scope & Prompt Bounty
        </button>
        <button class="aether-btn" onclick="openCveInspectorTab()" style="font-size:0.64rem; padding:4px 8px;" title="Ouvrir la liste détaillée à droite">
          📑 Liste CVE
        </button>
      </div>
    </div>
  `;
}
window.renderSynthesisTopDiagnostics = renderSynthesisTopDiagnostics;

function filterCveAnomalies(lang, btnEl) {
  document.querySelectorAll('#cve-lang-filters .cve-filter-btn').forEach(b => b.classList.remove('active'));
  if (btnEl) btnEl.classList.add('active');
  renderCveAnomaliesPanel(lang);
  AudioSys.playTone(520, 'sine', 0.1, 0.03);
}
window.filterCveAnomalies = filterCveAnomalies;

function teleportToCveAnomaly(idx) {
  const a = currentCveAnomaliesCache[idx];
  if (!a) return;

  const nodes = state.data?.nodes || [];
  let targetNode = null;
  if (a.symbol_id) {
    targetNode = nodes.find(n => n.id === a.symbol_id);
  }
  if (!targetNode && a.file_path) {
    targetNode = nodes.find(n => n.file_path === a.file_path && Math.abs((n.line_number || 0) - a.line_number) <= 15)
      || nodes.find(n => n.file_path === a.file_path && n.is_file)
      || nodes.find(n => n.file_path === a.file_path);
  }

  if (targetNode) {
    state.camera.targetX = targetNode.x;
    state.camera.targetY = targetNode.y;
    state.camera.targetZoom = 1.6;
    state.beaconNode = targetNode;
    state.beaconStart = performance.now();
    state.selectedAnomalyNodeId = targetNode.id;
    if (typeof selectNode === 'function') {
      selectNode(targetNode);
    }
    wakePhysics(60);
    AudioSys.nodeSelect();
    showToast(`🚨 Focus Faille CVE : ${a.cve_id} (${targetNode.name})`);
  } else {
    showToast(`⚠️ Nœud cible introuvable pour ${a.file_path}:${a.line_number}`);
  }
}
window.teleportToCveAnomaly = teleportToCveAnomaly;

const CVE_GOLD_REMEDIATION_PATTERNS = {
  'CVE-RUST-DESER-BOMB-OOM': {
    root_cause: "Les formats binaires réseau (bincode, MessagePack) ou `Vec::with_capacity(wire_len)` lisent un préfixe de taille sur 64 bits avant d'allouer. Un attaquant envoyant seulement 8 octets (`0xFFFFFFFFFFFFFFFF`) provoque un `handle_alloc_error` qui abort() instantanément le processus Rust entier (impossible à intercepter via `catch_unwind`).",
    gold_pattern: `const MAX_PACKET_BYTES: u64 = 64 * 1024; // 64 KiB max
// 1. Avec bincode : borner via with_limit()
use bincode::Options;
let packet: ClientPacket = bincode::DefaultOptions::new()
    .with_limit(MAX_PACKET_BYTES)
    .with_fixint_encoding()
    .deserialize(raw_frame)?;
// 2. Avec Vec::with_capacity : plafonner avec .min(MAX_FRAME_SIZE)
let safe_cap = claimed_len.min(MAX_FRAME_SIZE);
let mut buf = Vec::with_capacity(safe_cap);`
  },
  'CVE-RUST-CANCEL-SAFETY-LOSS': {
    root_cause: "Dans `tokio::select!`, lorsqu'une branche gagne la course, les futures des autres branches sont immédiatement droppées (`Future::drop`). Or `AsyncReadExt::read_exact`, `read_to_end`, `read_line` et `AsyncWriteExt::write_all` ne sont PAS cancel-safe : les octets déjà lus ou écrits partiellement sur le socket TCP sont définitivement perdus, corrompant le framing binaire.",
    gold_pattern: `// Solution Cancel-Safe : utiliser un buffer persistant hors du select! ou tokio_util::codec::Framed
// (StreamExt::next() sur Framed<TcpStream, LengthDelimitedCodec> est 100% cancel-safe)
let mut frame_buf = BytesMut::with_capacity(4096);
tokio::select! {
    res = socket.read_buf(&mut frame_buf) => { /* read_buf est 100% cancel-safe */ }
    _ = heartbeat_tick.tick() => { /* envoi hors du select ou via mpsc bufferisé */ }
}`
  },
  'CVE-RUST-TIMING-SIDECHANNEL': {
    root_cause: "L'opérateur `==` ou `!=` en Rust sur `&str`, `String` ou `&[u8]` utilise `memcmp` vectorisé SIMD qui s'arrête dès le premier octet divergent (short-circuit). Sur un réseau local ou Internet, un attaquant statistique (Crosby-Wallach) reconstruit le nonce, HMAC ou jeton secret octet par octet en mesurant le temps de réponse.",
    gold_pattern: `// Comparaison cryptographique en temps constant (sans branchement conditionnel sur les octets)
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
// Ou via la crate officielle subtle : bool::from(stored.as_bytes().ct_eq(expected.as_bytes()))`
  },
  'CVE-NET-SLOWLORIS-KEEPALIVE-DOS': {
    root_cause: "Accepter un socket TCP ou un handshake WebSocket (`accept_async`, `listener.accept()`, `axum::serve`) sans `tokio::time::timeout` permet à un attaquant d'ouvrir des milliers de connexions envoyant 1 octet toutes les 30 secondes (Slowloris), épuisant les descripteurs de fichiers OS (`EMFILE`) et bloquant les tâches Tokio.",
    gold_pattern: `use std::time::Duration;
// Envelopper systématiquement le handshake et la lecture dans un tokio::time::timeout strict :
let ws_stream = tokio::time::timeout(
    Duration::from_secs(5),
    tokio_tungstenite::accept_async(tcp_stream)
).await.map_err(|_| "Handshake timeout (Slowloris protection)")??;`
  },
  'CVE-NET-SSRF-DNS-REBIND': {
    root_cause: "Valider une URL par chaîne de caractères avant d'appeler `reqwest::get(url)` ou `TcpStream::connect(host)` est vulnérable au DNS Rebinding (TOCTOU : TTL=0 résolvant d'abord vers une IP publique, puis vers `127.0.0.1` ou `169.254.169.254` métadonnées Cloud) ainsi qu'aux redirections HTTP 302.",
    gold_pattern: `use std::net::{IpAddr, ToSocketAddrs};
// 1. Résoudre le DNS une seule fois et interdire toute IP interne/loopback/link-local :
fn validate_public_ip(ip: IpAddr) -> Result<IpAddr, &'static str> {
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return Err("SSRF Blocked: loopback/unspecified IP");
    }
    if let IpAddr::V4(v4) = ip {
        if v4.is_private() || v4.is_link_local() || v4.octets()[0] == 169 {
            return Err("SSRF Blocked: private or cloud metadata IPv4");
        }
    }
    Ok(ip)
}
// 2. Sur reqwest::Client : appliquer .redirect(reqwest::redirect::Policy::none())`
  },
  'CVE-RUST-BLOCKING-IN-ASYNC': {
    root_cause: "Exécuter une opération bloquante synchrone (`std::fs::*`, `std::thread::sleep`, `std::net::TcpStream`) à l'intérieur d'une `async fn` bloque le thread OS worker de Tokio. Avec seulement N cœurs CPU, N requêtes simultanées gèlent l'intégralité du serveur (Executor Starvation DoS).",
    gold_pattern: `// 1. Préférer les primitives asynchrones natives Tokio :
let content = tokio::fs::read_to_string(path).await?;
tokio::time::sleep(Duration::from_millis(50)).await;
// 2. Ou déporter le calcul/I/O lourd sur le pool dédié aux opérations bloquantes :
let res = tokio::task::spawn_blocking(move || std::fs::read(path)).await??;`
  },
  'CVE-WEB3-AUTH-BYPASS-MOCK': {
    root_cause: "Autoriser un préfixe de signature de test (`0xmock...`, `bypass_sig`) en dehors des tests unitaires isolés ou omettre de vérifier que `ecrecover` ne renvoie pas l'adresse zéro (`0x0000000000000000000000000000000000000000`) permet à n'importe quel attaquant d'usurper n'importe quel portefeuille Web3 sans clé privée.",
    gold_pattern: `// Vérification cryptographique stricte EIP-191 / EIP-712 avec rejet de l'adresse nulle :
const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';
const recovered = ethers.verifyMessage(challengeMessage, signature);
if (!recovered || recovered === ZERO_ADDRESS || recovered.toLowerCase() !== walletAddress.toLowerCase()) {
    throw new Error('CVE-WEB3-AUTH-BYPASS-MOCK: Invalid cryptographic signature');
}`
  },
  'CVE-MMO-IDOR-TRUSTED-USERID': {
    root_cause: "Extraire `userId` directement depuis le corps JSON de la requête (`req.body.userId`) sans vérifier ni résoudre le jeton de session (`Authorization: Bearer <token>`) permet à n'importe quel client de téléporter, débiter ou modifier l'inventaire d'un autre joueur (IDOR critique).",
    gold_pattern: `// Résoudre l'identité du joueur exclusivement depuis le token de session authentifié :
const authHeader = req.headers['authorization'] || '';
const token = authHeader.replace(/^Bearer\\s+/i, '');
const authenticatedUserId = await sessionsCache.verifyAndGetUserId(token);
if (!authenticatedUserId) {
    res.writeHead(401); return res.end(JSON.stringify({ error: 'Unauthorized session' }));
}`
  },
  'CVE-DB-REDIS-UNAUTH-KEYS-DOS': {
    root_cause: "La commande Redis `KEYS *` s'exécute en complexité $O(N)$ sur le thread unique de Redis/Cache, gelant toutes les lectures de sessions et de salles MMO pendant plusieurs secondes. De plus, un serveur RESP TCP sans `AUTH` expose toutes les sessions en clair sur le réseau interne.",
    gold_pattern: `// 1. Exiger l'authentification RESP avant toute commande et remplacer KEYS par SCAN (curseur O(1)) :
if (!clientSession.authenticated && cmd !== 'AUTH') {
    socket.write('-NOAUTH Authentication required.\\r\\n');
    return;
}
// 2. Utiliser SCAN avec COUNT borné au lieu de KEYS * :
// SCAN cursor MATCH prefix:* COUNT 100`
  },
  'CVE-PY-PATH-TRAVERSAL-ARCHIVE': {
    root_cause: "Ouvrir un chemin fourni par l'utilisateur (`open(os.path.join(base, user_file))`) ou extraire une archive (`zip.extractall()`) sans résolution canonique permet d'écrire hors du répertoire cible via des séquences `../../` (Path Traversal / ZipSlip RCE).",
    gold_pattern: `from pathlib import Path
BASE_DIR = Path("/var/app/uploads").resolve()
target_path = (BASE_DIR / user_filename).resolve()
if not target_path.is_relative_to(BASE_DIR):
    raise PermissionError("CVE-PY-PATH-TRAVERSAL-ARCHIVE: Illegal path traversal attempt")`
  },
  'CVE-JS-EVENTLOOP-REDOS-JSON': {
    root_cause: "Accumuler les chunks HTTP via `req.on('data', chunk => body += chunk)` sans vérifier la taille cumulée permet à un attaquant d'envoyer un flux infini qui sature la heap V8 (`FATAL ERROR: Reached heap limit`) et bloque l'Event-Loop Node.js.",
    gold_pattern: `const MAX_BODY_BYTES = 64 * 1024; // 64 KiB max
let body = '';
req.on('data', chunk => {
    body += chunk;
    if (Buffer.byteLength(body) > MAX_BODY_BYTES) {
        res.writeHead(413); res.end('Payload Too Large');
        req.destroy();
    }
});`
  },
  'CVE-CPP-UNBOUNDED-COPY-UAF': {
    root_cause: "Les fonctions C historiques (`strcpy`, `strcat`, `sprintf`, `gets`, `memcpy` sans vérification de la capacité destination) écrivent au-delà du tampon sur la pile ou le tas, permettant l'écrasement de l'adresse de retour (Stack Smashing / RCE).",
    gold_pattern: `// Utiliser snprintf / strncpy_s avec la taille exacte du buffer destination :
char dest[256];
std::snprintf(dest, sizeof(dest), "%s", untrusted_src);`
  },
  'CVE-CPP-NET-INTEGER-OVERFLOW-ALLOC': {
    root_cause: "L'addition arithmétique `payload_len + sizeof(Header)` sur des types non bornés peut déborder 32-bit / 64-bit (`UINT32_MAX`), provoquant une allocation mémoire sous-dimensionnée suivie d'un dépassement de tampon massif sur le tas (Heap Overflow).",
    gold_pattern: `// Vérifier l'overflow avant l'allocation mémoire :
if (payload_len > MAX_PACKET_SIZE - sizeof(PacketHeader)) {
    throw std::runtime_error("Taille de paquet excessive ou overflow potentiel");
}
char* buf = new char[payload_len + sizeof(PacketHeader)];`
  },
  'CVE-CPP-NET-TYPE-PUNNING-STRICT-ALIASING': {
    root_cause: "Caster directement un buffer d'octets `char*` en `PacketHeader*` via `reinterpret_cast` viole la règle de Strict Aliasing de la norme C++ (UB : optimisations dévastatrices du compilateur) et risque un crash matériel SIGBUS sur architecture à alignement strict (ARM/Apple Silicon).",
    gold_pattern: `// Copier dans une structure correctement alignée :
PacketHeader hdr;
std::memcpy(&hdr, raw_bytes, sizeof(PacketHeader));
// Ou en C++20 : auto hdr = std::bit_cast<PacketHeader>(raw_array);`
  },
  'CVE-CPP-NET-ASYNC-CALLBACK-UAF': {
    root_cause: "Dans un framework réseau asynchrone (Boost.Asio, libuv), capturer `[this]` dans une lambda de lecture/écriture sans prolonger la durée de vie de la session entraîne un Use-After-Free si le socket est déconnecté avant l'achèvement de l'opération.",
    gold_pattern: `// Hériter de std::enable_shared_from_this<Session> et capturer le shared_ptr :
auto self = shared_from_this();
socket.async_read_some(buffer, [self](const boost::system::error_code& ec, size_t bytes) {
    if (!ec) self->handle_read(bytes);
});`
  },
  'CVE-CPP-NET-PARTIAL-READ-SHORT-FRAME': {
    root_cause: "TCP est un flux d'octets sans délimitation de trame. Traiter le buffer d'un simple `recv()` sans boucle d'accumulation suppose une arrivée atomique impossible à garantir sur réseau réel, corrompant les paquets fragmentés.",
    gold_pattern: `// Boucler jusqu'à accumulation de la trame complète :
size_t total_read = 0;
while (total_read < sizeof(FrameHeader)) {
    ssize_t n = recv(fd, buf + total_read, sizeof(FrameHeader) - total_read, 0);
    if (n <= 0) break;
    total_read += n;
}`
  },
  'CVE-CPP-NET-SOCKET-FD-LEAK-EXCEPTION': {
    root_cause: "Un socket descripteur brut `int sock = socket(...)` sans destructeur automatique fuit si une exception C++ est levée ou en cas de `return` anticipé sans `close()`, saturant les descripteurs ouverts du système.",
    gold_pattern: `// Classe RAII assurant la fermeture immédiate :
struct SocketGuard {
    int fd;
    ~SocketGuard() { if (fd >= 0) close(fd); }
};
SocketGuard sock{ socket(AF_INET, SOCK_STREAM, 0) };`
  },
  'CVE-CPP-NET-HOST-BYTE-ORDER-CONFUSION': {
    root_cause: "Les protocoles réseau transmettent les entiers en Big-Endian. Accéder à un champ `length` directement sur processeur Little-Endian (x86_64/ARM) donne une valeur erronée contournant les vérifications de sécurité.",
    gold_pattern: `// Convertir de l'ordre réseau vers l'ordre hôte :
uint32_t packet_len = ntohl(header.length);
if (packet_len > MAX_ALLOWED_SIZE) return false;`
  },
  'CVE-CPP-NET-STRING-NULL-TERMINATION-OFF-BY-ONE': {
    root_cause: "Les paquets réseau ne contiennent pas nécessairement d'octet nul de fin `\\0`. Utiliser `strlen()` ou `std::string(buf)` sur un tampon brut lit la mémoire adjacente non initialisée jusqu'au prochain zéro.",
    gold_pattern: `// Allouer N + 1 et positionner le null-terminator explicitement :
char buf[MAX_LEN + 1];
int n = recv(sock, buf, MAX_LEN, 0);
if (n >= 0) buf[n] = '\\0';
std::string msg(buf);`
  },
  'CVE-CPP-NET-FORMAT-STRING-LOGGING': {
    root_cause: "Passer une chaîne reçue du réseau comme chaîne de format à `syslog()` ou `printf()` permet à un attaquant d'injecter des spécificateurs `%n` (écriture mémoire arbitraire) et `%x` (lecture de la pile).",
    gold_pattern: `// Spécifier systématiquement le format littéral "%s" :
syslog(LOG_ERR, "%s", client_message.c_str());
std::printf("%s", packet_str);`
  },
  'CVE-CPP-NET-SOCKET-RACE-DOUBLE-CLOSE': {
    root_cause: "Fermer un socket descripteur de manière concurrente sans atomicité peut fermer par erreur un nouveau descripteur réattribué entre-temps par le noyau OS à une autre connexion cliente.",
    gold_pattern: `// Fermeture atomique avec réinitialisation à -1 :
std::atomic<int> m_socket_fd;
int fd = m_socket_fd.exchange(-1);
if (fd >= 0) {
    ::close(fd);
}`
  },
  'CVE-CPP-NET-UNBOUNDED-RECURSION-TLV-PARSER': {
    root_cause: "L'analyse récursive de protocoles imbriqués (TLV/JSON/ASN.1) sans limite de profondeur consomme la totalité de la pile d'exécution sur trame forgée, provoquant un crash `SIGSEGV` immédiat du serveur.",
    gold_pattern: `// Transmettre un compteur de profondeur et plafonner la récursion :
Node parse_tlv(const uint8_t* stream, size_t depth = 0) {
    if (depth > MAX_RECURSION_DEPTH) {
        throw std::runtime_error("Dépassement de profondeur de récursion");
    }
    return parse_tlv(stream + 4, depth + 1);
}`
  },
  'CVE-WEB3-UNVALIDATED-CHAINID': {
    root_cause: "Une signature cryptographique Web3 (EIP-191 / EIP-712) qui n'inclut pas explicitement `chain_id` et l'adresse du contrat/domaine vérificateur peut être rejouée à l'identique sur un autre réseau (Polygon, Arbitrum, Testnet, Fork).",
    gold_pattern: `pub struct AuthVerifySignatureRequest {
    pub wallet_address: String,
    pub message: String,
    pub signature: String,
    pub chain_id: u64, // Obligatoire : lier la signature au Chain ID actif (ex: 1 ou 137)
}`
  },
  'CVE-MMO-UNCHECKED-SPEEDHACK': {
    root_cause: "Appliquer directement le delta ou les coordonnées reçues d'un paquet client sans plafonner la distance maximale par tick (`max_speed * dt`) permet aux clients modifiés de se téléporter instantanément (Speedhack).",
    gold_pattern: `pub const MAX_SPEED_PER_TICK: f32 = 5.0;
pub fn update_player_coord(current: f32, delta: f32) -> f32 {
    let safe_delta = delta.clamp(-MAX_SPEED_PER_TICK, MAX_SPEED_PER_TICK);
    current + safe_delta
}`
  },
  'CVE-WEB3-ORACLE-SPOT-MANIPULATION': {
    root_cause: "Lire le prix directement depuis les réserves instantanées d'un pool AMM (Uniswap getReserves / slot0) permet à un attaquant d'emprunter 50M$ en Flash Loan, déséquilibrer la pool dans le même bloc et drainer la trésorerie du protocole.",
    gold_pattern: `// Remplacer les réserves spot par un flux Chainlink vérifié avec contrôle de fraîcheur :
AggregatorV3Interface internal priceFeed;
function getLatestPrice() public view returns (int256) {
    (uint80 roundId, int256 price,, uint256 updatedAt, uint80 answeredInRound) = priceFeed.latestRoundData();
    require(updatedAt > block.timestamp - 3600, "Stale price oracle feed");
    require(answeredInRound >= roundId, "Round incomplete");
    return price;
}`
  },
  'CVE-WEB3-TX-ORIGIN-PHISHING': {
    root_cause: "Utiliser tx.origin pour valider les droits d'administration autorise n'importe quel contrat intermédiaire tiers à exécuter des ordres privilégiés si le propriétaire interagit avec un site malveillant.",
    gold_pattern: `// Remplacer tx.origin par msg.sender (ou _msgSender() si meta-transactions ERC-2771) :
function adminWithdraw(uint amount) external {
    require(msg.sender == owner, "Caller is not the owner");
    payable(owner).transfer(amount);
}`
  },
  'CVE-WEB3-UNCHECKED-TRANSFER-RETURN': {
    root_cause: "Des tokens majeurs comme USDT ou BNB ne renvoient pas de booléen true/false sur transfer(), violant la spécification ERC-20 standard et provoquant des échecs silencieux ou des reverts.",
    gold_pattern: `// Utiliser SafeERC20 d'OpenZeppelin qui gère les tokens avec ou sans retour booléen :
using SafeERC20 for IERC20;
IERC20 public token;
function deposit(uint amount) external {
    token.safeTransferFrom(msg.sender, address(this), amount);
}`
  },
  'CVE-WEB3-PERMIT-FRONT-RUNNING': {
    root_cause: "Des bots mempool peuvent extraire la signature ERC-2612 permit(v, r, s) et l'exécuter avant la victime, faisant échouer la transaction de l'utilisateur légitime par revert (Signature Griefing DoS).",
    gold_pattern: `// Envelopper permit dans un try/catch pour tolérer le front-running de signature :
function depositWithPermit(uint amount, uint deadline, uint8 v, bytes32 r, bytes32 s) external {
    try token.permit(msg.sender, address(this), amount, deadline, v, r, s) {} catch {}
    token.safeTransferFrom(msg.sender, address(this), amount);
}`
  },
  'CVE-WEB3-HOOK-REENTRANCY': {
    root_cause: "Les tokens ERC-777 et ERC-1363 invoquent un hook automatique tokensReceived() chez le destinataire pendant un simple transfer(), permettant la réentrance sans aucun flux d'Ether brut.",
    gold_pattern: `// Appliquer nonReentrant d'OpenZeppelin sur toutes les fonctions manipulant des tokens :
function withdrawTokens(uint amount) external nonReentrant {
    userBalances[msg.sender] -= amount;
    token.safeTransfer(msg.sender, amount);
}`
  },
  'CVE-VM-ACCOUNT-MISSING-OWNER': {
    root_cause: "Désérialiser un AccountInfo sans vérifier account.owner == program_id permet à un attaquant d'injecter des comptes forgés par un programme malveillant tiers.",
    gold_pattern: `// Vérification stricte du propriétaire du compte dans les programmes à modèle de comptes :
if account.owner != &expected_program_id {
    return Err(ProgramError::IllegalOwner.into());
}
// Ou via la contrainte d'owner équivalente de votre framework`
  },
  'CVE-WEB3-READONLY-REENTRANCY': {
    root_cause: "Les fonctions view/read-only interrogeant get_virtual_price() sur Curve ou les réserves Balancer pendant un retrait de liquidité en cours renvoient des cours faussés.",
    gold_pattern: `// Vérifier le verrou de réentrance de la pool avant de consommer son virtual_price :
ICurvePool pool = ICurvePool(poolAddress);
// Ou forcer une interaction non-reentrante d'état pour valider l'absence de reentrancy active
require(!pool.is_locked(), "Read-only reentrancy detected on Curve pool");`
  },
  'CVE-WEB3-DIRTY-MEMORY-ASSEMBLY': {
    root_cause: "Les types inférieurs à 32 octets (bool, uint8, address) manipulés en inline assembly sans masque binaire conservent des bits hauts sales qui inversent les conditions logiques Solidity.",
    gold_pattern: `// Nettoyer systématiquement les 248 bits supérieurs lors de l'assignation en assembleur :
assembly {
    let raw := mload(slot)
    // Masque strict pour bool / uint8 :
    clean_val := and(raw, 0xff)
    // Ou pour une adresse (20 octets) :
    clean_addr := and(raw, 0xffffffffffffffffffffffffffffffffffffffff)
}`
  },
  'CVE-WEB3-ERC4626-INFLATION-ATTACK': {
    root_cause: "Dans un Vault ERC-4626 sans parts virtuelles, le premier déposant mint 1 wei de part puis fait une donation massive, provoquant une division entière par zéro part pour les déposants suivants.",
    gold_pattern: `// Implémenter OpenZeppelin v4.9+ avec décalage de décimales (virtual shares / virtual assets) :
contract SafeVault is ERC4626 {
    function _decimalsOffset() internal view virtual override returns (uint8) {
        return 3; // Ajoute 1000 parts et actifs virtuels contre l'inflation attack
    }
}`
  },
  'CVE-WEB3-VAULT-ROUNDING-DIRECTION': {
    root_cause: "Divisions entières sans arrondi dirigé (Rounding.Floor en dépôt, Rounding.Ceil en rachat) favorisant l'attaquant au lieu du coffre.",
    gold_pattern: `// Utiliser OpenZeppelin Math.mulDiv avec Rounding explicite :
function _convertToShares(uint256 assets, Math.Rounding rounding) internal view virtual returns (uint256) {
    return Math.mulDiv(assets, totalSupply() + 10 ** _decimalsOffset(), totalAssets() + 1, rounding);
}
function convertToShares(uint256 assets) public view virtual override returns (uint256) {
    return _convertToShares(assets, Math.Rounding.Floor); // Floor impératif sur les dépôts !
}`
  },
  'CVE-WEB3-STALE-ORACLE-HEARTBEAT': {
    root_cause: "latestRoundData() interrogé sans validation de fraîcheur temporelle et sans contrôle de la ronde de prix.",
    gold_pattern: `// Contrôle de fraîcheur strict Chainlink :
(uint80 roundId, int256 price, , uint256 updatedAt, uint80 answeredInRound) = priceFeed.latestRoundData();
require(price > 0, "Prix négatif ou nul");
require(updatedAt != 0, "Ronde incomplète");
require(block.timestamp - updatedAt <= HEARTBEAT_TIMEOUT, "Prix oraculaire périmé");
require(answeredInRound >= roundId, "Ronde de prix obsolète");`
  },
  'CVE-WEB3-UNPROTECTED-CALLBACK-SELECTOR': {
    root_cause: "Callback de prêt flash (onFlashLoan) public sans contrôle d'identité sur msg.sender.",
    gold_pattern: `// Restreindre l'appelant au pool de prêt autorisé :
function onFlashLoan(address initiator, address token, uint256 amount, uint256 fee, bytes calldata data) external returns (bytes32) {
    require(msg.sender == address(AUTHORIZED_POOL), "Seul le pool de prêt est autorisé à invoquer le callback");
    require(initiator == address(this), "Seul ce contrat peut être l'initiateur du flash loan");
    // Traitement sécurisé...
    return keccak256("ERC3156FlashBorrower.onFlashLoan");
}`
  },
  'CVE-WEB3-SIGNATURE-DEADLINE-MISSING': {
    root_cause: "Signature cryptographique acceptée sans vérification de date d'expiration (deadline).",
    gold_pattern: `// Validation impérative de la date d'expiration :
function permit(address owner, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s) public {
    require(block.timestamp <= deadline, "La signature permit a expiré");
    bytes32 structHash = keccak256(abi.encode(_PERMIT_TYPEHASH, owner, spender, value, _useNonce(owner), deadline));
    bytes32 hash = _hashTypedDataV4(structHash);
    address signer = ECDSA.recover(hash, v, r, s);
    require(signer == owner, "Signature invalide");
    _approve(owner, spender, value);
}`
  },
  'CVE-WEB3-HOOK-DEFERRED-CHECKS-DESYNC': {
    root_cause: "Hook de Vault exécuté pendant que les contrôles de solvabilité sont différés dans l'EVC.",
    gold_pattern: `// Contrôle de l'état des vérifications différées EVC :
function checkVaultStatus() external view override {
    require(msg.sender == address(authorizedVault), "Appelant non autorisé");
    require(!IEVC(evc).isChecksDeferred(account) || IEVC(evc).areChecksInProgress(), "Vérifications de liquidité différées actives");
}`
  },
  'CVE-WEB3-DECIMAL-SCALING-TRUNCATION': {
    root_cause: "Normalisation statique avec 10**(18 - decimals) ou division entière avant multiplication.",
    gold_pattern: `// Mise à l'échelle symétrique sécurisée des décimales :
function scaleTo18(uint256 amount, uint8 decimals) internal pure returns (uint256) {
    if (decimals == 18) return amount;
    if (decimals < 18) return amount * (10 ** (18 - decimals));
    return amount / (10 ** (decimals - 18));
}`
  },
  'CVE-WEB3-FORK-REWARD-STAKING-DILUTION': {
    root_cause: "Distribution de tokens de récompense sans checkpoint préalable de l'accumulateur de reward.",
    gold_pattern: `// Actualiser l'accumulateur de rendement avant toute mutation :
function deposit(uint256 amount) external {
    updatePool(); // Met à jour accRewardPerShare et lastRewardBlock
    if (user.amount > 0) {
        uint256 pending = (user.amount * accRewardPerShare) / 1e12 - user.rewardDebt;
        if (pending > 0) rewardToken.safeTransfer(msg.sender, pending);
    }
    // Traitement du dépôt...
}`
  },
  'DEFI-ASYMMETRIC-ROUNDING': {
    root_cause: "Divisions entières sans arrondi explicite dans les fonctions de conversion d'actifs/parts ou calculs de taxes économiques.",
    gold_pattern: `// Solvabilité garantie : Arrondi en faveur du protocole (mulDiv / FixedPointMathLib)
function _convertToShares(uint256 assets, Math.Rounding rounding) internal view virtual returns (uint256) {
    return assets.mulDiv(totalSupply() + 10 ** _decimalsOffset(), totalAssets() + 1, rounding);
}

// En Rust / Jeu (economy.rs) :
let fee = (amount * fee_basis_points / 10_000).max(1); // Évite l'évasion de taxe par troncature à 0`
  },
  'DEFI-STALE-CACHE-AFTER-CALL': {
    root_cause: "Réutilisation d'un cache local (VaultCache / snapshot) après un appel externe (.call / HookTarget / .await) sans réactualisation.",
    gold_pattern: `// Toujours rafraîchir le cache local immédiatement après un hook externe :
target.call(payload);
// 🛡️ Invalidation & Rechargement du cache d'état :
vaultCache = _loadVaultCache(); // Re-lecture fraîche de l'état du coffre
uint256 currentBalance = vaultCache.balance;`
  },
  'EVC-HOOK-BYPASS': {
    root_cause: "Mutation d'état ou transfert de fonds dans un hook de Vault alors que les contrôles de solvabilité de l'EVC sont différés.",
    gold_pattern: `// Restreindre les hooks et vérifier les invariants EVC :
function checkVaultStatus() external view override {
    // Vérifier que les contrôles différés ne sont pas détournés en réentrance
    if (IEVC(evc).isChecksDeferred(msg.sender)) {
        require(IEVC(evc).areChecksInProgress(), "EVC_CHECKS_DEFERRED_NOT_IN_PROGRESS");
    }
}`
  },
  'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER': {
    root_cause: "Chainlink renvoie minAnswer/maxAnswer en cas d'extrême krach sans avertir le contrat appelant.",
    gold_pattern: `// Vérification stricte des bornes physiques Chainlink (AggregatorV2V3Interface) :
(, int256 answer, , , ) = priceFeed.latestRoundData();
require(answer > 0, "Prix invalide");
// Récupérer les bornes de l'agrégateur sous-jacent :
int256 minAnswer = AggregatorV2V3Interface(feed).minAnswer();
int256 maxAnswer = AggregatorV2V3Interface(feed).maxAnswer();
require(answer > minAnswer, "Prix au plancher: possible crash de prix");
require(answer < maxAnswer, "Prix au plafond artificiel");`
  },
  'DEFI-ORACLE-L2-SEQUENCER-RACE': {
    root_cause: "Absence de vérification du statut du séquenceur L2 et de la période de grâce après redémarrage.",
    gold_pattern: `// Vérification du séquenceur Chainlink Uptime Feed sur L2 (Arbitrum / Base) :
(, int256 answer, uint256 startedAt, , ) = sequencerUptimeFeed.latestRoundData();
// 0 = Séquenceur actif, 1 = Séquenceur en panne
require(answer == 0, "Sequencer down");
// Période de grâce obligatoire (ex: 3600 secondes = 1 heure) :
require(block.timestamp - startedAt > GRACE_PERIOD_TIME, "Grace period not over");`
  },
  'DEFI-TWAP-SHORT-WINDOW': {
    root_cause: "Fenêtre TWAP inférieure à 30 minutes vulnérable à la manipulation MEV multi-blocs.",
    gold_pattern: `// Imposer une fenêtre TWAP minimale de 30 minutes (1800 secondes) :
uint32 constant MIN_TWAP_WINDOW = 1800; // 30 minutes
function getTwap() external view returns (uint256) {
    uint32[] memory secondsAgos = new uint32[](2);
    secondsAgos[0] = twapWindow; // Garanti >= MIN_TWAP_WINDOW dans le constructeur
    secondsAgos[1] = 0;
    (int56[] memory tickCumulatives, ) = pool.observe(secondsAgos);
    // Calcul de la moyenne géométrique sécurisée...
}`
  },

  // 🐍 6 Nouveaux Patrons Python V2
  'CVE-PY-JINJA2-SSTI-INJECTION': {
    root_cause: "Concaténation directe de données utilisateurs dans un template Jinja2 via render_template_string() ou Template(). Un attaquant injecte {{ config.__class__.__init__.__globals__['os'].popen('id').read() }} pour exécuter du code à distance (RCE).",
    gold_pattern: `# Patron de Remédiation Certifié :
# 1. Utiliser un template statique avec variables de contexte :
from flask import render_template_string

SAFE_TEMPLATE = "<h1>Bonjour {{ user_name }} !</h1>"
# Les variables passées en kwargs sont automatiquement échappées par Jinja2 :
return render_template_string(SAFE_TEMPLATE, user_name=untrusted_input)`
  },
  'CVE-PY-YAML-UNSAFE-LOAD': {
    root_cause: "yaml.load(data) sans loader spécifique ou yaml.unsafe_load(data) autorise l'exécution de constructeurs Python arbitraires via des balises de type !!python/object/apply.",
    gold_pattern: `# Patron de Remédiation Certifié :
import yaml

# Toujours utiliser safe_load ou spécifier explicitement SafeLoader :
config_data = yaml.safe_load(raw_yaml_stream)
# Alternative équivalente :
# config_data = yaml.load(raw_yaml_stream, Loader=yaml.SafeLoader)`
  },
  'CVE-PY-SQL-ALCHEMY-TEXT-INJECTION': {
    root_cause: "Passer une f-string ou une chaîne formatée à sqlalchemy.text() injecte du code SQL brut directement dans la base sans passer par le paramétrage sécurisé de l'ORM.",
    gold_pattern: `# Patron de Remédiation Certifié :
from sqlalchemy import text

# Utiliser la syntaxe bindparams avec :variable :
stmt = text("SELECT * FROM users WHERE id = :user_id AND status = :status")
result = db.session.execute(stmt, {"user_id": user_input_id, "status": "active"}).fetchall()`
  },
  'CVE-PY-JWT-NONE-ALGORITHM': {
    root_cause: "Décoder un jeton JWT avec verify=False ou autoriser l'algorithme 'none' permet à un attaquant de forger un jeton sans clé secrète et de s'octroyer des privilèges administrateur.",
    gold_pattern: `# Patron de Remédiation Certifié :
import jwt

# Imposer la validation de signature avec une liste d'algorithmes autorisés stricte :
payload = jwt.decode(
    token,
    JWT_SECRET_KEY,
    algorithms=["HS256"],
    options={"verify_signature": True, "require": ["exp", "sub"]}
)`
  },
  'CVE-PY-PARAMIKO-MISSING-HOST-KEY': {
    root_cause: "AutoAddPolicy() ajoute aveuglément n'importe quelle clé d'hôte SSH inconnue dans le cache local, neutralisant la vérification d'authenticité du serveur distant et permettant une interception Man-In-The-Middle.",
    gold_pattern: `# Patron de Remédiation Certifié :
import paramiko

client = paramiko.SSHClient()
# Charger les clés d'hôtes de confiance du système hôte :
client.load_system_host_keys()
# Rejeter formellement toute clé inconnue :
client.set_missing_host_key_policy(paramiko.RejectPolicy())
client.connect(hostname, port=22, username=user, key_filename=key_path)`
  },
  'CVE-PY-XML-XXE-ENTITY-EXPANSION': {
    root_cause: "Le parseur XML standard de Python résout les entités externes DTD (SYSTEM 'file:///etc/passwd'), permettant la lecture arbitraire de fichiers ou un déni de service Billion Laughs.",
    gold_pattern: `# Patron de Remédiation Certifié :
# Remplacer xml.etree par defusedxml :
import defusedxml.ElementTree as ET

# defusedxml désactive automatiquement la résolution d'entités externes et DTD :
root = ET.fromstring(untrusted_xml_data)`
  },

  // 🦀 6 Nouveaux Patrons Rust V2
  'CVE-RUST-INTEGER-OVERFLOW-CAST': {
    root_cause: "L'opérateur 'as' en Rust (ex: val as u32) tronque silencieusement les bits de poids fort en mode release sans paniquer, ce qui provoque des désynchronisations de taille de buffer ou de solde monétaire.",
    gold_pattern: `// Patron de Remédiation Certifié :
use std::convert::TryFrom;

// Remplacer 'val as u32' par TryFrom avec mapping d'erreur propre :
let safe_len = u32::try_from(wire_len)
    .map_err(|_| PacketError::LengthOverflow { claimed: wire_len, max: u32::MAX as u64 })?;`
  },
  'CVE-RUST-TRANSMUTE-SIZE-MISMATCH': {
    root_cause: "std::mem::transmute contourne le système de types de Rust. Si les types sources et destinations ont un layout ou une taille divergente, la mémoire est immédiatement corrompue (Undefined Behavior).",
    gold_pattern: `// Patron de Remédiation Certifié :
// 1. Pour les conversions numériques/octets, utiliser from_le_bytes :
let val = u64::from_le_bytes(bytes[0..8].try_into().map_err(...)?);

// 2. Pour les structs en mémoire, utiliser bytemuck (garantie de sûreté à la compilation) :
// #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
// let pod: MyStruct = bytemuck::cast(raw_data);`
  },
  'CVE-RUST-FORGOTTEN-MEM-LEAK': {
    root_cause: "std::mem::forget() neutralise l'exécution du trait Drop. Si appelé sur un garde de synchronisation (MutexGuard), le verrou n'est jamais libéré, provoquant un deadlock permanent de tous les threads futurs.",
    gold_pattern: `// Patron de Remédiation Certifié :
// Ne JAMAIS appeler std::mem::forget(guard) sur un MutexGuard ou Handle réseau.
// Laisser la portée RAII fermer le garde naturellement :
{
    let mut data = mutex.lock().map_err(...)?;
    data.update();
} // guard dropped automatiquement ici

// Ou libérer explicitement plus tôt :
drop(guard);`
  },
  'CVE-RUST-UNCHECKED-INDEX-SLICING': {
    root_cause: "L'indexation directe [offset..offset + len] panique immédiatement si le buffer réseau reçu est tronqué ou altéré, permettant à un attaquant de crasher le serveur à distance avec un seul paquet UDP/TCP malformé.",
    gold_pattern: `// Patron de Remédiation Certifié :
// Remplacer le découpage direct par .get() sécurisé :
let slice = packet.get(offset..offset + len)
    .ok_or(PacketError::TruncatedFrame { expected: len, available: packet.len().saturating_sub(offset) })?;`
  },
  'CVE-RUST-THREAD-POISON-IGNORED': {
    root_cause: "Avaler un PoisonError avec .unwrap_or_else(|e| e.into_inner()) continue le traitement sur des données qui étaient en cours de modification lors d'une panique précédente, propageant un état corrompu.",
    gold_pattern: `// Patron de Remédiation Certifié :
// Traiter le Mutex empoisonné comme une condition d'erreur bloquante :
let mut guard = match shared_state.lock() {
    Ok(g) => g,
    Err(poisoned) => {
        tracing::error!("Mutex poisoned! Rebuilding state invariants...");
        let mut g = poisoned.into_inner();
        g.repair_invariants(); // Réparer formellement les données avant réutilisation
        g
    }
};`
  },
  'CVE-RUST-FFI-UNCHECKED-NULL-PTR': {
    root_cause: "Déréférencer un pointeur brut *const T ou *mut T sans tester ptr.is_null() provoque un crash de processus (SIGSEGV) si le code appelant C transmet NULL.",
    gold_pattern: `// Patron de Remédiation Certifié :
use std::ptr::NonNull;

pub unsafe extern "C" fn process_native(ptr: *const u8, len: usize) -> i32 {
    // 1. Vérifier la nullité via NonNull :
    let valid_ptr = match NonNull::new(ptr as *mut u8) {
        Some(p) => p,
        None => return -1, // Code d'erreur propre FFI_ERR_NULL_PTR
    };
    // 2. Construire une tranche sécurisée avec les bornes validées :
    let slice = std::slice::from_raw_parts(valid_ptr.as_ptr(), len);
    0
}`
  },

  // 🟢 6 Nouveaux Patrons Node.js / TypeScript V2
  'CVE-JS-VM-SANDBOX-ESCAPE': {
    root_cause: "Le module 'vm' de Node.js n'est pas conçu pour exécuter du code non fiable. Le code évalué peut faire this.constructor.constructor('return process')().mainModule.require('child_process').execSync('id') pour s'évader de la sandbox.",
    gold_pattern: `// Patron de Remédiation Certifié :
// Utiliser isolated-vm (sandbox isolée V8 avec mémoire distincte) :
const ivm = require('isolated-vm');
const isolate = new ivm.Isolate({ memoryLimit: 128 });
const context = isolate.createContextSync();
const jail = context.global;
jail.setSync('global', jail.derefInto());

const script = isolate.compileScriptSync(untrustedCode);
const result = script.runSync(context, { timeout: 1000 });`
  },
  'CVE-JS-SSRF-DYNAMIC-AXIOS-FETCH': {
    root_cause: "Passer une URL utilisateur non filtrée à axios ou fetch permet à un attaquant de requêter des services internes (localhost:6379, metadata AWS 169.254.169.254).",
    gold_pattern: `// Patron de Remédiation Certifié :
const ALLOWED_DOMAINS = new Set(['api.aether.engine', 'cdn.habbo2026.com']);

function isSafeUrl(rawUrl) {
    try {
        const parsed = new URL(rawUrl);
        if (parsed.protocol !== 'https:') return false;
        if (!ALLOWED_DOMAINS.has(parsed.hostname)) return false;
        // Bloquer toute tentative d'encodage décimal / octal / hex d'IP privées
        return true;
    } catch {
        return false;
    }
}
if (!isSafeUrl(userUrl)) throw new Error('Cible d\\'URL interdite (SSRF Guard)');
const res = await axios.get(userUrl, { maxRedirects: 0, timeout: 5000 });`
  },
  'CVE-JS-PATH-TRAVERSAL-SENDFILE': {
    root_cause: "res.sendFile(req.query.file) sans confinement de racine permet d'utiliser des chemins relatifs ../../../etc/passwd pour extraire des fichiers arbitraires du serveur.",
    gold_pattern: `// Patron de Remédiation Certifié :
const path = require('path');
const PUBLIC_DIR = path.resolve(__dirname, '../public');

app.get('/download', (req, res) => {
    // 1. Spécifier root avec interdiction des fichiers cachés :
    res.sendFile(req.query.file, { root: PUBLIC_DIR, dotfiles: 'deny' }, (err) => {
        if (err) res.status(404).send('Fichier introuvable');
    });
});`
  },
  'CVE-JS-REGEX-DOS-DYNAMIC-INPUT': {
    root_cause: "new RegExp(userInput) sans échappement permet d'injecter des expressions vulnérables au 'Catastrophic Backtracking' (ex: (a+)+$) qui gèlent l'Event-Loop Node.js à 100% CPU.",
    gold_pattern: `// Patron de Remédiation Certifié :
const escapeRegExp = require('lodash.escaperegexp');

// Toujours échapper les caractères spéciaux regex avant compilation :
const safePattern = escapeRegExp(req.query.search);
const rx = new RegExp(safePattern, 'i');`
  },
  'CVE-JS-CORS-WILDCARD-CREDENTIALS': {
    root_cause: "Configurer CORS avec credentials: true tout en renvoyant l'origine du client (origin: true) permet à n'importe quel site web d'effectuer des requêtes authentifiées au nom de la victime.",
    gold_pattern: `// Patron de Remédiation Certifié :
const cors = require('cors');

const TRUSTED_ORIGINS = ['https://app.aether.engine', 'https://game.habbo2026.com'];

app.use(cors({
    origin: (origin, callback) => {
        if (!origin || TRUSTED_ORIGINS.includes(origin)) {
            callback(null, true);
        } else {
            callback(new Error('Origine non autorisée par la politique CORS'));
        }
    },
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'DELETE']
}));`
  },
  'CVE-JS-UNHANDLED-PROMISE-CRASH': {
    root_cause: "Sous Node.js 15+, tout rejet de promesse non intercepté dans un gestionnaire de route async déclenche un 'UnhandledPromiseRejection' qui termine immédiatement le processus Node.js entier.",
    gold_pattern: `// Patron de Remédiation Certifié :
// 1. Importer express-async-errors tout en haut du point d'entrée :
require('express-async-errors');

// 2. Ou envelopper le gestionnaire dans try/catch avec next(err) :
app.get('/api/user', async (req, res, next) => {
    try {
        const user = await db.getUser(req.userId);
        res.json(user);
    } catch (err) {
        next(err); // Transmettre proprement au middleware d'erreur global
    }
});`
  },

  // 💎 4 Patrons Web3 Next-Gen Ultra-Critiques & Inédits
  'CVE-WEB3-EIP1153-TRANSIENT-COMPOSABILITY-LEAK': {
    root_cause: "L'opcode EIP-1153 TSTORE conserve ses données jusqu'à la fin de la transaction entière (et non de la frame d'appel CALL). Ne pas remettre un slot transitoire à 0 avant de quitter la fonction permet au second appel d'un multicall, d'un batch EVC ou d'un bundler ERC-4337 d'hériter de l'état authentifié ou des crédits de Flash Accounting de l'appel précédent.",
    gold_pattern: `// Patron de Remédiation Certifié (Nettoyage Symétrique EIP-1153 TSTORE) :
uint256 constant AUTH_CALLER_TSLOT = 0x01;

modifier withTransientContext(address authorizedCaller) {
    assembly {
        tstore(AUTH_CALLER_TSLOT, authorizedCaller)
    }
    _;
    // 🛡️ Impératif : Purger le slot transitoire avant de rendre la main au batch/multicall
    assembly {
        tstore(AUTH_CALLER_TSLOT, 0)
    }
}`
  },
  'CVE-WEB3-EIP7702-EOA-CODE-BYPASS-COLLISION': {
    root_cause: "Vérifier msg.sender.code.length == 0 ou extcodesize == 0 ne garantit plus un appelant non-programmable : un constructeur de contrat a une taille de code de 0, et EIP-7702 permet à tout EOA de déléguer son exécution à un contrat (0xef0100) tout en conservant tx.origin == msg.sender. De plus, un contrat délégué sans stockage ERC-7201 écrase le slot 0 de l'EOA.",
    gold_pattern: `// Patron de Remédiation Certifié (Stockage Isolé ERC-7201 pour Délégation EIP-7702) :
/// @custom:storage-location erc7201:aether.storage.DelegatedAccount
struct AccountStorage {
    uint256 nonce;
    address sessionKey;
}

// Slot calculé cryptographiquement loin du slot 0 :
// keccak256(abi.encode(uint256(keccak256("aether.storage.DelegatedAccount")) - 1)) & ~bytes32(uint256(0xff))
bytes32 private constant ACCOUNT_STORAGE_LOCATION =
    0x8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd100;

function _getAccountStorage() private pure returns (AccountStorage storage $) {
    assembly { $.slot := ACCOUNT_STORAGE_LOCATION }
}`
  },
  'CVE-WEB3-ERC4337-PAYMASTER-GAS-SIPHON': {
    root_cause: "Dans ERC-4337, si le Paymaster valide validatePaymasterUserOp sans pré-séquestrer les tokens ERC-20 et attend _postOp pour exécuter transferFrom, l'utilisateur peut révoquer son allowance ou dépenser ses tokens pendant l'exécution de son UserOp, faisant échouer _postOp et forçant le Paymaster à payer l'ETH à l'EntryPoint de sa propre poche.",
    gold_pattern: `// Patron de Remédiation Certifié (Pré-Séquestration ERC-4337 + Plafond maxCost) :
function _validatePaymasterUserOp(
    PackedUserOperation calldata userOp,
    bytes32 /*userOpHash*/,
    uint256 maxCost
) internal override returns (bytes memory context, uint256 validationData) {
    require(maxCost <= MAX_SPONSORED_ETH_COST, "Paymaster: maxCost exceeds safety cap");
    uint256 maxTokenCost = getTokenAmountForEth(maxCost);
    // 1. Pré-séquestrer (pre-charge) immédiatement les tokens ERC-20 pendant la phase de validation :
    feeToken.safeTransferFrom(userOp.sender, address(this), maxTokenCost);
    return (abi.encode(userOp.sender, maxTokenCost, maxCost), 0);
}

function _postOp(PostOpMode /*mode*/, bytes calldata context, uint256 actualGasCost, uint256 /*actualUserOpFeePerGas*/) internal override {
    (address sender, uint256 preChargedTokens, uint256 maxCost) = abi.decode(context, (address, uint256, uint256));
    uint256 actualTokenCost = (preChargedTokens * actualGasCost) / maxCost;
    // 2. Rembourser uniquement l'excédent pré-prélevé :
    if (preChargedTokens > actualTokenCost) {
        feeToken.safeTransfer(sender, preChargedTokens - actualTokenCost);
    }
}`
  },
  'CVE-WEB3-OMNICHAIN-LZ-CHANNEL-BRICKING': {
    root_cause: "Dans un récepteur inter-chaînes (_lzReceive / _ccipReceive), les tokens ont déjà été brûlés sur la chaîne de départ. Si la livraison sur la chaîne d'arrivée revert (ex: destinataire blacklisté par USDC, hook externe qui échoue), les fonds brûlés sont perdus à jamais ou le canal inter-chaînes est paralysé.",
    gold_pattern: `// Patron de Remédiation Certifié (Quarantaine Non-Bloquante Inter-Chaînes) :
mapping(bytes32 => bytes32) public failedMessages;

function _lzReceive(Origin calldata _origin, bytes32 _guid, bytes calldata _message, address, bytes calldata) internal override {
    // Isoler l'exécution métier dans un sous-appel externe pour intercepter tout revert :
    try this.processInboundPayload(_message) {
        // Livraison réussie
    } catch {
        // Stocker l'empreinte du payload en quarantaine sans faire réverter le canal :
        failedMessages[_guid] = keccak256(_message);
        emit MessageQuarantined(_origin.srcEid, _guid, _message);
    }
}`
  },

  // 🏆 6 Patrons Masterclass Deep-Core & Zero-Day Class
  'CVE-WEB3-EIP150-63-64-GAS-STIPEND-GRIEFING': {
    root_cause: "Depuis l'EIP-150, un sous-appel (.call) ne reçoit au maximum que 63/64 du gaz restant dans la frame parente. Un relayer malveillant peut calibrer exactement le gasLimit de la transaction pour que le sous-appel échoue par Out-Of-Gas, tandis que le contrat parent conserve 1/64 de gaz, consomme le nonce de l'utilisateur et empoche le remboursement.",
    gold_pattern: `// Patron de Remédiation Certifié (Protection Anti-Griefing EIP-150 63/64th Gas) :
function executeMetaTx(address target, bytes calldata data, uint256 innerGasLimit, uint256 nonce) external {
    nonces[msg.sender] = nonce;
    // 1. Vérifier que 63/64 du gaz restant couvre largement innerGasLimit :
    require(gasleft() >= (innerGasLimit * 64) / 63 + 15_000, "EIP-150: Insufficient gas for subcall");
    (bool success, ) = target.call{gas: innerGasLimit}(data);
    // 2. Si le sous-appel échoue par manque de gaz fourni par le relayer, annuler toute la transaction :
    if (!success) {
        require(gasleft() > innerGasLimit / 63, "Relayer OOG griefing detected");
    }
}`
  },
  'CVE-WEB3-ZK-SNARK-FIELD-OVERFLOW-ALIASING': {
    root_cause: "Les précompilés de courbes elliptiques BN254 (alt_bn128) opèrent modulo r = 21888242871839275222246405745257275088548364400416034343698204186575808495617 (≈ 2^254), tandis qu'un uint256 EVM accepte jusqu'à 2^256 - 1. Sans vérification < SNARK_SCALAR_FIELD, nullifierHash + k*r valide exactement la même preuve ZK tout en contournant le mapping Solidity nullifiers[nullifierHash] jusqu'à 5 fois.",
    gold_pattern: `// Patron de Remédiation Certifié (Borne Stricte du Corps Scalaire BN254) :
uint256 constant SNARK_SCALAR_FIELD = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

function withdrawWithZkProof(uint256[8] calldata proof, uint256 root, uint256 nullifierHash) external {
    // 🛡️ Impératif : Rejeter tout scalaire >= r pour éliminer l'aliasing modulo r (5x Spend)
    require(root < SNARK_SCALAR_FIELD && nullifierHash < SNARK_SCALAR_FIELD, "ZK input exceeds BN254 scalar field");
    require(!nullifiers[nullifierHash], "Nullifier already spent");
    nullifiers[nullifierHash] = true;
    require(verifier.verifyProof(proof, [root, nullifierHash]), "Invalid ZK proof");
}`
  },
  'CVE-WEB3-MERKLE-SECOND-PREIMAGE-LEAF-NODE-COLLISION': {
    root_cause: "Un nœud interne d'arbre de Merkle Keccak256 est le hachage de deux sous-nœuds de 32 octets (64 octets au total). Si une feuille est construite via keccak256(abi.encode(address, uint256)) (qui mesure aussi exactement 64 octets), un attaquant peut soumettre n'importe quelle paire de nœuds internes de l'arbre comme feuille légitime.",
    gold_pattern: `// Patron de Remédiation Certifié (Double Hachage Standard OpenZeppelin v5) :
function claimAirdrop(address account, uint256 amount, bytes32[] calldata proof) external {
    // 🛡️ Double hachage : le hash interne fait 32 octets, rendant impossible toute collision avec un nœud interne de 64 octets
    bytes32 leaf = keccak256(bytes.concat(keccak256(abi.encode(account, amount))));
    require(MerkleProof.verifyCalldata(proof, merkleRoot, leaf), "Invalid Merkle proof");
}`
  },
  'CVE-WEB3-CREATE2-METAMORPHIC-REINIT-RUG': {
    root_cause: "Passer un salt utilisateur brut à CREATE2 sans y hacher msg.sender permet à un attaquant dans le mempool (ou sur un autre L2) de déployer en premier à la même adresse déterministe. De plus, un initCode non constant ouvre aux contrats métamorphiques.",
    gold_pattern: `// Patron de Remédiation Certifié (Salt CREATE2 Lié Cryptographiquement au Déployeur) :
function deployDeterministicVault(bytes32 userSalt, bytes calldata immutableArgs) external returns (address) {
    // Lier l'identité de msg.sender et le chainId dans le salt CREATE2 :
    bytes32 boundSalt = keccak256(abi.encode(msg.sender, block.chainid, userSalt));
    return address(new Vault{salt: boundSalt}(immutableArgs));
}`
  },
  'CVE-RUST-PIN-STRUCTURAL-UNPIN-SOUNDNESS-UB': {
    root_cause: "Appeler Pin::get_unchecked_mut() ou Pin::new_unchecked() manuellement sur une Future ou un Stream contenant des références internes brise la garantie de non-déplacement en mémoire de Rust : un std::mem::swap ultérieur provoque un Use-After-Free silencieux au prochain poll().",
    gold_pattern: `// Patron de Remédiation Certifié (Projection Structurelle Sûre via pin_project_lite) :
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    pub struct SafeStreamFuture<F> {
        #[pin]
        inner_future: F,
        state_counter: u64,
    }
}

impl<F: std::future::Future> std::future::Future for SafeStreamFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project(); // 100% sound : génère Pin<&mut F> et &mut u64 sans unsafe manuel
        this.inner_future.poll(cx)
    }
}`
  },
  'CVE-JS-V8-TYPEDARRAY-SHARED-AB-RACE-OOB': {
    root_cause: "Buffer.allocUnsafe(size) alloue un segment non nettoyé de la heap V8 contenant d'anciens secrets (clés JWT, mots de passe, TLS), et lire un SharedArrayBuffer sans Atomics.load permet à un Worker Thread concurrent de modifier la longueur d'une trame après sa validation (TOCTOU).",
    gold_pattern: `// Patron de Remédiation Certifié (Zéro-Fill V8 & Lecture Atomique SharedArrayBuffer) :
// 1. Toujours allouer des buffers initialisés à zéro :
const safeFrame = Buffer.alloc(frameSize);

// 2. Sur SharedArrayBuffer, lire l'en-tête via Atomics.load et copier dans un buffer privé avant validation :
const sharedI32 = new Int32Array(sharedArrayBuffer);
const claimedLen = Atomics.load(sharedI32, 0);
if (claimedLen < 0 || claimedLen > MAX_FRAME_LEN) throw new RangeError("Invalid frame length");
const localCopy = new Uint8Array(claimedLen);
localCopy.set(new Uint8Array(sharedArrayBuffer, 4, claimedLen)); // Copie locale immuable (Anti-TOCTOU)`
  },
  'CVE-L1-BORSH-UNBOUNDED-DESER-OOM': {
    root_cause: "Désérialisation Borsh (try_from_slice) ou décompression Zstd directe sur une trame P2P/RPC non bornée, permettant à un pair malveillant de provoquer un crash Out-Of-Memory (OOM) du validateur via un préfixe de longueur forgé.",
    gold_pattern: `// Patron de Remédiation Certifié (Borne Stricte Pré-Désérialisation Borsh & Décompression Zstd Bornée) :
use std::io::Read;
const MAX_NETWORK_MESSAGE_BYTES: usize = 64 * 1024 * 1024; // 64 MiB max
const MAX_DECOMPRESSED_WITNESS_BYTES: u64 = 128 * 1024 * 1024;

pub fn decode_peer_message(raw_payload: &[u8]) -> Result<PeerMessage, NetworkError> {
    if raw_payload.len() > MAX_NETWORK_MESSAGE_BYTES {
        return Err(NetworkError::PayloadTooLarge(raw_payload.len()));
    }
    let decoder = zstd::stream::read::Decoder::new(raw_payload)?;
    let mut bounded_reader = decoder.take(MAX_DECOMPRESSED_WITNESS_BYTES);
    let mut decompressed = Vec::new();
    bounded_reader.read_to_end(&mut decompressed)?;
    PeerMessage::try_from_slice(&decompressed).map_err(NetworkError::InvalidBorsh)
}`
  },
  'CVE-L1-RUNTIME-GAS-PREPAY-BYPASS': {
    root_cause: "Une Host Function Wasm (VMLogic) alloue un buffer sur le tas hôte ou lit la mémoire linéaire du contrat avant d'avoir facturé le gaz proportionnel (burn_gas), permettant un déni de service gratuit par épuisement CPU/RAM.",
    gold_pattern: `// Patron de Remédiation Certifié (Prépaiement Atomique de Gaz AVANT Allocation Mémoire Hôte) :
pub fn host_read_memory(&mut self, ptr: u64, len: u64) -> Result<Vec<u8>, VMLogicError> {
    // 1. Vérifier les bornes de registre avant toute opération :
    if len > self.config.limit_config.max_register_size {
        return Err(HostError::MemoryAccessViolation.into());
    }
    // 2. Facturer le gaz de base + linéaire AVANT d'allouer ou de toucher la RAM :
    let gas_cost = self.gas_cost_base.checked_add(len.checked_mul(self.gas_per_byte)?)?;
    self.result_state.gas_counter.burn_gas(gas_cost)?;
    // 3. Seulement après prépaiement validé, allouer et lire :
    let mut buf = vec![0u8; len as usize];
    self.memory.read_memory(ptr, &mut buf)?;
    Ok(buf)
}`
  },
  'CVE-L1-BALANCE-STORAGE-STAKING-DESYNC': {
    root_cause: "Mutation directe de Account (set_amount / set_locked) avec saturating_sub ou sans vérification finale de l'invariant de Storage Staking (check_storage_stake), exposant le protocole à une insolvabilité d'état ou une inflation de jetons.",
    gold_pattern: `// Patron de Remédiation Certifié (Arithmétique Checked & Invariant de Storage Staking) :
pub fn debit_account_and_verify_storage(
    account: &mut Account,
    debit_amount: Balance,
    runtime_config: &RuntimeConfig,
) -> Result<(), RuntimeError> {
    // 1. Interdire saturating_sub sur les soldes : exiger checked_sub
    let new_amount = account
        .amount()
        .checked_sub(debit_amount)
        .ok_or(ActionErrorKind::LackBalanceForState { amount: debit_amount })?;
    account.set_amount(new_amount);

    // 2. Vérifier systématiquement que le solde restant couvre storage_usage * storage_amount_per_byte
    check_storage_stake(account, account.amount(), runtime_config)?;
    Ok(())
}`
  },
  'CVE-L1-CONSENSUS-UNVERIFIED-WITNESS-CHUNK': {
    root_cause: "Un ChunkStateWitness ou PartialEncodedChunk est décompressé, appliqué au runtime ou stocké dans l'Orphan Pool avant la vérification cryptographique de la signature du validateur assigné dans l'EpochManager.",
    gold_pattern: `// Patron de Remédiation Certifié (Vérification Cryptographique EpochManager Pré-Décompression) :
pub fn process_incoming_state_witness(
    epoch_manager: &dyn EpochManagerAdapter,
    signed_witness: &SignedEncodedChunkStateWitness,
) -> Result<(), Error> {
    // 1. Vérifier que l'émetteur est bien le chunk producer légitime et valider sa signature :
    if !epoch_manager.verify_chunk_state_witness_signature(signed_witness)? {
        return Err(Error::InvalidWitnessSignature);
    }
    // 2. Seulement après authentification cryptographique, décompresser et appliquer :
    let witness = signed_witness.decompress_bounded()?;
    apply_chunk_state_witness(witness)
}`
  },
  'CVE-L1-CROSS-SHARD-RECEIPT-NONCE-REPLAY': {
    root_cause: "Validation incomplète d'une Meta-Transaction NEP-366 (DelegateAction) ou d'un reçu inter-shard : absence de contrôle sur max_block_height ou non-incrémentation atomique de access_key.nonce avant création du Receipt.",
    gold_pattern: `// Patron de Remédiation Certifié (Validation NEP-366 Anti-Replay & Mise à Jour Atomique du Nonce) :
pub fn validate_and_commit_delegate_action(
    state_update: &mut TrieUpdate,
    delegate_action: &DelegateAction,
    current_block_height: BlockHeight,
    access_key: &mut AccessKey,
) -> Result<(), InvalidTxError> {
    if current_block_height > delegate_action.max_block_height {
        return Err(InvalidTxError::Expired);
    }
    if delegate_action.nonce <= access_key.nonce {
        return Err(InvalidTxError::InvalidNonce { tx_nonce: delegate_action.nonce, ak_nonce: access_key.nonce });
    }
    // Persister atomiquement le nouveau nonce AVANT d'émettre le Receipt cross-shard :
    access_key.nonce = delegate_action.nonce;
    set_access_key(state_update, &delegate_action.sender_id, &delegate_action.public_key, access_key);
    Ok(())
}`
  },
  'CVE-L1-CRYPTO-BLIND-SCALAR-MERKLE-MALLEABILITY': {
    root_cause: "Décompression de point elliptique sans vérification d'appartenance à la courbe/sous-groupe (*_unchecked) ou vérification Ed25519 non stricte acceptant une composante de torsion d'ordre 8 (malléabilité de signature/VRF).",
    gold_pattern: `// Patron de Remédiation Certifié (Vérification Stricte Ed25519 & Contrôle de Sous-Groupe d'Ordre Premier) :
pub fn verify_validator_signature_strict(
    pub_key_bytes: &[u8; 32],
    msg: &[u8],
    sig_bytes: &[u8; 64],
) -> Result<(), CryptoError> {
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(pub_key_bytes)?;
    if verifying_key.is_weak() {
        return Err(CryptoError::SmallOrderSubgroupPoint);
    }
    let signature = ed25519_dalek::Signature::from_bytes(sig_bytes);
    // verify_strict rejette les points de torsion et garantit S < L (unicité canonique) :
    verifying_key.verify_strict(msg, &signature).map_err(|_| CryptoError::InvalidSignature)
}`
  },
  'CVE-RUST-LLVM-DEAD-STORE-SECRET-LEAK': {
    root_cause: "Les passes d'optimisation Dead Store Elimination (DSE) de LLVM suppriment toute écriture mémoire (fill(0), write_bytes) sur un tampon local dont la valeur n'est plus relue avant la fin de sa durée de vie, laissant la clé secrète en clair dans la RAM.",
    gold_pattern: `use zeroize::Zeroize;
pub fn clear_secret_key(mut secret_key: [u8; 32]) {
    // zeroize utilise write_volatile + atomic::compiler_fence(Ordering::SeqCst)
    // garantissant que LLVM ne supprimera jamais l'effacement :
    secret_key.zeroize();
}`
  },
  'CVE-RUST-UNWIND-PANIC-DOUBLE-DROP-UB': {
    root_cause: "Dupliquer une valeur par ptr::read() ou étendre set_len() avant d'invoquer une closure ou fonction pouvant paniquer déclenche un double-free lorsque le déroulement de pile (stack unwinding) exécute le Drop du conteneur.",
    gold_pattern: `// Réduire la longueur à un état sûr AVANT l'opération susceptible de paniquer :
unsafe {
    let old_len = buf.len();
    buf.set_len(0); // Guard contre le double-drop en cas d'unwind
    let item = std::ptr::read(buf.as_ptr());
    user_callback(&item)?;
    buf.set_len(old_len);
}`
  },
  'CVE-RUST-MAYBEUNINIT-PADDING-AND-POISON-UB': {
    root_cause: "Appeler MaybeUninit::uninit().assume_init() produit une valeur LLVM poison (UB immédiat), et sérialiser un struct via slice::from_raw_parts expose sur le réseau les octets de padding d'alignement non initialisés de la pile.",
    gold_pattern: `// 1. Utiliser MaybeUninit::zeroed() ou initialiser chaque champ via addr_of_mut!
// 2. Pour la sérialisation binaire zero-copy, dériver bytemuck::NoUninit :
#[repr(C)]
#[derive(Copy, Clone, bytemuck::NoUninit)]
pub struct WireHeader {
    pub magic: u32,
    pub flags: u16,
    pub _pad: u16, // Padding explicite mis à zéro
}`
  },
  'CVE-RUST-ALLOC-LAYOUT-ZERO-OR-ISIZE-OVERFLOW': {
    root_cause: "Passer un Layout de taille 0 à GlobalAlloc::alloc() est un Comportement Indéfini selon la spécification Rust, et calculer size * count sans checked_mul permet un dépassement entier allouant un tampon sous-dimensionné.",
    gold_pattern: `pub fn safe_aligned_alloc(count: usize, elem_size: usize, align: usize) -> Result<std::ptr::NonNull<u8>, &'static str> {
    let total_size = count.checked_mul(elem_size).ok_or("Allocation overflow")?;
    if total_size == 0 || total_size > isize::MAX as usize {
        return Err("Invalid zero or oversized allocation layout");
    }
    let layout = std::alloc::Layout::from_size_align(total_size, align).map_err(|_| "Bad layout")?;
    let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
    std::ptr::NonNull::new(ptr).ok_or("Out of memory")
}`
  },
  'CVE-RUST-FFI-PANIC-ACROSS-ABI-BOUNDARY': {
    root_cause: "Une panique Rust traversant une frontière extern \"C\" (#[no_mangle]) sans catch_unwind corrompt les trames d'appel de l'hôte C/C++/Python/JVM (Comportement Indéfini immédiat).",
    gold_pattern: `#[no_mangle]
pub extern "C" fn ffi_process_frame(ptr: *const u8, len: usize) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if ptr.is_null() { return -1; }
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        handle_frame_internal(slice)
    })) {
        Ok(code) => code,
        Err(_) => -500, // Panique interceptée proprement à la frontière ABI
    }
}`
  },
  'CVE-RUST-UNSAFECELL-CONCURRENT-MUT-NOALIAS-UB': {
    root_cause: "Créer &mut *cell.get() depuis un UnsafeCell partagé entre plusieurs threads (impl Sync) viole la règle d'unicité exclusive (&mut = LLVM noalias), causant des corruptions de données sous optimisation.",
    gold_pattern: `// Utiliser Mutex<T> / RwLock<T> ou Atomic* au lieu d'exposer &mut T depuis UnsafeCell :
pub struct SafeSharedState<T> {
    inner: std::sync::RwLock<T>,
}
impl<T> SafeSharedState<T> {
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.write().unwrap_or_else(|e| e.into_inner());
        f(&mut *guard)
    }
}`
  },
  'CVE-RUST-MMAP-EXTERNAL-TRUNCATION-SIGBUS': {
    root_cause: "Mapper un fichier avec memmap2::Mmap sans verrouillage exclusif permet à un processus externe de tronquer le fichier sous-jacent ; tout accès mémoire au-delà de la nouvelle fin de fichier déclenche un signal matériel SIGBUS fatal.",
    gold_pattern: `use fs2::FileExt;
pub fn map_file_exclusively(file: &std::fs::File) -> std::io::Result<memmap2::Mmap> {
    // Verrouiller le fichier au niveau OS avant de projeter ses pages en mémoire :
    file.lock_exclusive()?;
    unsafe { memmap2::MmapOptions::new().map(file) }
}`
  }
};

function extractCodeSnippetForAnomaly(filePath, lineNumber) {
  const nodes = state.data?.nodes || [];
  const fileNode = nodes.find(n => n.file_path === filePath && n.metadata?.raw_content)
    || nodes.find(n => n.file_path === filePath);
  const raw = fileNode?.metadata?.raw_content;
  if (!raw) return null;
  const lines = raw.split(/\r?\n/);
  const start = Math.max(0, (lineNumber || 1) - 10);
  const end = Math.min(lines.length, (lineNumber || 1) + 14);
  return lines.slice(start, end).map((l, i) => {
    const ln = start + i + 1;
    const marker = ln === lineNumber ? '>> ' : '   ';
    return `${marker}${ln}: ${l}`;
  }).join('\n');
}

function buildSingleAnomalyPerfectBlock(a, indexNum = null, includeGoldPatch = false) {
  const gold = CVE_GOLD_REMEDIATION_PATTERNS[a.rule_id] || null;
  const expl = CVE_ATTACK_EXPLANATIONS[a.rule_id] || null;
  const snippet = extractCodeSnippetForAnomaly(a.file_path, a.line_number);
  const prefix = indexNum !== null ? `### ${indexNum}. ` : `## `;

  let block = `${prefix}🎯 Cible : \`${a.file_path}:${a.line_number}\` — [${a.severity}] \`${a.cve_id}\` (\`${a.rule_id}\`)\n`;
  block += `- **Symbole / Fonction Cible :** \`${a.symbol_name || a.file_path}\`\n`;
  block += `- **Vecteur d'Entrée / Donnée Contaminée (Tainted Flow) :** \`${a.transmitted_data || 'N/A'}\` (\`${a.transmitted_type || 'N/A'}\`)\n`;
  block += `- **Diagnostic AST :** ${a.message}\n`;
  if (expl?.attack) {
    block += `- **💥 Mécanisme de la Faille & Scénario de Déclenchement :** ${expl.attack}\n`;
  }
  if (gold?.root_cause) {
    block += `- **🔬 Root Cause Technique :** ${gold.root_cause}\n`;
  }
  if (!includeGoldPatch) {
    block += `- **🧪 Objectif Proof of Concept (PoC) :** Construire un test unitaire isolé et reproductible (\`#[test]\` / \`#[tokio::test]\` ou Foundry \`test_poc_...\`) qui appelle \`${a.symbol_name || a.file_path}\` avec une entrée contrôlée déclenchant la ligne **${a.line_number}** et démontre concrètement l'impact (Panic, UB, corruption d'état, OOM borné, contournement d'invariant ou perte de fonds) **sans modifier le fichier source**.\n`;
  } else {
    if (expl?.shield) {
      block += `- **🛡️ Protection Requise :** ${expl.shield}\n`;
    }
    block += `- **Directive de Remédiation :** ${a.remediation}\n`;
  }
  if (snippet) {
    block += `\n**Extrait du Code Source Cible (\`${a.file_path}\`, ligne ${a.line_number} marquée par \`>>\`) :**\n\`\`\`rust\n${snippet}\n\`\`\`\n`;
  }
  if (includeGoldPatch && gold?.gold_pattern) {
    block += `\n**Patron de Correction Idiomatique Attendu (Gold Standard) :**\n\`\`\`rust\n${gold.gold_pattern}\n\`\`\`\n`;
  }
  return block;
}

function getActiveBountyScopeSummaryBlock() {
  const inRaw = (document.getElementById('bounty-inscope-input')?.value || '').trim();
  const outRaw = (document.getElementById('bounty-outscope-input')?.value || '').trim();
  const projName = state.data?.project_name || 'Projet Cible';
  const projPath = state.data?.project_path || '';

  let s = `## 🧭 PÉRIMÈTRE BUG BOUNTY ACTIF (${projName})\n`;
  if (projPath) s += `- **Dépôt analysé :** \`${projPath}\`\n`;
  s += `- **✅ IN-SCOPE :** ${inRaw ? inRaw.replace(/\n+/g, ' | ') : 'Modules de production principaux (hors tests/exemples/outillage)'}\n`;
  s += `- **🚫 OUT-OF-SCOPE :** ${outRaw ? outRaw.replace(/\n+/g, ' | ') : 'tests/, testsuite/, benches/, examples/, fixtures/, outillage interne, rapports théoriques sans PoC'}\n`;
  return s;
}

function copyCvePocBountyPrompt(idx) {
  const a = currentCveAnomaliesCache[idx];
  if (!a) return;

  const detailBlock = buildSingleAnomalyPerfectBlock(a, null, false);
  const scopeBlock = getActiveBountyScopeSummaryBlock();

  const prompt = `# 🧪 PROMPT AUTO PROOF OF CONCEPT (PoC) & RAPPORT BUG BOUNTY — AETHERSCOPE V2

Tu agis en tant que **Chercheur Principal Zero-Day & Triager Bug Bounty Tier-1 (Rust / Systèmes / Web3 / L1)**.

> ⛔ **RÈGLE ABSOLUE (ZÉRO CORRECTION BARBARE) :**  
> **NE MODIFIE AUCUN FICHIER SOURCE DU PROJET** et ne propose pas une réécriture massive du code.  
> Ton unique mission est de **vérifier l'exploitabilité réelle In-Scope** de la faille ci-dessous et de produire un **Proof of Concept (PoC) exécutable et reproductible** ainsi que le **Rapport Bug Bounty** prêt à soumettre.

${scopeBlock}

---

${detailBlock}
---

### 🎯 LIVRABLES ATTENDUS (PROTOCOLE BUG BOUNTY)

1. **🔍 Étape 1 — Qualification In-Scope & Vérification Source-to-Sink (Anti-Faux-Positif) :**
   - Vérifie que \`${a.file_path}:${a.line_number}\` appartient bien au périmètre **IN-SCOPE** et n'est pas un code de test/mock/outil interne.
   - Trace le chemin exact d'atteignabilité (*Call Graph / Data Flow*) depuis un point d'entrée externe (message réseau P2P, requête RPC, transaction, bloc, fichier ou appel public) jusqu'au sink ligne **${a.line_number}**.
   - Si une vérification en amont rend l'exploitation impossible, indique honnêtement **"FAUX POSITIF / NON ATTEIGNABLE"** en expliquant quel garde bloque l'entrée.

2. **🧪 Étape 2 — Proof of Concept (PoC) Reproductible & Autonome :**
   - Écris un test de reproduction complet et directement exécutable (\`#[test]\` ou \`#[tokio::test]\` en Rust, \`test_poc_...\` sous Foundry, ou script de reproduction isolé) qui :
     - Initialise uniquement l'état minimal requis,
     - Injecte le vecteur déclencheur précis (\`${a.transmitted_data || 'payload'}\`),
     - Appelle \`${a.symbol_name || a.file_path}\` et déclenche concrètement la faille (assertion sur l'état corrompu, contournement d'invariant, panic, OOM borné ou UB détectable par \`cargo test\` / \`cargo miri test\`),
   - Fournis la commande exacte pour lancer uniquement ce PoC (ex: \`cargo test poc_${(a.rule_id || 'vuln').toLowerCase().replace(/[^a-z0-9]/g, '_')} -- --nocapture\`).

3. **🏆 Étape 3 — Rapport Bug Bounty Prêt à Soumettre (Format Immunefi / HackerOne / Cantina) :**
   - **Title :** Titre percutant résumant la cause et l'impact critique
   - **Severity & Impact Category :** Critical / High + catégorie d'impact In-Scope
   - **Vulnerable Component :** \`${a.file_path}:${a.line_number}\` (\`${a.symbol_name || 'N/A'}\`)
   - **Root Cause & Step-by-Step Exploitation Scenario :** Explication chirurgicale
   - **Proof of Concept (PoC Code) :** Le code du test reproductible
   - **Suggested Minimal Fix (3-5 lignes max en commentaire uniquement) :** Juste pour la section "Recommendation" du rapport, sans modifier les fichiers du dépôt.

<aether_handshake version="2.0">
  <security_target file="${a.file_path}" line="${a.line_number}" rule="${a.rule_id}" cve="${a.cve_id}" severity="${a.severity}">
    <tainted_flow data="${a.transmitted_data || ''}" type="${a.transmitted_type || ''}"/>
    <diagnostic>${a.message}</diagnostic>
  </security_target>
  <task objective="Générer un Proof of Concept (PoC) reproductible et un rapport Bug Bounty pour ${a.cve_id} sans modifier le code source">
    Analyse \`${a.file_path}:${a.line_number}\`, valide le chemin Source-to-Sink In-Scope et rédige le test PoC exécutable + rapport Bug Bounty complet.
  </task>
</aether_handshake>
`;

  navigator.clipboard.writeText(prompt).then(() => {
    AudioSys.promptCopied();
    showToast(`🧪 Prompt PoC Bug Bounty (${a.cve_id}) copié !`);
  });
}
window.copyCvePocBountyPrompt = copyCvePocBountyPrompt;

function copyCveSinglePatchPrompt(idx) {
  const a = currentCveAnomaliesCache[idx];
  if (!a) return;

  const detailBlock = buildSingleAnomalyPerfectBlock(a, null, true);

  const prompt = `# 🛡️ PROMPT DE PATCH MINIMAL CIBLÉ — AETHERSCOPE V2

Corrige de manière minimale et non-destructive la vulnérabilité identifiée dans \`${a.file_path}:${a.line_number}\` sans refactoriser le reste du fichier :

${detailBlock}
---

<aether_handshake version="2.0">
  <security_target file="${a.file_path}" line="${a.line_number}" rule="${a.rule_id}" cve="${a.cve_id}" severity="${a.severity}">
    <tainted_flow data="${a.transmitted_data || ''}" type="${a.transmitted_type || ''}"/>
    <diagnostic>${a.message}</diagnostic>
    <remediation>${a.remediation}</remediation>
  </security_target>
  <task objective="Patch minimal sans régression de ${a.rule_id}">
    Applique uniquement le correctif minimal sur \`${a.file_path}:${a.line_number}\` sans modifier les signatures publiques.
  </task>
</aether_handshake>
`;

  navigator.clipboard.writeText(prompt).then(() => {
    AudioSys.promptCopied();
    showToast('🛡️ Prompt de Patch Minimal copié !');
  });
}
window.copyCveSinglePatchPrompt = copyCveSinglePatchPrompt;

// Par défaut, copyCveRemediationPrompt génère désormais le Prompt PoC Bug Bounty
function copyCveRemediationPrompt(idx) {
  copyCvePocBountyPrompt(idx);
}
window.copyCveRemediationPrompt = copyCveRemediationPrompt;

function copyAllCveHandshakes() {
  const filteredInScope = typeof getFilteredBountyAnomalies === 'function'
    ? getFilteredBountyAnomalies()
    : (currentCveAnomaliesCache || []);

  const securityAnomalies = (filteredInScope.length > 0 ? filteredInScope : (currentCveAnomaliesCache || [])).filter(a => {
    const isWarn = (a.severity || '').toUpperCase() === 'WARNING';
    const isMaxLines = a.rule_id === 'RULE_MAX_LINES' || (a.cve_id && a.cve_id.includes('ARCH-BUDGET-EXCEEDED'));
    return !isWarn && !isMaxLines;
  });

  if (securityAnomalies.length === 0) {
    showToast('✅ Zéro anomalie CVE In-Scope restante !');
    return;
  }

  const scopeBlock = getActiveBountyScopeSummaryBlock();

  let prompt = `# 🧪 PROMPT GLOBAL DE TRIAGE & PROOF OF CONCEPT (PoC) BUG BOUNTY — AETHERSCOPE V2 (${securityAnomalies.length} Cibles In-Scope)\n\n`;
  prompt += `Tu agis en tant que **Chercheur Principal Zero-Day & Triager Bug Bounty Tier-1**.\n\n`;
  prompt += `> ⛔ **INTERDICTION DE CORRECTION BARBARE :** Ne modifie aucun fichier source du projet. Ton rôle est de trier les ${securityAnomalies.length} alertes SAST ci-dessous selon le périmètre Bug Bounty, d'éliminer impitoyablement les faux positifs, et de rédiger un **Proof of Concept (PoC) exécutable** pour chaque vraie vulnérabilité exploitable.\n\n`;
  prompt += `${scopeBlock}\n\n---\n\n`;

  const maxToInclude = Math.min(securityAnomalies.length, 25);
  for (let i = 0; i < maxToInclude; i++) {
    prompt += buildSingleAnomalyPerfectBlock(securityAnomalies[i], i + 1, false) + '\n---\n\n';
  }
  if (securityAnomalies.length > maxToInclude) {
    prompt += `*... (+ ${securityAnomalies.length - maxToInclude} autres alertes disponibles dans AetherScope)*\n\n`;
  }

  prompt += `### 🎯 Protocole de Sortie Attendu (Triage & PoC Bug Bounty)\n`;
  prompt += `1. **Tableau de Triage Rapide :** Classe les alertes en **✅ VRAI POSITIF EXPLOITABLE IN-SCOPE** vs **🚫 FAUX POSITIF / HORS-SCOPE**.\n`;
  prompt += `2. **Proof of Concept (PoC) Complet :** Pour chaque vrai positif confirmé, fournis un test de reproduction autonome (\`#[test]\` Rust / Foundry \`test_poc\`) démontrant le crash, l'UB, le DoS mémoire ou la perte de fonds sans toucher au code de production.\n`;
  prompt += `3. **Rapport Bug Bounty Prêt à Soumettre :** Titre, Sévérité CVSS, Root Cause, Scénario d'attaque Source-to-Sink et PoC.\n\n`;

  prompt += `<aether_handshake version="2.0">\n`;
  prompt += `  <task objective="Triage Bug Bounty et génération de Proofs of Concept (PoC) sur ${securityAnomalies.length} alertes In-Scope">\n`;
  prompt += `    Trie les ${securityAnomalies.length} alertes ci-dessus, écarte les faux positifs et génère les Proofs of Concept (PoC) reproductibles.\n`;
  prompt += `  </task>\n`;
  prompt += `</aether_handshake>\n`;

  navigator.clipboard.writeText(prompt).then(() => {
    AudioSys.promptCopied();
    showToast(`🧪 Prompt PoC Bug Bounty Global (${securityAnomalies.length} CVE In-Scope) copié !`);
  });
}
window.copyAllCveHandshakes = copyAllCveHandshakes;

function toggleSingleRelation(relKey, rowEl) {
  if (!state.activeRelations) {
    state.activeRelations = new Set();
  }

  const synonyms = [relKey];
  if (relKey === 'realtime') synonyms.push('realtimews', 'realtime_ws');
  if (relKey === 'violates') synonyms.push('violation', 'security_violation', 'security_violates');

  const currentlyActive = synonyms.some(k => state.activeRelations.has(k));

  if (currentlyActive) {
    synonyms.forEach(k => state.activeRelations.delete(k));
    if (rowEl) {
      rowEl.classList.add('disabled');
      const b = rowEl.querySelector('.legend-toggle-badge');
      if (b) b.innerText = 'OFF';
    }
    AudioSys.playTone(340, 'triangle', 0.12, 0.03);
    showToast(`🚫 Liens masqués : ${relKey}`);
  } else {
    synonyms.forEach(k => state.activeRelations.add(k));
    if (rowEl) {
      rowEl.classList.remove('disabled');
      const b = rowEl.querySelector('.legend-toggle-badge');
      if (b) b.innerText = 'ON';
    }
    AudioSys.playTone(660, 'sine', 0.12, 0.04);
    showToast(`👁️ Liens affichés : ${relKey}`);
  }

  invalidateEdgePathCache();
  wakePhysics(30);
}
window.toggleSingleRelation = toggleSingleRelation;

function toggleDataPipelineTracker() {
  state.dataPipelineMode = !state.dataPipelineMode;
  const btn = document.getElementById('btn-toggle-data-pipeline');
  if (btn) {
    btn.classList.toggle('active', state.dataPipelineMode);
    const badge = btn.querySelector('.pipeline-tracker-badge');
    if (badge) {
      badge.innerText = state.dataPipelineMode ? 'ON' : 'OFF';
      badge.style.color = state.dataPipelineMode ? '#00f0ff' : '#94a3b8';
      badge.style.background = state.dataPipelineMode ? 'rgba(0, 240, 255, 0.3)' : 'rgba(0, 240, 255, 0.1)';
    }
  }
  state.instanceDataDirty = true;
  invalidateEdgePathCache();
  wakePhysics(60);
  AudioSys.playTone(state.dataPipelineMode ? 780 : 380, 'sine', 0.15, 0.04);
  showToast(state.dataPipelineMode ? '⚡ Traçabilité Données E2E Activée (Form Submit ➔ Net HTTP ➔ WS Temps Réel ➔ DB)' : '⚡ Traçabilité Données : Désactivée');
}
window.toggleDataPipelineTracker = toggleDataPipelineTracker;

document.getElementById('btn-toggle-all-edges')?.addEventListener('click', () => {
  const allKnown = [
    'submits', 'network', 'realtime', 'realtimews', 'realtime_ws', 'queries',
    'calls', 'renders', 'imports', 'contains', 'inherits',
    'violates', 'violation', 'security_violation', 'security_violates',
    'documents', 'binds', 'inter_module'
  ];
  (state.data?.edges || []).forEach(e => {
    if (e.relation) allKnown.push(e.relation);
  });

  const rows = document.querySelectorAll('.legend-row');
  if (state.activeRelations.size > 0) {
    state.activeRelations.clear();
    rows.forEach(r => {
      r.classList.add('disabled');
      const b = r.querySelector('.legend-toggle-badge');
      if (b) b.innerText = 'OFF';
    });
    AudioSys.playTone(280, 'triangle', 0.15, 0.04);
    showToast('🚫 Tous les liens sont masqués');
  } else {
    allKnown.forEach(r => state.activeRelations.add(r));
    rows.forEach(r => {
      r.classList.remove('disabled');
      const b = r.querySelector('.legend-toggle-badge');
      if (b) b.innerText = 'ON';
    });
    AudioSys.playTone(720, 'sine', 0.15, 0.04);
    showToast('👁️ Tous les liens sont affichés');
  }
  invalidateEdgePathCache();
  wakePhysics(30);
});

// --- Left Synthesis Menu Burger Controller ---
function toggleSynthesisMenu() {
  const card = document.getElementById('synthesis-card');
  const trigger = document.getElementById('btn-toggle-synthesis-burger');
  if (!card) return;
  const isCollapsed = card.classList.toggle('collapsed');
  state.synthesisMenuCollapsed = isCollapsed;
  if (trigger) {
    trigger.classList.toggle('active', !isCollapsed);
  }
  AudioSys.playTone(isCollapsed ? 320 : 640, 'sine', 0.10, 0.03);
  showToast(isCollapsed ? '📁 Panneau Synthèse replié (B)' : '🪐 Panneau Synthèse ouvert (B)');
}
window.toggleSynthesisMenu = toggleSynthesisMenu;

function openSynthesisMenu() {
  const card = document.getElementById('synthesis-card');
  const trigger = document.getElementById('btn-toggle-synthesis-burger');
  if (card && card.classList.contains('collapsed')) {
    card.classList.remove('collapsed');
    state.synthesisMenuCollapsed = false;
    if (trigger) trigger.classList.add('active');
  }
}
window.openSynthesisMenu = openSynthesisMenu;

function closeSynthesisMenu() {
  const card = document.getElementById('synthesis-card');
  const trigger = document.getElementById('btn-toggle-synthesis-burger');
  if (card && !card.classList.contains('collapsed')) {
    card.classList.add('collapsed');
    state.synthesisMenuCollapsed = true;
    if (trigger) trigger.classList.remove('active');
  }
}
window.closeSynthesisMenu = closeSynthesisMenu;

document.getElementById('btn-toggle-synthesis-burger')?.addEventListener('click', toggleSynthesisMenu);
document.getElementById('btn-close-synthesis')?.addEventListener('click', closeSynthesisMenu);
document.getElementById('btn-collapse-synthesis')?.addEventListener('click', toggleSynthesisMenu);

// Snapshot 4K / HD Viewport Exporter
function takeSnapshot() {
  showToast('📸 Génération de la capture HD...');
  try {
    const offscreen = document.createElement('canvas');
    offscreen.width = canvas.width;
    offscreen.height = canvas.height;
    const oCtx = offscreen.getContext('2d');
    if (!oCtx) return;

    oCtx.drawImage(canvas, 0, 0);
    oCtx.drawImage(lineCanvas, 0, 0);

    oCtx.font = '700 13px ui-monospace, Menlo, Consolas, monospace';
    oCtx.fillStyle = 'rgba(56, 189, 248, 0.85)';
    oCtx.textAlign = 'right';
    const label = state.xrayMode && state.xrayTunnel 
      ? `✦ AetherScope X-Ray Focus [${state.xrayTunnel.centerNode.name}]` 
      : `✦ AetherScope • ${state.data.project_name || 'Cosmos'}`;
    oCtx.fillText(label, offscreen.width - 24, offscreen.height - 24);

    const dataUrl = offscreen.toDataURL('image/png');
    const a = document.createElement('a');
    const nodeName = state.selectedNode ? state.selectedNode.name.replace(/[^a-zA-Z0-9_-]/g, '_') : 'cosmos';
    const prefix = state.xrayMode ? 'xray_' : '';
    a.download = `aetherscope_${prefix}${nodeName}_${Date.now()}.png`;
    a.href = dataUrl;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    AudioSys.playTone(880, 'sine', 0.22, 0.05);
    showToast(`✅ Snapshot PNG HD enregistré : ${a.download}`);
  } catch (err) {
    showToast('❌ Erreur lors de la capture');
  }
}
window.takeSnapshot = takeSnapshot;

// HUD Buttons
document.getElementById('btn-xray')?.addEventListener('click', toggleXrayMode);
document.getElementById('btn-xray-exit')?.addEventListener('click', toggleXrayMode);
document.getElementById('btn-xray-snapshot')?.addEventListener('click', takeSnapshot);

document.getElementById('btn-deadcode')?.addEventListener('click', toggleDeadCodeMode);
document.getElementById('btn-deadcode-exit')?.addEventListener('click', toggleDeadCodeMode);
document.getElementById('btn-deadcode-ai-clean')?.addEventListener('click', () => {
  const p = generateDeadCodeCleanPrompt();
  navigator.clipboard.writeText(p).then(() => {
    AudioSys.promptCopied();
    showToast('📋 Prompt Nettoyage Code Mort copié !');
  });
});

document.getElementById('btn-heatmap')?.addEventListener('click', toggleHeatmapMode);
document.getElementById('btn-heatmap-exit')?.addEventListener('click', toggleHeatmapMode);
document.getElementById('btn-heatmap-ai-audit')?.addEventListener('click', () => {
  const p = generateHeatmapArchitecturePrompt();
  navigator.clipboard.writeText(p).then(() => {
    AudioSys.promptCopied();
    showToast('📋 Prompt Audit Architecture Heatmap copié !');
  });
});

document.getElementById('btn-snapshot')?.addEventListener('click', takeSnapshot);

function initDisplaySettingsUI() {
  const displaySettings = [
    { key: 'showFolderBadges', def: false },
    { key: 'showFolderMembranes', def: true },
    { key: 'showGrid', def: true },
    { key: 'showStarfield', def: true },
    { key: 'showShadows', def: true }
  ];
  displaySettings.forEach(s => {
    if (state[s.key] === undefined) state[s.key] = s.def;
    const isEnabled = state[s.key];
    const item = document.getElementById(`btn-menu-toggle-${s.key}`);
    if (item) {
      const badge = item.querySelector('.menu-toggle-badge');
      if (badge) {
        badge.innerText = isEnabled ? 'ON' : 'OFF';
        badge.style.color = isEnabled ? '#38bdf8' : '#64748b';
      }
      item.classList.toggle('active', isEnabled);
    }
  });
  if (typeof syncFolderBadgesButtonUI === 'function') syncFolderBadgesButtonUI();
}
window.initDisplaySettingsUI = initDisplaySettingsUI;

// =========================================================================
// 🎯 PRÉPARATEUR DE PROMPT CVE & SCOPE BUG BOUNTY (IN-SCOPE / OUT-OF-SCOPE)
// =========================================================================
let currentBountyMode = 'triage_poc';
let bountyExcludedFoldersSet = new Set();

function isTestOrInternalToolFolder(folderName) {
  const f = (folderName || '').toLowerCase();
  return f.includes('test') || f.includes('bench') || f.includes('example') ||
         f.includes('fixture') || f.includes('mock') || f.includes('devtool') ||
         f.includes('security-linter') || f.includes('ast-scanner') ||
         f.includes('arch-templates') || f.includes('doc');
}

function parseScopePatterns(rawText) {
  if (!rawText) return { paths: [], rules: [] };
  const lines = rawText.split(/[\n,]+/).map(s => s.trim()).filter(Boolean);
  const paths = [];
  const rules = [];
  lines.forEach(line => {
    if (line.startsWith('[') || line.includes(' ') && !line.includes('/') && !line.includes('\\') && !line.includes('.')) {
      rules.push(line);
    } else {
      paths.push(line.replace(/\\/g, '/').toLowerCase());
    }
  });
  return { paths, rules };
}

function renderBountyFoldersPills() {
  const box = document.getElementById('bounty-folders-pills');
  if (!box) return;
  const nodes = state.data?.nodes || [];
  const folderCounts = new Map();
  nodes.forEach(n => {
    const f = n.folder || 'root';
    folderCounts.set(f, (folderCounts.get(f) || 0) + 1);
  });

  if (folderCounts.size === 0) {
    box.innerHTML = '<span style="font-size:0.65rem; color:#64748b;">Aucun dossier chargé.</span>';
    return;
  }

  const sorted = Array.from(folderCounts.entries()).sort((a, b) => a[0].localeCompare(b[0]));
  box.innerHTML = '';
  sorted.forEach(([folder, count]) => {
    const isOut = bountyExcludedFoldersSet.has(folder);
    const pill = document.createElement('button');
    pill.type = 'button';
    pill.className = `bounty-folder-pill ${isOut ? 'out-scope' : 'in-scope'}`;
    const shortName = typeof formatFolderLabel === 'function' ? formatFolderLabel(folder) : folder;
    pill.innerHTML = `<span>${isOut ? '🚫' : '✅'}</span> <span title="${folder}">${shortName}</span> <span style="opacity:0.7;">(${count})</span>`;
    pill.addEventListener('click', () => {
      if (bountyExcludedFoldersSet.has(folder)) {
        bountyExcludedFoldersSet.delete(folder);
      } else {
        bountyExcludedFoldersSet.add(folder);
      }
      syncBountyFoldersToTextareas();
      renderBountyFoldersPills();
      updateBountyPromptPreview();
      AudioSys.playTone(bountyExcludedFoldersSet.has(folder) ? 330 : 620, 'sine', 0.08, 0.02);
    });
    box.appendChild(pill);
  });
}

function syncBountyFoldersToTextareas() {
  const inEl = document.getElementById('bounty-inscope-input');
  const outEl = document.getElementById('bounty-outscope-input');
  if (!inEl || !outEl) return;

  const nodes = state.data?.nodes || [];
  const allFolders = Array.from(new Set(nodes.map(n => n.folder || 'root'))).sort();

  const existingInRules = (inEl.value || '').split('\n').map(s => s.trim()).filter(l => l.startsWith('['));
  const existingOutRules = (outEl.value || '').split('\n').map(s => s.trim()).filter(l => l.startsWith('['));

  const inFolders = allFolders.filter(f => !bountyExcludedFoldersSet.has(f));
  const outFolders = allFolders.filter(f => bountyExcludedFoldersSet.has(f));

  inEl.value = [...inFolders, ...existingInRules].join('\n');
  outEl.value = [...outFolders, 'tests/', 'testsuite/', 'examples/', ...existingOutRules].filter((v, i, a) => a.indexOf(v) === i).join('\n');
}

function setAllBountyFoldersScope(allInScope) {
  bountyExcludedFoldersSet.clear();
  if (!allInScope) {
    const nodes = state.data?.nodes || [];
    nodes.forEach(n => bountyExcludedFoldersSet.add(n.folder || 'root'));
  }
  syncBountyFoldersToTextareas();
  renderBountyFoldersPills();
  updateBountyPromptPreview();
}
window.setAllBountyFoldersScope = setAllBountyFoldersScope;

function autoExcludeTestAndToolFolders() {
  bountyExcludedFoldersSet.clear();
  const nodes = state.data?.nodes || [];
  nodes.forEach(n => {
    const f = n.folder || 'root';
    if (isTestOrInternalToolFolder(f)) {
      bountyExcludedFoldersSet.add(f);
    }
  });
  syncBountyFoldersToTextareas();
  renderBountyFoldersPills();
  updateBountyPromptPreview();
  showToast('🎯 Dossiers de tests, exemples et outils internes basculés en Out-of-Scope');
}
window.autoExcludeTestAndToolFolders = autoExcludeTestAndToolFolders;

function applyBountyScopePreset(preset) {
  const inEl = document.getElementById('bounty-inscope-input');
  const outEl = document.getElementById('bounty-outscope-input');
  if (!inEl || !outEl) return;

  if (preset === 'auto') {
    autoExcludeTestAndToolFolders();
    appendBountyScopeLine('bounty-inscope-input', '[Impact Critique] Vol/Gel de Fonds, Corruption Mémoire (UB), RCE, Partition de Consensus');
    appendBountyScopeLine('bounty-outscope-input', '[Exclu] Rapports théoriques sans PoC reproductible, DoS volumétrique réseau, Dépendances tiers non exploitables');
  } else if (preset === 'php_web') {
    inEl.value = [
      'src/',
      'app/',
      'controllers/',
      'models/',
      'api/',
      'routes/',
      'includes/',
      'config/',
      '[Impact Critique] RCE (Command Injection, Eval, Unserialize POP Gadgets)',
      '[Impact Critique] SQL Injection directe (Vol / Altération BDD)',
      '[Impact Critique] Arbitrary File Upload (Webshell .php)',
      '[Impact High] Server-Side Request Forgery (SSRF) & XXE Injection',
      '[Impact High] Path Traversal, Session Fixation, XSS'
    ].join('\n');
    outEl.value = [
      'vendor/',
      'tests/',
      'test/',
      'cache/',
      'var/cache/',
      'storage/logs/',
      'storage/framework/',
      '[Exclu] Dépendances vendor/ non modifiables directement',
      '[Exclu] Attaques par force brute volumétrique sans impact de bypass logique'
    ].join('\n');
  } else if (preset === 'l1_consensus') {
    inEl.value = [
      'consensus/',
      'execution/',
      'vm/',
      'vm-validator/',
      'network/',
      'storage/',
      'mempool/',
      'state-sync/',
      '[Impact Critique] Consensus Split / Fork / Non-Déterminisme VM',
      '[Impact Critique] Loss or Freeze of Funds / Minting Illimité',
      '[Impact Critique] Crash Validateur à Distance (OOM, Panic, SIGBUS, UB) via message P2P/Transaction'
    ].join('\n');
    outEl.value = [
      'tests/',
      'testsuite/',
      'examples/',
      'devtools/',
      'crates/security-linter/',
      'crates/ast-scanner/',
      'crates/core-graph/',
      '[Exclu] DoS volumétrique réseau ou épuisement RPC public sans crash validateur',
      '[Exclu] Bugs dans les outils CLI locaux ou fonctionnalités expérimentales désactivées sur Mainnet',
      '[Exclu] Attaques nécessitant > 1/3 de validateurs Byzantins malveillants'
    ].join('\n');
  } else if (preset === 'web3_defi') {
    inEl.value = [
      'contracts/',
      'vault/',
      'pools/',
      'oracle/',
      'bridge/',
      '[Impact Critique] Vol direct de collatéral ou drainage de trésorerie (Reentrancy, Arrondi, Flash Loan)',
      '[Impact Critique] Insolvency / Manipulation d\'Oracle TWAP ou Chainlink Min/Max',
      '[Impact High] Blocage permanent des fonds utilisateurs (Channel Bricking / Griefing EIP-150)'
    ].join('\n');
    outEl.value = [
      'test/',
      'mocks/',
      'script/',
      'audits/',
      '[Exclu] Centralisation Admin / Rugpull par clé multisig légitime (Owner)',
      '[Exclu] Front-running MEV standard inhérent aux AMMs sans perte protocole',
      '[Exclu] Tokens ERC-20 malveillants non listés dans la whitelist officielle'
    ].join('\n');
  } else if (preset === 'rust_lowlevel') {
    inEl.value = [
      'src/',
      'crates/',
      'server/',
      'runtime/',
      '[Impact Critique] LLVM Dead Store Elimination (Fuite de clés secrètes en RAM)',
      '[Impact Critique] Panic Unwind Double-Drop / Use-After-Free & MaybeUninit Poison UB',
      '[Impact Critique] GlobalAlloc Layout taille 0 ou overflow isize::MAX',
      '[Impact Critique] Panique FFI traversant extern "C", UnsafeCell noalias UB, Mmap SIGBUS'
    ].join('\n');
    outEl.value = [
      'tests/',
      'benches/',
      'examples/',
      'crates/security-linter/',
      '[Exclu] Fuites mémoire mineures sans impact de sécurité (mem::forget isolé hors verrou)',
      '[Exclu] Paniques dans les binaires de test ou scripts de build (build.rs)'
    ].join('\n');
  }
  updateBountyPromptPreview();
  AudioSys.playTone(680, 'sine', 0.10, 0.03);
  showToast(`⚡ Preset Bug Bounty appliqué : ${preset.toUpperCase()}`);
}
window.applyBountyScopePreset = applyBountyScopePreset;

function appendBountyScopeLine(textareaId, textLine) {
  const el = document.getElementById(textareaId);
  if (!el) return;
  const lines = el.value.split('\n').map(s => s.trim()).filter(Boolean);
  if (!lines.includes(textLine)) {
    lines.push(textLine);
    el.value = lines.join('\n');
  }
  updateBountyPromptPreview();
}
window.appendBountyScopeLine = appendBountyScopeLine;

let currentBountyTargetCveIdx = 'all';

function setBountyPromptMode(mode, btnEl) {
  currentBountyMode = mode;
  document.querySelectorAll('#bounty-mode-selector .task-btn').forEach(b => b.classList.remove('active'));
  if (btnEl) btnEl.classList.add('active');
  updateBountyPromptPreview();
  AudioSys.playTone(560, 'sine', 0.08, 0.02);
}
window.setBountyPromptMode = setBountyPromptMode;

function onBountyTargetCveChange(val) {
  currentBountyTargetCveIdx = val === 'all' ? 'all' : parseInt(val, 10);
  updateBountyPromptPreview();
  AudioSys.playTone(580, 'sine', 0.08, 0.02);
}
window.onBountyTargetCveChange = onBountyTargetCveChange;

function populateBountyTargetCveSelect(filteredAnomalies) {
  const sel = document.getElementById('bounty-target-cve-select');
  if (!sel) return;
  const list = filteredAnomalies || getFilteredBountyAnomalies();
  const prevVal = currentBountyTargetCveIdx;

  let html = `<option value="all">📂 Toutes les CVE In-Scope (${list.length} cibles — Triage & PoC Auto)</option>`;
  (currentCveAnomaliesCache || []).forEach((a, idx) => {
    const isSelected = prevVal !== 'all' && Number(prevVal) === idx;
    const shortFile = (a.file_path || '').split(/[/\\]/).slice(-2).join('/');
    html += `<option value="${idx}" ${isSelected ? 'selected' : ''}>🎯 #${idx + 1} [${a.severity}] ${a.cve_id} — ${shortFile}:${a.line_number}</option>`;
  });
  sel.innerHTML = html;
  if (prevVal === 'all') {
    sel.value = 'all';
  }
}

function getFilteredBountyAnomalies() {
  const allAnomalies = currentCveAnomaliesCache || [];
  const inRaw = document.getElementById('bounty-inscope-input')?.value || '';
  const outRaw = document.getElementById('bounty-outscope-input')?.value || '';
  const onlyCritHigh = document.getElementById('bounty-only-crit-high')?.checked !== false;

  const inParsed = parseScopePatterns(inRaw);
  const outParsed = parseScopePatterns(outRaw);

  return allAnomalies.filter(a => {
    const sev = (a.severity || '').toUpperCase();
    if (onlyCritHigh && sev !== 'CRITICAL' && sev !== 'HIGH') {
      return false;
    }
    const fp = (a.file_path || '').replace(/\\/g, '/').toLowerCase();

    // 1. Check Out-of-Scope path exclusions first
    for (let i = 0; i < outParsed.paths.length; i++) {
      const pat = outParsed.paths[i].replace(/^\*+|\*+$/g, '');
      if (pat && fp.includes(pat)) {
        return false;
      }
    }

    // 2. Check In-Scope path inclusions (if any path patterns are specified)
    if (inParsed.paths.length > 0) {
      const matchesIn = inParsed.paths.some(pat => {
        const cleanPat = pat.replace(/^\*+|\*+$/g, '');
        return !cleanPat || cleanPat === 'root' || fp.includes(cleanPat);
      });
      if (!matchesIn) return false;
    }

    return true;
  });
}

function updateBountyPromptPreview() {
  const inRaw = (document.getElementById('bounty-inscope-input')?.value || '').trim();
  const outRaw = (document.getElementById('bounty-outscope-input')?.value || '').trim();
  const filtered = getFilteredBountyAnomalies();
  const totalCount = (currentCveAnomaliesCache || []).length;
  const excludedCount = Math.max(0, totalCount - filtered.length);

  populateBountyTargetCveSelect(filtered);

  const badgeEl = document.getElementById('bounty-filtered-badge');
  if (badgeEl) {
    if (currentBountyTargetCveIdx !== 'all' && currentCveAnomaliesCache[currentBountyTargetCveIdx]) {
      const targetA = currentCveAnomaliesCache[currentBountyTargetCveIdx];
      badgeEl.innerText = `🎯 Focus PoC : ${targetA.cve_id} (${targetA.file_path}:${targetA.line_number})`;
    } else {
      badgeEl.innerText = `🎯 ${filtered.length} CVE In-Scope (${excludedCount} hors-scope ignorées)`;
    }
  }

  const projName = state.data?.project_name || 'Projet Cible';
  const projPath = state.data?.project_path || '';
  const includePatch = currentBountyMode === 'surgical_fix';

  let p = `# 🧪 PROMPT AUTO PROOF OF CONCEPT (PoC) CVE & BUG BOUNTY — ${projName.toUpperCase()}\n`;
  if (projPath) p += `> **Dépôt local analysé :** \`${projPath}\`\n`;
  p += `> **Moteur SAST :** AetherScope V2 (Rust AST + Taint Analysis + LLVM/Memory Low-Level Rules)\n\n`;

  if (!includePatch) {
    p += `> ⛔ **RÈGLE ABSOLUE (ZÉRO CORRECTION BARBARE) :**  \n`;
    p += `> **NE MODIFIE AUCUN FICHIER SOURCE DU PROJET** et ne réécris pas le code de production.  \n`;
    p += `> Ton unique objectif est de **valider l'exploitabilité réelle In-Scope**, d'éliminer les faux positifs, et de générer un **Proof of Concept (PoC) exécutable et reproductible** (\`#[test]\` Rust, Foundry \`test_poc\`, ou harnais d'exploitation isolé) accompagné du **Rapport Bug Bounty** prêt à soumettre.\n\n`;
  }

  p += `## 🧭 1. PÉRIMÈTRE STRICT DU BUG BOUNTY (SCOPE BOUNDARIES)\n\n`;
  p += `### ✅ IN-SCOPE (Cibles, Composants & Impacts Autorisés)\n`;
  p += `\`\`\`text\n${inRaw || 'Tous les modules de production principaux du projet'}\n\`\`\`\n\n`;

  p += `### 🚫 OUT-OF-SCOPE (Exclusions Strictes — À Ignorer Impérativement)\n`;
  p += `\`\`\`text\n${outRaw || 'tests/, testsuite/, examples/, fixtures/, outillage interne'}\n\`\`\`\n\n`;

  if (currentBountyMode === 'triage_poc') {
    p += `## 🧪 2. OBJECTIF : PREUVE DE CONCEPT (PoC) EXÉCUTABLE, TRIAGE & RAPPORT BUG BOUNTY\n`;
    p += `Tu agis en tant que **Chercheur Principal Zero-Day & Triager Bug Bounty Tier-1** :\n`;
    p += `1. **Filtrage In-Scope & Anti-Faux-Positif :** Rejette immédiatement toute alerte située dans un dossier **OUT-OF-SCOPE** ou dont le vecteur d'entrée est déjà assaini en amont.\n`;
    p += `2. **Traçage d'Atteignabilité (Source-to-Sink) :** Pour chaque CVE retenue dans le **IN-SCOPE**, trace le chemin exact depuis une entrée externe non fiable (paquet P2P, requête RPC, transaction utilisateur, bloc, mémoire partagée ou fichier) jusqu'à la ligne vulnérable.\n`;
    p += `3. **Génération du Proof of Concept (PoC) Reproductible (Sans modifier le code cible) :**\n`;
    p += `   - Fournis un test de reproduction complet et autonome (\`#[test]\` / \`#[tokio::test]\` en Rust, \`test_poc_...\` sous Foundry, ou script de déclenchement minimal) qui appelle la fonction cible avec le payload déclencheur.\n`;
    p += `   - Le PoC doit prouver concrètement l'impact (Panic, UB sous Miri/ASAN, corruption d'état, OOM, contournement de signature/gaz ou vol/gel de fonds) avec la commande exacte pour l'exécuter (\`cargo test ...\`).\n`;
    p += `4. **Rapport Bug Bounty Prêt à Soumettre (Format Immunefi / HackerOne / Cantina) :**\n`;
    p += `   - **Title & Severity (CVSS / Bounty Tier)**\n`;
    p += `   - **In-Scope Asset & Location (\`fichier:ligne\`)**\n`;
    p += `   - **Root Cause & Step-by-Step Exploitation Path**\n`;
    p += `   - **Executable Proof of Concept (PoC)**\n`;
    p += `   - **Minimal Suggested Fix (3-5 lignes en commentaire pour le rapport uniquement, sans toucher aux fichiers)**\n\n`;
  } else if (currentBountyMode === 'deep_hunt') {
    p += `## 🔬 2. OBJECTIF : CHASSE ZERO-DAY & PoC RUST ULTRA-BAS-NIVEAU (IN-SCOPE UNIQUEMENT)\n`;
    p += `Tu agis en tant qu'**Auditeur Système & Chercheur en Vulnérabilités Bas-Niveau (Rust / LLVM / VM / Crypto)**.\n`;
    p += `Sans modifier le code source, concentre 100% de ton analyse sur les dossiers **IN-SCOPE** et construis des **Proofs of Concept (PoC)** sur les failles profondes :\n`;
    p += `- **Optimisations LLVM & Mémoire :** Dead Store Elimination (DSE) effaçant le nettoyage de clés privées sans \`write_volatile\`/\`zeroize\`, lecture de \`MaybeUninit\` non initialisé (LLVM Poison UB), fuite d'octets de padding mémoire sur le réseau.\n`;
    p += `- **Unwind Safety & FFI ABI :** Double-Drop / Use-After-Free lors d'un déroulement de pile (\`panic!\` entre \`ptr::read\`/\`set_len\` et sa restauration), panique traversant une frontière \`extern "C"\` sans \`catch_unwind\`.\n`;
    p += `- **Allocations & Pages OS :** \`GlobalAlloc::alloc\` avec \`Layout\` de taille 0 ou overflow \`size * count > isize::MAX\`, références \`&mut T\` concurrentes depuis \`UnsafeCell::get()\` violant \`noalias\`, troncature externe de fichier \`Mmap\` provoquant un \`SIGBUS\`.\n`;
    p += `- **Divergence d'État & Consensus :** Désynchronisation entre cache mémoire et stockage persistant après erreur partielle, malléabilité cryptographique ou contournement de prépaiement de gaz.\n\n`;
  } else {
    p += `## 🛡️ 2. OBJECTIF : PATCH MINIMAL CIBLÉ (IN-SCOPE UNIQUEMENT)\n`;
    p += `Applique uniquement le correctif minimal nécessaire sur les fichiers **IN-SCOPE** listés ci-dessous, sans toucher aux fichiers **OUT-OF-SCOPE** ni casser les signatures publiques.\n\n`;
  }

  const singleTarget = currentBountyTargetCveIdx !== 'all' ? currentCveAnomaliesCache[currentBountyTargetCveIdx] : null;
  const targetList = singleTarget ? [singleTarget] : filtered;

  p += `## 🚨 3. CIBLE(S) CVE ANALYSÉE(S) (${singleTarget ? '1 CVE Ciblée pour PoC' : `${filtered.length} retenues In-Scope / ${totalCount} totales`})\n\n`;
  if (targetList.length === 0) {
    p += `*Aucune violation active ne correspond simultanément aux filtres In-Scope et Out-of-Scope actuels. Utilise les directives de la Section 2 pour mener une recherche manuelle de PoC Zero-Day sur les modules In-Scope.*\n\n`;
  } else {
    const maxIncluded = Math.min(targetList.length, 30);
    for (let i = 0; i < maxIncluded; i++) {
      p += buildSingleAnomalyPerfectBlock(targetList[i], singleTarget ? null : i + 1, includePatch) + '\n---\n\n';
    }
    if (targetList.length > maxIncluded) {
      p += `*... (+ ${targetList.length - maxIncluded} autres anomalies In-Scope supplémentaires disponibles dans le panneau CVE AetherScope)*\n\n`;
    }
  }

  p += `<aether_bounty_scope version="2.0" mode="${currentBountyMode}" in_scope_cve_count="${targetList.length}">\n`;
  p += `  <instruction>Respecte strictement les frontières In-Scope et Out-of-Scope ci-dessus. Produis un Proof of Concept (PoC) reproductible et le rapport Bug Bounty sans modifier les fichiers sources du projet.</instruction>\n`;
  p += `</aether_bounty_scope>\n`;

  const area = document.getElementById('bounty-prompt-textarea');
  if (area) area.value = p;

  const tokens = Math.round(p.length / 3.8);
  const tokEl = document.getElementById('bounty-token-count');
  if (tokEl) tokEl.innerText = `~${tokens} tokens`;
}
window.updateBountyPromptPreview = updateBountyPromptPreview;

function openBountyScopeModal(targetIdx = 'all') {
  const modal = document.getElementById('bounty-scope-modal');
  if (!modal) return;

  currentBountyTargetCveIdx = (targetIdx !== undefined && targetIdx !== null && targetIdx !== 'all') ? Number(targetIdx) : 'all';

  const inEl = document.getElementById('bounty-inscope-input');
  const outEl = document.getElementById('bounty-outscope-input');

  if (inEl && !inEl.value.trim() && outEl && !outEl.value.trim()) {
    autoExcludeTestAndToolFolders();
    appendBountyScopeLine('bounty-inscope-input', '[Impact Critique] Vol/Gel de Fonds, Corruption Mémoire (UB), RCE, Partition de Consensus');
    appendBountyScopeLine('bounty-outscope-input', '[Exclu] Rapports théoriques sans PoC reproductible, DoS volumétrique réseau, Tests & Outillage interne');
  } else {
    renderBountyFoldersPills();
    updateBountyPromptPreview();
  }

  modal.classList.add('open');
  AudioSys.playTone(640, 'sine', 0.12, 0.03);
}
window.openBountyScopeModal = openBountyScopeModal;

function openBountyScopeModalForAnomaly(idx) {
  openBountyScopeModal(idx);
  const a = currentCveAnomaliesCache[idx];
  if (a) {
    showToast(`🎯 Prompt PoC Bug Bounty préparé pour ${a.cve_id}`);
  }
}
window.openBountyScopeModalForAnomaly = openBountyScopeModalForAnomaly;

function closeBountyScopeModal() {
  document.getElementById('bounty-scope-modal')?.classList.remove('open');
}
window.closeBountyScopeModal = closeBountyScopeModal;

function copyBountyScopePrompt() {
  const txt = document.getElementById('bounty-prompt-textarea')?.value || '';
  if (!txt) return;
  navigator.clipboard.writeText(txt).then(() => {
    AudioSys.promptCopied();
    showToast('🎯 Prompt CVE & Scope Bug Bounty copié dans le presse-papier !');
  });
}
window.copyBountyScopePrompt = copyBountyScopePrompt;

function sendBountyPromptToCopilot() {
  const txt = document.getElementById('bounty-prompt-textarea')?.value || '';
  if (!txt) return;
  closeBountyScopeModal();
  const popup = document.getElementById('ai-chat-popup');
  if (popup && popup.classList.contains('hidden') && typeof toggleAiChatPopup === 'function') {
    toggleAiChatPopup();
  }
  const input = document.getElementById('ai-chat-user-input');
  if (input) {
    input.value = txt;
    if (typeof sendUserChatMessage === 'function') {
      sendUserChatMessage();
    }
  }
}
window.sendBountyPromptToCopilot = sendBountyPromptToCopilot;

async function syncBountyOutScopeToScanner() {
  const outRaw = document.getElementById('bounty-outscope-input')?.value || '';
  const parsed = parseScopePatterns(outRaw);
  if (parsed.paths.length === 0) {
    showToast('⚠️ Aucun motif de dossier dans Out-of-Scope à synchroniser');
    return;
  }
  if (!state.settings) state.settings = {};
  const merged = Array.from(new Set([...(state.settings.exclusions || []), ...parsed.paths]));
  state.settings.exclusions = merged;
  try {
    localStorage.setItem('aetherscope_settings', JSON.stringify(state.settings));
    await fetch('/api/settings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        exclusions: merged,
        active_cve_rules: state.settings.activeCveRules || []
      })
    });
    showToast(`🔄 ${parsed.paths.length} motifs Out-of-Scope synchronisés avec le Scanner AST ! Relancez Analyser.`);
  } catch (e) {
    showToast('⚠️ Exclusions enregistrées localement');
  }
}
window.syncBountyOutScopeToScanner = syncBountyOutScopeToScanner;

// Initialize affiliation & display toggles on load
updateAffiliationUI();
initDisplaySettingsUI();

window.toggleXrayMode = toggleXrayMode;
window.toggleDeadCodeMode = toggleDeadCodeMode;
window.toggleHeatmapMode = toggleHeatmapMode;
window.computeXrayTunnel = computeXrayTunnel;
window.updateXrayHud = updateXrayHud;
