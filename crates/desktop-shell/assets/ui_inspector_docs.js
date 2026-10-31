// =========================================================================
// AETHERSCOPE V2 — INSPECTOR & DOCUMENTATION ENGINE (ui_inspector_docs.js)
// Debriefs, UML, Mini-DAG, Markdown Reader, BLAKE3 Seal & 1-Click Handshake
// =========================================================================

// Inspector Tab Switching Controller
function switchInspectorTab(tabName) {
  document.querySelectorAll('.insp-tab').forEach(b => {
    b.classList.toggle('active', b.getAttribute('data-tab') === tabName);
  });
  const debriefView = document.getElementById('insp-debrief-view');
  const archView = document.getElementById('insp-arch-view');
  const relationsView = document.getElementById('insp-relations-view');
  const cveView = document.getElementById('insp-cve-view');
  if (debriefView) debriefView.classList.toggle('hidden', tabName !== 'debrief');
  if (archView) archView.classList.toggle('hidden', tabName !== 'arch');
  if (relationsView) relationsView.classList.toggle('hidden', tabName !== 'relations');
  if (cveView) cveView.classList.toggle('hidden', tabName !== 'cve');

  if (tabName === 'cve' && typeof renderCveAnomaliesPanel === 'function') {
    renderCveAnomaliesPanel();
  }
}
window.switchInspectorTab = switchInspectorTab;

function toggleInspector(forceOpen) {
  const inspector = document.getElementById('inspector');
  const dockTrigger = document.getElementById('btn-expand-inspector-dock');
  const topbarBtn = document.getElementById('btn-inspector');
  if (!inspector) return;

  const shouldOpen = forceOpen !== undefined ? !!forceOpen : inspector.classList.contains('hidden');
  if (shouldOpen) {
    inspector.classList.remove('hidden');
    if (dockTrigger) dockTrigger.classList.add('hidden');
    if (topbarBtn) topbarBtn.classList.add('active');
  } else {
    inspector.classList.add('hidden');
    if (dockTrigger) dockTrigger.classList.remove('hidden');
    if (topbarBtn) topbarBtn.classList.remove('active');
  }
}
window.toggleInspector = toggleInspector;

function openCveInspectorTab() {
  toggleInspector(true);
  switchInspectorTab('cve');
  if (typeof renderCveAnomaliesPanel === 'function') {
    renderCveAnomaliesPanel();
  }
  AudioSys.playTone(600, 'sine', 0.12, 0.03);
}
window.openCveInspectorTab = openCveInspectorTab;

document.querySelectorAll('.insp-tab').forEach(btn => {
  btn.addEventListener('click', () => {
    switchInspectorTab(btn.getAttribute('data-tab'));
  });
});

document.getElementById('btn-inspector')?.addEventListener('click', () => {
  toggleInspector();
});

document.getElementById('btn-collapse-inspector')?.addEventListener('click', () => {
  toggleInspector(false);
  AudioSys.playTone(400, 'sine', 0.1, 0.03);
});

document.getElementById('btn-expand-inspector-dock')?.addEventListener('click', () => {
  toggleInspector(true);
  AudioSys.playTone(620, 'sine', 0.12, 0.04);
});

// Dedicated Folder Selection & Architectural Débrief
window.selectFolder = function(folderName) {
  if (!folderName) {
    state.selectedFolder = null;
    return;
  }
  state.selectedFolder = folderName;
  state.selectedNode = null;
  wakePhysics(120);
  AudioSys.nodeSelect();

  const nodes = state.data.nodes || [];
  const edges = state.data.edges || [];
  const folderNodes = nodes.filter(n => (n.folder || 'root') === folderName);
  if (folderNodes.length === 0) return;

  // Pan camera smoothly to folder cluster center
  let sumX = 0, sumY = 0;
  folderNodes.forEach(n => { sumX += n.x; sumY += n.y; });
  const cx = sumX / folderNodes.length;
  const cy = sumY / folderNodes.length;
  state.camera.targetX = cx;
  state.camera.targetY = cy;
  if (state.camera.zoom < 0.42) {
    state.camera.targetZoom = 0.65;
  }

  // Aquatic ripple wave radiating from selected folder
  if (state.waterRipples) {
    const colRgba = getFolderColor(folderName);
    state.waterRipples.push({
      x: cx,
      y: cy,
      startTime: performance.now(),
      duration: 1200,
      baseRadius: 90,
      r: Math.round(colRgba[0] * 255),
      g: Math.round(colRgba[1] * 255),
      b: Math.round(colRgba[2] * 255)
    });
  }

  // Update Inspector Header
  const stEl = document.getElementById('insp-stereo');
  const nmEl = document.getElementById('insp-name');
  const ptEl = document.getElementById('insp-path');
  if (stEl) stEl.innerText = '«Dossier / Module»';
  if (nmEl) nmEl.innerText = '📁 ' + folderName;
  if (ptEl) ptEl.innerText = 'Module architectural (' + folderName + ')';

  renderFolderDebrief(folderName, folderNodes, edges);
  renderFolderUml(folderName, folderNodes, edges);
  renderFolderRelations(folderName, folderNodes, edges);

  const mdNodes = folderNodes.filter(n => n.layer === 'docs' || n.symbol_type === 'document' || (n.file_path && n.file_path.endsWith('.md')));
  if (mdNodes.length > 0) {
    mdNodes.sort((a, b) => {
      const aLower = a.name.toLowerCase();
      const bLower = b.name.toLowerCase();
      if (aLower.includes('readme')) return -1;
      if (bLower.includes('readme')) return 1;
      return aLower.localeCompare(bLower);
    });
    openFolderDocReader(folderName, mdNodes, 0, false);
  } else {
    openEmptyFolderDoc(folderName, folderNodes, false);
  }

  switchInspectorTab('debrief');
  toggleInspector(true);
  showToast(`📁 Débrief du dossier : ${folderName}`);
};

// Node Selection & Left Doc Reader / Right Inspector Activation
function selectNode(n) {
  state.selectedNode = n;
  state.selectedFolder = null;
  if (!n) return;

  if (state.xrayMode && typeof computeXrayTunnel === 'function') {
    computeXrayTunnel(n);
    if (typeof updateXrayHud === 'function') updateXrayHud();
  }

  AudioSys.nodeSelect();

  // Smooth camera fly-to
  state.camera.targetX = n.x;
  state.camera.targetY = n.y;

  // Zero-Shell AI bridge handshake registration
  if (window.AetherAPI && typeof AetherAPI.aiTarget === 'function') {
    AetherAPI.aiTarget(n.id, 'inspect').catch(() => {});
  }

  // Open associated README on left only if doc panel is already open or explicitly requested
  openNodeDocReader(n, 0, false);

  const stEl = document.getElementById('insp-stereo');
  const nmEl = document.getElementById('insp-name');
  const ptEl = document.getElementById('insp-path');
  if (stEl) stEl.innerText = n.stereotype || '«Symbol»';
  if (nmEl) nmEl.innerText = n.name;
  if (ptEl) ptEl.innerText = n.file_path || n.folder || '';

  const inc = n._incidentEdges || (state.data.edges || []).filter(e => e.target === n.id || e.source === n.id);
  const incoming = inc.filter(e => e.target === n.id);
  const outgoing = inc.filter(e => e.source === n.id);

  renderNodeDebrief(n, incoming, outgoing);
  renderUmlCard(n, incoming, outgoing);
  renderMiniDagSvg(n, incoming, outgoing);
  renderNodeRelations(n, incoming, outgoing);

  switchInspectorTab('debrief');
  toggleInspector(true);
}
window.selectNode = selectNode;
window.updateInspector = selectNode; // Global alias preventing crashes

// Cryptographic BLAKE3 Contract Signing Handler
window.signContractForCurrent = async function(folder) {
  const proj = state.data.project_path || '';
  showToast('🔒 Signature cryptographique BLAKE3 en cours...');
  try {
    const res = await AetherAPI.signContracts(proj, folder);
    if (res.status === 'ok') {
      const count = res.signed_contracts ? res.signed_contracts.length : 1;
      showToast(`✅ ${count} contrat(s) scellé(s) dans aether.lock.toml !`);
      const g = await AetherAPI.fetchGraph();
      if (g && g.nodes) {
        state.data = g;
        precomputeDocMetadata(g);
        wakePhysics(60);
        const refreshed = g.nodes.find(x => x.id === (state.selectedNode && state.selectedNode.id));
        if (refreshed) selectNode(refreshed);
      }
    } else {
      showToast(`⚠️ Erreur signature: ${res.message || 'échec'}`);
    }
  } catch (err) {
    showToast(`⚠️ Erreur réseau signature: ${err}`);
  }
};

// 1-Click Quick AI Handshake Buttons
document.querySelectorAll('.quick-ai-btn').forEach(btn => {
  btn.addEventListener('click', async () => {
    const n = state.selectedNode;
    if (!n) {
      showToast('⚠️ Sélectionnez d\'abord un symbole dans le graphe.');
      return;
    }
    const action = btn.getAttribute('data-action') || 'refactor';
    showToast(`🤖 Génération prompt IA (${action})...`);
    try {
      const res = await AetherAPI.aiTarget(n.id, action);
      if (res && res.status === 'ok' && res.prompt) {
        await navigator.clipboard.writeText(res.prompt);
        AudioSys.promptCopied();
        showToast('✨ Prompt IA v2.0 copié avec Handshake XML !');
      } else {
        throw new Error(res.message || 'Erreur backend');
      }
    } catch (err) {
      console.warn('Fallback prompt local:', err);
      const fallbackPrompt = `<aether_handshake version="2.0">\n  <target id="${n.id}" name="${n.name}" action="${action}" />\n</aether_handshake>`;
      await navigator.clipboard.writeText(fallbackPrompt);
      showToast('📋 Prompt copié dans le presse-papier !');
    }
  });
});

// Render Detailed Débrief for Folder
function renderFolderDebrief(folderName, folderNodes, edges) {
  const box = document.getElementById('debrief-content');
  if (!box) return;

  const files = folderNodes.filter(n => n.is_file);
  const classes = folderNodes.filter(n => n.is_class);
  const funcs = folderNodes.filter(n => n.symbol_type === 'function');
  const totalLoc = folderNodes.reduce((acc, n) => acc + ((n.metadata && n.metadata.lines) || 0), 0);

  const nodeMap = new Map();
  (state.data.nodes || []).forEach(n => nodeMap.set(n.id, n));

  const inCallers = new Map();
  const outImports = new Map();

  edges.forEach(e => {
    const s = nodeMap.get(e.source);
    const t = nodeMap.get(e.target);
    if (!s || !t) return;
    const sFolder = s.folder || 'root';
    const tFolder = t.folder || 'root';

    if (sFolder !== folderName && tFolder === folderName) {
      inCallers.set(sFolder, (inCallers.get(sFolder) || 0) + 1);
    }
    if (sFolder === folderName && tFolder !== folderName) {
      outImports.set(tFolder, (outImports.get(tFolder) || 0) + 1);
    }
  });

  let roleDesc = "Module applicatif";
  let roleCol = "#38bdf8";
  const fLower = folderName.toLowerCase();
  if (fLower.includes('ui') || fLower.includes('component') || fLower.includes('view') || fLower.includes('page') || fLower.includes('client')) {
    roleDesc = "Couche Présentation & UI"; roleCol = "#ec4899";
  } else if (fLower.includes('contract') || fLower.includes('doc')) {
    roleDesc = "Contrats & Spécifications EARS"; roleCol = "#10b981";
  } else if (fLower.includes('model') || fLower.includes('domain') || fLower.includes('entity') || fLower.includes('data-nodes')) {
    roleDesc = "Couche Métier & Données"; roleCol = "#fbbf24";
  } else if (fLower.includes('controller') || fLower.includes('service') || fLower.includes('server')) {
    roleDesc = "Couche Services & Backend"; roleCol = "#38bdf8";
  } else if (fLower.includes('db') || fLower.includes('repo')) {
    roleDesc = "Couche Persistance & Stockage"; roleCol = "#818cf8";
  } else if (fLower.includes('config') || fLower.includes('util') || fLower.includes('lib')) {
    roleDesc = "Socle Technique & Utilitaires"; roleCol = "#c084fc";
  }

  let html = `
    <div class="debrief-badge-row">
      <span class="debrief-badge" style="background:${roleCol}22; border:1px solid ${roleCol}; color:${roleCol};">🪐 ${roleDesc}</span>
      <span class="debrief-badge" style="background:rgba(56,189,248,0.12); border:1px solid rgba(56,189,248,0.3); color:#38bdf8;">📁 ${folderName}</span>
    </div>

    <div class="debrief-metrics-grid">
      <div class="debrief-kpi">
        <span class="debrief-kpi-val">${files.length}</span>
        <span class="debrief-kpi-lbl">📄 Fichiers</span>
      </div>
      <div class="debrief-kpi">
        <span class="debrief-kpi-val">${folderNodes.length}</span>
        <span class="debrief-kpi-lbl">⚡ Symboles</span>
      </div>
      <div class="debrief-kpi">
        <span class="debrief-kpi-val">${totalLoc > 0 ? totalLoc : '~' + files.length * 85}</span>
        <span class="debrief-kpi-lbl">📏 Lignes de Code</span>
      </div>
      <div class="debrief-kpi">
        <span class="debrief-kpi-val">${classes.length} cls • ${funcs.length} fns</span>
        <span class="debrief-kpi-lbl">🏛️ Typologie</span>
      </div>
    </div>

    <div style="margin: 6px 0;">
      ${(() => {
        const mdFiles = folderNodes.filter(n => n.layer === 'docs' || n.symbol_type === 'document' || (n.file_path && n.file_path.endsWith('.md')));
        if (mdFiles.length > 0) {
          const docNode = mdFiles[0];
          const isVerified = docNode.metadata?.lock_verified;
          const hashShort = docNode.metadata?.blake3_hash ? docNode.metadata.blake3_hash.substring(0, 10) + '…' : '';
          return `
            <div style="background:rgba(16,185,129,0.12); border:1px solid rgba(16,185,129,0.35); border-radius:6px; padding:8px 10px; display:flex; flex-direction:column; gap:6px; min-width:0; box-sizing:border-box;">
              <div style="display:flex; align-items:center; justify-content:space-between; gap:8px; min-width:0;">
                <span style="color:#6ee7b7; font-weight:700; font-size:0.72rem; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; min-width:0; flex:1;" title="${escapeHtml(docNode.name)}">📖 Spécification active : ${escapeHtml(docNode.name)}</span>
                <button class="aether-btn primary" onclick="openFolderDocReader('${folderName}', state.folderMdMap.get('${folderName}') || [], 0, true)">Voir à Gauche</button>
              </div>
              <div style="display:flex; align-items:center; justify-content:space-between; font-size:0.65rem; gap:8px; min-width:0;">
                ${isVerified 
                  ? `<span class="contract-seal-badge verified" style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap; min-width:0;">🔒 BLAKE3 Sceau Vérifié [${hashShort}]</span>`
                  : `<span class="contract-seal-badge warning" style="white-space:nowrap;">⚠️ BLAKE3 Non Synchronisé</span>`
                }
                <button class="aether-btn primary" onclick="signContractForCurrent('${folderName}')">🔒 Signer Contrat</button>
              </div>
            </div>
          `;
        } else {
          return `
            <div style="background:rgba(245,158,11,0.12); border:1px solid rgba(245,158,11,0.35); border-radius:6px; padding:8px 10px; display:flex; align-items:center; justify-content:space-between;">
              <span style="color:#fbbf24; font-weight:700; font-size:0.72rem;">⚠️ Aucune spécification .md</span>
              <button class="aether-btn export" style="font-size:0.65rem; padding:3px 8px;" onclick="openEmptyFolderDoc('${folderName}', state.data.nodes.filter(n => (n.folder||'root')==='${folderName}'), true)">Rédiger à Gauche</button>
            </div>
          `;
        }
      })()}
    </div>

    <div>
      <div class="debrief-sec-title">
        <span>📦 Fichiers du dossier (${files.length})</span>
        <span style="font-size:0.6rem; color:#94a3b8;">Cliquer pour zoomer</span>
      </div>
      <div class="debrief-list">
  `;

  if (files.length > 0) {
    files.forEach(f => {
      const loc = (f.metadata && f.metadata.lines) || '';
      html += `
        <div class="debrief-list-item" onclick="jumpToNode('${f.id}')">
          <span style="color:#f8fafc; font-weight:600;">${(f.name && f.name.endsWith('.java')) ? '☕' : '📄'} ${f.name}</span>
          <span class="debrief-tag">${loc ? loc + ' LOC' : f.layer}</span>
        </div>
      `;
    });
  } else {
    html += `<div style="font-size:0.7rem; color:#64748b; font-style:italic;">Aucun nœud de fichier direct.</div>`;
  }

  html += `
      </div>
    </div>

    <div>
      <div class="debrief-sec-title">
        <span>🌐 Dépendances Inter-Dossiers</span>
      </div>
      <div style="display:flex; flex-direction:column; gap:6px; font-size:0.72rem;">
  `;

  if (inCallers.size > 0) {
    html += `<div style="color:#38bdf8; font-weight:700; font-size:0.65rem;">📥 Appelants externes :</div>`;
    inCallers.forEach((cnt, srcF) => {
      html += `
        <div class="debrief-list-item" onclick="selectFolder('${srcF}')">
          <span style="color:#e2e8f0;">📁 ${srcF}</span>
          <span class="debrief-tag" style="color:#38bdf8;">${cnt} appels</span>
        </div>
      `;
    });
  } else {
    html += `<div style="color:#64748b; font-style:italic;">📥 Aucun dossier externe n'appelle ce dossier.</div>`;
  }

  if (outImports.size > 0) {
    html += `<div style="color:#c084fc; font-weight:700; font-size:0.65rem; margin-top:4px;">📤 Imports vers l'extérieur :</div>`;
    outImports.forEach((cnt, tgtF) => {
      html += `
        <div class="debrief-list-item" onclick="selectFolder('${tgtF}')">
          <span style="color:#e2e8f0;">📁 ${tgtF}</span>
          <span class="debrief-tag" style="color:#c084fc;">${cnt} imports</span>
        </div>
      `;
    });
  } else {
    html += `<div style="color:#64748b; font-style:italic;">📤 Ce dossier n'importe aucun autre dossier.</div>`;
  }

  html += `
      </div>
    </div>

    <div style="margin-top:6px;">
      <button class="aether-btn ai" style="width:100%; justify-content:center; padding:8px;" onclick="openAiModal()">
        🤖 Générer un Débrief IA de ce Dossier
      </button>
    </div>
  `;

  box.innerHTML = html;
}

// Render Detailed Débrief for Node
function renderNodeDebrief(n, incoming, outgoing) {
  const box = document.getElementById('debrief-content');
  if (!box) return;

  const meta = n.metadata || {};
  const isFile = n.is_file || n.symbol_type === 'file';
  const isClass = n.is_class;
  const isDoc = n.layer === 'docs' || n.symbol_type === 'document' || (n.file_path && n.file_path.endsWith('.md'));
  const folder = n.folder || 'root';

  const isDeadCode = state.deadCodeSet && state.deadCodeSet.has(n.id);
  const hmInfo = state.heatmapScores && state.heatmapScores.get(n.id);
  let extraBadges = '';
  if (isDeadCode) {
    extraBadges += `<span class="debrief-badge" style="background:#ef444422; border:1px solid #ef4444; color:#f87171;">💀 Orphelin</span>`;
  }
  if (hmInfo) {
    const hmCol = hmInfo.score > 0.65 ? '#ef4444' : (hmInfo.score > 0.35 ? '#fbbf24' : '#10b981');
    extraBadges += `<span class="debrief-badge" style="background:${hmCol}22; border:1px solid ${hmCol}; color:${hmCol};">🌡️ Complexité: ${(hmInfo.score * 100).toFixed(0)}%</span>`;
  }
  if (n.has_security_violation) {
    extraBadges += `<span class="debrief-badge" style="background:rgba(239,68,68,0.22); border:1px solid #ef4444; color:#f87171; animation:pulseViolate 0.8s infinite alternate;">🚨 Violation SAST / Contrat</span>`;
  }

  // Cryptographic BLAKE3 badge
  let cryptoBadge = '';
  if (meta.blake3_hash) {
    const hashShort = meta.blake3_hash.substring(0, 12);
    if (meta.lock_verified) {
      cryptoBadge = `<span class="contract-seal-badge verified" title="BLAKE3: ${meta.blake3_hash}">🔒 BLAKE3 Sceau Vérifié [${hashShort}…]</span>`;
    } else {
      cryptoBadge = `
        <span class="contract-seal-badge warning" title="BLAKE3: ${meta.blake3_hash}">⚠️ BLAKE3 Non Scellé</span>
        <button class="aether-btn primary" onclick="signContractForCurrent('${folder}')">🔒 Signer Contrat</button>
      `;
    }
  }

  // Violations list block
  let violationsHtml = '';
  const anomaliesList = meta.cve_anomalies || [];
  if ((meta.violations && meta.violations.length > 0) || anomaliesList.length > 0) {
    const totalV = Math.max(meta.violations ? meta.violations.length : 0, anomaliesList.length);
    violationsHtml = `
      <div style="background:rgba(239,68,68,0.12); border:1px solid rgba(239,68,68,0.45); border-radius:6px; padding:8px 10px; margin:6px 0;">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:6px;">
          <div style="color:#f87171; font-weight:800; font-size:0.68rem;">🚨 ANOMALIES SAST & CVE DÉTECTÉES (${totalV}) :</div>
          <span style="font-size:0.58rem; background:rgba(239,68,68,0.25); color:#fca5a5; padding:1px 5px; border-radius:3px; font-weight:800;">CRITICAL RISK</span>
        </div>
        <div style="display:flex; flex-direction:column; gap:6px;">
          ${anomaliesList.length > 0 ? anomaliesList.map(a => {
            const sClass = (a.severity || 'warning').toLowerCase();
            let defiLaserBadge = '';
            let defiPatchBtn = '';
            if (a.rule_id === 'DEFI-ASYMMETRIC-ROUNDING' || (a.cve_id && a.cve_id.includes('ASYMMETRIC_ROUNDING_RISK'))) {
              defiLaserBadge = `<div class="defi-laser-badge" style="margin:4px 0;">☣️ [CWE-682] ASYMMETRIC_ROUNDING_RISK</div>`;
              defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="if(window.copyDefiPatchPrompt)window.copyDefiPatchPrompt('${escapeForBacktick(n.file_path)}', ${a.line_number}, 'DEFI-ASYMMETRIC-ROUNDING')">🛡️ Patch Arrondi Asymétrique</button>`;
            } else if (a.rule_id === 'DEFI-STALE-CACHE-AFTER-CALL' || (a.cve_id && a.cve_id.includes('STALE_CACHE_TOCTOU'))) {
              defiLaserBadge = `<div class="defi-laser-badge" style="margin:4px 0;">☣️ [CWE-362] STALE_CACHE_TOCTOU</div>`;
              defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="if(window.copyDefiPatchPrompt)window.copyDefiPatchPrompt('${escapeForBacktick(n.file_path)}', ${a.line_number}, 'DEFI-STALE-CACHE-AFTER-CALL')">🛡️ Invalidation Cache Post-Call</button>`;
            } else if (a.rule_id === 'EVC-HOOK-BYPASS' || (a.cve_id && a.cve_id.includes('EVC_HOOK_BYPASS_RISK'))) {
              defiLaserBadge = `<div class="defi-laser-badge" style="margin:4px 0;">☣️ [CWE-841] EVC_HOOK_BYPASS_RISK</div>`;
              defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="if(window.copyDefiPatchPrompt)window.copyDefiPatchPrompt('${escapeForBacktick(n.file_path)}', ${a.line_number}, 'EVC-HOOK-BYPASS')">🛡️ Invariant EVC nonReentrant</button>`;
            } else if (a.rule_id === 'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER' || (a.cve_id && a.cve_id.includes('ORACLE_MIN_MAX_CIRCUIT_BREAKER'))) {
              defiLaserBadge = `<div class="defi-laser-badge" style="margin:4px 0;">☣️ [LOGIC-CRITICAL] ORACLE_MIN_MAX_CIRCUIT_BREAKER</div>`;
              defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="if(window.copyDefiPatchPrompt)window.copyDefiPatchPrompt('${escapeForBacktick(n.file_path)}', ${a.line_number}, 'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER')">🛡️ Patch Circuit Breaker Min/Max</button>`;
            } else if (a.rule_id === 'DEFI-ORACLE-L2-SEQUENCER-RACE' || (a.cve_id && a.cve_id.includes('L2_SEQUENCER_ORACLE_RACE'))) {
              defiLaserBadge = `<div class="defi-laser-badge" style="margin:4px 0;">☣️ [LOGIC-CRITICAL] L2_SEQUENCER_ORACLE_RACE</div>`;
              defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="if(window.copyDefiPatchPrompt)window.copyDefiPatchPrompt('${escapeForBacktick(n.file_path)}', ${a.line_number}, 'DEFI-ORACLE-L2-SEQUENCER-RACE')">🛡️ Patch Sequencer Grace Period</button>`;
            } else if (a.rule_id === 'DEFI-TWAP-SHORT-WINDOW' || (a.cve_id && a.cve_id.includes('TWAP_SHORT_WINDOW'))) {
              defiLaserBadge = `<div class="defi-laser-badge" style="margin:4px 0;">☣️ [LOGIC-CRITICAL] TWAP_SHORT_WINDOW</div>`;
              defiPatchBtn = `<button class="cve-btn-patch defi-patch" onclick="if(window.copyDefiPatchPrompt)window.copyDefiPatchPrompt('${escapeForBacktick(n.file_path)}', ${a.line_number}, 'DEFI-TWAP-SHORT-WINDOW')">🛡️ Patch TWAP Safe Window (>= 30m)</button>`;
            }

            return `
            <div class="cve-card ${sClass}" style="margin-bottom:6px;">
              <div class="cve-card-header">
                <div class="cve-badge-group">
                  <span class="cve-badge ${sClass}">${escapeHtml(a.cve_id || a.rule_id)}</span>
                  <span class="cve-sev-tag ${sClass}">${escapeHtml(a.severity)}</span>
                </div>
                <span class="cve-line-tag">L${a.line_number}</span>
              </div>
              ${defiLaserBadge}
              <div class="cve-msg-box">${escapeHtml(a.message)}</div>
              ${a.transmitted_data ? `
                <div class="cve-transmitted-box">
                  <span><strong>📦 Donnée :</strong> <code>${escapeHtml(a.transmitted_data)}</code></span>
                  <span><strong>🏷️ Type :</strong> <code>${escapeHtml(a.transmitted_type || 'Untyped')}</code></span>
                </div>
              ` : ''}
              <div class="cve-card-actions">
                <button class="cve-action-btn poc" onclick="copySingleAnomalyPrompt('${escapeForBacktick(n.file_path)}', ${a.line_number}, '${escapeForBacktick(a.rule_id)}', '${escapeForBacktick(a.cve_id)}', '${escapeForBacktick(a.severity)}', '${escapeForBacktick(a.message)}', '${escapeForBacktick(a.transmitted_data || '')}', '${escapeForBacktick(a.transmitted_type || '')}', '${escapeForBacktick(a.remediation)}')">
                  🧪 PoC Bug Bounty
                </button>
                ${defiPatchBtn}
              </div>
            </div>
            `;
          }).join('') : meta.violations.map(v => `<div style="color:#fca5a5; font-size:0.65rem; font-family:var(--font-mono);">• ${escapeHtml(v)}</div>`).join('')}
        </div>
      </div>
    `;
  }

  let html = '';

  if (isDoc) {
    const raw = meta.raw_content || n.docstring || '';
    const wordCount = raw.trim() ? raw.trim().split(/\s+/).length : 0;
    const headers = (raw.match(/^#{1,3}\s+(.+)$/gm) || []).map(h => h.trim());

    html = `
      <div class="debrief-badge-row">
        <span class="debrief-badge" style="background:#10b98122; border:1px solid #10b981; color:#10b981;">📖 Documentation & Spécifications</span>
        <span class="debrief-badge" style="background:rgba(56,189,248,0.12); border:1px solid rgba(56,189,248,0.3); color:#38bdf8; cursor:pointer;" onclick="selectFolder('${folder}')">📁 ${folder}</span>
        ${extraBadges}
      </div>

      ${cryptoBadge ? `<div style="display:flex; align-items:center; gap:8px; margin:4px 0;">${cryptoBadge}</div>` : ''}
      ${violationsHtml}

      <div class="debrief-metrics-grid">
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${wordCount}</span>
          <span class="debrief-kpi-lbl">📝 Mots Rédigés</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${headers.length}</span>
          <span class="debrief-kpi-lbl">📑 Sections (H1-H3)</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${incoming.length}</span>
          <span class="debrief-kpi-lbl">🔗 Documents Liés</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${meta.lines || raw.split('\n').length}</span>
          <span class="debrief-kpi-lbl">📏 Lignes Markdown</span>
        </div>
      </div>

      <div>
        <div class="debrief-sec-title"><span>📑 Sommaire Détecté (${headers.length})</span></div>
        <div class="debrief-list">
          ${headers.length > 0 ? headers.map(h => `<div class="debrief-list-item"><span style="color:#e2e8f0;">${h}</span></div>`).join('') : '<div style="color:#64748b;font-style:italic;">Aucun titre H1-H3 détecté.</div>'}
        </div>
      </div>

      <div style="display:flex; gap:6px; margin-top:8px;">
        <button class="aether-btn primary" style="flex:1; justify-content:center; padding:7px;" onclick="openDocReader(state.selectedNode)">
          📖 Ouvrir le Lecteur Markdown
        </button>
        <button class="aether-btn" style="flex:1; justify-content:center; padding:7px;" onclick="document.getElementById('btn-copy-doc')?.click()">
          📋 Copier Texte
        </button>
      </div>
    `;
  } else if (isFile) {
    const nodesInFile = (state.data.nodes || []).filter(sub => sub.file_path === n.file_path && sub.id !== n.id);
    const loc = meta.lines || 1;
    const cyclomatic = meta.complexity || n.complexity || 1;

    html = `
      <div class="debrief-badge-row">
        <span class="debrief-badge" style="background:#38bdf822; border:1px solid #38bdf8; color:#38bdf8;">${(n.name && n.name.endsWith('.java')) ? '☕ Fichier Java' : '📄 Fichier Source'}</span>
        <span class="debrief-badge" style="background:rgba(245,158,11,0.12); border:1px solid rgba(245,158,11,0.3); color:#fbbf24; cursor:pointer;" onclick="selectFolder('${folder}')">📁 ${folder}</span>
        ${extraBadges}
      </div>

      ${cryptoBadge ? `<div style="display:flex; align-items:center; gap:8px; margin:4px 0;">${cryptoBadge}</div>` : ''}
      ${violationsHtml}

      <div class="debrief-metrics-grid">
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${loc}</span>
          <span class="debrief-kpi-lbl">📏 Lignes (LOC)</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${cyclomatic}</span>
          <span class="debrief-kpi-lbl">🔀 Complexité (McCabe)</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${nodesInFile.length}</span>
          <span class="debrief-kpi-lbl">⚡ Symboles</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${incoming.length + outgoing.length}</span>
          <span class="debrief-kpi-lbl">🔗 Flux Liés</span>
        </div>
      </div>

      <div>
        <div class="debrief-sec-title"><span>🧩 Symboles Contenus (${nodesInFile.length})</span></div>
        <div class="debrief-list">
          ${nodesInFile.length > 0 ? nodesInFile.map(s => `
            <div class="debrief-list-item" onclick="jumpToNode('${s.id}')">
              <span style="color:#f8fafc; font-weight:600;">${s.is_class ? '🏛️' : '⚡'} ${s.name}</span>
              <span class="debrief-tag">${s.stereotype}</span>
            </div>
          `).join('') : '<div style="color:#64748b;font-style:italic;">Aucun sous-symbole direct répertorié.</div>'}
        </div>
      </div>

      <div style="margin-top:6px;">
        <button class="aether-btn ai" style="width:100%; justify-content:center; padding:8px;" onclick="openAiModal()">
          🤖 Audit IA de ce Fichier
        </button>
      </div>
    `;
  } else {
    // Class or Function
    const loc = meta.lines || 1;
    const cyclomatic = meta.complexity || n.complexity || 1;

    html = `
      <div class="debrief-badge-row">
        <span class="debrief-badge" style="background:#c084fc22; border:1px solid #c084fc; color:#c084fc;">${isClass ? '🏛️ Classe POO' : '⚡ Fonction / Méthode'}</span>
        <span class="debrief-badge" style="background:rgba(56,189,248,0.12); border:1px solid rgba(56,189,248,0.3); color:#38bdf8; cursor:pointer;" onclick="selectFolder('${folder}')">📁 ${folder}</span>
        ${extraBadges}
      </div>

      ${cryptoBadge ? `<div style="display:flex; align-items:center; gap:8px; margin:4px 0;">${cryptoBadge}</div>` : ''}
      ${violationsHtml}

      <div class="debrief-metrics-grid">
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${loc}</span>
          <span class="debrief-kpi-lbl">📏 Lignes (LOC)</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${cyclomatic}</span>
          <span class="debrief-kpi-lbl">🔀 Branches Cyclo</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${incoming.length}</span>
          <span class="debrief-kpi-lbl">📥 Appelants</span>
        </div>
        <div class="debrief-kpi">
          <span class="debrief-kpi-val">${outgoing.length}</span>
          <span class="debrief-kpi-lbl">📤 Sous-Flux</span>
        </div>
      </div>

      ${(() => {
        let sig = n.signature || meta.signature || '';
        if (!sig && (n.symbol_type === 'function' || n.is_function || n.stereotype === '«Function»' || n.stereotype === '«Method»')) {
          const params = n.parameters || meta.parameters || [];
          const retType = n.return_type || meta.return_type || '';
          if (params.length > 0) {
            sig = `function ${n.name}(${params.join(', ')})${retType ? ': ' + retType : ''}`;
          } else {
            sig = `function ${n.name}()${retType ? ': ' + retType : ''}`;
          }
        } else if (!sig && (isClass || n.stereotype === '«Class»' || n.stereotype === '«Model»')) {
          sig = `class ${n.name}`;
        }

        const doc = n.docstring || meta.docstring || '';
        if (!sig && !doc) return '';

        return `
          <div style="background: rgba(15, 23, 42, 0.75); border: 1px solid rgba(56, 189, 248, 0.35); border-radius: 8px; padding: 10px 12px; margin: 8px 0; box-shadow: 0 4px 14px rgba(0,0,0,0.3);">
            <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom: 6px;">
              <span style="font-size: 0.68rem; font-weight: 700; color: #38bdf8; text-transform: uppercase; letter-spacing: 0.05em;">⚡ Déclaration & Contrat PHPDoc</span>
              <span style="font-size: 0.62rem; color: #94a3b8; font-family: var(--font-mono);">${meta.lines ? meta.lines + ' LOC' : ''}</span>
            </div>
            ${sig ? `
              <pre style="margin: 0 0 6px 0; padding: 6px 8px; background: rgba(0,0,0,0.5); border-radius: 5px; font-family: var(--font-mono); font-size: 0.72rem; color: #38bdf8; overflow-x: auto; white-space: pre-wrap; word-break: break-all;"><code>${escapeHtml(sig)}</code></pre>
            ` : ''}
            ${doc && !doc.startsWith('Module source ') ? `
              <div style="font-size: 0.70rem; color: #cbd5e1; line-height: 1.45; background: rgba(56,189,248,0.04); border-left: 2px solid #38bdf8; padding: 6px 8px; border-radius: 0 4px 4px 0; font-style: italic; white-space: pre-wrap;">
                ${escapeHtml(doc)}
              </div>
            ` : ''}
          </div>
        `;
      })()}

      <div>
        <div class="debrief-sec-title"><span>📥 Fonctions Appelantes (${incoming.length})</span></div>
        <div class="debrief-list">
          ${incoming.length > 0 ? incoming.map(e => `
            <div class="debrief-list-item" onclick="jumpToNode('${e.source}')">
              <span style="color:#e2e8f0;">${e.source.replace('file::', '')}</span>
              <span class="debrief-tag">${e.relation}</span>
            </div>
          `).join('') : '<div style="color:#64748b;font-style:italic;">Point d\'entrée (aucun appelant interne).</div>'}
        </div>
      </div>

      <div>
        <div class="debrief-sec-title"><span>📤 Sous-Appels Sortants (${outgoing.length})</span></div>
        <div class="debrief-list">
          ${outgoing.length > 0 ? outgoing.map(e => `
            <div class="debrief-list-item" onclick="jumpToNode('${e.target}')">
              <span style="color:#e2e8f0;">${e.target.replace('file::', '')}</span>
              <span class="debrief-tag">${e.relation}</span>
            </div>
          `).join('') : '<div style="color:#64748b;font-style:italic;">Fonction feuille (aucun sous-appel).</div>'}
        </div>
      </div>

      <div style="margin-top:6px;">
        <button class="aether-btn ai" style="width:100%; justify-content:center; padding:8px;" onclick="openAiModal()">
          🤖 Générer le Prompt IA de ce Symbole
        </button>
      </div>
    `;
  }

  box.innerHTML = html;
}

// Render UML Architecture Card
function renderUmlCard(n, incoming, outgoing) {
  const box = document.getElementById('insp-uml-card');
  if (!box) return;
  const meta = n.metadata || {};
  const attrs = meta.attributes || [];
  const methods = meta.methods || [];

  let html = `
    <div class="uml-header">
      <div class="uml-stereo-label">${n.stereotype || '«Symbol»'}</div>
      <div class="uml-name-label">${n.name}</div>
    </div>
  `;

  html += `<div class="uml-section">`;
  if (attrs.length > 0) {
    attrs.forEach(a => {
      html += `<div class="uml-item"><span class="uml-vis-plus">+</span> <span>${a.name}</span> <span class="uml-tag">${a.type || 'prop'}</span></div>`;
    });
  } else {
    html += `<div class="uml-item" style="color:#64748b;font-style:italic;">+ lines: ${meta.lines || 1} [LOC]</div>`;
    if (n.symbol_type === 'file') {
      html += `<div class="uml-item" style="color:#64748b;font-style:italic;">+ folder: ${n.folder || 'root'}</div>`;
    }
  }
  html += `</div>`;

  html += `<div class="uml-section">`;
  if (methods.length > 0) {
    methods.forEach(m => {
      html += `<div class="uml-item"><span class="uml-vis-plus">+</span> <span>${m.name}</span> <span class="uml-tag">${m.returns || 'void'}</span></div>`;
    });
  } else {
    html += `<div class="uml-item" style="color:#64748b;font-style:italic;">+ in: ${n.in_degree || 0} • out: ${n.out_degree || 0}</div>`;
    if (outgoing.length > 0) {
      html += `<div class="uml-item" style="color:#38bdf8;">+ flux sortants: ${outgoing.length}</div>`;
    }
  }
  html += `</div>`;

  box.innerHTML = html;
}

// Render Folder UML
function renderFolderUml(folderName, folderNodes, edges) {
  const box = document.getElementById('insp-uml-card');
  if (!box) return;
  const files = folderNodes.filter(n => n.is_file);
  const classes = folderNodes.filter(n => n.is_class);

  let html = `
    <div class="uml-header">
      <div class="uml-stereo-label">«Module / Folder»</div>
      <div class="uml-name-label">📁 ${folderName}</div>
    </div>
    <div class="uml-section">
      <div class="uml-item"><span class="uml-vis-plus">+</span> <span>fichiers: ${files.length}</span> <span class="uml-tag">count</span></div>
      <div class="uml-item"><span class="uml-vis-plus">+</span> <span>classes: ${classes.length}</span> <span class="uml-tag">types</span></div>
    </div>
    <div class="uml-section">
  `;
  files.slice(0, 6).forEach(f => {
    html += `<div class="uml-item"><span class="uml-vis-plus">+</span> <span>${f.name}</span> <span class="uml-tag">${f.layer}</span></div>`;
  });
  if (files.length > 6) {
    html += `<div class="uml-item" style="color:#64748b;font-style:italic;">+ ... et ${files.length - 6} autres fichiers</div>`;
  }
  html += `</div>`;
  box.innerHTML = html;

  const dagBox = document.getElementById('insp-svg-dag');
  if (dagBox) {
    dagBox.innerHTML = `
      <div style="text-align:center; padding:16px 0; color:#38bdf8; font-family:var(--font-mono); font-size:0.75rem;">
        🪐 Cluster de ${folderNodes.length} entités actives dans <strong>${folderName}</strong>
      </div>
    `;
  }
}

// Render Mini-DAG SVG
function renderMiniDagSvg(n, incoming, outgoing) {
  const box = document.getElementById('insp-svg-dag');
  if (!box) return;
  const inNodes = incoming.slice(0, 3).map(e => e.source.split('::').pop().replace('file::', ''));
  const outNodes = outgoing.slice(0, 3).map(e => e.target.split('::').pop().replace('file::', ''));

  let svg = `<svg viewBox="0 0 380 90" style="width:100%;height:90px;font-family:var(--font-mono);font-size:10px;">
    <defs>
      <marker id="arr" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
        <path d="M 0 1 L 9 5 L 0 9 z" fill="#38bdf8"/>
      </marker>
    </defs>
  `;

  svg += `
    <rect x="135" y="24" width="110" height="42" rx="6" fill="#0f172a" stroke="#38bdf8" stroke-width="1.8"/>
    <text x="190" y="42" fill="#fff" font-weight="bold" text-anchor="middle">${n.name.length > 12 ? n.name.substring(0, 11) + '…' : n.name}</text>
    <text x="190" y="56" fill="#38bdf8" font-size="8px" text-anchor="middle">${n.stereotype || 'AST'}</text>
  `;

  if (inNodes.length > 0) {
    const topY = inNodes.length === 1 ? 35 : (inNodes.length === 2 ? 20 : 10);
    inNodes.forEach((src, idx) => {
      const y = topY + idx * 28;
      svg += `
        <rect x="10" y="${y}" width="85" height="22" rx="4" fill="#030712" stroke="#c084fc" stroke-width="1"/>
        <text x="52" y="${y + 14}" fill="#e2e8f0" text-anchor="middle">${src.length > 10 ? src.substring(0, 9) + '…' : src}</text>
        <path d="M 95 ${y + 11} L 132 40" fill="none" stroke="#c084fc" stroke-width="1.2" stroke-dasharray="3,3" marker-end="url(#arr)"/>
      `;
    });
  } else {
    svg += `<text x="50" y="48" fill="#64748b" font-style="italic" text-anchor="middle">[Entrée]</text>`;
  }

  if (outNodes.length > 0) {
    const topY = outNodes.length === 1 ? 35 : (outNodes.length === 2 ? 20 : 10);
    outNodes.forEach((tgt, idx) => {
      const y = topY + idx * 28;
      svg += `
        <rect x="285" y="${y}" width="85" height="22" rx="4" fill="#030712" stroke="#38bdf8" stroke-width="1"/>
        <text x="327" y="${y + 14}" fill="#e2e8f0" text-anchor="middle">${tgt.length > 10 ? tgt.substring(0, 9) + '…' : tgt}</text>
        <path d="M 245 45 L 282 ${y + 11}" fill="none" stroke="#38bdf8" stroke-width="1.2" marker-end="url(#arr)"/>
      `;
    });
  } else {
    svg += `<text x="330" y="48" fill="#64748b" font-style="italic" text-anchor="middle">[Feuille]</text>`;
  }

  svg += `</svg>`;
  box.innerHTML = svg;
}

// Relations renderers
function renderNodeRelations(n, incoming, outgoing) {
  const relContainer = document.getElementById('insp-relations');
  if (!relContainer) return;
  let html = '';

  if (incoming.length > 0) {
    html += `<div style="font-size:0.65rem;font-weight:700;color:#38bdf8;margin:8px 0 4px;">📥 FLUX ENTRANTS (${incoming.length}) :</div>`;
    incoming.forEach(e => {
      html += `<div class="rel-card" onclick="jumpToNode('${e.source}')" style="cursor:pointer;"><span style="color:#f8fafc;">${e.source.replace('file::', '')}</span><span class="rel-tag">${e.relation}</span></div>`;
    });
  }

  if (outgoing.length > 0) {
    html += `<div style="font-size:0.65rem;font-weight:700;color:#c084fc;margin:8px 0 4px;">📤 FLUX SORTANTS (${outgoing.length}) :</div>`;
    outgoing.forEach(e => {
      html += `<div class="rel-card" onclick="jumpToNode('${e.target}')" style="cursor:pointer;"><span style="color:#f8fafc;">${e.target.replace('file::', '')}</span><span class="rel-tag">${e.relation}</span></div>`;
    });
  }

  if (incoming.length === 0 && outgoing.length === 0) {
    html += `<div style="font-size:0.72rem;color:#64748b;font-style:italic;">Aucune relation directe détectée.</div>`;
  }

  relContainer.innerHTML = html;
}

function renderFolderRelations(folderName, folderNodes, edges) {
  const relContainer = document.getElementById('insp-relations');
  if (!relContainer) return;
  const nodeMap = new Map();
  (state.data.nodes || []).forEach(n => nodeMap.set(n.id, n));

  const inEdges = edges.filter(e => {
    const s = nodeMap.get(e.source);
    const t = nodeMap.get(e.target);
    return s && t && (s.folder || 'root') !== folderName && (t.folder || 'root') === folderName;
  });

  const outEdges = edges.filter(e => {
    const s = nodeMap.get(e.source);
    const t = nodeMap.get(e.target);
    return s && t && (s.folder || 'root') === folderName && (t.folder || 'root') !== folderName;
  });

  let html = '';
  if (inEdges.length > 0) {
    html += `<div style="font-size:0.65rem;font-weight:700;color:#38bdf8;margin:8px 0 4px;">📥 ENTRANTS DEPUIS D'AUTRES DOSSIERS (${inEdges.length}) :</div>`;
    inEdges.slice(0, 10).forEach(e => {
      html += `<div class="rel-card" onclick="jumpToNode('${e.source}')" style="cursor:pointer;"><span style="color:#f8fafc;">${e.source.replace('file::', '')}</span><span class="rel-tag">${e.relation}</span></div>`;
    });
  }
  if (outEdges.length > 0) {
    html += `<div style="font-size:0.65rem;font-weight:700;color:#c084fc;margin:8px 0 4px;">📤 SORTANTS VERS D'AUTRES DOSSIERS (${outEdges.length}) :</div>`;
    outEdges.slice(0, 10).forEach(e => {
      html += `<div class="rel-card" onclick="jumpToNode('${e.target}')" style="cursor:pointer;"><span style="color:#f8fafc;">${e.target.replace('file::', '')}</span><span class="rel-tag">${e.relation}</span></div>`;
    });
  }
  if (inEdges.length === 0 && outEdges.length === 0) {
    html += `<div style="font-size:0.72rem;color:#64748b;font-style:italic;">Aucune dépendance externe inter-dossiers.</div>`;
  }
  relContainer.innerHTML = html;
}

// Markdown & Mermaid Rendering
function renderVisualMermaid(code) {
  const trimmed = code.trim();
  const firstLine = trimmed.split('\n')[0].trim().toLowerCase();

  if (firstLine.startsWith('sequencediagram')) {
    const lines = trimmed.split('\n').slice(1);
    const steps = [];
    lines.forEach(line => {
      const l = line.trim();
      if (!l || l.startsWith('autonumber') || l.startsWith('%%')) return;
      const m = l.match(/([A-Za-z0-9_]+)\s*(-->>|->>|->|-->)\s*([A-Za-z0-9_]+)\s*:\s*(.+)/);
      if (m) {
        steps.push({ from: m[1], type: m[2], to: m[3], desc: m[4] });
      }
    });

    let cardHtml = `
      <div class="rendered-diagram-box">
        <div class="rendered-diagram-header">
          <span class="rendered-diagram-title">⏱️ DIAGRAMME DE SÉQUENCE</span>
          <span class="token-pill" style="font-size:0.62rem; color:#38bdf8;">${steps.length} étapes</span>
        </div>
        <div style="display:flex; flex-direction:column; gap:6px;">
    `;
    steps.forEach((st, idx) => {
      const isReturn = st.type.includes('--');
      cardHtml += `
        <div class="diagram-seq-row" style="${isReturn ? 'border-style:dashed; opacity:0.9;' : ''}">
          <div style="display:flex; align-items:center; gap:8px;">
            <span style="font-size:0.62rem; color:#64748b; font-weight:700;">#${idx+1}</span>
            <span class="diagram-seq-actor">${escapeHtml(st.from)}</span>
            <span class="diagram-seq-arrow">${isReturn ? '⇠┈┈' : '━━➤'}</span>
            <span class="diagram-seq-actor" style="color:#38bdf8;">${escapeHtml(st.to)}</span>
          </div>
          <span class="diagram-seq-msg">${escapeHtml(st.desc)}</span>
        </div>
      `;
    });
    cardHtml += `</div></div>`;
    return cardHtml;
  }

  return `<pre><code>${escapeHtml(code.trim())}</code></pre>`;
}

function parseSimpleMarkdown(md) {
  if (!md) return '<p><em>Document vide.</em></p>';
  let html = '';
  const lines = md.split('\n');
  let inCode = false, codeBlock = '', codeLang = '';

  for (let line of lines) {
    if (line.trim().startsWith('```')) {
      if (inCode) {
        if (codeLang === 'mermaid') {
          html += renderVisualMermaid(codeBlock);
        } else {
          html += `<pre><code>${escapeHtml(codeBlock.trim())}</code></pre>`;
        }
        codeBlock = '';
        codeLang = '';
        inCode = false;
      } else {
        inCode = true;
        codeLang = line.trim().substring(3).trim().toLowerCase();
      }
      continue;
    }
    if (inCode) {
      codeBlock += line + '\n';
      continue;
    }

    const trimmed = line.trim();
    if (trimmed.startsWith('# ')) {
      html += `<h1>${escapeHtml(trimmed.substring(2))}</h1>`;
    } else if (trimmed.startsWith('## ')) {
      html += `<h2>${escapeHtml(trimmed.substring(3))}</h2>`;
    } else if (trimmed.startsWith('### ')) {
      html += `<h3>${escapeHtml(trimmed.substring(4))}</h3>`;
    } else if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
      html += `<ul><li>${formatInlineMd(trimmed.substring(2))}</li></ul>`;
    } else if (trimmed.length > 0) {
      html += `<p>${formatInlineMd(trimmed)}</p>`;
    }
  }
  return html;
}

function formatInlineMd(str) {
  let res = escapeHtml(str);
  res = res.replace(/`([^`]+)`/g, '<code>$1</code>');
  res = res.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  res = res.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" style="color:#38bdf8;text-decoration:underline;">$1</a>');
  return res;
}

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function escapeForBacktick(str) {
  return (str || '').replace(/\\/g, '\\\\').replace(/`/g, '\\`').replace(/\$/g, '\\$');
}

function escapeRegExp(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

// Markdown Live Editor & Auto-Save
let docSaveTimeout = null;
let docActiveContext = { folder: 'root', fileName: 'README.md', isEditorMode: false };

function setDocSaveStatus(status, text) {
  const badge = document.getElementById('doc-save-status');
  if (!badge) return;
  badge.className = 'doc-status-badge ' + (status || '');
  badge.innerText = text;
}

function toggleDocEditorMode() {
  docActiveContext.isEditorMode = !docActiveContext.isEditorMode;
  const btn = document.getElementById('btn-toggle-doc-mode');
  const icon = document.getElementById('doc-mode-icon');
  const label = document.getElementById('doc-mode-label');

  if (btn) btn.classList.toggle('active', docActiveContext.isEditorMode);
  if (icon) icon.innerText = docActiveContext.isEditorMode ? '👁️' : '✏️';
  if (label) label.innerText = docActiveContext.isEditorMode ? 'Aperçu' : 'Éditer';

  const previewBox = document.getElementById('doc-rendered-view');
  const editorBox = document.getElementById('doc-live-editor-container');
  const textarea = document.getElementById('doc-live-editor-textarea');

  if (docActiveContext.isEditorMode) {
    if (previewBox) previewBox.style.display = 'none';
    if (editorBox) editorBox.style.display = 'flex';
    if (textarea) textarea.focus();
  } else {
    const currentMd = textarea ? textarea.value : '';
    if (previewBox) {
      const bodyEl = document.getElementById('doc-rendered-body');
      if (bodyEl) {
        bodyEl.innerHTML = parseSimpleMarkdown(currentMd);
      }
      previewBox.style.display = 'block';
    }
    if (editorBox) editorBox.style.display = 'none';
  }
}
document.getElementById('btn-toggle-doc-mode')?.addEventListener('click', toggleDocEditorMode);

function triggerDocAutoSave(currentContent) {
  setDocSaveStatus('modified', '⏳ Saisie...');
  if (docSaveTimeout) clearTimeout(docSaveTimeout);
  docSaveTimeout = setTimeout(() => {
    executeDocAutoSave(currentContent);
  }, 750);
}

async function executeDocAutoSave(content) {
  setDocSaveStatus('saving', '💾 Enregistrement...');
  const folderName = docActiveContext.folder || 'root';
  const fileName = docActiveContext.fileName || 'README.md';
  const projectPath = state.data.project_path || '';

  if (state.activeDocNode) {
    if (!state.activeDocNode.metadata) state.activeDocNode.metadata = {};
    state.activeDocNode.metadata.raw_content = content;
    state.activeDocNode.docstring = content;
  }
  docActiveContext.rawContent = content;

  if (!projectPath) {
    setDocSaveStatus('', '💾 Mémoire locale');
    return;
  }

  try {
    const d = await AetherAPI.saveDoc(projectPath, folderName, fileName, content);
    if (d && d.status === 'ok') {
      const timeStr = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
      setDocSaveStatus('', `💾 Enregistré (${timeStr})`);
    } else {
      setDocSaveStatus('saving', '⚠️ Erreur save');
    }
  } catch (err) {
    setDocSaveStatus('saving', '⚠️ Erreur réseau');
  }
}
window.executeDocAutoSave = executeDocAutoSave;

window.saveCurrentDoc = function() {
  const textarea = document.getElementById('doc-live-editor-textarea');
  const content = textarea ? textarea.value : (docActiveContext.rawContent || '');
  executeDocAutoSave(content);
  showToast('💾 Document sauvegardé avec succès !');
};

window.copyCurrentDoc = function() {
  const textarea = document.getElementById('doc-live-editor-textarea');
  const content = textarea ? textarea.value : (docActiveContext.rawContent || '');
  navigator.clipboard.writeText(content);
  showToast('📋 Markdown copié dans le presse-papier !');
};

function formatAstDocstring(rawDoc) {
  if (!rawDoc) return '';
  let cleaned = rawDoc
    .replace(/^(\s*(\/\/\/|\/\/!|\/\/|\*|#)\s?)/gm, '')
    .trim();

  cleaned = escapeHtml(cleaned);

  cleaned = cleaned.replace(/@param\s+(?:\{([^}]+)\}\s+)?([a-zA-Z0-9_$]+)(?:\s+-\s+|\s+)?([^\n<]*)/g, (m, type, name, desc) => {
    return `<div class="doc-param-row"><span class="doc-tag-badge param">@param</span> <code>${name}</code> ${type ? `<span class="doc-tag-type">{${type}}</span>` : ''} <span class="doc-param-desc">${desc || ''}</span></div>`;
  });
  cleaned = cleaned.replace(/@returns?\s+(?:\{([^}]+)\}\s+)?([^\n<]*)/g, (m, type, desc) => {
    return `<div class="doc-param-row"><span class="doc-tag-badge return">@returns</span> ${type ? `<span class="doc-tag-type">{${type}}</span>` : ''} <span class="doc-param-desc">${desc || ''}</span></div>`;
  });
  cleaned = cleaned.replace(/@(?:throws|exception)\s+(?:\{([^}]+)\}\s+)?([^\n<]*)/g, (m, type, desc) => {
    return `<div class="doc-param-row"><span class="doc-tag-badge throws">@throws</span> ${type ? `<span class="doc-tag-type">{${type}}</span>` : ''} <span class="doc-param-desc">${desc || ''}</span></div>`;
  });
  cleaned = cleaned.replace(/@(?:type|typedef)\s+(?:\{([^}]+)\}\s+)?([^\n<]*)/g, (m, type, desc) => {
    return `<div class="doc-param-row"><span class="doc-tag-badge type">@type</span> ${type ? `<span class="doc-tag-type">{${type}}</span>` : ''} <span class="doc-param-desc">${desc || ''}</span></div>`;
  });

  cleaned = cleaned.replace(/^(?:#|===)\s+(Arguments|Errors|Panics|Returns|Examples?|Safety)\b/gim, (m, sec) => {
    return `<div class="doc-docstring-section">📌 ${sec}</div>`;
  });

  cleaned = cleaned.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  cleaned = cleaned.replace(/`([^`]+)`/g, '<code>$1</code>');
  cleaned = cleaned.replace(/\n\n+/g, '<br><br>');
  cleaned = cleaned.replace(/\n(?!\s*<div)/g, '<br>');

  return cleaned;
}

function getSymbolSignature(n) {
  if (!n) return '';
  const meta = n.metadata || {};
  let sig = n.signature || meta.signature || '';
  if (!sig) {
    if (meta.methods && meta.methods.length === 1 && meta.methods[0].name === n.name) {
      const m = meta.methods[0];
      sig = `${m.is_async ? 'async ' : ''}fn ${m.name}(${m.args || ''})${m.returns ? ' -> ' + m.returns : ''}`;
    } else if (n.symbol_type === 'function' || n.is_function || n.stereotype === '«Function»' || n.stereotype === '«Method»' || n.stereotype === '«Action»') {
      const params = n.parameters || meta.parameters || [];
      const retType = n.return_type || meta.return_type || '';
      sig = `fn ${n.name}(${params.join(', ')})${retType ? ' -> ' + retType : ''}`;
    } else if (n.is_class || n.stereotype === '«Class»' || n.stereotype === '«Model»' || n.stereotype === '«Service»') {
      sig = `class ${n.name}`;
    } else if (n.is_file || n.symbol_type === 'file') {
      sig = `// File: ${n.name}`;
    }
  }
  return sig;
}

function renderDocLiaisonCard(e, isIncoming) {
  const otherId = isIncoming ? e.source : e.target;
  const nodes = state.data.nodes || [];
  const otherNode = nodes.find(x => x.id === otherId);

  const rawName = otherNode ? otherNode.name : otherId.split('::').pop().replace('file::', '');
  const cleanName = rawName || otherId;
  const path = otherNode ? (otherNode.file_path || otherNode.folder || '') : '';
  const stereotype = otherNode ? (otherNode.stereotype || '«Symbol»') : '';
  const layer = otherNode ? (otherNode.layer || 'generic') : 'generic';

  let relEmoji = '⚡';
  let relClass = 'calls';
  const r = (e.relation || 'calls').toLowerCase();
  if (r === 'imports') { relEmoji = '📦'; relClass = 'imports'; }
  else if (r === 'submits') { relEmoji = '📤'; relClass = 'submits'; }
  else if (r === 'queries') { relEmoji = '🗄️'; relClass = 'queries'; }
  else if (r === 'network') { relEmoji = '🌐'; relClass = 'network'; }
  else if (r === 'renders') { relEmoji = '🎨'; relClass = 'renders'; }
  else if (r === 'inherits') { relEmoji = '🏛️'; relClass = 'inherits'; }
  else if (r === 'implements') { relEmoji = '⚙️'; relClass = 'implements'; }
  else if (r === 'validates') { relEmoji = '🛡️'; relClass = 'validates'; }
  else if (r === 'documents') { relEmoji = '📖'; relClass = 'documents'; }

  return `
    <div class="doc-liaison-card" onclick="jumpToNode('${escapeForBacktick(otherId)}')" title="Naviguer vers ${escapeHtml(cleanName)}">
      <div class="doc-liaison-card-main">
        <div class="doc-liaison-name-row">
          <span class="doc-liaison-icon">${relEmoji}</span>
          <span class="doc-liaison-name">${escapeHtml(cleanName)}</span>
          ${stereotype ? `<span class="doc-mini-tag ${layer}">${escapeHtml(stereotype)}</span>` : ''}
        </div>
        ${path ? `<div class="doc-liaison-path" title="${escapeHtml(path)}">${escapeHtml(path)}</div>` : ''}
      </div>
      <div class="doc-liaison-badges">
        <span class="doc-rel-chip ${relClass}">${relEmoji} ${escapeHtml(e.relation || 'calls')}</span>
        <span class="doc-liaison-jump-arrow">➔</span>
      </div>
    </div>
  `;
}

window.filterDocLiaisons = function(keyword) {
  const term = (keyword || '').toLowerCase().trim();
  const cards = document.querySelectorAll('.doc-liaison-card');
  cards.forEach(card => {
    const text = card.textContent.toLowerCase();
    card.style.display = text.includes(term) ? 'flex' : 'none';
  });
};

function renderDocContainedMethods(n) {
  const methods = (n.metadata && n.metadata.methods) || [];
  const nodes = state.data.nodes || [];

  if (n.is_file) {
    const fileSymbols = nodes.filter(x => x.file_path === n.file_path && x.id !== n.id);
    if (fileSymbols.length === 0) return '';
    return `
      <div class="doc-contained-box">
        <div class="doc-contained-header">
          <span>🧩 Fonctions & Classes du fichier (${fileSymbols.length})</span>
        </div>
        <div class="doc-contained-list">
          ${fileSymbols.map(s => {
            const docSnippet = (s.docstring || (s.metadata && s.metadata.docstring) || '').replace(/^[\s\S]*?(?=\w)/, '').split('\n')[0] || '';
            return `
              <div class="doc-contained-item" onclick="jumpToNode('${escapeForBacktick(s.id)}')">
                <div class="doc-contained-info">
                  <div class="doc-contained-name">
                    <span>${s.is_class ? '🏛️' : '⚡'}</span>
                    <strong>${escapeHtml(s.name)}</strong>
                    <span class="doc-mini-tag ${s.layer || 'generic'}">${escapeHtml(s.stereotype || '')}</span>
                  </div>
                  ${docSnippet ? `<div class="doc-contained-desc">${escapeHtml(docSnippet)}</div>` : ''}
                </div>
                <span class="doc-contained-jump">➔</span>
              </div>
            `;
          }).join('')}
        </div>
      </div>
    `;
  }

  if (methods.length > 0) {
    return `
      <div class="doc-contained-box">
        <div class="doc-contained-header">
          <span>⚡ Méthodes & Fonctions membres (${methods.length})</span>
        </div>
        <div class="doc-contained-list">
          ${methods.map(m => {
            const sig = `${m.is_async ? 'async ' : ''}${m.name}(${m.args || ''})${m.returns ? ': ' + m.returns : ''}`;
            const mDoc = m.docstring ? formatAstDocstring(m.docstring) : '';
            return `
              <div class="doc-method-card">
                <div class="doc-method-sig-row">
                  <span class="doc-method-icon">⚡</span>
                  <code class="doc-method-sig">${escapeHtml(sig)}</code>
                  ${m.is_async ? '<span class="doc-async-pill">async</span>' : ''}
                </div>
                ${mDoc ? `<div class="doc-method-doc">${mDoc}</div>` : ''}
                ${m.calls && m.calls.length > 0 ? `
                  <div class="doc-method-calls">
                    <span style="color:#94a3b8; font-size:0.64rem;">Appelle :</span>
                    ${m.calls.map(c => `<span class="doc-call-chip">${escapeHtml(c)}</span>`).join('')}
                  </div>
                ` : ''}
              </div>
            `;
          }).join('')}
        </div>
      </div>
    `;
  }

  return '';
}

function renderFolderLiaisonsCard(folderName) {
  if (!folderName || folderName === 'root') return '';
  const nodes = state.data.nodes || [];
  const edges = state.data.edges || [];

  const folderNodes = nodes.filter(n => (n.folder || 'root') === folderName);
  if (folderNodes.length === 0) return '';

  const folderNodeIds = new Set(folderNodes.map(n => n.id));

  // Cross-module incoming: target belongs to folder, source is outside
  const incoming = edges.filter(e => folderNodeIds.has(e.target) && !folderNodeIds.has(e.source) && e.relation !== 'contains');
  // Cross-module outgoing: source belongs to folder, target is outside
  const outgoing = edges.filter(e => folderNodeIds.has(e.source) && !folderNodeIds.has(e.target) && e.relation !== 'contains');

  const incomingHtml = incoming.length > 0
    ? incoming.slice(0, 50).map(e => renderDocLiaisonCard(e, true)).join('')
    : '<div class="doc-liaison-empty">Aucun appelant externe direct détecté pour ce module.</div>';

  const outgoingHtml = outgoing.length > 0
    ? outgoing.slice(0, 50).map(e => renderDocLiaisonCard(e, false)).join('')
    : '<div class="doc-liaison-empty">Aucune dépendance externe sortante vers d\'autres modules.</div>';

  const filesCount = folderNodes.filter(n => n.is_file).length;

  return `
    <div class="doc-symbol-section" style="margin-top:18px;">
      <div class="doc-symbol-card">
        <div class="doc-symbol-header">
          <div class="doc-symbol-title-row">
            <span class="doc-stereo-tag" style="background:rgba(16,185,129,0.2); color:#6ee7b7; border-color:rgba(16,185,129,0.5);">«Module»</span>
            <span class="doc-symbol-title">📁 ${escapeHtml(folderName)}</span>
          </div>
          <div class="doc-symbol-subline">
            <span class="doc-symbol-path">Liaisons architecturales et connectivité inter-modules</span>
          </div>
        </div>

        <div class="doc-symbol-metrics">
          <div class="doc-symbol-metric">
            <span class="doc-metric-num">${filesCount || folderNodes.length}</span>
            <span class="doc-metric-lbl">Fichiers</span>
          </div>
          <div class="doc-symbol-metric">
            <span class="doc-metric-num">${folderNodes.length}</span>
            <span class="doc-metric-lbl">Symboles</span>
          </div>
          <div class="doc-symbol-metric">
            <span class="doc-metric-num cyan">${incoming.length}</span>
            <span class="doc-metric-lbl">📥 Appels Amont</span>
          </div>
          <div class="doc-symbol-metric">
            <span class="doc-metric-num purple">${outgoing.length}</span>
            <span class="doc-metric-lbl">📤 Dépendances</span>
          </div>
        </div>

        <div class="doc-liaison-container">
          <div class="doc-liaison-sec-header">
            <div class="doc-liaison-sec-title">
              <span>🔗 Informations de liaison du Module (${incoming.length + outgoing.length} flux inter-modules)</span>
            </div>
            <div class="doc-liaison-filter-box">
              <input type="text" class="doc-liaison-search" placeholder="🔍 Filtrer les flux..." oninput="filterDocLiaisons(this.value)">
            </div>
          </div>

          <div class="doc-liaison-grid">
            <div class="doc-liaison-col incoming">
              <div class="doc-liaison-col-header">
                <span>📥 Flux Entrants (Modules qui appellent ${escapeHtml(folderName)})</span>
                <span class="doc-count-pill cyan">${incoming.length}</span>
              </div>
              <div class="doc-liaison-list" id="doc-incoming-list">
                ${incomingHtml}
              </div>
            </div>

            <div class="doc-liaison-col outgoing">
              <div class="doc-liaison-col-header">
                <span>📤 Flux Sortants (Ce que ${escapeHtml(folderName)} appelle)</span>
                <span class="doc-count-pill purple">${outgoing.length}</span>
              </div>
              <div class="doc-liaison-list" id="doc-outgoing-list">
                ${outgoingHtml}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  `;
}

function renderSymbolLiaisonCard(n, incoming, outgoing) {
  if (!n) return '';
  const loc = n.lines_count || (n.metadata && n.metadata.lines) || 1;
  const cyclo = n.complexity || (n.metadata && n.metadata.complexity) || 1;
  const sig = getSymbolSignature(n);
  const rawDoc = (n.docstring || (n.metadata && n.metadata.docstring) || '').trim();
  const hasRealDoc = rawDoc && !rawDoc.startsWith('Fichier source') && !rawDoc.startsWith('Module source') && !rawDoc.startsWith('Module Rust') && !rawDoc.startsWith('Module Python') && !rawDoc.startsWith('Documentation Markdown');
  const formattedDoc = hasRealDoc ? formatAstDocstring(rawDoc) : '';

  const incomingHtml = incoming.length > 0
    ? incoming.map(e => renderDocLiaisonCard(e, true)).join('')
    : '<div class="doc-liaison-empty">Point d\'entrée racine ou aucun appelant direct.</div>';

  const outgoingHtml = outgoing.length > 0
    ? outgoing.map(e => renderDocLiaisonCard(e, false)).join('')
    : '<div class="doc-liaison-empty">Fonction feuille (aucun sous-appel) ou pure.</div>';

  const containedHtml = renderDocContainedMethods(n);

  return `
    <div id="doc-symbol-section" class="doc-symbol-section ${docActiveContext.viewMode === 'doc' ? 'hidden' : ''}" style="${docActiveContext.viewMode === 'doc' ? 'display:none;' : ''}">
      <div class="doc-symbol-card">
        <div class="doc-symbol-header">
          <div class="doc-symbol-title-row">
            <span class="doc-stereo-tag ${n.layer || 'generic'}">${escapeHtml(n.stereotype || '«Symbol»')}</span>
            <span class="doc-symbol-title">${escapeHtml(n.name)}</span>
            ${n.is_dead_code ? '<span class="doc-mini-pill dead">💀 Orphelin</span>' : ''}
            ${n.has_security_violation ? '<span class="doc-mini-pill alert">🚨 Alerte CVE</span>' : ''}
          </div>
          <div class="doc-symbol-subline">
            <span class="doc-symbol-path" title="${escapeHtml(n.file_path || n.folder || '')}">
              📁 ${escapeHtml(n.file_path || n.folder || '')}${n.line_number ? `:${n.line_number}` : ''}
            </span>
            <button class="doc-path-copy-btn" onclick="navigator.clipboard.writeText('${escapeForBacktick(n.file_path || n.folder || '')}'); showToast('📋 Chemin copié !');" title="Copier le chemin">📋</button>
          </div>
        </div>

        <div class="doc-symbol-metrics">
          <div class="doc-symbol-metric">
            <span class="doc-metric-num">${loc}</span>
            <span class="doc-metric-lbl">LOC</span>
          </div>
          <div class="doc-symbol-metric">
            <span class="doc-metric-num">${cyclo}</span>
            <span class="doc-metric-lbl">Cyclomatique</span>
          </div>
          <div class="doc-symbol-metric">
            <span class="doc-metric-num cyan">${incoming.length}</span>
            <span class="doc-metric-lbl">📥 Appelants</span>
          </div>
          <div class="doc-symbol-metric">
            <span class="doc-metric-num purple">${outgoing.length}</span>
            <span class="doc-metric-lbl">📤 Dépendances</span>
          </div>
        </div>

        ${sig ? `
          <div class="doc-sig-card">
            <div class="doc-sig-header">
              <span>⚡ Signature AST</span>
              <button class="doc-sig-copy-btn" onclick="navigator.clipboard.writeText('${escapeForBacktick(sig)}'); showToast('📋 Signature copiée !');">📋 Copier</button>
            </div>
            <pre class="doc-sig-code"><code>${escapeHtml(sig)}</code></pre>
          </div>
        ` : ''}

        <div id="doc-symbol-comments-container">
          ${hasRealDoc ? `
            <div class="doc-comments-box">
              <div class="doc-comments-header">
                <span>💬 Spécifications & Commentaires de fonction</span>
                <span class="doc-badge-chip">AST Doc</span>
              </div>
              <div class="doc-comments-body" id="doc-symbol-comments-body">${formattedDoc}</div>
            </div>
          ` : `
            <div class="doc-comments-box empty" id="doc-comments-empty-notice">
              <div class="doc-comments-header">
                <span>💬 Commentaires de fonction & Docstrings AST</span>
                <span class="doc-badge-chip muted">En attente / Source</span>
              </div>
              <div class="doc-comments-empty-text">
                Analyse du fichier source pour les commentaires et spécifications…
              </div>
              <div class="doc-comments-body" id="doc-symbol-comments-body" style="margin-top:4px;"></div>
            </div>
          `}
        </div>

        ${containedHtml}

        <div class="doc-liaison-container">
          <div class="doc-liaison-sec-header">
            <div class="doc-liaison-sec-title">
              <span>🔗 Informations de liaison & Connectivité (${incoming.length + outgoing.length})</span>
            </div>
            <div class="doc-liaison-filter-box">
              <input type="text" class="doc-liaison-search" id="doc-liaison-search-input" placeholder="🔍 Filtrer les flux..." oninput="filterDocLiaisons(this.value)">
            </div>
          </div>

          <div class="doc-liaison-grid">
            <div class="doc-liaison-col incoming">
              <div class="doc-liaison-col-header">
                <span>📥 Flux Amont (Qui l'appelle / Importers)</span>
                <span class="doc-count-pill cyan">${incoming.length}</span>
              </div>
              <div class="doc-liaison-list" id="doc-incoming-list">
                ${incomingHtml}
              </div>
            </div>

            <div class="doc-liaison-col outgoing">
              <div class="doc-liaison-col-header">
                <span>📤 Flux Aval (Ce qu'il appelle / Dépendances)</span>
                <span class="doc-count-pill purple">${outgoing.length}</span>
              </div>
              <div class="doc-liaison-list" id="doc-outgoing-list">
                ${outgoingHtml}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  `;
}

async function ensureNodeDocstring(n) {
  if (!n) return;
  const existingDoc = (n.docstring || (n.metadata && n.metadata.docstring) || '').trim();
  if (existingDoc && !existingDoc.startsWith('Fichier source') && !existingDoc.startsWith('Module source') && !existingDoc.startsWith('Module Rust') && !existingDoc.startsWith('Module Python') && !existingDoc.startsWith('Documentation Markdown')) {
    const commentContainer = document.getElementById('doc-symbol-comments-body');
    if (commentContainer && !commentContainer.innerHTML.trim()) {
      commentContainer.innerHTML = formatAstDocstring(existingDoc);
      const emptyBox = document.getElementById('doc-comments-empty-notice');
      if (emptyBox) emptyBox.classList.remove('empty');
    }
    return existingDoc;
  }
  if (!n.file_path || !n.line_number) {
    const emptyBox = document.getElementById('doc-comments-empty-notice');
    if (emptyBox) {
      const txt = emptyBox.querySelector('.doc-comments-empty-text');
      if (txt) txt.innerText = 'Aucun commentaire AST détecté directement sur ce symbole.';
    }
    return '';
  }

  try {
    const res = await AetherAPI.fetchFileContent(n.file_path);
    if (res && res.status === 'ok' && res.content) {
      const lines = res.content.split('\n');
      const targetIdx = n.line_number - 1;
      const collected = [];
      let inBlock = false;
      let skippedBlanks = 0;

      for (let i = targetIdx - 1; i >= 0 && i >= targetIdx - 35; i--) {
        const raw = lines[i].trim();
        if (!raw) {
          if (collected.length === 0 && skippedBlanks < 2) {
            skippedBlanks++;
            continue;
          } else {
            break;
          }
        }
        if (raw.startsWith('#[') || raw.startsWith('#![') || raw === ']' || (raw.startsWith('@') && !raw.startsWith('@param') && !raw.startsWith('@return'))) {
          continue;
        }
        if (inBlock) {
          if (raw.includes('/*')) {
            const p = raw.substring(raw.indexOf('/*') + 2).replace(/^\*+/, '').trim();
            if (p) collected.push(p);
            inBlock = false;
          } else {
            collected.push(raw.replace(/^\*+/, '').trim());
          }
          continue;
        }
        if (raw.endsWith('*/')) {
          inBlock = true;
          if (raw.includes('/*')) {
            const inner = raw.substring(raw.indexOf('/*') + 2, raw.length - 2).replace(/^\*+/, '').trim();
            if (inner) collected.push(inner);
            inBlock = false;
          } else {
            const inner = raw.substring(0, raw.length - 2).replace(/^\*+/, '').trim();
            if (inner) collected.push(inner);
          }
          continue;
        }
        if (raw.startsWith('///') || raw.startsWith('//!')) {
          collected.push(raw.substring(3).trim());
        } else if (raw.startsWith('//')) {
          collected.push(raw.substring(2).trim());
        } else if (raw.startsWith('#')) {
          collected.push(raw.substring(1).trim());
        } else {
          break;
        }
      }

      if (collected.length > 0) {
        collected.reverse();
        const extracted = collected.join('\n');
        n.docstring = extracted;
        const commentContainer = document.getElementById('doc-symbol-comments-body');
        const emptyBox = document.getElementById('doc-comments-empty-notice');
        if (commentContainer) {
          commentContainer.innerHTML = formatAstDocstring(extracted);
          if (emptyBox) {
            emptyBox.classList.remove('empty');
            const emptyTxt = emptyBox.querySelector('.doc-comments-empty-text');
            if (emptyTxt) emptyTxt.style.display = 'none';
            const chip = emptyBox.querySelector('.doc-badge-chip');
            if (chip) { chip.classList.remove('muted'); chip.innerText = 'Source Doc'; }
          }
        }
        return extracted;
      } else {
        const emptyBox = document.getElementById('doc-comments-empty-notice');
        if (emptyBox) {
          const emptyTxt = emptyBox.querySelector('.doc-comments-empty-text');
          if (emptyTxt) emptyTxt.innerText = 'Aucun commentaire AST détecté directement sur ce symbole.';
        }
      }
    }
  } catch(e) {}
  return '';
}

window.setDocViewMode = function(mode) {
  docActiveContext.viewMode = mode;
  try { localStorage.setItem('aether_doc_view_mode', mode); } catch(e) {}

  const symSec = document.getElementById('doc-symbol-section');
  const mdSec = document.getElementById('doc-markdown-section');
  const tabs = document.querySelectorAll('.doc-view-tab');

  tabs.forEach(t => {
    t.classList.toggle('active', t.getAttribute('data-mode') === mode);
  });

  if (symSec) {
    symSec.style.display = (mode === 'doc') ? 'none' : 'block';
  }
  if (mdSec) {
    mdSec.style.display = (mode === 'symbol') ? 'none' : 'block';
  }
};

function renderDocReaderUI(title, folder, fileName, rawContent, isDoc, targetNodeName, tabsHtml, filePath, targetNode) {
  targetNode = targetNode || state.selectedNode;
  docActiveContext.folder = folder;
  docActiveContext.fileName = fileName;
  docActiveContext.rawContent = rawContent;

  const savedMode = (function() {
    try { return localStorage.getItem('aether_doc_view_mode') || 'combined'; } catch(e) { return 'combined'; }
  })();
  if (!docActiveContext.viewMode) {
    docActiveContext.viewMode = isDoc ? 'doc' : savedMode;
  }

  const titleEl = document.getElementById('doc-reader-title');
  if (titleEl) {
    titleEl.innerText = title;
    titleEl.title = title;
  }
  setDocSaveStatus('', '💾 Prêt');

  let parsed = parseSimpleMarkdown(rawContent);
  if (!isDoc && targetNodeName) {
    try {
      const regex = new RegExp(`(<code>\\s*${escapeRegExp(targetNodeName)}\\s*<\\/code>|\\b${escapeRegExp(targetNodeName)}\\b)`, 'g');
      parsed = parsed.replace(regex, `<mark style="background:rgba(56,189,248,0.35); color:#38bdf8; padding:1px 4px; border-radius:3px; font-weight:bold;">$1</mark>`);
    } catch(e) {}
  }

  const displayPath = filePath || fileName;
  let displayShortPath = displayPath;
  const pathParts = displayPath.split(/[/\\]/);
  if (pathParts.length > 2) {
    displayShortPath = '…/' + pathParts.slice(-2).join('/');
  }

  const currentMode = docActiveContext.viewMode;
  const showSymbol = targetNode && !isDoc;
  const edges = state.data.edges || [];
  const incoming = targetNode ? edges.filter(e => e.target === targetNode.id) : [];
  const outgoing = targetNode ? edges.filter(e => e.source === targetNode.id) : [];

  const viewSwitcherHtml = showSymbol ? `
    <div class="doc-view-switcher">
      <button class="doc-view-tab ${currentMode === 'combined' ? 'active' : ''}" data-mode="combined" onclick="setDocViewMode('combined')">
        📑 Vue Combinée
      </button>
      <button class="doc-view-tab ${currentMode === 'symbol' ? 'active' : ''}" data-mode="symbol" onclick="setDocViewMode('symbol')">
        ⚡ Symbole & Liaisons (${incoming.length + outgoing.length})
      </button>
      <button class="doc-view-tab ${currentMode === 'doc' ? 'active' : ''}" data-mode="doc" onclick="setDocViewMode('doc')">
        📖 Spécification: ${escapeHtml(fileName || 'README.md')}
      </button>
    </div>
  ` : '';

  const symbolHtml = showSymbol ? renderSymbolLiaisonCard(targetNode, incoming, outgoing) : '';

  const actionsHtml = `
    <div class="doc-actions-bar">
      <div class="doc-actions-path">
        <span class="doc-actions-path-label">
          ${isDoc ? '📖 Document' : `🎯 Spécification ${targetNodeName ? '<code>' + escapeHtml(targetNodeName) + '</code>' : ''}`} :
        </span>
        <code class="doc-actions-path-code" title="${escapeHtml(displayPath)}">${escapeHtml(displayShortPath)}</code>
      </div>
      <div class="doc-actions-btns">
        <button class="aether-btn primary" onclick="saveCurrentDoc()">💾 Sauvegarder</button>
        <button class="aether-btn primary" onclick="signContractForCurrent('${escapeHtml(folder)}')">🔒 Signer Contrat</button>
        <button class="aether-btn" onclick="copyCurrentDoc()">📋 Copier</button>
      </div>
    </div>
  `;

  const isEdit = docActiveContext.isEditorMode;
  const markdownHtml = `
    <div id="doc-markdown-section" style="display:${currentMode === 'symbol' ? 'none' : 'block'};">
      ${tabsHtml}
      ${actionsHtml}
      <div id="doc-rendered-view" style="display:${isEdit ? 'none' : 'block'};">
        <div id="doc-rendered-body">${parsed}</div>
      </div>
      <div id="doc-live-editor-container" style="display:${isEdit ? 'flex' : 'none'}; flex-direction:column; flex:1; gap:6px;">
        <div style="display:flex; justify-content:space-between; align-items:center; font-size:0.68rem; color:#94a3b8; font-family:var(--font-mono);">
          <span>✏️ Modifiez directement ci-dessous (sauvegarde automatique en direct) :</span>
          <span style="color:#6ee7b7;">● Auto-save actif</span>
        </div>
        <textarea id="doc-live-editor-textarea" class="doc-live-textarea" placeholder="Rédigez vos spécifications en Markdown...">${escapeHtml(rawContent)}</textarea>
      </div>
    </div>
  `;

  const folderLiaisonsHtml = (!showSymbol && folder && folder !== 'root') ? renderFolderLiaisonsCard(folder) : '';

  const html = `
    ${viewSwitcherHtml}
    ${markdownHtml}
    ${symbolHtml}
    ${folderLiaisonsHtml}
  `;

  const contentBox = document.getElementById('doc-reader-content');
  if (contentBox) {
    contentBox.innerHTML = html;
    contentBox.scrollTop = 0;
  }

  const textarea = document.getElementById('doc-live-editor-textarea');
  if (textarea) {
    textarea.addEventListener('input', () => {
      triggerDocAutoSave(textarea.value);
    });
  }

  if (targetNode) {
    ensureNodeDocstring(targetNode);
  }
}

window.openNodeDocReader = async function(n, activeIndex = 0, forceOpen = false) {
  if (!n) return;
  state.selectedNode = n;
  const panel = document.getElementById('doc-reader-panel');
  const shouldOpen = forceOpen === true || (panel && !panel.classList.contains('hidden'));
  const nodes = state.data.nodes || [];
  const edges = state.data.edges || [];

  const isDoc = n.layer === 'docs' || n.symbol_type === 'document' || (n.file_path && n.file_path.endsWith('.md'));
  let docList = [];

  if (isDoc) {
    docList = [{ doc: n, reason: 'Sélection directe', priority: 1 }];
  } else {
    const seenDocIds = new Set();
    const candidateDocs = [];

    edges.forEach(e => {
      if (e.target === n.id && e.relation === 'documents') {
        const dn = nodes.find(x => x.id === e.source);
        if (dn && !seenDocIds.has(dn.id)) {
          seenDocIds.add(dn.id);
          candidateDocs.push({ doc: dn, reason: '⚡ Mot-clé direct', priority: 1 });
        }
      }
    });

    const nodeFolder = n.folder || 'root';
    nodes.forEach(dn => {
      const isDnDoc = dn.layer === 'docs' || dn.symbol_type === 'document' || (dn.file_path && dn.file_path.endsWith('.md'));
      if (isDnDoc && !seenDocIds.has(dn.id)) {
        if ((dn.folder || 'root') === nodeFolder) {
          seenDocIds.add(dn.id);
          candidateDocs.push({ doc: dn, reason: '📁 Module dossier', priority: 2 });
        }
      }
    });

    candidateDocs.sort((a, b) => a.priority - b.priority);
    docList = candidateDocs;
  }

  if (docList.length === 0) {
    const defaultTemplate = `# Module Architectural : ${n.folder || 'root'}\n\n## 🎯 Spécifications pour ${n.name}\n- Rôle du symbole : \`${n.name}\` (${n.stereotype || 'Symbol'})\n- Fichier : \`${n.file_path || ''}\`\n\n## ⚙️ Règles & Contrats (EARS)\n- Condition EARS 1 : Invariants vérifiés\n`;
    renderDocReaderUI(
      `⚡ ${n.name} [Spécification & Liaisons]`,
      n.folder || 'root',
      'README.md',
      defaultTemplate,
      false,
      n.name,
      '',
      n.file_path,
      n
    );
    if (shouldOpen) {
      panel?.classList.remove('hidden');
      document.getElementById('btn-expand-doc-dock')?.classList.add('hidden');
      document.getElementById('btn-topbar-docs')?.classList.add('active');
    }
    return;
  }

  const safeIndex = Math.min(activeIndex, docList.length - 1);
  const activeEntry = docList[safeIndex];
  const activeDoc = activeEntry.doc;
  const cleanDocName = (activeDoc.name || '').replace(/\s*\([^)]*src\/[^)]*\)/gi, '').trim();
  const title = isDoc ? `📖 ${cleanDocName || activeDoc.name}` : `📖 ${n.name} ➔ ${cleanDocName || activeDoc.name}`;

  let tabsHtml = '';
  if (docList.length > 1) {
    tabsHtml = `<div class="doc-tabs-vertical">
      <div class="doc-tabs-header">
        <span>📚 Documents associés (${docList.length})</span>
        <span style="font-size:0.66rem; color:#94a3b8; font-weight:500;">Spécification active</span>
      </div>`;
    docList.forEach((entry, idx) => {
      const isActive = idx === safeIndex;
      tabsHtml += `<div class="doc-tab-row ${isActive ? 'active' : ''}" onclick="openNodeDocReader(state.selectedNode, ${idx}, true)" title="${escapeHtml(entry.doc.file_path || entry.doc.name)}">
        <div class="doc-tab-info">
          <div class="doc-tab-name">
            <span>${isActive ? '👉' : '📄'}</span>
            <span>${escapeHtml(entry.doc.name)}</span>
          </div>
          <div class="doc-tab-path">${escapeHtml(entry.doc.file_path || '')}</div>
        </div>
        <span class="doc-tab-reason">${escapeHtml(entry.reason)}</span>
      </div>`;
    });
    tabsHtml += `</div>`;
  }

  const raw = await fetchDocContentOnDemand(activeDoc);
  state.activeDocNode = activeDoc;
  const cleanFileName = (activeDoc.file_path ? activeDoc.file_path.split('/').pop() : '') || cleanDocName || activeDoc.name;
  renderDocReaderUI(title, activeDoc.folder || 'root', cleanFileName, raw, isDoc, n.name, tabsHtml, activeDoc.file_path, n);
  if (shouldOpen) {
    panel?.classList.remove('hidden');
    document.getElementById('btn-expand-doc-dock')?.classList.add('hidden');
    document.getElementById('btn-topbar-docs')?.classList.add('active');
  }
};

function openDocReader(n) {
  openNodeDocReader(n, 0, true);
}
window.openDocReader = openDocReader;

window.openFolderDocReader = async function(folderName, mdNodes, activeIndex = 0, forceOpen = false) {
  const panel = document.getElementById('doc-reader-panel');
  if (!mdNodes || mdNodes.length === 0) return;
  const shouldOpen = forceOpen === true || (panel && !panel.classList.contains('hidden'));
  state.folderMdMap.set(folderName, mdNodes);
  const activeNode = mdNodes[activeIndex] || mdNodes[0];

  let tabsHtml = '';
  if (mdNodes.length > 0) {
    tabsHtml = `<div class="doc-tabs-vertical">
      <div class="doc-tabs-header">
        <span>📁 Spécifications du module (${mdNodes.length})</span>
        <span style="font-size:0.66rem; color:#94a3b8; font-weight:500;">Fichier actif</span>
      </div>`;
    mdNodes.forEach((mn, idx) => {
      const isActive = idx === activeIndex;
      tabsHtml += `<div class="doc-tab-row ${isActive ? 'active' : ''}" onclick="openFolderDocReader('${folderName}', state.folderMdMap.get('${folderName}'), ${idx}, true)" title="${escapeHtml(mn.file_path || mn.name)}">
        <div class="doc-tab-info">
          <div class="doc-tab-name">
            <span>${isActive ? '👉' : '📄'}</span>
            <span>${escapeHtml(mn.name)}</span>
          </div>
          <div class="doc-tab-path">${escapeHtml(mn.file_path || '')}</div>
        </div>
        <span class="doc-tab-reason">Module</span>
      </div>`;
    });
    tabsHtml += `</div>`;
  }

  const cleanNodeName = (activeNode.name || '').replace(/\s*\([^)]*src\/[^)]*\)/gi, '').trim();
  const title = `📁 ${folderName} / ${cleanNodeName || activeNode.name}`;
  const raw = await fetchDocContentOnDemand(activeNode);
  state.activeDocNode = activeNode;
  const cleanFileName = (activeNode.file_path ? activeNode.file_path.split('/').pop() : '') || cleanNodeName || activeNode.name;
  renderDocReaderUI(title, folderName, cleanFileName, raw, true, null, tabsHtml, activeNode.file_path);
  if (shouldOpen) {
    panel?.classList.remove('hidden');
    document.getElementById('btn-expand-doc-dock')?.classList.add('hidden');
    document.getElementById('btn-topbar-docs')?.classList.add('active');
  }
};

window.openEmptyFolderDoc = function(folderName, folderNodes, forceOpen = false) {
  const panel = document.getElementById('doc-reader-panel');
  const shouldOpen = forceOpen === true || (panel && !panel.classList.contains('hidden'));
  const titleEl = document.getElementById('doc-reader-title');
  if (titleEl) titleEl.innerText = `📁 ${folderName} / README.md`;
  const files = folderNodes.filter(n => n.is_file);

  const template = `# Module Architectural : ${folderName}

## 🎯 Rôle & Responsabilités Attendues
Ce module regroupe la logique associée à : ${folderName}.

## 📦 Fichiers & Composants (${files.length})
${files.map(f => `- **${f.name}** (${f.layer})`).join('\n') || '- Aucun fichier'}

## ⚙️ Contrats & Règles Métier (EARS)
- Condition EARS 1 : Entrées/sorties validées
- Condition EARS 2 : Gestion stricte des erreurs
`;

  docActiveContext.folder = folderName;
  docActiveContext.fileName = 'README.md';
  setDocSaveStatus('modified', '⚠️ Nouveau fichier');

  const html = `
    <div style="background:rgba(245,158,11,0.12); border:1px solid rgba(245,158,11,0.35); border-radius:6px; padding:10px; margin-bottom:12px; font-size:0.75rem;">
      <div style="color:#fbbf24; font-weight:700; margin-bottom:4px;">⚠️ Spécification Markdown manquante dans ${folderName}</div>
      <div style="color:#cbd5e1; font-size:0.7rem;">Rédigez directement vos attentes pour ce dossier ci-dessous :</div>
    </div>
    <textarea id="doc-live-editor-textarea" class="doc-live-textarea" style="height:320px;">${template}</textarea>
    <div style="display:flex; gap:8px; margin-top:8px;">
      <button class="aether-btn primary" style="flex:1; justify-content:center;" onclick="executeDocAutoSave(document.getElementById('doc-live-editor-textarea').value); showToast('💾 Sauvegardé !');">💾 Enregistrer dans le projet</button>
      <button class="aether-btn primary" style="flex:1; justify-content:center;" onclick="signContractForCurrent('${folderName}')">🔒 Signer Contrat</button>
      <button class="aether-btn" style="flex:1; justify-content:center;" onclick="navigator.clipboard.writeText(document.getElementById('doc-live-editor-textarea').value); showToast('📋 Markdown copié !');">📋 Copier</button>
    </div>
  `;
  const contentBox = document.getElementById('doc-reader-content');
  if (contentBox) contentBox.innerHTML = html;

  const textarea = document.getElementById('doc-live-editor-textarea');
  if (textarea) {
    textarea.addEventListener('input', () => {
      triggerDocAutoSave(textarea.value);
    });
  }

  if (shouldOpen) {
    panel?.classList.remove('hidden');
    document.getElementById('btn-expand-doc-dock')?.classList.add('hidden');
    document.getElementById('btn-topbar-docs')?.classList.add('active');
  }
};

function toggleDocReader(forceOpen) {
  const panel = document.getElementById('doc-reader-panel');
  const dockTrigger = document.getElementById('btn-expand-doc-dock');
  const topbarBtn = document.getElementById('btn-topbar-docs');
  if (!panel) return;

  const isHidden = panel.classList.contains('hidden');
  const shouldOpen = forceOpen !== undefined ? !!forceOpen : isHidden;

  if (shouldOpen) {
    panel.classList.remove('hidden');
    if (dockTrigger) dockTrigger.classList.add('hidden');
    if (topbarBtn) topbarBtn.classList.add('active');

    const contentBox = document.getElementById('doc-reader-content');
    if (!contentBox || !contentBox.innerHTML.trim()) {
      if (state.selectedNode) {
        openNodeDocReader(state.selectedNode, 0, false);
      } else if (state.activeDocNode) {
        openNodeDocReader(state.activeDocNode, 0, false);
      } else {
        const anyDoc = (state.data.nodes || []).find(n => n.layer === 'docs' || (n.file_path && n.file_path.endsWith('.md')));
        if (anyDoc) {
          openNodeDocReader(anyDoc, 0, false);
        }
      }
    }
  } else {
    panel.classList.add('hidden');
    if (dockTrigger) dockTrigger.classList.remove('hidden');
    if (topbarBtn) topbarBtn.classList.remove('active');
  }
}
window.toggleDocReader = toggleDocReader;

document.getElementById('btn-topbar-docs')?.addEventListener('click', () => {
  toggleDocReader();
  AudioSys.playTone(550, 'sine', 0.1, 0.03);
});

document.getElementById('btn-minimize-doc')?.addEventListener('click', () => {
  toggleDocReader(false);
  AudioSys.playTone(380, 'sine', 0.08, 0.03);
});

document.getElementById('btn-close-doc')?.addEventListener('click', () => {
  toggleDocReader(false);
  AudioSys.playTone(380, 'sine', 0.08, 0.03);
});

document.getElementById('btn-expand-doc-dock')?.addEventListener('click', () => {
  toggleDocReader(true);
  AudioSys.playTone(650, 'sine', 0.12, 0.04);
});

document.getElementById('btn-copy-doc')?.addEventListener('click', () => {
  const editor = document.getElementById('doc-live-editor-textarea');
  let raw = editor ? editor.value : '';
  if (!raw) {
    const n = state.activeDocNode || state.selectedNode;
    if (n) raw = (n.metadata && n.metadata.raw_content) || n.docstring || '';
  }
  navigator.clipboard.writeText(raw).then(() => {
    showToast('📋 Markdown copié dans le presse-papier !');
  });
});

window.jumpToNode = function(id) {
  const n = (state.data.nodes || []).find(node => node.id === id);
  if (n) selectNode(n);
};

document.getElementById('btn-insp-mermaid')?.addEventListener('click', () => {
  const n = state.selectedNode;
  if (!n) return;
  const edges = state.data.edges || [];
  const outgoing = edges.filter(e => e.source === n.id);

  let code = `classDiagram\n  class ${n.name.replace(/[^A-Za-z0-9_]/g, '_')} {\n`;
  const meta = n.metadata || {};
  (meta.attributes || []).forEach(a => code += `    +${a.name}: ${a.type || 'any'}\n`);
  (meta.methods || []).forEach(m => code += `    +${m.name}()\n`);
  code += `  }\n`;

  outgoing.forEach(e => {
    const tgt = e.target.split('::').pop().replace(/[^A-Za-z0-9_]/g, '_');
    code += `  ${n.name.replace(/[^A-Za-z0-9_]/g, '_')} --> ${tgt} : ${e.relation}\n`;
  });

  navigator.clipboard.writeText(code).then(() => {
    showToast('📋 Code Mermaid copié dans le presse-papier !');
  });
});

window.copySingleAnomalyPrompt = function(filePath, line, ruleId, cveId, severity, message, data, ty, remediation) {
  if (typeof window.copyCvePocBountyPrompt === 'function' && Array.isArray(window.currentCveAnomaliesCache)) {
    const foundIdx = window.currentCveAnomaliesCache.findIndex(x =>
      x.file_path === filePath && Number(x.line_number) === Number(line) && (x.rule_id === ruleId || x.cve_id === cveId)
    );
    if (foundIdx !== -1) {
      window.copyCvePocBountyPrompt(foundIdx);
      return;
    }
  }

  const prompt = `# 🧪 PROMPT AUTO PROOF OF CONCEPT (PoC) & RAPPORT BUG BOUNTY — AETHERSCOPE V2

> ⛔ **RÈGLE ABSOLUE (ZÉRO CORRECTION BARBARE) :**  
> **Ne modifie aucun fichier source du projet.** Ton unique objectif est de vérifier l'exploitabilité réelle In-Scope de cette alerte et de rédiger un **Proof of Concept (PoC) reproductible** + le **Rapport Bug Bounty**.

> **Fichier & Ligne Cible :** \`${filePath}:${line}\`  
> **Identifiant CVE/CWE :** \`${cveId || ruleId}\` (Sévérité : **${severity}**)  
> **Règle Déclenchée :** \`${ruleId}\`  
> **Vecteur / Donnée Contaminée :** \`${data || 'N/A'}\` (\`${ty || 'N/A'}\`)  
> **Diagnostic AST :** ${message}  

---

### 🎯 Livrables Attendus (Bug Bounty Triage & PoC)
1. **Qualification Source-to-Sink (Anti-Faux-Positif) :** Vérifie si \`${filePath}:${line}\` est atteignable depuis une entrée externe non fiable dans le périmètre In-Scope.
2. **Proof of Concept (PoC) Exécutable :** Fournis un test unitaire isolé et reproductible (\`#[test]\` / \`#[tokio::test]\` en Rust ou \`test_poc\` sous Foundry) qui invoque la fonction cible avec le payload déclencheur et prouve l'impact (Panic, UB, corruption d'état, OOM ou perte de fonds) sans modifier le fichier source.
3. **Rapport Bug Bounty Prêt à Soumettre :** Title, Severity (CVSS), Root Cause, Step-by-Step Attack Scenario, PoC Code & commande d'exécution (\`cargo test ...\`).

<aether_handshake version="2.0">
  <security_target file="${filePath}" line="${line}" rule="${ruleId}" cve="${cveId}" severity="${severity}">
    <tainted_flow data="${data}" type="${ty}"/>
    <diagnostic>${message}</diagnostic>
  </security_target>
  <task objective="Générer un Proof of Concept (PoC) reproductible et un rapport Bug Bounty sans modifier le code source">
    Vérifie l'atteignabilité In-Scope de \`${filePath}:${line}\` et génère le test de Proof of Concept (PoC) autonome ainsi que le rapport Bug Bounty.
  </task>
</aether_handshake>
`;

  navigator.clipboard.writeText(prompt).then(() => {
    if (window.AudioSys && AudioSys.promptCopied) AudioSys.promptCopied();
    showToast('🧪 Prompt PoC Bug Bounty copié dans le presse-papier !');
  });
};

window.copyDefiPatchPrompt = function(filePath, line, ruleId) {
  let patchTitle = '';
  let patchInstructions = '';
  let patchCode = '';

  if (ruleId === 'DEFI-ASYMMETRIC-ROUNDING') {
    patchTitle = 'Arrondi Asymétrique (CWE-682 / ASYMMETRIC_ROUNDING_RISK)';
    patchInstructions = `1. Remplacer les divisions entières sans arrondi dans les fonctions de conversion (convertToShares, convertToAssets, previewDeposit, previewWithdraw) par des appels stricts à Math.mulDiv avec Rounding explicite (Math.Rounding.Floor sur les dépôts, Math.Rounding.Ceil sur les retraits).
2. Si le calcul concerne des taxes ou frais (ex: economy.rs), appliquer un plancher minimum de 1 wei / token via .max(1) ou mulDivUp.`;
    patchCode = `// 🛡️ Pattern Sécurisé Solidity (OpenZeppelin Math.mulDiv) :
function _convertToShares(uint256 assets, Math.Rounding rounding) internal view virtual returns (uint256) {
    return assets.mulDiv(totalSupply() + 10 ** _decimalsOffset(), totalAssets() + 1, rounding);
}

// 🛡️ Pattern Sécurisé Rust (economy.rs) :
let fee = (amount * fee_basis_points / 10_000).max(1);`;
  } else if (ruleId === 'DEFI-STALE-CACHE-AFTER-CALL') {
    patchTitle = 'Lecture d\'État Périmé Post-Appel (CWE-362 / STALE_CACHE_TOCTOU)';
    patchInstructions = `1. Identifier l'appel externe (.call, invoke, HookTarget, .await).
2. Immédiatement APRÈS cet appel externe, forcer la réactualisation ou l'invalidation du cache local (VaultCache, local_balance, snapshot) avant tout calcul critique de solde ou de collatéral.`;
    patchCode = `// 🛡️ Pattern Sécurisé Solidity / EVC :
// Appel externe pouvant altérer l'état du coffre
(bool success, ) = target.call(payload);
require(success, "CALL_FAILED");

// Invalidation & Rechargement impératif du cache d'état :
vaultCache = _loadVaultCache(); // Rechargement frais depuis le storage
uint256 updatedBalance = vaultCache.balance;`;
  } else if (ruleId === 'EVC-HOOK-BYPASS') {
    patchTitle = 'Invariant EVC & NonReentrant Hook (CWE-841 / EVC_HOOK_BYPASS_RISK)';
    patchInstructions = `1. Ajouter le modificateur nonReentrant sur la fonction de callback ou hook.
2. Vérifier l'état global de l'EVC pour empêcher toute exploitation pendant que les contrôles de solvabilité sont différés (ChecksDeferred).`;
    patchCode = `// 🛡️ Pattern Sécurisé HookTarget / EVC :
function checkVaultStatus() external view override {
    // Empêcher l'exécution transitoire sous-collatéralisée :
    if (IEVC(evc).isChecksDeferred(msg.sender)) {
        require(IEVC(evc).areChecksInProgress(), "EVC_CHECKS_DEFERRED_NOT_IN_PROGRESS");
    }
}`;
  } else if (ruleId === 'DEFI-ORACLE-MIN-MAX-CIRCUIT-BREAKER') {
    patchTitle = 'Contrôle Bornes Min/Max Oracle & Circuit Breaker (LOGIC-CRITICAL / Oracle Bounds Vector)';
    patchInstructions = `1. Interroger les bornes matérielles minAnswer et maxAnswer de l'agrégateur Chainlink via AggregatorV2V3Interface.
2. Rejeter ou basculer sur un circuit breaker / oracle secondaire si answer <= minAnswer ou answer >= maxAnswer afin de bloquer les collatéraux effondrés valorisés à un plancher factice.`;
    patchCode = `// 🛡️ Pattern Sécurisé Bornes Min/Max Chainlink :
(, int256 answer, , , ) = priceFeed.latestRoundData();
require(answer > 0, "Prix invalide");

int256 minAnswer = AggregatorV2V3Interface(feed).minAnswer();
int256 maxAnswer = AggregatorV2V3Interface(feed).maxAnswer();
require(answer > minAnswer, "Prix au plancher: possible krach de prix");
require(answer < maxAnswer, "Prix au plafond artificiel");`;
  } else if (ruleId === 'DEFI-ORACLE-L2-SEQUENCER-RACE') {
    patchTitle = 'Contrôle Uptime Séquenceur L2 & Période de Grâce (LOGIC-CRITICAL / Arbitrum-Base)';
    patchInstructions = `1. Interroger le sequencerUptimeFeed Chainlink propre au réseau L2 (Arbitrum/Optimism/Base).
2. Valider que le séquenceur est actif (answer == 0) ET que la période de grâce est expirée (block.timestamp - startedAt > GRACE_PERIOD_TIME) pour empêcher les liquidations déloyales post-reprise.`;
    patchCode = `// 🛡️ Pattern Sécurisé L2 Sequencer Uptime + Période de Grâce :
(, int256 answer, uint256 startedAt, , ) = sequencerUptimeFeed.latestRoundData();
require(answer == 0, "Séquenceur L2 indisponible");
require(block.timestamp - startedAt > GRACE_PERIOD_TIME, "Période de grâce L2 active");`;
  } else if (ruleId === 'DEFI-TWAP-SHORT-WINDOW') {
    patchTitle = 'Plancher Sécurisé Fenêtre TWAP >= 30 Minutes (LOGIC-CRITICAL / MEV Attack)';
    patchInstructions = `1. Bannir toute fenêtre TWAP inférieure à 1800 secondes (30 minutes).
2. Ajouter une contrainte stricte require(twapWindow >= 1800) dans les constructeurs et setters pour neutraliser la rentabilité des attaques MEV multi-blocs.`;
    patchCode = `// 🛡️ Pattern Sécurisé TWAP AMM :
uint32 public constant MIN_TWAP_WINDOW = 1800; // 30 minutes minimum

function setTwapWindow(uint32 _newWindow) external onlyOwner {
    require(_newWindow >= MIN_TWAP_WINDOW, "Fenêtre TWAP trop courte / manipulable");
    twapWindow = _newWindow;
}`;
  }

  const prompt = `# 🛡️ PATCH CHIRURGICAL DÉFI & EVC — ${patchTitle}

> **Cible :** \`${filePath}:${line}\`
> **Règle :** \`${ruleId}\`

### Directives d'Application du Correctif :
${patchInstructions}

### Code Modèle & Pattern Certifié :
\`\`\`solidity
${patchCode}
\`\`\`

---
<aether_handshake version="2.0">
  <security_target file="${filePath}" line="${line}" rule="${ruleId}">
    <action>Apply certified zero-loss patch</action>
  </security_target>
</aether_handshake>
`;

  navigator.clipboard.writeText(prompt).then(() => {
    if (window.AudioSys && AudioSys.promptCopied) AudioSys.promptCopied();
    showToast(`🛡️ Patch ${ruleId} copié dans le presse-papier !`);
  });
};

