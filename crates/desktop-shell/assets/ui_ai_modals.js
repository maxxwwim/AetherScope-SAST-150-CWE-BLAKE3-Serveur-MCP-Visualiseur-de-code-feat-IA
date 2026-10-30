// =========================================================================
// AETHERSCOPE V2 — AI, MODALS, CHAT & COCKPIT CONTROLLER (ui_ai_modals.js)
// Search Omnibox, AI Context Packer, Copilot Chat, Telemetry, Project Hub,
// MCP Bridge, Settings, Shortcuts & Lifecycle Initialization
// =========================================================================

// --- Search Omnibox & Beacon Illumination System ---
const searchModal = document.getElementById('search-modal');
const omniInput = document.getElementById('omni-search-input');
const searchList = document.getElementById('search-results-list');
let searchSelectedIndex = 0;

// Topbar Search Cockpit Elements
const topbarSearchInput = document.getElementById('topbar-search-input');
const topbarSearchDropdown = document.getElementById('topbar-search-dropdown');
const topbarSearchResults = document.getElementById('topbar-search-results');
const topbarSearchCount = document.getElementById('topbar-search-count');
const topbarSearchClear = document.getElementById('btn-topbar-search-clear');
let topbarSelectedIndex = 0;

const safeEscape = (str) => {
  if (typeof escapeHtml === 'function') return escapeHtml(String(str || ''));
  return String(str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
};

// Core query engine shared by Topbar and Modal (Full Deep Search across functions, methods, calls, fields, docs, violations)
function querySearchNodes(q, cat) {
  q = (q || '').toLowerCase().trim();
  cat = cat || 'all';
  const nodes = state.data?.nodes || [];
  let scoredResults = [];

  // 1. Folders search
  if (cat === 'all' || cat === 'folder') {
    const uniqueFolders = new Set();
    nodes.forEach(n => {
      if (n.folder && n.folder !== 'root') uniqueFolders.add(n.folder);
    });
    Array.from(uniqueFolders).sort().forEach(f => {
      const fLower = f.toLowerCase();
      if (q.length === 0 || fLower.includes(q)) {
        let score = 25;
        if (fLower === q) score = 100;
        else if (fLower.startsWith(q)) score = 80;
        scoredResults.push({
          score,
          item: {
            is_folder: true,
            name: f,
            file_path: '📁 Dossier Architectural',
            stereotype: '«Dossier»'
          }
        });
      }
    });
  }

  // 2. Nodes Deep Search
  if (cat !== 'folder') {
    nodes.forEach(n => {
      if (cat === 'form' && !(n.stereotype === '«Form»' || n.stereotype === '«Endpoint»' || n.is_form || (n.id && n.id.startsWith('api::')))) return;
      if (cat === 'class' && !n.is_class) return;
      if (cat === 'func' && n.symbol_type !== 'function') return;
      if (cat === 'ui' && n.layer !== 'ui') return;
      if (cat === 'doc' && n.layer !== 'docs' && !(n.file_path && n.file_path.endsWith('.md'))) return;
      if (cat === 'file' && !n.is_file) return;
      if (cat === 'violation' && !n.has_security_violation && !(n.metadata?.violations && n.metadata.violations.length > 0)) return;

      if (q.length === 0) {
        let defaultScore = 10;
        if (n.has_security_violation) defaultScore = 95;
        else if (n.is_class) defaultScore = 30;
        else if (n.is_file) defaultScore = 20;
        scoredResults.push({ score: defaultScore, item: n });
        return;
      }

      const nameLower = (n.name || '').toLowerCase();
      const pathLower = (n.file_path || '').toLowerCase();
      const folderLower = (n.folder || '').toLowerCase();
      const idLower = (n.id || '').toLowerCase();
      const stereoLower = (n.stereotype || '').toLowerCase();
      const docLower = (n.docstring || '').toLowerCase();
      const contentLower = (n.metadata?.raw_content || '').toLowerCase();

      // Check methods & method calls
      let methodMatchName = null;
      if (n.metadata?.methods) {
        for (const m of n.metadata.methods) {
          const mName = (m.name || '').toLowerCase();
          if (mName.includes(q)) {
            methodMatchName = m.name;
            break;
          }
          if (m.calls && m.calls.some(c => (c || '').toLowerCase().includes(q))) {
            methodMatchName = `${m.name} ➔ appel(${q})`;
            break;
          }
        }
      }

      // Check struct/class attributes
      let attrMatchName = null;
      if (!methodMatchName && n.metadata?.attributes) {
        for (const a of n.metadata.attributes) {
          if ((a.name || '').toLowerCase().includes(q)) {
            attrMatchName = a.name;
            break;
          }
        }
      }

      // Check security violations
      let violationMatch = false;
      if (n.metadata?.violations) {
        violationMatch = n.metadata.violations.some(v => (v || '').toLowerCase().includes(q));
      }

      let score = 0;
      if (nameLower === q) score = 100;
      else if (nameLower.startsWith(q)) score = 90;
      else if (nameLower.includes(q)) score = 75;
      else if (methodMatchName) score = 65;
      else if (attrMatchName) score = 60;
      else if (pathLower.includes(q)) score = 50;
      else if (idLower.includes(q)) score = 45;
      else if (folderLower.includes(q)) score = 40;
      else if (stereoLower.includes(q)) score = 35;
      else if (violationMatch || (n.has_security_violation && q.includes('violat'))) score = 70;
      else if (docLower.includes(q)) score = 25;
      else if (contentLower.includes(q)) score = 15;

      if (score > 0) {
        let displayItem = n;
        if (methodMatchName) {
          displayItem = Object.assign({}, n, {
            stereotype: `⚙️ ${methodMatchName}`
          });
        } else if (attrMatchName) {
          displayItem = Object.assign({}, n, {
            stereotype: `🏷️ Champ: ${attrMatchName}`
          });
        }
        scoredResults.push({ score, item: displayItem });
      }
    });
  }

  scoredResults.sort((a, b) => b.score - a.score);
  return scoredResults.slice(0, 60).map(s => s.item);
}
window.querySearchNodes = querySearchNodes;

// --- Topbar Search Dropdown Controls ---
function openTopbarSearchDropdown() {
  if (topbarSearchDropdown) {
    topbarSearchDropdown.classList.add('open');
    topbarSearchDropdown.style.display = 'flex';
  }
}
window.openTopbarSearchDropdown = openTopbarSearchDropdown;

function closeTopbarSearchDropdown() {
  if (topbarSearchDropdown) {
    topbarSearchDropdown.classList.remove('open');
    topbarSearchDropdown.style.display = 'none';
  }
}
window.closeTopbarSearchDropdown = closeTopbarSearchDropdown;

function focusTopbarSearch() {
  if (topbarSearchInput) {
    topbarSearchInput.focus();
    topbarSearchInput.select();
    openTopbarSearchDropdown();
    updateTopbarSearchResults();
  } else {
    openSearchModal();
  }
}
window.focusTopbarSearch = focusTopbarSearch;

function updateTopbarSearchResults() {
  if (!topbarSearchResults) return;
  const q = topbarSearchInput ? topbarSearchInput.value : '';
  const cat = state.searchCategory || 'all';
  const results = querySearchNodes(q, cat);

  if (topbarSearchCount) {
    topbarSearchCount.innerText = `${results.length} résultat${results.length > 1 ? 's' : ''}`;
  }

  topbarSearchResults.innerHTML = '';
  if (results.length === 0) {
    topbarSearchResults.innerHTML = '<div style="padding:16px;text-align:center;color:#64748b;font-style:italic;font-size:12px;">Aucun symbole, fonction, classe ou dossier correspondant.</div>';
    return;
  }

  results.forEach((itemData, idx) => {
    const item = document.createElement('div');
    item.className = `search-result-item ${idx === 0 ? 'selected' : ''}`;
    let icon = '⚡';
    if (itemData.is_folder) icon = '📁';
    else if (itemData.has_security_violation) icon = '🚨';
    else if (itemData.is_file) icon = '📄';
    else if (itemData.is_class) icon = '🏛️';
    else if (itemData.layer === 'docs') icon = '📖';
    else if (itemData.layer === 'ui') icon = '🎨';

    const pathText = itemData.file_path || (itemData.folder ? `📁 ${itemData.folder}` : '');
    const stereoText = itemData.has_security_violation ? '🚨 Alerte SAST' : (itemData.stereotype || (itemData.is_folder ? '«Dossier»' : (itemData.layer ? itemData.layer.toUpperCase() : '')));

    item.innerHTML = `
      <div class="search-res-info">
        <div class="search-res-name">${icon} ${safeEscape(itemData.name)}</div>
        <div class="search-res-path">${safeEscape(pathText)}${itemData.folder && !itemData.is_folder ? ' • 📁 ' + safeEscape(itemData.folder) : ''}</div>
      </div>
      <div class="search-res-tag" style="${itemData.has_security_violation ? 'color:#ef4444;font-weight:800;' : ''}">${safeEscape(stereoText)}</div>
    `;

    item.addEventListener('click', (e) => {
      e.stopPropagation();
      closeTopbarSearchDropdown();
      if (itemData.is_folder) {
        if (typeof selectFolder === 'function') selectFolder(itemData.name);
      } else {
        activateSearchTarget(itemData);
      }
    });

    topbarSearchResults.appendChild(item);
  });

  topbarSelectedIndex = 0;
}

topbarSearchInput?.addEventListener('input', () => {
  const hasVal = topbarSearchInput.value.trim().length > 0;
  if (topbarSearchClear) {
    topbarSearchClear.style.display = hasVal ? 'block' : 'none';
    topbarSearchClear.classList.toggle('visible', hasVal);
  }
  openTopbarSearchDropdown();
  updateTopbarSearchResults();
});

topbarSearchInput?.addEventListener('focus', () => {
  openTopbarSearchDropdown();
  updateTopbarSearchResults();
});

topbarSearchInput?.addEventListener('click', (e) => {
  e.stopPropagation();
  openTopbarSearchDropdown();
  updateTopbarSearchResults();
});

document.getElementById('btn-expand-search-modal')?.addEventListener('click', (e) => {
  e.stopPropagation();
  const currentVal = topbarSearchInput ? topbarSearchInput.value : '';
  closeTopbarSearchDropdown();
  openSearchModal();
  if (omniInput) {
    omniInput.value = currentVal;
    updateSearchResults();
  }
});

topbarSearchClear?.addEventListener('click', (e) => {
  e.stopPropagation();
  if (topbarSearchInput) {
    topbarSearchInput.value = '';
    topbarSearchClear.style.display = 'none';
    topbarSearchClear.classList.remove('visible');
    updateTopbarSearchResults();
    topbarSearchInput.focus();
  }
});

// Topbar Category Pills
document.querySelectorAll('#topbar-search-cats .search-cat-pill').forEach(btn => {
  btn.addEventListener('click', (e) => {
    e.stopPropagation();
    document.querySelectorAll('#topbar-search-cats .search-cat-pill').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    state.searchCategory = btn.getAttribute('data-cat') || 'all';

    // Synchronize modal category pills
    document.querySelectorAll('#search-modal .cat-pill').forEach(b => {
      b.classList.toggle('active', b.getAttribute('data-cat') === state.searchCategory);
    });

    openTopbarSearchDropdown();
    updateTopbarSearchResults();
    if (topbarSearchInput) topbarSearchInput.focus();
  });
});

topbarSearchInput?.addEventListener('keydown', (e) => {
  if (!topbarSearchResults) return;
  const items = topbarSearchResults.querySelectorAll('.search-result-item');
  if (items.length === 0) {
    if (e.key === 'Escape') {
      closeTopbarSearchDropdown();
      topbarSearchInput.blur();
    }
    return;
  }

  if (e.key === 'ArrowDown') {
    e.preventDefault();
    items[topbarSelectedIndex]?.classList.remove('selected');
    topbarSelectedIndex = (topbarSelectedIndex + 1) % items.length;
    items[topbarSelectedIndex]?.classList.add('selected');
    items[topbarSelectedIndex]?.scrollIntoView({ block: 'nearest' });
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    items[topbarSelectedIndex]?.classList.remove('selected');
    topbarSelectedIndex = (topbarSelectedIndex - 1 + items.length) % items.length;
    items[topbarSelectedIndex]?.classList.add('selected');
    items[topbarSelectedIndex]?.scrollIntoView({ block: 'nearest' });
  } else if (e.key === 'Enter') {
    e.preventDefault();
    const selectedEl = items[topbarSelectedIndex];
    if (selectedEl) selectedEl.click();
  } else if (e.key === 'Escape') {
    e.preventDefault();
    closeTopbarSearchDropdown();
    topbarSearchInput.blur();
  }
});

// Click outside topbar search cockpit closes dropdown
document.addEventListener('click', (e) => {
  const cockpit = document.getElementById('topbar-search-cockpit') || document.getElementById('topbar-search-box');
  if (cockpit && !cockpit.contains(e.target)) {
    closeTopbarSearchDropdown();
  }
});

// --- Modal Search Controls (Legacy & Secondary Access) ---
function openSearchModal() {
  if (searchModal) {
    searchModal.classList.add('open');
  }
  if (omniInput) {
    omniInput.value = '';
    omniInput.focus();
  }
  updateSearchResults();
}
window.openSearchModal = openSearchModal;

function closeSearchModal() {
  if (searchModal) {
    searchModal.classList.remove('open');
  }
}
window.closeSearchModal = closeSearchModal;

document.getElementById('btn-trigger-search')?.addEventListener('click', focusTopbarSearch);
document.getElementById('btn-close-search-modal')?.addEventListener('click', closeSearchModal);

document.querySelectorAll('#search-modal .cat-pill').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('#search-modal .cat-pill').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    state.searchCategory = btn.getAttribute('data-cat') || 'all';

    // Synchronize topbar category pills
    document.querySelectorAll('#topbar-search-cats .search-cat-pill').forEach(b => {
      b.classList.toggle('active', b.getAttribute('data-cat') === state.searchCategory);
    });

    updateSearchResults();
  });
});

omniInput?.addEventListener('input', updateSearchResults);

function updateSearchResults() {
  if (!omniInput || !searchList) return;
  const q = omniInput.value;
  const cat = state.searchCategory || 'all';
  const results = querySearchNodes(q, cat);

  const countPill = document.getElementById('search-count-pill');
  if (countPill) countPill.innerText = `${results.length} résultat${results.length > 1 ? 's' : ''}`;

  searchList.innerHTML = '';
  if (results.length === 0) {
    searchList.innerHTML = '<div style="padding:16px;text-align:center;color:#64748b;font-style:italic;">Aucun symbole ou dossier correspondant trouvé.</div>';
    return;
  }

  results.forEach((itemData, idx) => {
    const item = document.createElement('div');
    item.className = `search-result-item ${idx === 0 ? 'selected' : ''}`;
    item.innerHTML = `
      <div class="search-res-info">
        <div class="search-res-name">${itemData.is_folder ? '📁 ' + safeEscape(itemData.name) : safeEscape(itemData.name)}</div>
        <div class="search-res-path">${safeEscape(itemData.file_path)}${itemData.folder ? ' • 📁 ' + safeEscape(itemData.folder) : ''}</div>
      </div>
      <div class="search-res-tag">${safeEscape(itemData.stereotype || '')}</div>
    `;

    item.addEventListener('click', () => {
      if (itemData.is_folder) {
        closeSearchModal();
        if (typeof selectFolder === 'function') selectFolder(itemData.name);
      } else {
        activateSearchTarget(itemData);
      }
    });

    searchList.appendChild(item);
  });

  searchSelectedIndex = 0;
}

omniInput?.addEventListener('keydown', (e) => {
  if (!searchList) return;
  const items = searchList.querySelectorAll('.search-result-item');
  if (items.length === 0) return;

  if (e.key === 'ArrowDown') {
    e.preventDefault();
    items[searchSelectedIndex]?.classList.remove('selected');
    searchSelectedIndex = (searchSelectedIndex + 1) % items.length;
    items[searchSelectedIndex]?.classList.add('selected');
    items[searchSelectedIndex]?.scrollIntoView({ block: 'nearest' });
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    items[searchSelectedIndex]?.classList.remove('selected');
    searchSelectedIndex = (searchSelectedIndex - 1 + items.length) % items.length;
    items[searchSelectedIndex]?.classList.add('selected');
    items[searchSelectedIndex]?.scrollIntoView({ block: 'nearest' });
  } else if (e.key === 'Enter') {
    e.preventDefault();
    const selectedEl = items[searchSelectedIndex];
    if (selectedEl) selectedEl.click();
  }
});

// Illuminate Node with Shockwave Beacon & Fly-to Redirection
function activateSearchTarget(n) {
  closeSearchModal();
  closeTopbarSearchDropdown();
  if (topbarSearchInput) topbarSearchInput.blur();

  // 1. Set pulsating beacon
  state.beaconNode = n;
  state.beaconStart = performance.now();
  if (window.AudioSys?.beaconPulse) AudioSys.beaconPulse();

  // 2. Camera cinematic fly-to
  state.camera.targetX = n.x;
  state.camera.targetY = n.y;
  state.camera.targetZoom = 1.15;

  // 3. Select node
  if (typeof selectNode === 'function') selectNode(n);
  showToast(`🎯 Cible illuminée : ${n.name}`);
}
window.activateSearchTarget = activateSearchTarget;

// --- AI Context Packer & Prompt Generator ---
const TASK_TEMPLATES = {
  refactor: {
    title: "⚡ Refactorisation & Performance",
    prompt: "Analyser la structure ci-dessous, identifier les goulots d'étranglement, la dette technique et proposer une refactorisation modulaire à haute performance sans rompre les contrats d'interface existants."
  },
  security: {
    title: "🛡️ Audit de Sécurité & Robustesse",
    prompt: "Effectuer un audit de sécurité approfondi (injections, validation des entrées, gestion des erreurs, fuites de mémoire) sur ce composant et ses interactions avec ses dépendances."
  },
  tests: {
    title: "🧪 Génération de Tests Unitaires",
    prompt: "Générer une suite complète de tests unitaires et d'intégration (cas nominaux, cas limites, mocking des dépendances identifiées ci-dessous) avec couverture maximale."
  },
  feature: {
    title: "✨ Nouvelle Fonctionnalité / Extension",
    prompt: "Étendre ce composant pour lui ajouter une nouvelle fonctionnalité tout en respectant scrupuleusement les contrats d'interface, le typage strict et les conventions d'architecture."
  },
  doc: {
    title: "📖 Spécifications & Documentation Technique",
    prompt: "Rédiger la documentation technique complète de ce symbole : rôle architectural, flux de données entrants et sortants, entrées/sorties et guide d'intégration."
  }
};

function generateAiPrompt(n, taskKey) {
  const currentPort = window.location.port || '3030';
  const apiOrigin = window.location.origin || `http://127.0.0.1:${currentPort}`;

  if (state.selectedFolder && !state.selectedNode) {
    const folder = state.selectedFolder;
    const nodes = (state.data.nodes || []).filter(node => (node.folder || 'root') === folder);
    const files = nodes.filter(node => node.is_file);
    const classes = nodes.filter(node => node.is_class);
    const funcs = nodes.filter(node => node.symbol_type === 'function');
    const t = TASK_TEMPLATES[taskKey] || TASK_TEMPLATES.refactor;

    return `<context>
  <project name="${state.data.project_name || 'Project'}"/>
  <target_folder name="${folder}"/>
  <metrics total_files="${files.length}" total_classes="${classes.length}" total_functions="${funcs.length}"/>
  
  <contained_files>
${files.map(f => `    - ${f.name} (${f.file_path}) [${(f.metadata && f.metadata.lines) || 0} LOC]`).join('\n')}
  </contained_files>
</context>

<task objective="${t.title} sur le module ${folder}">
${t.prompt}
Analyser la cohésion interne du module, le couplage avec les autres parties du projet et proposer les axes d'amélioration.
</task>

<aetherscope_api_and_research>
[RÉCUPÉRATION D'INFORMATIONS & RECHERCHE AVEC L'API AETHERSCOPE V2]
1. Documentation & Contrats Modulaires : Consulter impérativement AETHER_AI_RULES.md et aether.lock.toml à la racine, ainsi que le README.md de ce dossier (${folder}) pour connaître les spécifications, contrats BLAKE3/EARS et flux.
2. Télémétrie Spatiale Temps Réel (OBLIGATOIRE) : Pour chaque symbole, fichier ou module inspecté ou modifié, émettre la télémétrie HTTP locale :
   POST ${apiOrigin}/api/ai/telemetry
   Body JSON: {"target": "${folder}", "action": "${t.title}", "details": "Inspection / modification du module ${folder}"}
   (Téléporte instantanément la caméra WebGL et allume la balise quantique en direct dans AetherScope).
3. Exploration du Graphe : Consulter l'API locale AetherScope sur ${apiOrigin} si nécessaire (POST /api/analyze).
</aetherscope_api_and_research>

<response_format_rules>
[CONSIGNES DE RÉPONSE OPTIMISÉE - PETIT PROMPT UPDATE OPTI]
- Concision & Zéro Blabla : Réponse technique, dense et directe. Aucune politesse superflue.
- Diffs & Updates Chirurgicaux : Ne JAMAIS réécrire un fichier complet si seules quelques lignes changent. Fournir uniquement les extraits ciblés à remplacer ou diffs précis.
- Règle d'Atomicité (< 35 lignes) : Découper le code en fonctions atomiques pures de moins de 35 lignes (règle stricte utils/ et domain/).
- Respect Strict des Couches & Contrats : Préserver la séparation (domain/ = logique pure sans UI/DB, application/ = services, infrastructure/ = I/O & API, ui/ = rendu). Ne jamais casser les signatures existantes.
- Réflexion Structurée : Penser à haute voix de manière ordonnée (Diagnostic ➔ Impact ➔ Modification chirurgicale).
</response_format_rules>`;
  }

  if (!n) n = state.data.nodes && state.data.nodes[0] ? state.data.nodes[0] : { name: 'Codebase', file_path: '', layer: 'general', stereotype: '«System»' };
  const edges = state.data.edges || [];
  const incoming = edges.filter(e => e.target === n.id).map(e => e.source.split('::').pop().replace('file::', ''));
  const outgoing = edges.filter(e => e.source === n.id).map(e => `${e.target.split('::').pop().replace('file::', '')} (${e.relation})`);

  const t = TASK_TEMPLATES[taskKey] || TASK_TEMPLATES.refactor;
  const isFormNode = n.stereotype === '«Form»' || n.is_form;
  const formNetRoutes = outgoing.filter(e => e.includes('submits') || e.includes('network'));

  return `<context>
  <project name="${state.data.project_name || 'Project'}"/>
  <target_symbol name="${n.name}" type="${n.stereotype}" path="${n.file_path}"/>
  <architectural_layer name="${(n.layer || 'general').toUpperCase()}"/>
  <metrics lines="${(n.metadata && n.metadata.lines) || 1}" in_degree="${n.in_degree || 0}" out_degree="${n.out_degree || 0}"/>
${isFormNode ? `  <form_spec is_form="true">
    <network_targets>
${formNetRoutes.length > 0 ? formNetRoutes.map(s => `      - ${s}`).join('\n') : '      (Aucun appel réseau direct détecté)'}
    </network_targets>
  </form_spec>\n` : ''}  
  <incoming_callers>
${incoming.length > 0 ? incoming.map(s => `    - ${s}`).join('\n') : '    (Point d\'entrée / Aucun appelant interne direct)'}
  </incoming_callers>

  <outgoing_dependencies>
${outgoing.length > 0 ? outgoing.map(s => `    - ${s}`).join('\n') : '    (Feuille / Aucune dépendance externe directe)'}
  </outgoing_dependencies>
</context>

<task objective="${t.title}">
${t.prompt}
</task>

<aetherscope_api_and_research>
[RÉCUPÉRATION D'INFORMATIONS & RECHERCHE AVEC L'API AETHERSCOPE V2]
1. Documentation & Contrats Modulaires : Consulter impérativement AETHER_AI_RULES.md et aether.lock.toml à la racine, ainsi que le README.md du dossier de ce symbole (${n.file_path}) pour valider les contrats d'interface BLAKE3/EARS.
2. Télémétrie Spatiale Temps Réel (OBLIGATOIRE) : Pour chaque symbole ou fichier inspecté / modifié, émettre la télémétrie HTTP locale :
   POST ${apiOrigin}/api/ai/telemetry
   Body JSON: {"target": "${n.name}", "action": "${t.title}", "details": "Inspection / modification de ${n.name} (${n.file_path})" }
   (Téléporte instantanément la caméra WebGL et allume la balise quantique en direct dans AetherScope).
3. Exploration du Graphe : Consulter l'API locale AetherScope sur ${apiOrigin} si nécessaire (POST /api/analyze).
</aetherscope_api_and_research>

<response_format_rules>
[CONSIGNES DE RÉPONSE OPTIMISÉE - PETIT PROMPT UPDATE OPTI]
- Concision & Zéro Blabla : Réponse technique, dense et directe. Aucune politesse superflue.
- Diffs & Updates Chirurgicaux : Ne JAMAIS réécrire un fichier complet si seules quelques lignes changent. Fournir uniquement les extraits ciblés à remplacer ou diffs précis.
- Règle d'Atomicité (< 35 lignes) : Découper le code en fonctions atomiques pures de moins de 35 lignes (règle stricte utils/ et domain/).
- Respect Strict des Couches & Contrats : Couche ${(n.layer || 'domain').toUpperCase()} respectée (domain/ = règles pures sans UI/DB, application/ = services, infrastructure/ = I/O & API, ui/ = rendu). Préserver strictement les signatures et contrats utilisés par les appelants.
- Réflexion Structurée : Penser à haute voix de manière ordonnée (Diagnostic ➔ Impact ➔ Modification chirurgicale).
</response_format_rules>`;
}
window.generateAiPrompt = generateAiPrompt;

function openAiModal() {
  if (state.selectedFolder && !state.selectedNode) {
    const targetEl = document.getElementById('ai-modal-target');
    if (targetEl) targetEl.innerText = '📁 ' + state.selectedFolder;
    updateModalPrompt();
    document.getElementById('ai-modal')?.classList.add('open');
    return;
  }
  const n = state.selectedNode || (state.data.nodes && state.data.nodes[0]);
  if (!n) {
    showToast('Sélectionnez un symbole ou un dossier pour générer son prompt.');
    return;
  }
  const targetEl = document.getElementById('ai-modal-target');
  if (targetEl) targetEl.innerText = n.name;
  updateModalPrompt();
  document.getElementById('ai-modal')?.classList.add('open');
}
window.openAiModal = openAiModal;

function updateModalPrompt() {
  const n = state.selectedNode || (state.data.nodes && state.data.nodes[0]);
  let p = generateAiPrompt(n, state.currentTask);
  const customInput = document.getElementById('ai-custom-input')?.value?.trim();
  if (customInput) {
    p += `\n\n<custom_user_instructions>\n${customInput}\n</custom_user_instructions>`;
  }
  const preview = document.getElementById('ai-prompt-preview');
  if (preview) preview.value = p;
  const tokens = Math.round(p.length / 3.8);
  const countEl = document.getElementById('ai-token-count');
  if (countEl) countEl.innerText = `~${tokens} tokens context pack`;
}

document.getElementById('ai-custom-input')?.addEventListener('input', updateModalPrompt);

document.querySelectorAll('.task-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.task-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    state.currentTask = btn.getAttribute('data-task') || 'refactor';
    updateModalPrompt();
  });
});

document.getElementById('btn-insp-ai')?.addEventListener('click', openAiModal);
document.getElementById('btn-ai-prompt')?.addEventListener('click', openAiModal);
document.getElementById('btn-close-ai-modal')?.addEventListener('click', () => document.getElementById('ai-modal')?.classList.remove('open'));
document.getElementById('btn-dismiss-ai-modal')?.addEventListener('click', () => document.getElementById('ai-modal')?.classList.remove('open'));

document.getElementById('btn-copy-ai-prompt')?.addEventListener('click', () => {
  const txt = document.getElementById('ai-prompt-preview')?.value || '';
  navigator.clipboard.writeText(txt).then(() => {
    showToast('📋 Prompt IA copié dans le presse-papier !');
    document.getElementById('ai-modal')?.classList.remove('open');
  });
});

// --- Master Blueprint IA Project Generator (Gemini / Claude / GPT) ---
let currentMasterLlm = 'claude';

function generateMasterBlueprintPrompt(llm = 'claude') {
  const projName = state.data.project_name || 'NouveauProjet';
  const isClaude = llm === 'claude';
  const isGemini = llm === 'gemini';

  return `# 🚀 MASTER BLUEPRINT : ARCHITECTURE & MÉTHODE DE DÉVELOPPEMENT OPTIMISÉE POUR IA
# Compatible : ${isClaude ? 'Claude 3.5 Sonnet (Recommandé)' : (isGemini ? 'Google Gemini 1.5 Pro / Flash' : 'OpenAI GPT-4o / o1')}
# Visualiseur & Cartographie : AetherScope V2 (Rust WebGL Engine)

Tu es l'Architecte Logiciel Principal et Expert Technique sur le projet "${projName}".
Nous développons ce projet selon la **MÉTHODE PAR SPÉCIFICATION MODULAIRE (Spec-Driven Architecture)**, conçue pour permettre aux LLMs de coder à pleine vitesse avec un contexte ciblé sans jamais avoir à analyser tout le projet à la fois.

---

### 🌟 1. PRINCIPE CARDINAL : DÉVELOPPEMENT PAR FICHIER .MD (ZÉRO CONTEXT LOSS)
- **Chaque dossier doit contenir son propre \`README.md\`** (ex: \`src/domain/README.md\`, \`src/ui/README.md\`).
- Ce \`README.md\` documente :
  1. Le rôle architectural précis du dossier
  2. Les entrées / sorties attendues et contrats d'interfaces
  3. La liste des fichiers et fonctions hébergés
  4. Les flux réseau ou dépendances autorisées
- **Sceau d'intégrité BLAKE3 :** Tout contrat est scellé dans \`aether.lock.toml\` pour garantir l'absence de régression.

---

### 📁 2. ARBORESCENCE & DÉCOUPAGE STRICT DES DOSSIERS
Organise systématiquement les fichiers selon la structure modulaire suivante :

\`\`\`text
${projName}/
├── aether.lock.toml          # 🔒 Verrou BLAKE3 de conformité architecturale
├── AETHER_AI_RULES.md        # 📖 Spécifications globales & directives
├── contracts/                # 📜 Schémas d'interface et spécifications formelles EARS
├── client/                   # 🎨 Interface utilisateur, composants & visualisations
├── server/                   # ⚡ Moteur backend, logique métier & API Axum/Actix
└── data-nodes/               # 💾 Persistance, modèles de données & connecteurs
\`\`\`

---

### ⚡ 3. RÈGLES SUR LES FICHIERS ET LES FONCTIONS (POUR UN GRAPH AETHERSCOPE OPTIMAL)
Dans notre visualiseur AetherScope, chaque fichier est une planète céleste et chaque fonction est un point lumineux :

1. **Fonctions Atomiques & Pures :**
   - Écris des fonctions courtes (10 à 35 lignes maximum), pures et réutilisables.
   - Une fonction = une seule responsabilité.
2. **Conventions de Préfixes :**
   - \`handle_*\` : Fonctions d'événements et gestionnaires d'actions.
   - \`fetch_*\` ou \`api_*\` : Appels réseau sortants (faisceaux lasers).
   - \`render_*\` : Logique de rendu UI ou d'affichage.
   - \`validate_*\` : Vérification des formulaires et payloads.
3. **5 Filtres d'Affiliation AetherScope V2 :**
   - Bit 1 (1) : Structure Physique (contains)
   - Bit 2 (2) : Réseau RPC (submits)
   - Bit 3 (4) : Canaux WS Temps Réel (RealtimeWS - lasers Cyan)
   - Bit 4 (8) : Sémantique (imports/calls)
   - Bit 5 (16) : Violations de Sécurité (SECURITY_VIOLATES - lasers Rouges clignotants)
`;
}
window.generateMasterBlueprintPrompt = generateMasterBlueprintPrompt;

function openMasterPromptModal() {
  const modal = document.getElementById('master-prompt-modal');
  const input = document.getElementById('init-project-input');
  const currPath = state.data.project_path || document.getElementById('project-path-input')?.value.trim();
  if (input && !input.value) {
    input.value = currPath ? currPath + '_NouveauProjet' : 'C:\\Users\\1234\\Desktop\\Projets\\NouveauProjet';
  }
  updateMasterPrompt();
  modal?.classList.add('open');
}
window.openMasterPromptModal = openMasterPromptModal;

function updateMasterPrompt() {
  const area = document.getElementById('master-prompt-textarea');
  if (area) {
    area.value = generateMasterBlueprintPrompt(currentMasterLlm);
  }
}

document.getElementById('btn-master-prompt')?.addEventListener('click', openMasterPromptModal);
document.getElementById('btn-close-master-prompt-modal')?.addEventListener('click', () => {
  document.getElementById('master-prompt-modal')?.classList.remove('open');
});
document.getElementById('btn-dismiss-master-modal')?.addEventListener('click', () => {
  document.getElementById('master-prompt-modal')?.classList.remove('open');
});

document.querySelectorAll('#llm-target-selector .task-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('#llm-target-selector .task-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    currentMasterLlm = btn.getAttribute('data-llm') || 'claude';
    updateMasterPrompt();
  });
});

document.getElementById('btn-copy-master-prompt')?.addEventListener('click', () => {
  const txt = document.getElementById('master-prompt-textarea')?.value || '';
  navigator.clipboard.writeText(txt).then(() => {
    showToast('📋 Master Blueprint Prompt copié !');
  });
});

document.getElementById('btn-download-master-prompt')?.addEventListener('click', () => {
  const txt = document.getElementById('master-prompt-textarea')?.value || '';
  const blob = new Blob([txt], { type: 'text/markdown;charset=utf-8' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `AetherScope_Master_Blueprint_${currentMasterLlm}.md`;
  a.click();
  showToast(`💾 Fichier ${a.download} téléchargé !`);
});

document.getElementById('btn-init-skeleton')?.addEventListener('click', async () => {
  const p = document.getElementById('init-project-input')?.value.trim();
  if (!p) {
    showToast('Veuillez entrer un chemin de dossier.');
    return;
  }
  showToast(`🔨 Initialisation du projet dans ${p}...`);
  try {
    const d = await window.AetherAPI.initProject(p, 'modular');
    if (d.status === 'ok') {
      showToast(`✅ ${d.message} !`);
      const pInput = document.getElementById('project-path-input');
      if (pInput) pInput.value = p;
      setTimeout(() => {
        analyze(p);
        document.getElementById('master-prompt-modal')?.classList.remove('open');
      }, 600);
    } else {
      showToast(`❌ Erreur: ${d.message || 'initialisation projet'}`);
    }
  } catch (e) {
    showToast('❌ Erreur réseau / API');
  }
});

// --- Draggable AI Copilot Chat & History Engine ---
window.copyUserPromptForAntigravity = function(target, userText) {
  const projPath = state.data?.project_path || '';
  const prompt = `Dans le visualiseur AetherScope (${projPath}) :
${target ? `[Cible: ${target}] ` : ''}${userText}

Vérifie l'historique et aligne la caméra si nécessaire.`;

  navigator.clipboard.writeText(prompt).then(() => {
    if (window.AudioSys?.playTone) AudioSys.playTone(880, 'sine', 0.12, 0.04);
    showToast('⚡ Prêt ! Collez le message dans Antigravity (Ctrl+V)');
  }).catch(() => {
    showToast('⚠️ Impossible de copier dans le presse-papier');
  });
};

function safeEscapeHtml(str) {
  if (str == null) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function formatTime(ts) {
  if (!ts) return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  const ms = ts < 10000000000 ? ts * 1000 : ts;
  const d = new Date(ms);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function addAiChatMessage(telem, node) {
  if (!telem || !telem.target) return;
  const msgObj = {
    sender: 'ai',
    target: telem.target,
    action: telem.action || 'Navigation',
    details: telem.details || 'Déplacement sur la bulle stellaire',
    timestamp: telem.timestamp || Date.now(),
    nodeId: node ? node.id : null,
    nodeX: node ? node.x : null,
    nodeY: node ? node.y : null
  };

  state.aiChatHistory.push(msgObj);
  if (state.aiChatHistory.length > 150) {
    state.aiChatHistory.shift();
  }

  renderAiChatMessageEl(msgObj, node);
}

function renderAiChatMessageEl(msgObj, node) {
  const body = document.getElementById('ai-chat-body');
  const countPill = document.getElementById('ai-chat-count-pill');
  if (countPill) countPill.innerText = `${state.aiChatHistory.length} msg`;

  if (body) {
    const placeholder = body.querySelector('.ai-chat-placeholder');
    if (placeholder) {
      placeholder.remove();
    }

    const isUser = msgObj.sender === 'user';
    const msgEl = document.createElement('div');
    msgEl.className = `ai-chat-msg ${isUser ? 'user' : 'ai'}`;
    const rawText = msgObj.details || msgObj.text || '';
    let thoughtContent = '';
    let cleanText = rawText;

    // Parse <thought>...</thought> or <thinking>...</thinking>
    const thoughtMatch = rawText.match(/<(?:thought|thinking)>([\s\S]*?)<\/(?:thought|thinking)>/i);
    if (thoughtMatch) {
      thoughtContent = thoughtMatch[1].trim();
      cleanText = rawText.replace(/<(?:thought|thinking)>[\s\S]*?<\/(?:thought|thinking)>/i, '').trim();
    }

    let thoughtHtml = '';
    if (thoughtContent) {
      thoughtHtml = `
        <div class="ai-chat-thought-box">
          <div class="ai-thought-header" onclick="this.parentElement.classList.toggle('collapsed')">
            <span class="ai-thought-title">🧠 Réflexion & Analyse interne <span class="ai-typing-pulse"></span></span>
            <span style="font-size:0.60rem; color:#c084fc;">Afficher/Masquer ▾</span>
          </div>
          <div class="ai-thought-body">${safeEscapeHtml(thoughtContent)}</div>
        </div>
      `;
    }

    msgEl.innerHTML = `
      <div class="ai-chat-msg-meta">
        <div style="display:flex; align-items:center; gap:5px;">
          <span class="ai-chat-msg-target">${isUser ? '👤 Vous' : `📍 ${safeEscapeHtml(msgObj.target || 'AI Copilot')}`}</span>
          <span class="ai-chat-msg-action">${isUser ? 'Message' : safeEscapeHtml(msgObj.action || 'Réponse')}</span>
        </div>
        <div style="display:flex; align-items:center; gap:6px;">
          ${isUser ? `<button class="ai-chat-push-btn" title="Copier la question avec le contexte pour Antigravity" onclick="event.stopPropagation(); copyUserPromptForAntigravity('${safeEscapeHtml(msgObj.target || '')}', '${safeEscapeHtml(cleanText).replace(/'/g, "\\'")}')">⚡ Antigravity</button>` : ''}
          <span class="ai-chat-msg-time">${formatTime(msgObj.timestamp)}</span>
        </div>
      </div>
      ${thoughtHtml}
      <div class="ai-chat-msg-content">${safeEscapeHtml(cleanText)}</div>
    `;

    if (!isUser) {
      msgEl.addEventListener('click', () => {
        if (msgObj.nodeX != null && msgObj.nodeY != null) {
          state.camera.targetX = msgObj.nodeX;
          state.camera.targetY = msgObj.nodeY;
          state.camera.targetZoom = 1.05;
          if (node && typeof selectNode === 'function') selectNode(node);
          if (window.AudioSys?.playTone) AudioSys.playTone(880, 'sine', 0.15, 0.05);
          showToast(`🎯 Caméra alignée sur : ${msgObj.target}`);
        } else if (msgObj.target) {
          const tLower = msgObj.target.toLowerCase();
          const n = (state.data.nodes || []).find(x => 
            x.name.toLowerCase() === tLower || 
            x.id.toLowerCase() === tLower || 
            (x.file_path && x.file_path.toLowerCase().includes(tLower))
          );
          if (n) {
            state.camera.targetX = n.x;
            state.camera.targetY = n.y;
            state.camera.targetZoom = 1.05;
            if (typeof selectNode === 'function') selectNode(n);
            if (window.AudioSys?.playTone) AudioSys.playTone(880, 'sine', 0.15, 0.05);
            showToast(`🎯 Caméra alignée sur : ${n.name}`);
          }
        }
      });
    }

    body.appendChild(msgEl);
    if (state.aiChatAutoScroll) {
      body.scrollTop = body.scrollHeight;
    }
  }
}

async function sendUserChatMessage() {
  const input = document.getElementById('ai-chat-user-input');
  if (!input) return;
  const text = input.value.trim();
  if (!text) return;

  input.value = '';
  const targetSymbol = state.selectedNode ? state.selectedNode.name : (state.selectedFolder || null);
  updateAiStatus('thinking');

  const now = Date.now();
  const optimisticMsg = {
    id: `temp_${now}`,
    sender: 'user',
    target: targetSymbol || 'Copilot',
    action: 'Question',
    details: text,
    timestamp: now
  };
  state.aiChatHistory.push(optimisticMsg);
  renderAiChatMessageEl(optimisticMsg, null);

  try {
    const d = await window.AetherAPI.sendChatMessage(text, 'user', targetSymbol);
    if (d && d.status === 'ok') {
      if (window.AudioSys?.playTone) AudioSys.playTone(740, 'sine', 0.12, 0.04);
      showToast('⚡ Message transmis au Copilot IA !');
      if (d.message && d.message.id) {
        optimisticMsg.id = d.message.id;
      }
      pollServerChatMessages();
    } else {
      showToast('⚠️ Erreur envoi au serveur');
    }
  } catch (err) {
    console.error('sendUserChatMessage error:', err);
    showToast('⚠️ Erreur transmission message (serveur déconnecté ?)');
  }
}
window.sendUserChatMessage = sendUserChatMessage;

function toggleAiChatPopup() {
  const popup = document.getElementById('ai-chat-popup');
  const btn = document.getElementById('btn-ai-chat-toggle');
  if (!popup) return;
  const isHidden = popup.classList.toggle('hidden');
  if (!isHidden) {
    popup.classList.remove('minimized');
    if (btn) {
      btn.style.boxShadow = '0 0 16px rgba(192, 132, 252, 0.8)';
      btn.style.background = 'rgba(192, 132, 252, 0.35)';
    }
    const input = document.getElementById('ai-chat-user-input');
    if (input) setTimeout(() => input.focus(), 60);
  } else {
    if (btn) {
      btn.style.boxShadow = '';
      btn.style.background = 'rgba(168, 85, 247, 0.25)';
    }
  }
}
window.toggleAiChatPopup = toggleAiChatPopup;

// Make AI Chat popup Draggable
(function initDraggableChat() {
  const popup = document.getElementById('ai-chat-popup');
  const header = document.getElementById('ai-chat-header');
  if (!popup || !header) return;

  let isDragging = false;
  let startX = 0, startY = 0;
  let initialLeft = 0, initialTop = 0;

  header.addEventListener('mousedown', (e) => {
    if (e.target.closest('.ai-chat-header-btn') || e.target.closest('button')) return;
    isDragging = true;
    const rect = popup.getBoundingClientRect();
    startX = e.clientX;
    startY = e.clientY;
    initialLeft = rect.left;
    initialTop = rect.top;
    popup.style.bottom = 'auto';
    popup.style.right = 'auto';
    popup.style.left = `${initialLeft}px`;
    popup.style.top = `${initialTop}px`;
    e.preventDefault();
  });

  window.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    const newLeft = Math.max(10, Math.min(window.innerWidth - popup.offsetWidth - 10, initialLeft + dx));
    const newTop = Math.max(10, Math.min(window.innerHeight - 50, initialTop + dy));
    popup.style.left = `${newLeft}px`;
    popup.style.top = `${newTop}px`;
  });

  window.addEventListener('mouseup', () => {
    isDragging = false;
  });
})();

// Wire up AI Chat UI Toggles and Input Handlers
document.getElementById('btn-menu-ai-chat')?.addEventListener('click', toggleAiChatPopup);
document.getElementById('btn-ai-chat-toggle')?.addEventListener('click', toggleAiChatPopup);
document.getElementById('btn-ai-hud-open-chat')?.addEventListener('click', () => {
  const popup = document.getElementById('ai-chat-popup');
  if (popup && popup.classList.contains('hidden')) {
    toggleAiChatPopup();
  }
});
document.getElementById('btn-ai-chat-close')?.addEventListener('click', () => {
  const popup = document.getElementById('ai-chat-popup');
  if (popup && !popup.classList.contains('hidden')) {
    toggleAiChatPopup();
  }
});
document.getElementById('btn-ai-chat-min')?.addEventListener('click', () => {
  document.getElementById('ai-chat-popup')?.classList.toggle('minimized');
});
document.getElementById('btn-ai-chat-send')?.addEventListener('click', sendUserChatMessage);
document.getElementById('ai-chat-user-input')?.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    sendUserChatMessage();
  }
});

// --- Mode AI Copilot Telemetry Engine ---
function applyAiTelemetry(telem) {
  if (!telem || !telem.target) return;
  state.aiTelemetry = telem;
  state.aiLastTimestamp = telem.timestamp;

  // Update HUD banner
  const hud = document.getElementById('ai-telemetry-hud');
  const btn = document.getElementById('btn-ai-telemetry');
  if (hud) {
    hud.classList.remove('hidden');
    if (btn) btn.classList.add('active');
    const targetEl = document.getElementById('ai-target-badge');
    if (targetEl) targetEl.innerText = telem.target;
    const actEl = document.getElementById('ai-action-badge');
    if (actEl) actEl.innerText = telem.action || 'Exploration';
    const detEl = document.getElementById('ai-details-text');
    if (detEl) detEl.innerText = telem.details || '';
  }

  // Find node
  const nodes = state.data.nodes || [];
  const tLower = telem.target.toLowerCase();
  const node = nodes.find(n => 
    n.name.toLowerCase() === tLower || 
    n.id.toLowerCase() === tLower || 
    (n.file_path && n.file_path.toLowerCase().includes(tLower))
  );
  state.aiTelemetryNode = node || null;
  state.instanceDataDirty = true;

  // Add to Draggable Chat Popup History
  addAiChatMessage(telem, node);

  if (node) {
    if (typeof addToAiTrail === 'function') addToAiTrail(node, telem.action);

    const isCreation = (telem.action && (
      telem.action.toLowerCase().includes('créat') ||
      telem.action.toLowerCase().includes('creat') ||
      telem.action.toLowerCase().includes('naiss')
    ));

    if (isCreation) {
      state.aiSupernova = {
        x: node.x,
        y: node.y,
        name: node.name,
        startTime: performance.now(),
        duration: 3800
      };
      if (window.AudioSys?.supernovaBirth) AudioSys.supernovaBirth();
    } else {
      if (window.AudioSys?.playTone) {
        AudioSys.playTone(980, 'triangle', 0.20, 0.05);
        setTimeout(() => AudioSys.playTone(1470, 'sine', 0.25, 0.04), 70);
      }
    }

    if (state.aiAutoCam) {
      if (typeof smoothFlyTo === 'function') {
        smoothFlyTo(node.x, node.y, 0.95);
      } else {
        state.camera.targetX = node.x;
        state.camera.targetY = node.y;
        if (state.camera.targetZoom < 0.75) state.camera.targetZoom = 0.95;
      }
    }

    showToast(`🤖 L'IA explore : ${node.name} (${telem.action || 'Analyse'})`);
  }
}
window.applyAiTelemetry = applyAiTelemetry;

async function pollAiTelemetry() {
  try {
    const data = await window.AetherAPI.getTelemetry();
    if (data && data.telemetry && data.telemetry.timestamp !== state.aiLastTimestamp) {
      applyAiTelemetry(data.telemetry);
    }
  } catch (e) {}
}
setInterval(pollAiTelemetry, 400);

// --- Live Auto-Sync with Rust FileWatcher Bridge ---
let lastGraphVersion = 0;
let syncFailures = 0;
async function pollGraphVersion() {
  try {
    const res = await fetch('/api/graph/version');
    if (!res.ok) return;
    const d = await res.json();
    const sseDot = document.getElementById('sse-dot');
    if (sseDot) {
      sseDot.style.background = '#10b981';
      sseDot.style.boxShadow = '0 0 10px #10b981';
      sseDot.title = 'FileWatcher actif : synchronisation du code en temps réel';
    }
    if (lastGraphVersion === 0) {
      lastGraphVersion = d.version;
      return;
    }
    if (d.version !== lastGraphVersion) {
      lastGraphVersion = d.version;
      const gRes = await fetch('/api/graph');
      if (gRes.ok) {
        const newGraph = await gRes.json();
        applyDataKeepView(newGraph);
        showToast('⚡ Codebase actualisée en direct (FileWatcher)');
      }
    }
    syncFailures = 0;
  } catch (e) {
    syncFailures++;
    if (syncFailures > 4) {
      const sseDot = document.getElementById('sse-dot');
      if (sseDot) {
        sseDot.style.background = '#f59e0b';
        sseDot.style.boxShadow = 'none';
        sseDot.title = 'FileWatcher en pause';
      }
    }
  }
}
setInterval(pollGraphVersion, 800);

document.getElementById('btn-ai-autocam')?.addEventListener('click', () => {
  state.aiAutoCam = !state.aiAutoCam;
  const b = document.getElementById('btn-ai-autocam');
  if (b) b.innerText = state.aiAutoCam ? '🎥 Auto-Cam: ON' : '🎥 Auto-Cam: OFF';
  showToast(state.aiAutoCam ? 'Centrage automatique sur l\'IA activé' : 'Centrage automatique sur l\'IA désactivé');
});

function resetAiTrail() {
  state.aiTrail = [];
  if (window.AudioSys?.playTone) AudioSys.playTone(520, 'sine', 0.12, 0.04);
  showToast('🧹 Chemin IA effacé du graphe');
}
window.resetAiTrail = resetAiTrail;

document.getElementById('btn-ai-reset-trail')?.addEventListener('click', resetAiTrail);
document.getElementById('btn-ai-popup-reset-trail')?.addEventListener('click', resetAiTrail);

// --- Live AI Agent State & Full-Auto Copilot Bridge ---
function updateAiStatus(status, convId) {
  const badge = document.getElementById('ai-chat-status-badge');
  const statusText = document.getElementById('ai-chat-status-text');
  const btnCancelTop = document.getElementById('btn-ai-cancel-top');
  const btnCancelFooter = document.getElementById('btn-ai-cancel-footer');
  const sendLabel = document.getElementById('ai-chat-send-label');
  const sendIcon = document.getElementById('ai-chat-send-icon');
  const convPill = document.getElementById('ai-conv-id-pill');
  const body = document.getElementById('ai-chat-body');

  const st = (status || 'idle').toLowerCase();

  if (convPill) {
    if (convId) {
      convPill.innerText = `id:${convId.slice(0, 8)}...`;
      convPill.title = `Session Antigravity active : ${convId}`;
      convPill.style.display = 'inline-block';
    } else {
      convPill.style.display = 'none';
    }
  }

  if (st === 'thinking') {
    if (badge) {
      badge.className = 'ai-status-badge thinking';
      badge.innerHTML = '🧠 Réflexion...';
    }
    if (statusText) statusText.innerHTML = '🧠 Gemini réfléchit...';
    if (btnCancelTop) btnCancelTop.style.display = 'inline-flex';
    if (btnCancelFooter) btnCancelFooter.style.display = 'inline-flex';
    if (sendLabel) sendLabel.innerText = 'Réflexion...';
    if (sendIcon) sendIcon.innerText = '🧠';

    if (body && !document.getElementById('ai-thinking-live-bubble')) {
      const bubble = document.createElement('div');
      bubble.id = 'ai-thinking-live-bubble';
      bubble.className = 'ai-chat-msg ai ai-thinking-card';
      bubble.innerHTML = `
        <div class="ai-chat-msg-meta">
          <div style="display:flex; align-items:center; gap:5px;">
            <span class="ai-chat-msg-target">🤖 Antigravity Copilot</span>
            <span class="ai-chat-msg-action" style="color:#c084fc; background:rgba(192,132,252,0.2); border-color:rgba(192,132,252,0.4);">Réflexion...</span>
          </div>
          <span class="ai-chat-msg-time">En cours</span>
        </div>
        <div style="display:flex; align-items:center; justify-content:space-between; gap:10px; padding:4px 0;">
          <div style="display:flex; align-items:center; gap:9px;">
            <div class="ai-spinner-orbit"></div>
            <div>
              <div style="color:#f5d0fe; font-weight:700; font-size:0.75rem;">Gemini réfléchit...</div>
              <div style="color:#a5b4fc; font-size:0.65rem;">Analyse du code et élaboration de la réponse</div>
            </div>
          </div>
          <button class="ai-cancel-btn-pulse" onclick="cancelAiGeneration()" title="Interrompre la génération">🛑 Annuler</button>
        </div>
      `;
      body.appendChild(bubble);
      if (state.aiChatAutoScroll) body.scrollTop = body.scrollHeight;
    }
  } else {
    const bubble = document.getElementById('ai-thinking-live-bubble');
    if (bubble) bubble.remove();

    if (btnCancelTop) btnCancelTop.style.display = 'none';
    if (btnCancelFooter) btnCancelFooter.style.display = 'none';
    if (sendLabel) sendLabel.innerText = 'Envoyer';
    if (sendIcon) sendIcon.innerText = '⚡';

    if (st === 'completed') {
      if (badge) {
        badge.className = 'ai-status-badge completed';
        badge.innerHTML = '✅ Prêt';
      }
      if (statusText) statusText.innerHTML = '✅ Réponse reçue';
    } else if (st === 'cancelled') {
      if (badge) {
        badge.className = 'ai-status-badge cancelled';
        badge.innerHTML = '⚠️ Annulé';
      }
      if (statusText) statusText.innerHTML = '⚠️ Requête annulée';
    } else if (st === 'miss') {
      if (badge) {
        badge.className = 'ai-status-badge miss';
        badge.innerHTML = '❌ Échec';
      }
      if (statusText) statusText.innerHTML = '❌ Timeout / Erreur Antigravity';
    } else {
      if (badge) {
        badge.className = 'ai-status-badge idle';
        badge.innerHTML = '🟢 Prêt';
      }
      if (statusText) statusText.innerHTML = '🟢 En écoute active';
    }
  }
}
window.updateAiStatus = updateAiStatus;

window.cancelAiGeneration = async function() {
  try {
    updateAiStatus('cancelled');
    if (window.AudioSys?.playTone) AudioSys.playTone(440, 'triangle', 0.12, 0.04);
    showToast('🛑 Requête Antigravity annulée');
    await window.AetherAPI.cancelAi();
    pollServerChatMessages();
  } catch (e) {
    console.error(e);
  }
};

let lastChatSyncCount = 0;
async function pollServerChatMessages() {
  try {
    const d = await window.AetherAPI.fetchChatMessages();
    if (d) {
      if (d.agent_status !== undefined) {
        updateAiStatus(d.agent_status, d.conversation_id);
      }
      if (d.messages && Array.isArray(d.messages)) {
        d.messages.forEach(m => {
          const mTime = m.timestamp ? (m.timestamp < 10000000000 ? m.timestamp * 1000 : m.timestamp) : Date.now();
          const exists = state.aiChatHistory.some(x => {
            if (m.id && x.id === m.id) return true;
            const xTime = x.timestamp ? (x.timestamp < 10000000000 ? x.timestamp * 1000 : x.timestamp) : 0;
            return x.details === m.text && x.sender === m.sender && Math.abs(xTime - mTime) < 3000;
          });
          if (!exists) {
            const obj = {
              id: m.id,
              sender: m.sender || 'ai',
              target: m.target_symbol || 'Copilot',
              action: m.sender === 'user' ? 'Question' : 'Réponse IA',
              details: m.text,
              timestamp: mTime
            };
            state.aiChatHistory.push(obj);
            renderAiChatMessageEl(obj, null);
          }
        });
        lastChatSyncCount = d.messages.length;
      }
    }
  } catch (e) {}
}
setInterval(pollServerChatMessages, 500);

document.getElementById('btn-ai-cancel-top')?.addEventListener('click', cancelAiGeneration);
document.getElementById('btn-ai-cancel-footer')?.addEventListener('click', cancelAiGeneration);

document.getElementById('btn-ai-new-session')?.addEventListener('click', async () => {
  try {
    await window.AetherAPI.newAiSession();
    if (window.AudioSys?.playTone) AudioSys.playTone(660, 'sine', 0.15, 0.05);
    showToast('✨ Nouvelle session Antigravity démarrée !');
    state.aiChatHistory = [];
    const body = document.getElementById('ai-chat-body');
    if (body) {
      body.innerHTML = '<div class="ai-chat-placeholder" style="text-align:center; padding:24px 12px; color:#64748b; font-size:0.75rem; font-family:var(--font-mono);">Nouvelle session démarrée. En attente de messages...</div>';
    }
    const countPill = document.getElementById('ai-chat-count-pill');
    if (countPill) countPill.innerText = '0 msg';
    lastChatSyncCount = 0;
    updateAiStatus('idle', null);
    pollServerChatMessages();
  } catch (e) {
    showToast('⚠️ Erreur création nouvelle session');
  }
});

document.getElementById('btn-ai-chat-clear')?.addEventListener('click', async () => {
  state.aiChatHistory = [];
  const body = document.getElementById('ai-chat-body');
  if (body) {
    body.innerHTML = '<div class="ai-chat-placeholder" style="text-align:center; padding:24px 12px; color:#64748b; font-size:0.75rem; font-family:var(--font-mono);">Historique effacé. En attente de nouveaux messages...</div>';
  }
  const countPill = document.getElementById('ai-chat-count-pill');
  if (countPill) countPill.innerText = '0 msg';
  lastChatSyncCount = 0;
  updateAiStatus('idle', null);
  window.AetherAPI.clearChatMessages().catch(() => {});
  showToast('🗑️ Historique chat IA réinitialisé');
});

document.getElementById('btn-ai-chat-autoscroll')?.addEventListener('click', () => {
  state.aiChatAutoScroll = !state.aiChatAutoScroll;
  const b = document.getElementById('btn-ai-chat-autoscroll');
  if (b) b.innerText = state.aiChatAutoScroll ? 'Défilement Auto: OUI' : 'Défilement Auto: NON';
});

document.getElementById('btn-ai-flyto')?.addEventListener('click', () => {
  if (state.aiTelemetry && state.aiTelemetry.target) {
    const tLower = state.aiTelemetry.target.toLowerCase();
    const node = (state.data.nodes || []).find(n => 
      n.name.toLowerCase() === tLower || 
      n.id.toLowerCase() === tLower || 
      (n.file_path && n.file_path.toLowerCase().includes(tLower))
    );
    if (node) {
      state.camera.targetX = node.x;
      state.camera.targetY = node.y;
      state.camera.targetZoom = 1.05;
      showToast(`🎯 Caméra alignée sur l'IA : ${node.name}`);
    }
  }
});

document.getElementById('btn-close-ai-hud')?.addEventListener('click', () => {
  document.getElementById('ai-telemetry-hud')?.classList.add('hidden');
});

document.getElementById('btn-ai-telemetry')?.addEventListener('click', () => {
  const hud = document.getElementById('ai-telemetry-hud');
  if (hud) hud.classList.toggle('hidden');
});

// Multi-Layer Synthesis Pills Builder
function renderLayerPills() {
  const box = document.getElementById('synth-pills');
  if (!box) return;
  box.innerHTML = '';
  const layers = state.data.stats && state.data.stats.layers ? state.data.stats.layers : {};
  const layerHex = {
    ui: '#ec4899', action: '#38bdf8', domain: '#f59e0b',
    database: '#818cf8', config: '#94a3b8', utility: '#c084fc',
    docs: '#10b981'
  };

  Object.entries(layers).forEach(([layer, count]) => {
    const pill = document.createElement('div');
    const active = state.activeLayers.has(layer);
    const col = layerHex[layer] || '#38bdf8';
    pill.className = `synth-pill ${active ? '' : 'disabled'}`;
    pill.style.borderColor = col;
    pill.style.color = col;
    pill.style.background = active ? `${col}22` : 'transparent';
    pill.innerText = `${layer.toUpperCase()} (${count})`;

    pill.addEventListener('click', () => {
      if (state.activeLayers.has(layer)) {
        state.activeLayers.delete(layer);
        AudioSys.playTone(320, 'sine', 0.08, 0.02);
      } else {
        state.activeLayers.add(layer);
        AudioSys.playTone(550, 'sine', 0.08, 0.02);
      }
      state.instanceDataDirty = true;
      if (typeof invalidateEdgePathCache === 'function') invalidateEdgePathCache();
      wakePhysics(60);
      renderLayerPills();
    });
    box.appendChild(pill);
  });
}
window.renderLayerPills = renderLayerPills;

function setAllLayerFilters(enableAll) {
  const allLayers = ['ui', 'action', 'domain', 'database', 'config', 'utility', 'docs'];
  if (enableAll) {
    allLayers.forEach(l => state.activeLayers.add(l));
  } else {
    state.activeLayers.clear();
  }
  state.instanceDataDirty = true;
  if (typeof invalidateEdgePathCache === 'function') invalidateEdgePathCache();
  wakePhysics(60);
  renderLayerPills();
  AudioSys.playTone(enableAll ? 650 : 350, 'sine', 0.1, 0.03);
  showToast(enableAll ? '👁️ Toutes les couches affichées' : '🚫 Toutes les couches masquées');
}
window.setAllLayerFilters = setAllLayerFilters;

function getUniqueModules() {
  const nodes = state.data?.nodes || [];
  const map = new Map();
  nodes.forEach(n => {
    const f = n.folder || 'root';
    map.set(f, (map.get(f) || 0) + 1);
  });
  return map;
}
window.getUniqueModules = getUniqueModules;

function renderModulePills() {
  const box = document.getElementById('synth-module-pills');
  if (!box) return;
  box.innerHTML = '';
  if (!state.hiddenModules) state.hiddenModules = new Set();

  const moduleMap = getUniqueModules();
  if (moduleMap.size === 0) {
    box.innerHTML = '<span style="font-size:0.65rem; color:#64748b; padding:4px;">Aucun module détecté</span>';
    return;
  }

  const sorted = Array.from(moduleMap.entries()).sort((a, b) => a[0].localeCompare(b[0]));
  const badge = document.getElementById('synth-badge-modules');
  if (badge) badge.innerText = sorted.length;

  sorted.forEach(([folder, count]) => {
    const isHidden = state.hiddenModules.has(folder);
    const colRgba = typeof getFolderColor === 'function' ? getFolderColor(folder) : [0.22, 0.74, 0.97, 1];
    const colHex = `rgb(${Math.round(colRgba[0]*255)}, ${Math.round(colRgba[1]*255)}, ${Math.round(colRgba[2]*255)})`;

    const pill = document.createElement('div');
    pill.className = `synth-module-pill ${isHidden ? 'disabled' : ''}`;
    pill.style.borderColor = isHidden ? 'rgba(100, 116, 139, 0.3)' : colHex;
    pill.style.color = isHidden ? '#64748b' : '#f8fafc';
    pill.style.background = isHidden ? 'rgba(15, 23, 42, 0.45)' : `rgba(${Math.round(colRgba[0]*255)}, ${Math.round(colRgba[1]*255)}, ${Math.round(colRgba[2]*255)}, 0.14)`;

    const shortName = typeof formatFolderLabel === 'function' ? formatFolderLabel(folder) : folder;

    pill.innerHTML = `
      <span class="module-eye-icon">${isHidden ? '🚫' : '👁️'}</span>
      <span class="module-color-dot" style="background:${colHex};"></span>
      <span class="module-name" title="${escapeHtml(folder)}">${escapeHtml(shortName)}</span>
      <span class="module-count">${count}</span>
      <button class="module-doc-btn" title="Consulter la documentation du module ${escapeHtml(shortName)}">📖</button>
    `;

    // Click pill -> Toggle visibility (NEVER open doc)
    pill.addEventListener('click', (e) => {
      if (e.target.closest('.module-doc-btn')) return;

      if (state.hiddenModules.has(folder)) {
        state.hiddenModules.delete(folder);
        AudioSys.playTone(550, 'sine', 0.08, 0.02);
      } else {
        state.hiddenModules.add(folder);
        AudioSys.playTone(320, 'sine', 0.08, 0.02);
      }
      state.instanceDataDirty = true;
      if (typeof invalidateEdgePathCache === 'function') invalidateEdgePathCache();
      wakePhysics(60);
      renderModulePills();
    });

    // Click doc button -> Explicitly open documentation only
    const docBtn = pill.querySelector('.module-doc-btn');
    if (docBtn) {
      docBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        if (typeof openFolderDocReader === 'function') {
          const mdList = (state.folderMdMap && state.folderMdMap.get(folder)) || [];
          if (mdList.length > 0) {
            openFolderDocReader(folder, mdList, 0, true);
          } else {
            const folderNodes = (state.data.nodes || []).filter(n => (n.folder || 'root') === folder);
            openEmptyFolderDoc(folder, folderNodes, true);
          }
        }
      });
    }

    box.appendChild(pill);
  });
}
window.renderModulePills = renderModulePills;

function setAllModuleFilters(enableAll) {
  if (!state.hiddenModules) state.hiddenModules = new Set();
  const moduleMap = getUniqueModules();
  if (enableAll) {
    state.hiddenModules.clear();
  } else {
    moduleMap.forEach((_, f) => state.hiddenModules.add(f));
  }
  state.instanceDataDirty = true;
  if (typeof invalidateEdgePathCache === 'function') invalidateEdgePathCache();
  wakePhysics(60);
  renderModulePills();
  AudioSys.playTone(enableAll ? 650 : 350, 'sine', 0.1, 0.03);
  showToast(enableAll ? '👁️ Tous les modules affichés' : '🚫 Tous les modules masqués');
}
window.setAllModuleFilters = setAllModuleFilters;

function toggleSynthAccordion(catId) {
  const item = document.getElementById(`synth-acc-${catId}`);
  if (!item) return;
  const isCollapsed = item.classList.toggle('collapsed');
  try {
    localStorage.setItem(`aether_synth_acc_${catId}`, isCollapsed ? 'collapsed' : 'open');
  } catch (e) {}
  AudioSys.playTone(isCollapsed ? 340 : 480, 'sine', 0.08, 0.02);
}
window.toggleSynthAccordion = toggleSynthAccordion;

function initSynthAccordions() {
  ['modules', 'layers', 'pipeline', 'links', 'cve'].forEach(catId => {
    try {
      const saved = localStorage.getItem(`aether_synth_acc_${catId}`);
      if (saved === 'collapsed') {
        document.getElementById(`synth-acc-${catId}`)?.classList.add('collapsed');
      }
    } catch (e) {}
  });
}
window.initSynthAccordions = initSynthAccordions;
setTimeout(initSynthAccordions, 100);

// Server API & HUD Controls
async function loadProjects() {
  try {
    const d = await window.AetherAPI.fetchProjects();
    const sel = document.getElementById('project-select');
    if (sel && d.projects) {
      sel.innerHTML = '<option value="">-- Choisir un projet (' + d.projects.length + ') --</option>';
      d.projects.forEach(p => {
        const opt = document.createElement('option');
        opt.value = p.path;
        opt.innerText = p.name;
        sel.appendChild(opt);
      });
      const projToLoad = d.default_project || (d.projects && d.projects[0] && d.projects[0].path);
      if (projToLoad) {
        sel.value = projToLoad;
        const inp = document.getElementById('project-path-input');
        if (inp) inp.value = projToLoad;
        if (!state.data || !state.data.nodes || state.data.nodes.length === 0) {
          analyze(projToLoad);
        }
      }
    }
  } catch (e) {
    console.warn('Erreur chargement projets', e);
  }
}
window.loadProjects = loadProjects;

let loadingTimer = null;
function showLoadingScreen(title, stepText, folderPath) {
  const hud = document.getElementById('hydro-loading-hud');
  const titleEl = document.getElementById('hydro-project-title');
  const stepEl = document.getElementById('hydro-step-text');
  if (titleEl && title) titleEl.innerText = title;
  if (stepEl && (folderPath || stepText)) {
    stepEl.innerText = folderPath ? `Émergence de ${folderPath}...` : stepText;
  }
  if (hud) hud.classList.remove('hidden');

  const steps = [
    'Émergence aquatique des clusters de code...',
    'Scan AST multithread & linter SAST...',
    'Vérification des sceaux BLAKE3 et contrats EARS...',
    'Simulation des forces quantiques d\'affiliation...',
    'Projection spatiale WebGL 2 (120 FPS instancié)...'
  ];
  let stepIdx = 0;
  if (loadingTimer) clearInterval(loadingTimer);
  loadingTimer = setInterval(() => {
    stepIdx = (stepIdx + 1) % steps.length;
    if (stepEl) stepEl.innerText = steps[stepIdx];
  }, 700);
}

function hideLoadingScreen() {
  if (loadingTimer) {
    clearInterval(loadingTimer);
    loadingTimer = null;
  }
  const hud = document.getElementById('hydro-loading-hud');
  if (hud) {
    setTimeout(() => {
      hud.classList.add('hidden');
    }, 750);
  }
}

async function analyze(path) {
  if (!path) return;
  const projectName = path.split(/[\\/]/).filter(Boolean).pop() || path;
  showLoadingScreen(projectName, 'Démarrage du scan AST...', path);
  showToast('⚡ Analyse Rust V2 ultra-rapide...');
  try {
    const data = await window.AetherAPI.analyze(path);
    applyData(data);
    hideLoadingScreen();
    showToast(`✨ ${data.nodes.length} nœuds, ${data.edges.length} liens analysés en ${data.stats.elapsed_ms}ms !`);
  } catch (e) {
    hideLoadingScreen();
    showToast('❌ Erreur analyse serveur');
  }
}
window.analyze = analyze;

function applyData(data) {
  state.nodeMap = null;
  state.cachedFolderStats = null;
  state.cachedEdgePaths = null;
  state.waterRipples = [];
  if (typeof precomputeDocMetadata === 'function') precomputeDocMetadata(data);
  if (typeof wakePhysics === 'function') wakePhysics(180);

  const now = performance.now();
  const folderMap = new Map();
  (data.nodes || []).forEach(n => {
    const f = n.folder || 'root';
    if (!folderMap.has(f)) folderMap.set(f, []);
    folderMap.get(f).push(n);
  });

  // Staged Rhythmic Emergence Sequence ("1 x 2 x 3..." wave sequence)
  const sortedFolders = Array.from(folderMap.keys()).sort((a, b) => {
    if (a === 'root' || a === 'src' || a === 'core' || a === 'contracts') return -1;
    if (b === 'root' || b === 'src' || b === 'core' || b === 'contracts') return 1;
    const countA = folderMap.get(a).length;
    const countB = folderMap.get(b).length;
    return countB - countA;
  });

  // Batch folders into at most 8 waves so large projects emerge in ~1.0s instead of 15s+
  const MAX_WAVES = 8;
  const foldersPerWave = Math.max(1, Math.ceil(sortedFolders.length / MAX_WAVES));
  const WAVES = [];
  for (let i = 0; i < sortedFolders.length; i += foldersPerWave) {
    WAVES.push(sortedFolders.slice(i, i + foldersPerWave));
  }
  const totalWaves = Math.max(1, WAVES.length);
  const waveIntervalMs = Math.min(180, Math.max(70, Math.floor(900 / totalWaves)));

  state.bubbleBirthMap = new Map();
  const baseTime = now + 30;
  let maxSpawnTime = baseTime;
  let globalFolderIdx = 0;

  WAVES.forEach((waveFolders, wIdx) => {
    const folderBirth = baseTime + (wIdx * waveIntervalMs);
    waveFolders.forEach(folder => {
      const fNodes = folderMap.get(folder) || [];
      state.bubbleBirthMap.set(folder, {
        birthTime: folderBirth,
        poppedSound: false,
        phase: (globalFolderIdx * 1.7),
        folderIndex: globalFolderIdx,
        totalFolders: sortedFolders.length,
        name: folder,
        count: fNodes.length,
        wave: wIdx + 1
      });
      globalFolderIdx++;

      const files = fNodes.filter(n => n.is_file);
      const classes = fNodes.filter(n => !n.is_file && n.is_class);
      const others = fNodes.filter(n => !n.is_file && !n.is_class);

      files.forEach(n => {
        n.birth_time = folderBirth;
      });

      classes.forEach((n, ci) => {
        n.birth_time = folderBirth + Math.min(ci * 5, 50);
        if (n.birth_time > maxSpawnTime) maxSpawnTime = n.birth_time;
      });

      others.forEach((n, oi) => {
        n.birth_time = folderBirth + 12 + Math.min(oi * 3, 80);
        if (n.birth_time > maxSpawnTime) maxSpawnTime = n.birth_time;
      });
    });
  });

  const isLargeProject = (data.nodes || []).length > 450;
  state.spawnAnimationEndTime = maxSpawnTime + (isLargeProject ? 260 : 380);
  state.physicsCooldown = isLargeProject ? 110 : Math.min(200, Math.max(140, totalWaves * 18 + 60));
  state.physicsSleeping = false;
  state.instanceDataDirty = true;

  if (window.AudioSys && typeof AudioSys.waterRipple === 'function') {
    AudioSys.waterRipple(0.35);
  }

  // Realtime Animated HUD Banner
  const hud = document.getElementById('hydro-loading-hud');
  const hudTitle = document.getElementById('hydro-project-title');
  const hudStep = document.getElementById('hydro-step-text');

  if (hud) {
    hud.classList.remove('hidden');
    if (hudTitle) hudTitle.innerText = `✨ Émergence Céleste : Vague 1 / ${totalWaves}`;
    if (hudStep) hudStep.innerText = `Déploiement rapide des amas stellaires (${data.nodes.length} symboles)...`;

    WAVES.forEach((waveFolders, wIdx) => {
      setTimeout(() => {
        const folder = waveFolders[0];
        const fNodes = folderMap.get(folder) || [];
        const label = typeof formatFolderLabel === 'function' ? formatFolderLabel(folder) : folder;
        const extra = waveFolders.length > 1 ? ` +${waveFolders.length - 1}` : '';
        if (hudTitle) hudTitle.innerText = `✨ Émergence Céleste : Vague ${wIdx + 1} / ${totalWaves}`;
        if (hudStep) hudStep.innerText = `Naissance de [${label}${extra}] (${fNodes.length} symboles)...`;
        if (window.AudioSys && typeof AudioSys.waterBubblePop === 'function') {
          AudioSys.waterBubblePop(wIdx + 1);
        }
      }, 30 + wIdx * waveIntervalMs);
    });

    setTimeout(() => {
      if (hudTitle) hudTitle.innerText = `✨ Émergence Complète !`;
      if (hudStep) hudStep.innerText = `${data.nodes.length} symboles vivants • ${data.edges.length} flux quantiques`;
      setTimeout(() => {
        hud.classList.add('hidden');
      }, 500);
    }, 30 + totalWaves * waveIntervalMs + 220);
  }

  state.data = data;
  state.project_path = data.project_path || '';
  const topPill = document.getElementById('topbar-project-pill');
  if (topPill) topPill.innerText = data.project_name || 'Projet';
  const synthStats = document.getElementById('synth-stats');
  if (synthStats) synthStats.innerText = `${data.nodes.length} symboles • ${data.edges.length} flux`;
  const burgerBadge = document.getElementById('burger-count-badge');
  if (burgerBadge) burgerBadge.innerText = `${data.edges.length} flux`;
  renderLayerPills();
  if (typeof renderModulePills === 'function') renderModulePills();
  if (typeof renderAffiliationPills === 'function') renderAffiliationPills();
  if (typeof renderEdgeTypologyLegend === 'function') renderEdgeTypologyLegend();
  if (typeof renderCveAnomaliesPanel === 'function') renderCveAnomaliesPanel();
  if (typeof fitView === 'function') fitView();

  try {
    const projKey = (data.project_path || data.project_name || 'default').toLowerCase();
    const seenProjects = JSON.parse(localStorage.getItem('aetherscope_seen_projects') || '{}');
    if (!seenProjects[projKey]) {
      seenProjects[projKey] = Date.now();
      localStorage.setItem('aetherscope_seen_projects', JSON.stringify(seenProjects));
    }
  } catch (e) {
    console.warn('LocalStorage non disponible pour le suivi des projets', e);
  }
}
window.applyData = applyData;

function applyDataKeepView(data, focusSymbol) {
  state.nodeMap = null;
  state.cachedFolderStats = null;
  if (typeof precomputeDocMetadata === 'function') precomputeDocMetadata(data);
  if (typeof wakePhysics === 'function') wakePhysics(110);

  const now = performance.now();
  const folderMap = new Map();
  (data.nodes || []).forEach(n => {
    const f = n.folder || 'root';
    if (!folderMap.has(f)) folderMap.set(f, []);
    folderMap.get(f).push(n);
    if (!n.birth_time) n.birth_time = now;
  });

  const oldBirthMap = state.bubbleBirthMap || new Map();
  state.bubbleBirthMap = new Map();
  let newFolderIdx = 0;

  folderMap.forEach((fNodes, folder) => {
    let birthInfo = oldBirthMap.get(folder);
    if (!birthInfo) {
      birthInfo = {
        birthTime: now,
        poppedSound: true,
        phase: newFolderIdx * 1.37,
        folderIndex: newFolderIdx,
        totalFolders: folderMap.size,
        name: folder,
        count: fNodes.length
      };
      newFolderIdx++;
    } else {
      birthInfo.count = fNodes.length;
    }
    state.bubbleBirthMap.set(folder, birthInfo);
  });

  state.data = data;
  state.project_path = data.project_path || '';
  const topPill = document.getElementById('topbar-project-pill');
  if (topPill) topPill.innerText = data.project_name || 'Projet';
  const synthStats2 = document.getElementById('synth-stats');
  if (synthStats2) synthStats2.innerText = `${data.nodes.length} symboles • ${data.edges.length} flux`;
  const burgerBadge2 = document.getElementById('burger-count-badge');
  if (burgerBadge2) burgerBadge2.innerText = `${data.edges.length} flux`;
  renderLayerPills();
  if (typeof renderModulePills === 'function') renderModulePills();
  if (typeof renderAffiliationPills === 'function') renderAffiliationPills();
  if (typeof renderEdgeTypologyLegend === 'function') renderEdgeTypologyLegend();

  if (focusSymbol) {
    const sLower = focusSymbol.toLowerCase();
    const node = (data.nodes || []).find(n =>
      n.name.toLowerCase() === sLower ||
      n.id.toLowerCase() === sLower ||
      (n.file_path && n.file_path.toLowerCase().includes(sLower))
    );
    if (node) {
      state.camera.targetX = node.x;
      state.camera.targetY = node.y;
      state.camera.targetZoom = 1.15;
      state.selectedNode = node;
      state.aiSupernova = {
        x: node.x,
        y: node.y,
        name: node.name,
        startTime: performance.now(),
        duration: 3800
      };
      if (typeof addToAiTrail === 'function') addToAiTrail(node, 'Création Fichier');
      if (window.AudioSys?.supernovaBirth) AudioSys.supernovaBirth();
      if (typeof selectNode === 'function') selectNode(node);
      return;
    }
  }
}
window.applyDataKeepView = applyDataKeepView;

// Native Windows Dialog Handlers for Workspace & Single Project
document.getElementById('btn-pick-workspace')?.addEventListener('click', async () => {
  showToast('📁 Boîte de dialogue : choisissez le dossier racine de tous vos projets...');
  try {
    const d = await window.AetherAPI.pickWorkspace();
    if (d.status === 'ok' && d.projects) {
      const sel = document.getElementById('project-select');
      if (sel) {
        sel.innerHTML = '<option value="">-- Projets détectés (' + d.projects.length + ') --</option>';
        d.projects.forEach(p => {
          const opt = document.createElement('option');
          opt.value = p.path;
          opt.innerText = p.name;
          sel.appendChild(opt);
        });
      }
      showToast(`✅ Espace de travail : ${d.projects.length} projet(s) découvert(s)`);
      if (d.projects.length > 0) {
        const first = d.projects[0];
        const inp = document.getElementById('project-path-input');
        if (inp) inp.value = first.path;
        analyze(first.path);
      }
    } else if (d.status === 'cancelled') {
      showToast('Sélection annulée');
    }
  } catch (err) {
    showToast('❌ Erreur dialogue workspace');
  }
});

document.getElementById('btn-pick-project')?.addEventListener('click', async () => {
  showToast('🎯 Boîte de dialogue : sélectionnez un projet unique...');
  try {
    const d = await window.AetherAPI.pickProject();
    if (d.status === 'ok' && d.project_path) {
      const inp = document.getElementById('project-path-input');
      if (inp) inp.value = d.project_path;
      if (d.graph) {
        applyData(d.graph);
        showToast(`✅ Projet chargé : ${d.project_path}`);
      } else {
        analyze(d.project_path);
      }
    } else if (d.status === 'cancelled') {
      showToast('Sélection annulée');
    }
  } catch (err) {
    showToast('❌ Erreur dialogue projet');
  }
});

// --- Generic AI System Prompt Modal Handlers ---
let cachedSystemPrompt = '';

function generateFallbackSystemPrompt() {
  const projName = state.data?.project_name || window.__AETHERSCOPE_DATA__?.project_name || 'aether-engine';
  const projPath = state.data?.project_path || window.__AETHERSCOPE_DATA__?.project_path || 'C:\\Users\\1234\\Desktop\\Projets\\aether-engine';
  return `# 🪐 AETHERSCOPE V2 — DIRECTIVES SYSTÈME COPILOTE IA (ARCHITECTURE & PAIR-PROGRAMMING)

> **Projet :** \`${projName}\`  
> **Racine :** \`${projPath}\`  
> **Stack Détectée :** 🦀 Pure Rust (Cargo Workspace Hexagonal)  
> **Architecture Active :** Cargo Workspace : \`crates/domain\` (100% pur), \`crates/server-axum\`, \`crates/client-ui\`, \`crates/db-adapter\`  
> **Serveur Local AetherScope :** \`http://127.0.0.1:3030\`  

Tu opères en binôme direct avec le développeur et le moteur d'intelligence architecturale **AetherScope V2**.
Le graphe 3D WebGL 2 tourne en continu sur le port \`3030\` et audite chaque modification du codebase.

---

## 🚀 1. DIAGNOSTIC INITIAL AU LANCEMENT DE CONVERSATION
Dès ton premier message ou au démarrage d'une session :
1. **Émets un diagnostic de santé d'architecture & d'avancement des tâches :**
   - Appelle \`aetherscope_get_graph_summary\` (ou lis \`.aether/LIVE_AUDIT.md\`) et \`aetherscope_list_tasks\` (ou lis \`.aether/TASKS_SUMMARY.md\`).
   - Annonce en 2-3 lignes chirurgicales l'état du système :
     \`🪐 [AETHER_STATUS : Nœuds=<N>, Flux=<F>, Sécurité=<OK | X violations>, Tâches=<T actives>, Serveur=Connecté]\`
2. **Si l'utilisateur mentionne un module, symbole ou objectif :**
   - Appelle immédiatement \`aetherscope_get_active_target(symbol_id)\` pour extraire le contexte ciblé, les flux catégorisés, les tâches associées et les invariants EARS.
   - Aligne la caméra WebGL via \`aetherscope_teleport_camera\` (ou \`POST /api/ai/telemetry\`).

---

## 🛠️ 2. OUTILLAGE MCP COMPLET (AETHERSCOPE MCP ENGINE)
Utilise systématiquement les outils MCP mis à disposition :
- **\`aetherscope_teleport_camera(target, action, details)\`** :
  Déplace instantanément et avec fluidité la caméra 3D sur la bulle/dossier cible, déclenche le faisceau quantique et éclaire le fil d'Ariane stellaire en direct pour le développeur.
- **\`aetherscope_get_graph_summary(path)\`** :
  Fournit la vue globale : volumes de nœuds, relations, répartition des modules et détection proactive des violations CVE.
- **\`aetherscope_get_active_target(symbol_id)\`** :
  Génère le contexte chirurgical allégé avec les **flux catégorisés** et les tâches en cours sur la cible (UI, HTTP, WebSocket, DB/SQL, Contrats, Violations).
- **\`aetherscope_create_task(title, kind, target_symbol, target_file, priority, initial_readme)\`** :
  Crée une tâche (fonction, recherche, sécurité, refactor) avec son mini-README d'avancement consultable par le modèle et l'utilisateur.
- **\`aetherscope_update_task(task_id, status, progress, progress_note, full_readme)\`** :
  Met à jour le statut (\`pending\`, \`in_progress\`, \`blocked\`, \`done\`), le pourcentage d'avancement et ajoute des notes de progrès ou coche les étapes du README.
- **\`aetherscope_list_tasks(status_filter)\`** :
  Liste toutes les tâches du projet avec leur état, auteur et résumé d'avancement.
- **\`aetherscope_scan_project(path)\`** :
  Déclenche un rescan ultrarapide (< 100ms) pour rafraîchir l'AST et les métriques du graphe 3D.
- **\`aetherscope_send_chat_message(text, thought, target_symbol)\`** :
  Envoie un debrief technique ou compte-rendu directement dans le chat intégré du cockpit AetherScope.
- **\`aetherscope_get_chat_history()\`** :
  Lit l'historique récent des échanges et demandes passées dans le cockpit.

---

## 📋 3. GESTION DES TÂCHES & MINI-README D'AVANCEMENT
1. **Création Systématique :** Dès qu'une modification touche plus d'une fonction, requiert une recherche/audit ou une refactorisation :
   - Crée une tâche via \`aetherscope_create_task\` (ou modifie \`.aether/tasks/<task_id>/README.md\`).
   - Choisis le bon type : \`function\`, \`research\`, \`security_fix\`, \`arch_refactor\`, ou \`other\`.
2. **Suivi d'Avancement :**
   - Utilise le mini-README de la tâche pour documenter le plan, cocher les cases au fur et à mesure et lister les résultats d'investigation.
   - Mets à jour la progression via \`aetherscope_update_task\` au fil de tes actions.

---

## 🔒 4. ZERO-SHELL PROTOCOLE DE GOUVERNANCE
1. **Avant d'écrire ou modifier du code :**
   - Lis \`.aether/ACTIVE_TARGET.md\` (ou réponse de \`aetherscope_get_active_target\`).
   - Lis le \`README.md\` du dossier cible pour connaître les règles EARS et les invariants.
   - Ne modifie JAMAIS un \`README.md\` pour contourner une règle : le hash cryptographique BLAKE3 est scellé dans \`aether.lock.toml\`.
2. **Après chaque sauvegarde (\`Ctrl+S\`) :**
   - Le File-Watcher Rust réanalyse l'AST en < 50ms et régénère \`.aether/LIVE_AUDIT.md\`.
   - Si \`.aether/LIVE_AUDIT.md\` contient \`❌ VIOLATION\`, corrige immédiatement la ligne signalée avant de continuer.

---

## 🏛️ 5. INVARIANTS PAR COUCHE & SÉPARATION STRICTE
- \`contracts/\` : Traits Rust (\`RepositoryPort\`), DTOs partagés Serde et schémas RPC stricts
- \`client/\` : Rendu graphique, formulaires («Form»), état réactif. Ne doit JAMAIS importer \`server/domain\` ni \`data-nodes\`.
- \`server/domain/\` : **100% pur, déterministe, zéro I/O**. Interdiction formelle de requêtes SQL brutes, d'accès réseau ou de variables d'environnement (\`process.env\` / \`std::env::var\`).
- \`server/application/\` : Orchestrations des cas d'usage reliant le domaine aux interfaces de persistance.
- \`server/api/\` : Contrôleurs d'entrée (REST / WebSocket) validant tout payload via \`contracts/\`.
- \`data-nodes/\` : Adaptateurs de persistance (PostgreSQL / SQLite / Redis) isolables en microservices.

---

## ⚡ 6. CONSIGNES DE RÉPONSE OPTIMISÉE
- **Concision & Densité :** Réponse technique, chirurgicale et directe. Zéro bavardage inutile.
- **Diffs Chirurgicaux :** Ne réécris jamais un fichier complet si seules quelques lignes changent. Fournis uniquement les extraits ciblés.
- **Règle d'Atomicité (< 35 lignes) :** Découpe le code en fonctions atomiques pures de moins de 35 lignes maximum (particulièrement dans \`domain/\`).
- **Preuve de Lecture [AETHER_ACK] :** Démarre tes propositions techniques par :
  \`[AETHER_ACK: cible=<symbole>, couche=<layer>, imports_interdits=<liste>]\`;
`;
}

async function openSystemPromptModal(isFirstOpen = false) {
  const modal = document.getElementById('ai-sysprompt-modal');
  const textarea = document.getElementById('ai-sysprompt-textarea');
  const pill = document.getElementById('ai-first-open-pill');
  if (!modal || !textarea) return;
  try {
    const d = await window.AetherAPI.fetchSystemPrompt();
    if (d && d.system_prompt) {
      cachedSystemPrompt = d.system_prompt;
    }
  } catch (e) {
    // offline handled below
  }

  if (!cachedSystemPrompt || cachedSystemPrompt.startsWith('//')) {
    cachedSystemPrompt = state.data?.system_prompt || window.__AETHERSCOPE_DATA__?.system_prompt || generateFallbackSystemPrompt();
  }

  let promptText = cachedSystemPrompt;
  const projPath = state.data?.project_path || window.__AETHERSCOPE_DATA__?.project_path || '';
  if (projPath) {
    promptText = promptText.replaceAll('C:\\\\Chemin\\\\Vers\\\\Le\\\\Projet', projPath)
                           .replaceAll('C:\\Chemin\\Vers\\Le\\Projet', projPath);
  }
  textarea.value = promptText;

  if (pill) {
    pill.style.display = isFirstOpen ? 'block' : 'none';
  }
  modal.classList.add('open');
}
window.openSystemPromptModal = openSystemPromptModal;


document.getElementById('btn-ai-prompt-system')?.addEventListener('click', () => openSystemPromptModal(false));
document.getElementById('btn-close-ai-sysprompt-modal')?.addEventListener('click', () => {
  document.getElementById('ai-sysprompt-modal')?.classList.remove('open');
});
document.getElementById('btn-dismiss-ai-sysprompt-modal')?.addEventListener('click', () => {
  document.getElementById('ai-sysprompt-modal')?.classList.remove('open');
});
document.getElementById('btn-copy-ai-sysprompt')?.addEventListener('click', () => {
  const txt = document.getElementById('ai-sysprompt-textarea')?.value || '';
  navigator.clipboard.writeText(txt).then(() => {
    showToast('📋 Prompt Système Universel copié dans le presse-papier !');
  });
});
document.getElementById('btn-download-ai-sysprompt')?.addEventListener('click', () => {
  const txt = document.getElementById('ai-sysprompt-textarea')?.value || '';
  const blob = new Blob([txt], { type: 'text/markdown;charset=utf-8' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = 'aetherscope_system_prompt.md';
  a.click();
  showToast('💾 aetherscope_system_prompt.md téléchargé !');
});

// --- AI Instant File Creation & Supernova Birth Handlers ---
function openCreateFileModal() {
  const modal = document.getElementById('ai-create-file-modal');
  if (!modal) return;
  const projInput = document.getElementById('create-file-project-path');
  if (projInput) {
    const cur = document.getElementById('project-path-input')?.value.trim() || state.data.project_path || '';
    projInput.value = cur;
  }
  modal.classList.add('open');
}
window.openCreateFileModal = openCreateFileModal;

document.getElementById('btn-ai-newfile')?.addEventListener('click', openCreateFileModal);
document.getElementById('btn-close-create-file-modal')?.addEventListener('click', () => {
  document.getElementById('ai-create-file-modal')?.classList.remove('open');
});
document.getElementById('btn-dismiss-create-file-modal')?.addEventListener('click', () => {
  document.getElementById('ai-create-file-modal')?.classList.remove('open');
});

// Preset suggestion pills
document.getElementById('pill-suggest-ui')?.addEventListener('click', () => {
  const relInput = document.getElementById('create-file-rel-path');
  const codeInput = document.getElementById('create-file-code-content');
  if (relInput) relInput.value = 'src/ui/components/RoomHeroCard.jsx';
  if (codeInput) codeInput.value = `import React from 'react';\n\nexport function RoomHeroCard({ title, price, status }) {\n  return (\n    <div className="room-card">\n      <h3>{title}</h3>\n      <span className="price">{price} ETH</span>\n      <span className="badge">{status}</span>\n    </div>\n  );\n}\n`;
});

document.getElementById('pill-suggest-domain')?.addEventListener('click', () => {
  const relInput = document.getElementById('create-file-rel-path');
  const codeInput = document.getElementById('create-file-code-content');
  if (relInput) relInput.value = 'src/domain/models/RoomBooking.js';
  if (codeInput) codeInput.value = `export class RoomBooking {\n  constructor(bookingId, guestAddress, nights) {\n    this.bookingId = bookingId;\n    this.guestAddress = guestAddress;\n    this.nights = nights;\n  }\n\n  calculateTotal(ratePerNight) {\n    return this.nights * ratePerNight;\n  }\n}\n`;
});

document.getElementById('pill-suggest-service')?.addEventListener('click', () => {
  const relInput = document.getElementById('create-file-rel-path');
  const codeInput = document.getElementById('create-file-code-content');
  if (relInput) relInput.value = 'src/application/services/BookingEngine.js';
  if (codeInput) codeInput.value = `import { RoomBooking } from '../../domain/models/RoomBooking.js';\n\nexport class BookingEngine {\n  static processBooking(order) {\n    const booking = new RoomBooking(order.id, order.guest, order.nights);\n    return booking.calculateTotal(0.15);\n  }\n}\n`;
});

document.getElementById('pill-suggest-doc')?.addEventListener('click', () => {
  const relInput = document.getElementById('create-file-rel-path');
  const codeInput = document.getElementById('create-file-code-content');
  if (relInput) relInput.value = 'src/domain/README.md';
  if (codeInput) codeInput.value = `# Couche Domaine & Entités Métier\n\nCe dossier contient les entités pures, objets-valeurs et calculs fondamentaux.\n\n## Responsabilités\n- Calculs financiers et réservations hôtelières\n- Validation des règles immuables\n`;
});

// Submit File Creation
document.getElementById('btn-submit-create-file')?.addEventListener('click', async () => {
  const project_path = document.getElementById('create-file-project-path')?.value.trim();
  const file_path = document.getElementById('create-file-rel-path')?.value.trim();
  const content = document.getElementById('create-file-code-content')?.value;

  if (!project_path || !file_path) {
    showToast('⚠️ Veuillez renseigner le projet et le chemin du fichier !');
    return;
  }
  if (!content || !content.trim()) {
    showToast('⚠️ Le contenu du fichier ne peut pas être vide !');
    return;
  }

  showToast('🚀 Écriture & Naissance stellaire en cours...');
  try {
    const d = await window.AetherAPI.createFile(project_path, file_path, content);
    if (d.status === 'ok' && (d.data || d.graph)) {
      document.getElementById('ai-create-file-modal')?.classList.remove('open');
      applyDataKeepView(d.data || d.graph, d.created_symbol);
      showToast(`✨ Supernova ! Planète créée : ${d.created_symbol}`);
    } else {
      showToast(`❌ Erreur: ${d.message || 'Échec création'}`);
    }
  } catch (err) {
    showToast('❌ Erreur de communication serveur');
  }
});

document.getElementById('btn-analyze')?.addEventListener('click', () => {
  const p = document.getElementById('project-path-input')?.value.trim() || '';
  analyze(p);
});

document.getElementById('project-select')?.addEventListener('change', (e) => {
  if (e.target.value) {
    const inp = document.getElementById('project-path-input');
    if (inp) inp.value = e.target.value;
    analyze(e.target.value);
  }
});

document.getElementById('btn-export')?.addEventListener('click', async () => {
  const p = document.getElementById('project-path-input')?.value.trim() || state.data.project_path;
  if (!p) {
    showToast('Veuillez spécifier un projet.');
    return;
  }
  showToast('💾 Export du viewer autonome...');
  try {
    const d = await window.AetherAPI.exportHtml(p);
    if (d.status === 'ok') {
      showToast(`✅ Exporté dans ${d.viewer_path}`);
    }
  } catch (e) {
    showToast('❌ Erreur export');
  }
});

// --- Connect MCP Configuration Modal & CLI Bridge Handlers ---
const currentMcpPort = window.location.port || '3030';
// Chemin par défaut générique : remplacé par le chemin réel renvoyé par /api/settings (mcp_binary_path).
let mcpBinaryPath = "C:\\Chemin\\Vers\\AetherScope\\aetherscope-mcp.exe";

function buildMcpConfigs() {
  const json = JSON.stringify({
    mcpServers: {
      aetherscope: {
        command: mcpBinaryPath,
        args: ["--port", currentMcpPort]
      }
    }
  }, null, 2);
  return {
    antigravity: { label: "FICHIER : ~/.gemini/config/mcp_config.json (ou Antigravity MCP Settings)", json },
    claude: { label: "FICHIER : %APPDATA%\\Claude\\claude_desktop_config.json", json },
    cursor: { label: "FICHIER : .cursor/mcp.json ou Paramètres Cursor > Features > MCP", json }
  };
}

let mcpConfigs = buildMcpConfigs();

fetch('/api/settings')
  .then(res => (res.ok ? res.json() : null))
  .then(data => {
    if (data && data.mcp_binary_path) {
      mcpBinaryPath = data.mcp_binary_path;
      mcpConfigs = buildMcpConfigs();
      if (document.getElementById('ai-mcp-modal')?.classList.contains('open')) {
        updateMcpModalView(curMcpTab);
      }
    }
  })
  .catch(() => { /* chemin générique conservé */ });

let curMcpTab = 'antigravity';
function updateMcpModalView(tab) {
  curMcpTab = tab;
  const cfg = mcpConfigs[tab] || mcpConfigs.antigravity;
  const label = document.getElementById('mcp-config-label');
  const textarea = document.getElementById('mcp-config-textarea');
  if (label) label.innerText = cfg.label;
  if (textarea) textarea.value = cfg.json;

  document.querySelectorAll('#ai-mcp-modal .task-btn').forEach(btn => btn.classList.remove('active'));
  const activeBtn = document.getElementById(`tab-mcp-${tab}`);
  if (activeBtn) activeBtn.classList.add('active');
}

function openMcpModal() {
  const modal = document.getElementById('ai-mcp-modal');
  if (modal) {
    updateMcpModalView(curMcpTab);
    modal.classList.add('open');
  }
}
window.openMcpModal = openMcpModal;

function closeMcpModal() {
  document.getElementById('ai-mcp-modal')?.classList.remove('open');
}
window.closeMcpModal = closeMcpModal;

document.getElementById('btn-menu-mcp')?.addEventListener('click', openMcpModal);
document.getElementById('btn-ai-mcp-config')?.addEventListener('click', openMcpModal);
document.getElementById('btn-close-ai-mcp-modal')?.addEventListener('click', closeMcpModal);
document.getElementById('btn-dismiss-ai-mcp-modal')?.addEventListener('click', closeMcpModal);
document.getElementById('ai-mcp-modal')?.addEventListener('click', (e) => {
  if (e.target.id === 'ai-mcp-modal') closeMcpModal();
});

document.getElementById('tab-mcp-antigravity')?.addEventListener('click', () => updateMcpModalView('antigravity'));
document.getElementById('tab-mcp-claude')?.addEventListener('click', () => updateMcpModalView('claude'));
document.getElementById('tab-mcp-cursor')?.addEventListener('click', () => updateMcpModalView('cursor'));

document.getElementById('btn-copy-mcp-config')?.addEventListener('click', () => {
  const txt = document.getElementById('mcp-config-textarea')?.value || '';
  navigator.clipboard.writeText(txt).then(() => {
    if (window.AudioSys?.promptCopied) AudioSys.promptCopied();
    showToast('📋 Configuration MCP copiée dans le presse-papier !');
  });
});

document.getElementById('btn-ai-launch-cli')?.addEventListener('click', async () => {
  showToast('💻 Lancement du terminal Gemini CLI AetherScope...');
  try {
    const curPath = state.data?.project_path || document.getElementById('project-path-input')?.value.trim() || '';
    const d = await window.AetherAPI.launchCli('gemini', curPath, '');
    if (d.status === 'ok') {
      if (window.AudioSys?.playTone) AudioSys.playTone(580, 'triangle', 0.15, 0.04);
      showToast(`⚡ ${d.message}`);
    } else {
      showToast(`⚠️ ${d.message || 'Erreur au lancement du terminal'}`);
    }
  } catch (err) {
    showToast('❌ Échec de la connexion avec le serveur');
  }
});

// ========================================================
// PROJECT HUB & MULTI-ARCHITECTURE ORCHESTRATOR LOGIC
// ========================================================
let selectedArchStack = 'rust';

function openHubModal() {
  const modal = document.getElementById('project-hub-modal');
  if (!modal) return;
  const curDir = document.getElementById('project-path-input')?.value.trim();
  const hubDirInput = document.getElementById('hub-project-dir');
  if (hubDirInput && !hubDirInput.value) {
    if (curDir) {
      const parts = curDir.split(/[\\/]/);
      parts.pop();
      hubDirInput.value = parts.join('\\') || curDir;
    } else {
      hubDirInput.value = 'C:\\Users\\1234\\Desktop\\Projets';
    }
  }
  modal.classList.add('open');
}
window.openHubModal = openHubModal;

function closeHubModal() {
  document.getElementById('project-hub-modal')?.classList.remove('open');
}
window.closeHubModal = closeHubModal;

document.getElementById('btn-close-hub-modal')?.addEventListener('click', closeHubModal);
document.getElementById('btn-open-hub')?.addEventListener('click', openHubModal);

document.querySelectorAll('#arch-selector-grid .arch-card').forEach(card => {
  card.addEventListener('click', () => {
    document.querySelectorAll('#arch-selector-grid .arch-card').forEach(c => c.classList.remove('selected'));
    card.classList.add('selected');
    selectedArchStack = card.getAttribute('data-stack') || 'rust';
    if (window.AudioSys?.nodeSelect) AudioSys.nodeSelect();
  });
});

document.getElementById('btn-hub-browse-dir')?.addEventListener('click', async () => {
  try {
    const d = await window.AetherAPI.pickWorkspace();
    if (d.status === 'ok' && d.workspace) {
      const inp = document.getElementById('hub-project-dir');
      if (inp) inp.value = d.workspace;
    }
  } catch (e) {
    showToast('❌ Erreur sélection dossier');
  }
});

document.getElementById('btn-hub-action-workspace')?.addEventListener('click', () => {
  closeHubModal();
  document.getElementById('btn-pick-workspace')?.click();
});

document.getElementById('btn-hub-action-project')?.addEventListener('click', () => {
  closeHubModal();
  document.getElementById('btn-pick-project')?.click();
});

document.getElementById('btn-hub-create-scaffold')?.addEventListener('click', async () => {
  const baseDir = document.getElementById('hub-project-dir')?.value.trim();
  const projName = document.getElementById('hub-project-name')?.value.trim();

  if (!baseDir || !projName) {
    showToast('⚠️ Veuillez renseigner le dossier parent et le nom du projet !');
    return;
  }

  const fullPath = baseDir.replace(/[\\/]+$/, '') + '\\' + projName;
  showToast(`🔨 Forgeage de l'architecture [${selectedArchStack.toUpperCase()}] dans ${projName}...`);
  try {
    const d = await window.AetherAPI.initProject(fullPath, selectedArchStack);
    if (d.status === 'ok') {
      showToast(`✨ ${d.message} !`);
      closeHubModal();
      const pInput = document.getElementById('project-path-input');
      if (pInput) pInput.value = fullPath;
      setTimeout(() => {
        analyze(fullPath);
      }, 400);
    } else {
      showToast(`❌ Erreur: ${d.message || 'Échec initialisation'}`);
    }
  } catch (err) {
    showToast('❌ Erreur de communication serveur');
  }
});

setTimeout(() => {
  const hasSeenHub = localStorage.getItem('aetherscope_hub_seen_v2');
  if (!hasSeenHub) {
    localStorage.setItem('aetherscope_hub_seen_v2', 'true');
    openHubModal();
  }
}, 600);

// ========================================================
// COCKPIT SETTINGS & PREFERENCES CONTROLLER
// ========================================================
function openSettingsModal() {
  const modal = document.getElementById('settings-modal');
  if (!modal) return;
  const lenInput = document.getElementById('setting-label-len');
  const lenVal = document.getElementById('setting-label-len-val');
  const previewText = document.getElementById('setting-preview-result') || document.getElementById('setting-preview-text');
  const lazyInput = document.getElementById('setting-lazy-readmes');
  const fpsInput = document.getElementById('setting-fps-limit');
  const soundInput = document.getElementById('setting-sound-enabled') || document.getElementById('setting-sound-effects');

  const maxLen = state.settings?.maxLabelLength || 18;
  if (lenInput) lenInput.value = maxLen;
  if (lenVal) lenVal.innerText = `${maxLen} car.`;
  if (previewText) previewText.innerText = typeof truncateNodeName === 'function' ? truncateNodeName('BookingEngineServiceWorkerImpl', maxLen) : 'BookingEngine…';
  const bubbleToggle = document.getElementById('setting-hide-bubble-labels');
  const bubbleThresholdInput = document.getElementById('setting-bubble-threshold');
  if (bubbleToggle) bubbleToggle.checked = state.settings?.hideBubbleLabelsOver20 !== false;
  if (bubbleThresholdInput) bubbleThresholdInput.value = state.settings?.clusterLabelThreshold || 20;
  if (lazyInput) lazyInput.checked = state.settings?.lazyReadmes !== false;
  if (fpsInput) fpsInput.value = state.settings?.fpsLimit || '120';
  if (soundInput) soundInput.checked = state.settings?.sound !== false;

  const exclusionsInput = document.getElementById('setting-exclusions');
  if (exclusionsInput) {
    const currentExcl = state.settings?.exclusions || ['tests', '*.spec.*', '*.test.*'];
    exclusionsInput.value = Array.isArray(currentExcl) ? currentExcl.join('\n') : String(currentExcl || '');
  }

  const autocamToggle = document.getElementById('setting-ai-autocam');
  if (autocamToggle) autocamToggle.checked = state.aiAutoCam !== false;

  const trailToggle = document.getElementById('setting-ai-trail');
  if (trailToggle) trailToggle.checked = state.settings?.aiTrailEnabled !== false;

  const portInput = document.getElementById('setting-ai-port');
  if (portInput) portInput.value = state.settings?.mcpPort || window.location.port || 3030;

  const aiPromptInput = document.getElementById('setting-ai-custom-prompt');
  if (aiPromptInput) aiPromptInput.value = state.settings?.copilotPrompt || '';

  // 8. Load & Render CVE Rules
  renderCveRulesSettings();

  modal.classList.add('open');
}
window.openSettingsModal = openSettingsModal;

// CVE Rules in Settings State & Functions
let currentCveCatalogue = [];

async function renderCveRulesSettings() {
  const container = document.getElementById('settings-cve-rules-container');
  if (!container) return;

  try {
    if (!currentCveCatalogue || currentCveCatalogue.length === 0) {
      const res = await fetch('/api/settings');
      if (res.ok) {
        const data = await res.json();
        if (data.cve_catalogue) {
          currentCveCatalogue = data.cve_catalogue;
        }
        if (data.active_cve_rules && !state.settings?.activeCveRules) {
          if (!state.settings) state.settings = {};
          state.settings.activeCveRules = data.active_cve_rules;
        }
      }
    }
  } catch (e) {
    console.warn('Failed to fetch CVE rules from server:', e);
  }

  if (!currentCveCatalogue || currentCveCatalogue.length === 0) {
    container.innerHTML = '<div style="font-size:0.75rem; color:#64748b; padding:10px;">Catalogue 90+ Zero-Day Patterns & CWE Rules indisponible.</div>';
    return;
  }

  const activeRules = new Set(state.settings?.activeCveRules || currentCveCatalogue.map(r => r.rule_id));
  const searchInput = document.getElementById('setting-cve-search');
  const query = (searchInput?.value || '').toLowerCase().trim();

  const filtered = currentCveCatalogue.filter(r => {
    if (!query) return true;
    return r.rule_id.toLowerCase().includes(query) ||
           r.name.toLowerCase().includes(query) ||
           r.category.toLowerCase().includes(query) ||
           r.description.toLowerCase().includes(query) ||
           (r.cve_id && r.cve_id.toLowerCase().includes(query));
  });

  const counter = document.getElementById('cve-rules-counter');
  if (counter) {
    counter.innerText = `${activeRules.size}/${currentCveCatalogue.length} actives`;
  }

  if (filtered.length === 0) {
    container.innerHTML = '<div style="font-size:0.75rem; color:#64748b; padding:10px; text-align:center;">Aucune règle ne correspond à ce filtre.</div>';
    return;
  }

  let html = '';
  const getFileTypesBadge = (r) => {
    const rid = (r.rule_id || '').toUpperCase();
    const cat = (r.category || '').toLowerCase();
    if (rid.includes('-PHP-') || cat.includes('php')) return '<span class="cve-rule-cat" style="color:#c084fc; border-color:rgba(192,132,252,0.3); background:rgba(192,132,252,0.1);">📁 .php, .phtml</span>';
    if (rid.includes('-RUST-') || cat.includes('rust')) return '<span class="cve-rule-cat" style="color:#f97316; border-color:rgba(249,115,22,0.3); background:rgba(249,115,22,0.1);">📁 .rs</span>';
    if (rid.includes('-PY-') || cat.includes('python')) return '<span class="cve-rule-cat" style="color:#eab308; border-color:rgba(234,179,8,0.3); background:rgba(234,179,8,0.1);">📁 .py</span>';
    if (rid.includes('-CPP-') || cat.includes('c / c++')) return '<span class="cve-rule-cat" style="color:#38bdf8; border-color:rgba(56,189,248,0.3); background:rgba(56,189,248,0.1);">📁 .cpp, .c, .h</span>';
    if (rid.includes('-JS-') || cat.includes('node') || cat.includes('front-end')) return '<span class="cve-rule-cat" style="color:#4ade80; border-color:rgba(74,222,128,0.3); background:rgba(74,222,128,0.1);">📁 .ts, .js</span>';
    if (rid.includes('SOL') || cat.includes('defi') || cat.includes('crypto')) return '<span class="cve-rule-cat" style="color:#818cf8; border-color:rgba(129,140,248,0.3); background:rgba(129,140,248,0.1);">📁 .sol</span>';
    return '<span class="cve-rule-cat">📁 Tout fichier</span>';
  };

  filtered.forEach(rule => {
    const isChecked = activeRules.has(rule.rule_id);
    const sevClass = rule.severity.toLowerCase() === 'critical' ? 'critical' : (rule.severity.toLowerCase() === 'high' ? 'high' : 'warning');
    const sevBadge = rule.severity.toUpperCase();

    html += `
      <div class="cve-rule-row ${isChecked ? '' : 'disabled'}" data-rule-id="${rule.rule_id}">
        <label class="aether-toggle" style="margin-top:2px;">
          <input type="checkbox" class="cve-rule-checkbox" data-rule-id="${rule.rule_id}" ${isChecked ? 'checked' : ''} onchange="onCveRuleToggle('${rule.rule_id}', this.checked)" />
          <span class="toggle-slider"></span>
        </label>
        <div class="cve-rule-info">
          <div class="cve-rule-header">
            <span class="cve-rule-id">${rule.rule_id}</span>
            <span class="cve-badge ${sevClass}" style="font-size:0.60rem; padding:1px 5px;">${sevBadge}</span>
            <span class="cve-rule-cat">${rule.category}</span>
            ${getFileTypesBadge(rule)}
            ${rule.cve_id ? `<span style="font-size:0.62rem; font-family:var(--font-mono); color:#a855f7;">${rule.cve_id}</span>` : ''}
          </div>
          <div style="font-size:0.74rem; font-weight:700; color:#e2e8f0; margin-top:1px;">${rule.name}</div>
          <div class="cve-rule-desc">${rule.description}</div>
        </div>
      </div>
    `;
  });

  container.innerHTML = html;
}
window.renderCveRulesSettings = renderCveRulesSettings;

window.filterCveRulesList = function(val) {
  renderCveRulesSettings();
};

window.onCveRuleToggle = function(ruleId, isChecked) {
  if (!state.settings) state.settings = {};
  let active = new Set(state.settings.activeCveRules || currentCveCatalogue.map(r => r.rule_id));
  if (isChecked) {
    active.add(ruleId);
  } else {
    active.delete(ruleId);
  }
  state.settings.activeCveRules = Array.from(active);
  const row = document.querySelector(`.cve-rule-row[data-rule-id="${ruleId}"]`);
  if (row) {
    if (isChecked) row.classList.remove('disabled');
    else row.classList.add('disabled');
  }
  const counter = document.getElementById('cve-rules-counter');
  if (counter) {
    counter.innerText = `${active.size}/${currentCveCatalogue.length} actives`;
  }
};

window.toggleAllCveRules = function(enableAll) {
  if (!state.settings) state.settings = {};
  if (enableAll) {
    state.settings.activeCveRules = currentCveCatalogue.map(r => r.rule_id);
  } else {
    state.settings.activeCveRules = [];
  }
  renderCveRulesSettings();
};

window.addExclusionPill = function(pill) {
  const input = document.getElementById('setting-exclusions');
  if (!input) return;
  const current = input.value.split('\n').map(s => s.trim()).filter(Boolean);
  if (!current.includes(pill)) {
    current.push(pill);
    input.value = current.join('\n');
    if (window.AudioSys?.nodeSelect) AudioSys.nodeSelect();
  }
};

function closeSettingsModal() {
  document.getElementById('settings-modal')?.classList.remove('open');
}
window.closeSettingsModal = closeSettingsModal;

function toggleSettingsModal() {
  const modal = document.getElementById('settings-modal');
  if (modal && modal.classList.contains('open')) {
    closeSettingsModal();
  } else {
    openSettingsModal();
  }
}
window.toggleSettingsModal = toggleSettingsModal;

function syncSettingsToMenuShortcuts() {
  const lenBadge = document.getElementById('menu-current-label-len');
  if (lenBadge) lenBadge.innerText = `${state.settings?.maxLabelLength || 18} car.`;
  const bubbleBadge = document.getElementById('menu-current-bubble');
  if (bubbleBadge) {
    bubbleBadge.innerText = state.settings?.hideBubbleLabelsOver20 !== false ? `> ${state.settings?.clusterLabelThreshold || 20}` : 'Inactif';
  }
  const lazyBadge = document.getElementById('menu-current-lazy');
  if (lazyBadge) lazyBadge.innerText = state.settings?.lazyReadmes !== false ? 'Actif' : 'Inactif';
}

// Settings Slider Live Preview
document.getElementById('setting-label-len')?.addEventListener('input', (e) => {
  const val = parseInt(e.target.value, 10);
  if (!state.settings) state.settings = {};
  state.settings.maxLabelLength = val;
  const lenVal = document.getElementById('setting-label-len-val');
  if (lenVal) lenVal.innerText = `${val} car.`;
  const previewText = document.getElementById('setting-preview-result') || document.getElementById('setting-preview-text');
  if (previewText) previewText.innerText = typeof truncateNodeName === 'function' ? truncateNodeName('BookingEngineServiceWorkerImpl', val) : 'BookingEngine…';
  syncSettingsToMenuShortcuts();
});

document.getElementById('setting-hide-bubble-labels')?.addEventListener('change', (e) => {
  if (!state.settings) state.settings = {};
  state.settings.hideBubbleLabelsOver20 = e.target.checked;
  syncSettingsToMenuShortcuts();
});

document.getElementById('setting-bubble-threshold')?.addEventListener('input', (e) => {
  const val = parseInt(e.target.value, 10) || 20;
  if (!state.settings) state.settings = {};
  state.settings.clusterLabelThreshold = val;
  syncSettingsToMenuShortcuts();
});

document.getElementById('setting-lazy-readmes')?.addEventListener('change', (e) => {
  if (!state.settings) state.settings = {};
  state.settings.lazyReadmes = e.target.checked;
  syncSettingsToMenuShortcuts();
});

document.getElementById('setting-fps-limit')?.addEventListener('change', (e) => {
  if (!state.settings) state.settings = {};
  state.settings.fpsLimit = e.target.value;
});

const soundCheckbox = document.getElementById('setting-sound-enabled') || document.getElementById('setting-sound-effects');
soundCheckbox?.addEventListener('change', (e) => {
  if (!state.settings) state.settings = {};
  state.settings.sound = e.target.checked;
  if (window.AudioSys) {
    window.AudioSys.enabled = e.target.checked;
  }
});

// Save Settings Button
document.getElementById('btn-save-settings')?.addEventListener('click', async () => {
  try {
    if (!state.settings) state.settings = {};

    // 1. Label Length
    const lenInput = document.getElementById('setting-label-len');
    if (lenInput) state.settings.maxLabelLength = parseInt(lenInput.value, 10) || 18;

    // 2. Hide bubble labels
    const bubbleToggle = document.getElementById('setting-hide-bubble-labels');
    if (bubbleToggle) state.settings.hideBubbleLabelsOver20 = bubbleToggle.checked;

    // 2b. Threshold
    const bubbleThresh = document.getElementById('setting-bubble-threshold');
    if (bubbleThresh) state.settings.clusterLabelThreshold = parseInt(bubbleThresh.value, 10) || 20;

    // 3. Lazy Readmes
    const lazyInput = document.getElementById('setting-lazy-readmes');
    if (lazyInput) state.settings.lazyReadmes = lazyInput.checked;

    // 4. FPS Limit
    const fpsInput = document.getElementById('setting-fps-limit');
    if (fpsInput) state.settings.fpsLimit = fpsInput.value;

    // 5. Sound
    const soundInput = document.getElementById('setting-sound-enabled') || document.getElementById('setting-sound-effects');
    if (soundInput) {
      state.settings.sound = soundInput.checked;
      if (window.AudioSys) window.AudioSys.enabled = soundInput.checked;
    }

    // 6. Exclusions
    const exclusionsInput = document.getElementById('setting-exclusions');
    let lines = [];
    if (exclusionsInput) {
      lines = exclusionsInput.value.split(/[\n,]/).map(s => s.trim()).filter(Boolean);
      state.settings.exclusions = lines;
    }

    // 7. Copilot Preferences
    const autocamToggle = document.getElementById('setting-ai-autocam');
    if (autocamToggle) {
      state.aiAutoCam = autocamToggle.checked;
      state.settings.aiAutoCam = autocamToggle.checked;
      const btnAutocam = document.getElementById('btn-ai-autocam');
      if (btnAutocam) {
        btnAutocam.innerText = state.aiAutoCam ? '🎥 Auto-Cam: ON' : '🎥 Auto-Cam: OFF';
      }
    }

    const trailToggle = document.getElementById('setting-ai-trail');
    if (trailToggle) state.settings.aiTrailEnabled = trailToggle.checked;

    const portInput = document.getElementById('setting-ai-port');
    if (portInput) state.settings.mcpPort = parseInt(portInput.value, 10) || 3030;

    const aiPromptInput = document.getElementById('setting-ai-custom-prompt');
    if (aiPromptInput) state.settings.copilotPrompt = aiPromptInput.value.trim();

    // 8. CVE Active Rules
    const activeCve = state.settings.activeCveRules || (currentCveCatalogue ? currentCveCatalogue.map(r => r.rule_id) : []);

    // Persist to LocalStorage
    localStorage.setItem('aetherscope_settings', JSON.stringify(state.settings));

    // Send exclusions & active CVE rules to Server API
    try {
      await fetch('/api/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          exclusions: lines,
          active_cve_rules: activeCve
        })
      });
    } catch (_) {}

    syncSettingsToMenuShortcuts();
    if (typeof renderCveAnomaliesPanel === 'function') renderCveAnomaliesPanel();
    showToast('⚙️ Préférences du Copilote & Cockpit sauvegardées !');
    if (window.AudioSys?.nodeSelect) AudioSys.nodeSelect();
  } catch (err) {
    console.error('Save settings error:', err);
    showToast('❌ Erreur sauvegarde paramètres');
  }
  closeSettingsModal();
});

// Reset Settings Button (Support both IDs)
const resetSettingsHandler = () => {
  state.settings = {
    maxLabelLength: 18,
    lazyReadmes: true,
    fpsLimit: '120',
    sound: true,
    hideBubbleLabelsOver20: true,
    clusterLabelThreshold: 20,
    exclusions: ['tests', '*.spec.*', '*.test.*'],
    aiAutoCam: true,
    aiTrailEnabled: true,
    mcpPort: 3030,
    copilotPrompt: ''
  };
  state.aiAutoCam = true;
  if (window.AudioSys) {
    window.AudioSys.enabled = true;
  }
  try {
    localStorage.setItem('aetherscope_settings', JSON.stringify(state.settings));
  } catch (_) {}
  openSettingsModal();
  syncSettingsToMenuShortcuts();
  showToast('🔄 Paramètres réinitialisés aux valeurs optimales');
};
document.getElementById('btn-reset-default-settings')?.addEventListener('click', resetSettingsHandler);
document.getElementById('btn-reset-settings')?.addEventListener('click', resetSettingsHandler);

document.getElementById('btn-close-settings-modal')?.addEventListener('click', closeSettingsModal);
document.getElementById('btn-cancel-settings')?.addEventListener('click', closeSettingsModal);
document.getElementById('settings-modal')?.addEventListener('click', (e) => {
  if (e.target.id === 'settings-modal') closeSettingsModal();
});
document.getElementById('btn-settings-open-mcp')?.addEventListener('click', () => {
  closeSettingsModal();
  openMcpModal();
});
document.getElementById('btn-settings-open-sysprompt')?.addEventListener('click', () => {
  closeSettingsModal();
  openSystemPromptModal(false);
});
document.getElementById('btn-menu-ai-settings')?.addEventListener('click', openSettingsModal);
document.getElementById('btn-quick-settings')?.addEventListener('click', openSettingsModal);
document.getElementById('btn-menu-open-settings')?.addEventListener('click', openSettingsModal);
document.getElementById('btn-menu-open-settings-len')?.addEventListener('click', openSettingsModal);
document.getElementById('btn-menu-open-settings-bubble')?.addEventListener('click', openSettingsModal);
document.getElementById('btn-menu-open-settings-lazy')?.addEventListener('click', openSettingsModal);
document.getElementById('btn-menu-reset-view')?.addEventListener('click', () => { if (typeof fitView === 'function') fitView(); });
document.getElementById('btn-menu-clear-cache')?.addEventListener('click', () => {
  if (state.docCache) state.docCache.clear();
  state.settings = {
    maxLabelLength: 18,
    lazyReadmes: true,
    fpsLimit: '120',
    sound: true,
    hideBubbleLabelsOver20: true,
    clusterLabelThreshold: 20
  };
  if (window.AudioSys) {
    window.AudioSys.enabled = true;
  }
  try { localStorage.removeItem('aetherscope_settings'); } catch (_) {}
  syncSettingsToMenuShortcuts();
  showToast('🗑️ Cache de documentation & préférences réinitialisés');
});

// Keyboard Shortcuts Router
window.addEventListener('keydown', (e) => {
  if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT' || e.target.tagName === 'TEXTAREA') return;
  if (e.key === 'x' || e.key === 'X') { if (typeof toggleXrayMode === 'function') toggleXrayMode(); }
  else if (e.key === 'd' || e.key === 'D') { if (typeof toggleDeadCodeMode === 'function') toggleDeadCodeMode(); }
  else if (e.key === 'h' || e.key === 'H') { if (typeof toggleHeatmapMode === 'function') toggleHeatmapMode(); }
  else if (e.key === 'p' || e.key === 'P') toggleSettingsModal();
  else if (e.key === 's' || e.key === 'S') { if (typeof takeSnapshot === 'function') takeSnapshot(); }
  else if (e.key === 'a' || e.key === 'A') document.getElementById('btn-ai-telemetry')?.click();
  else if (e.key === 'c' || e.key === 'C') toggleAiChatPopup();
  else if (e.key === '1') { if (typeof switchDepth === 'function') switchDepth(1); }
  else if (e.key === '2') { if (typeof switchDepth === 'function') switchDepth(2); }
  else if (e.key === '3') { if (typeof switchDepth === 'function') switchDepth(3); }
  else if (e.key === '4') { if (typeof switchDepth === 'function') switchDepth(4); }
  else if (e.key === 'f' || e.key === 'F') { if (typeof fitView === 'function') fitView(); }
  else if (e.key === 'g' || e.key === 'G') document.getElementById('btn-gravity')?.click();
  else if (e.key === 'b' || e.key === 'B') { if (typeof toggleSynthesisMenu === 'function') toggleSynthesisMenu(); }
  else if (e.key === 'm' || e.key === 'M') { if (typeof toggleDocReader === 'function') toggleDocReader(); }
  else if (e.key === 'i' || e.key === 'I') { if (typeof toggleInspector === 'function') toggleInspector(); }
  else if (e.key === 'Escape') {
    if (document.getElementById('topbar-search-dropdown')?.classList.contains('open')) {
      closeTopbarSearchDropdown();
      document.getElementById('topbar-search-input')?.blur();
      return;
    }
    if (document.getElementById('settings-modal')?.classList.contains('open')) {
      closeSettingsModal();
      return;
    }
    if (document.getElementById('project-hub-modal')?.classList.contains('open')) {
      closeHubModal();
      return;
    }
    if (state.xrayMode && typeof toggleXrayMode === 'function') {
      toggleXrayMode();
      return;
    }
    if (state.deadCodeMode && typeof toggleDeadCodeMode === 'function') {
      toggleDeadCodeMode();
      return;
    }
    if (state.heatmapMode && typeof toggleHeatmapMode === 'function') {
      toggleHeatmapMode();
      return;
    }
    state.selectedNode = null;
    state.beaconNode = null;
    if (typeof toggleInspector === 'function') toggleInspector(false);
    else document.getElementById('inspector')?.classList.add('hidden');
    if (typeof toggleDocReader === 'function') toggleDocReader(false);
    else document.getElementById('doc-reader-panel')?.classList.add('hidden');
    document.getElementById('search-modal')?.classList.remove('open');
    closeTopbarSearchDropdown();
    document.getElementById('ai-modal')?.classList.remove('open');
    document.getElementById('ai-tasks-modal')?.classList.remove('open');
    document.getElementById('master-prompt-modal')?.classList.remove('open');
    document.getElementById('bounty-scope-modal')?.classList.remove('open');
    document.getElementById('ai-sysprompt-modal')?.classList.remove('open');
    document.getElementById('ai-create-file-modal')?.classList.remove('open');
    document.getElementById('ai-mcp-modal')?.classList.remove('open');
    document.getElementById('ai-telemetry-hud')?.classList.add('hidden');
  } else if (e.key === 'k' && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    focusTopbarSearch();
  } else if (e.key === '/') {
    e.preventDefault();
    focusTopbarSearch();
  } else if ((e.key === 't' || e.key === 'T') && !['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) {
    e.preventDefault();
    openTasksModal();
  }
});

// =========================================================================
// TASKS & QUESTS CONTROLLER (Model & User Hybrid Task Engine)
// =========================================================================
let currentTasksCache = [];
let currentTaskFilter = 'active';
let activeSelectedTaskId = null;

async function openTasksModal(prefillSymbol = null, prefillFile = null) {
  const modal = document.getElementById('ai-tasks-modal');
  if (!modal) return;
  modal.classList.add('open');

  if (prefillSymbol || prefillFile) {
    const formPanel = document.getElementById('new-task-form-panel');
    if (formPanel) formPanel.style.display = 'flex';
    const symInput = document.getElementById('task-input-symbol');
    const fileInput = document.getElementById('task-input-file');
    const titleInput = document.getElementById('task-input-title');
    if (symInput && prefillSymbol) symInput.value = prefillSymbol;
    if (fileInput && prefillFile) fileInput.value = prefillFile;
    if (titleInput) {
      titleInput.value = prefillSymbol ? `Travailler sur ${prefillSymbol}` : '';
      titleInput.focus();
    }
  }

  await refreshTasksList();
}
window.openTasksModal = openTasksModal;

function closeTasksModal() {
  document.getElementById('ai-tasks-modal')?.classList.remove('open');
  const readmePanel = document.getElementById('task-readme-panel');
  if (readmePanel) readmePanel.style.display = 'none';
  activeSelectedTaskId = null;
}
window.closeTasksModal = closeTasksModal;

async function refreshTasksList() {
  const container = document.getElementById('tasks-list-container');
  if (!container) return;
  container.innerHTML = '<div style="color:#64748b; font-size:0.75rem; padding:12px; text-align:center;">Chargement des tâches...</div>';

  const res = await AetherAPI.fetchTasks();
  if (res && res.tasks) {
    currentTasksCache = res.tasks;
    renderTasksList(currentTasksCache);
  } else {
    container.innerHTML = '<div style="color:#ef4444; font-size:0.75rem; padding:12px; text-align:center;">Impossible de charger les tâches.</div>';
  }
}

function renderTasksList(tasks) {
  const container = document.getElementById('tasks-list-container');
  const countPill = document.getElementById('tasks-count-pill');
  if (!container) return;

  const filtered = (tasks || []).filter(t => {
    if (currentTaskFilter === 'done') return t.status === 'done';
    if (currentTaskFilter === 'active') return t.status !== 'done' && t.status !== 'cancelled';
    return true;
  });

  if (countPill) countPill.innerText = `${tasks.length} tâche(s)`;

  if (filtered.length === 0) {
    container.innerHTML = `
      <div style="background:rgba(15, 23, 42, 0.6); border:1px dashed rgba(56, 189, 248, 0.2); border-radius:8px; padding:24px; text-align:center; color:#94a3b8; font-size:0.75rem;">
        Aucune tâche dans cette vue.<br>
        <span style="font-size:0.70rem; color:#64748b;">Cliquez sur <strong>➕ Nouvelle Tâche</strong> ou demandez à l'IA via MCP !</span>
      </div>
    `;
    return;
  }

  const kindIcons = {
    function: '⚙️',
    research: '🔍',
    security_fix: '🛡️',
    arch_refactor: '🏛️',
    documentation: '📖',
    test: '🧪',
    other: '📌'
  };

  const statusBadges = {
    pending: '<span style="background:rgba(148,163,184,0.15); color:#94a3b8; border:1px solid rgba(148,163,184,0.3); padding:2px 6px; border-radius:4px; font-size:0.65rem; font-weight:700;">⏳ EN ATTENTE</span>',
    in_progress: '<span style="background:rgba(56,189,248,0.15); color:#38bdf8; border:1px solid rgba(56,189,248,0.3); padding:2px 6px; border-radius:4px; font-size:0.65rem; font-weight:700;">⚡ EN COURS</span>',
    blocked: '<span style="background:rgba(239,68,68,0.15); color:#f87171; border:1px solid rgba(239,68,68,0.3); padding:2px 6px; border-radius:4px; font-size:0.65rem; font-weight:700;">⛔ BLOQUÉ</span>',
    done: '<span style="background:rgba(16,185,129,0.15); color:#34d399; border:1px solid rgba(16,185,129,0.3); padding:2px 6px; border-radius:4px; font-size:0.65rem; font-weight:700;">✅ TERMINÉ</span>',
    cancelled: '<span style="background:rgba(100,116,139,0.15); color:#64748b; border:1px solid rgba(100,116,139,0.3); padding:2px 6px; border-radius:4px; font-size:0.65rem; font-weight:700;">⚪ ANNULÉ</span>'
  };

  container.innerHTML = filtered.map(t => {
    const icon = kindIcons[t.kind] || '📌';
    const badge = statusBadges[t.status] || t.status;
    const target = t.target_symbol || t.target_file || '';
    const progress = t.progress || 0;

    return `
      <div class="task-card-item" data-id="${t.id}" style="background:rgba(15, 23, 42, 0.85); border:1px solid rgba(56, 189, 248, 0.22); border-radius:8px; padding:10px 14px; cursor:pointer; transition:all 0.18s; display:flex; flex-direction:column; gap:6px;">
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <div style="display:flex; align-items:center; gap:8px;">
            <span style="font-size:1.05rem;">${icon}</span>
            <span style="font-weight:700; color:#e2e8f0; font-size:0.80rem;">${t.title}</span>
            <span style="font-size:0.65rem; color:#64748b; font-family:var(--font-mono);">(${t.id})</span>
          </div>
          <div style="display:flex; align-items:center; gap:8px;">
            ${badge}
            <span style="font-size:0.72rem; font-weight:700; color:#38bdf8;">${progress}%</span>
          </div>
        </div>

        <div style="width:100%; height:4px; background:rgba(30, 41, 59, 0.8); border-radius:2px; overflow:hidden;">
          <div style="width:${progress}%; height:100%; background:linear-gradient(90deg, #38bdf8, #818cf8); border-radius:2px;"></div>
        </div>

        <div style="display:flex; justify-content:space-between; align-items:center; font-size:0.70rem; color:#94a3b8;">
          <div style="display:flex; gap:10px; align-items:center;">
            ${target ? `<span style="color:#c084fc; font-weight:600;">🎯 ${target}</span>` : '<span style="color:#64748b;">Global</span>'}
            <span>Auteur : <strong>${t.author || 'user'}</strong></span>
          </div>
          <div style="display:flex; gap:6px;">
            ${t.status !== 'done' ? `<button class="aether-btn mark-done-btn" data-id="${t.id}" style="font-size:0.62rem; padding:2px 6px; border-color:#10b981; color:#34d399;">✓ Terminer</button>` : ''}
            <button class="aether-btn open-readme-btn" data-id="${t.id}" style="font-size:0.62rem; padding:2px 6px;">📖 Ouvrir README</button>
          </div>
        </div>
      </div>
    `;
  }).join('');

  container.querySelectorAll('.task-card-item').forEach(card => {
    card.addEventListener('click', (e) => {
      if (e.target.closest('button')) return;
      const tid = card.getAttribute('data-id');
      const t = currentTasksCache.find(x => x.id === tid);
      if (t) selectTaskDetail(t);
    });
  });

  container.querySelectorAll('.open-readme-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const tid = btn.getAttribute('data-id');
      const t = currentTasksCache.find(x => x.id === tid);
      if (t) selectTaskDetail(t);
    });
  });

  container.querySelectorAll('.mark-done-btn').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const tid = btn.getAttribute('data-id');
      await AetherAPI.updateTask(tid, 'done', 100, 'Tâche marquée comme terminée par l\'utilisateur');
      showToast(`✅ Tâche ${tid} terminée !`);
      await refreshTasksList();
    });
  });
}

function selectTaskDetail(task) {
  activeSelectedTaskId = task.id;
  const panel = document.getElementById('task-readme-panel');
  if (!panel) return;

  panel.style.display = 'flex';
  const titleEl = document.getElementById('task-detail-title');
  if (titleEl) titleEl.innerText = `📖 MINI-README : ${task.title} (${task.id})`;

  const statusSel = document.getElementById('task-detail-status');
  if (statusSel) statusSel.value = task.status;

  const range = document.getElementById('task-detail-progress-range');
  const label = document.getElementById('task-detail-progress-label');
  if (range) range.value = task.progress || 0;
  if (label) label.innerText = `${task.progress || 0}%`;

  const textarea = document.getElementById('task-detail-readme-textarea');
  if (textarea) textarea.value = task.full_readme || task.readme_summary || '';

  panel.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

// Global button bindings for Tasks
document.getElementById('btn-menu-tasks')?.addEventListener('click', () => openTasksModal());
document.getElementById('btn-close-tasks-modal')?.addEventListener('click', closeTasksModal);
document.getElementById('btn-dismiss-tasks-modal')?.addEventListener('click', closeTasksModal);
document.getElementById('btn-refresh-tasks')?.addEventListener('click', refreshTasksList);

document.getElementById('btn-insp-task')?.addEventListener('click', () => {
  const node = state.selectedNode;
  const sym = node ? node.name : null;
  const file = node ? node.file_path : null;
  openTasksModal(sym, file);
});

document.getElementById('btn-toggle-new-task-form')?.addEventListener('click', () => {
  const panel = document.getElementById('new-task-form-panel');
  if (!panel) return;
  const isHidden = panel.style.display === 'none' || !panel.style.display;
  panel.style.display = isHidden ? 'flex' : 'none';
  if (isHidden) {
    document.getElementById('task-input-title')?.focus();
  }
});

document.getElementById('btn-cancel-new-task')?.addEventListener('click', () => {
  const panel = document.getElementById('new-task-form-panel');
  if (panel) panel.style.display = 'none';
});

document.getElementById('btn-submit-new-task')?.addEventListener('click', async () => {
  const title = document.getElementById('task-input-title')?.value?.trim();
  if (!title) {
    showToast('⚠️ Veuillez saisir un titre pour la tâche.');
    return;
  }
  const kind = document.getElementById('task-select-kind')?.value || 'function';
  const prio = document.getElementById('task-select-priority')?.value || 'medium';
  const sym = document.getElementById('task-input-symbol')?.value?.trim() || null;
  const file = document.getElementById('task-input-file')?.value?.trim() || null;
  const readme = document.getElementById('task-input-readme')?.value?.trim() || null;

  const res = await AetherAPI.createTask(title, kind, sym, file, prio, readme);
  if (res && res.status === 'ok') {
    showToast(`✅ Tâche créée avec succès !`);
    const titleInput = document.getElementById('task-input-title');
    if (titleInput) titleInput.value = '';
    const readmeInput = document.getElementById('task-input-readme');
    if (readmeInput) readmeInput.value = '';
    const panel = document.getElementById('new-task-form-panel');
    if (panel) panel.style.display = 'none';
    await refreshTasksList();
  } else {
    showToast(`⚠️ Erreur : ${(res && res.message) || 'Échec création'}`);
  }
});

document.getElementById('btn-close-task-detail')?.addEventListener('click', () => {
  const panel = document.getElementById('task-readme-panel');
  if (panel) panel.style.display = 'none';
  activeSelectedTaskId = null;
});

document.getElementById('task-detail-progress-range')?.addEventListener('input', (e) => {
  const val = e.target.value;
  const label = document.getElementById('task-detail-progress-label');
  if (label) label.innerText = `${val}%`;
});

document.getElementById('btn-save-task-detail')?.addEventListener('click', async () => {
  if (!activeSelectedTaskId) return;
  const status = document.getElementById('task-detail-status')?.value;
  const progress = parseInt(document.getElementById('task-detail-progress-range')?.value || '0', 10);
  const readme = document.getElementById('task-detail-readme-textarea')?.value;

  const res = await AetherAPI.updateTask(activeSelectedTaskId, status, progress, null, readme);
  if (res && res.status === 'ok') {
    showToast(`💾 Avancement de la tâche sauvegardé !`);
    await refreshTasksList();
  } else {
    showToast('⚠️ Erreur lors de la sauvegarde.');
  }
});

document.getElementById('btn-delete-task')?.addEventListener('click', async () => {
  if (!activeSelectedTaskId) return;
  if (!confirm(`Supprimer définitivement la tâche ${activeSelectedTaskId} ?`)) return;
  const res = await AetherAPI.deleteTask(activeSelectedTaskId);
  if (res && res.status === 'ok') {
    showToast('🗑️ Tâche supprimée.');
    const panel = document.getElementById('task-readme-panel');
    if (panel) panel.style.display = 'none';
    activeSelectedTaskId = null;
    await refreshTasksList();
  }
});

document.getElementById('btn-teleport-task-target')?.addEventListener('click', () => {
  if (!activeSelectedTaskId) return;
  const t = currentTasksCache.find(x => x.id === activeSelectedTaskId);
  if (!t) return;
  const target = t.target_symbol || t.target_file;
  if (!target) {
    showToast('Aucune cible spatiale associée à cette tâche.');
    return;
  }
  const nodes = (state.data && state.data.nodes) || [];
  const found = nodes.find(n => n.name === target || n.file_path === target || n.id === target);
  if (found) {
    closeTasksModal();
    if (typeof selectNode === 'function') selectNode(found);
    showToast(`🎯 Caméra alignée sur : ${target}`);
  } else {
    showToast(`Cible introuvable dans le graphe : ${target}`);
  }
});

document.querySelectorAll('#tasks-filter-tabs .task-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('#tasks-filter-tabs .task-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    currentTaskFilter = btn.getAttribute('data-filter') || 'active';
    renderTasksList(currentTasksCache);
  });
});


// Initial sync & Render Loop Bootstrap
syncSettingsToMenuShortcuts();
if (state.settings && window.AudioSys) {
  window.AudioSys.enabled = state.settings.sound !== false;
}
loadProjects();
if (state.data && state.data.nodes && state.data.nodes.length > 0) {
  applyData(state.data);
}
if (typeof renderEdgeTypologyLegend === 'function') {
  renderEdgeTypologyLegend();
}
if (typeof render === 'function') {
  requestAnimationFrame(render);
}
