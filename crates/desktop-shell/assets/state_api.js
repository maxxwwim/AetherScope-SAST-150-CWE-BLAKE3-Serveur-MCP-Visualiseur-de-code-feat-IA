// =========================================================================
// AETHERSCOPE V2 — CENTRALIZED STATE & API CLIENT (state_api.js)
// =========================================================================

// Embedded WebGL 2 Instanced Engine & Pure State Store
    const state = {
      data: window.__AETHERSCOPE_DATA__ || { project_name: 'AetherScope', nodes: [], edges: [], stats: {} },
      camera: { x: 0, y: 0, zoom: 0.65, targetX: 0, targetY: 0, targetZoom: 0.65 },
      depth: 1,
      selectedNode: null,
      selectedFolder: null,
      hoveredFolder: null,
      folderClusters: new Map(),
      folderAnchors: new Map(),
      showFolderMembranes: true,
      showFolderBadges: false,
      showStarfield: true,
      showGrid: true,
      showShadows: true,
      hoveredNode: null,
      beaconNode: null,
      beaconStart: 0,
      activeLayers: new Set(['ui', 'action', 'domain', 'database', 'config', 'utility', 'docs']),
      hiddenModules: new Set(),
      dataPipelineMode: false,
      activeRelations: new Set([
        'calls', 'renders', 'imports', 'contains', 'inherits',
        'submits', 'network', 'realtime', 'realtimews', 'realtime_ws',
        'queries',
        'violates', 'violation', 'security_violation', 'security_violates',
        'documents', 'binds', 'inter_module'
      ]),
      synthesisMenuCollapsed: true,
      searchCategory: 'all',
      searchQuery: '',
      inspectorOpen: false,
      physicsRunning: true,
      currentTask: 'refactor',
      folderPalettes: new Map(),
      folderMdMap: new Map(),
      activeDocNode: null,
      xrayMode: false,
      xrayTunnel: null,
      deadCodeMode: false,
      deadCodeSet: new Set(),
      heatmapMode: false,
      heatmapScores: new Map(),
      aiTelemetry: null,
      aiLastTimestamp: 0,
      aiAutoCam: true,
      aiTrail: [],
      aiSupernova: null,
      aiChatHistory: [],
      aiChatAutoScroll: true,
      settings: (function() {
        const defaults = {
          maxLabelLength: 18,
          lazyReadmes: true,
          fpsLimit: '120',
          sound: true,
          hideBubbleLabelsOver20: true,
          clusterLabelThreshold: 20,
          exclusions: ['tests', '*.spec.*', '*.test.*']
        };
        try {
          const parsed = JSON.parse(localStorage.getItem('aetherscope_settings'));
          return Object.assign({}, defaults, parsed);
        } catch (_) {
          return defaults;
        }
      })(),
      physicsSleeping: false,
      physicsCooldown: 120,
      cachedDocImpactZones: [],
      nodeMap: null,
      cachedFolderStats: null,
      docCache: new Map(),
      instanceDataDirty: true,
      lastRenderedSelId: null,
      lastRenderedHovId: null,
      lastRenderedLod: null,
      aiTelemetryNode: null,
      bubbleBirthMap: new Map(),
      waterRipples: [],
      filterBits: 31,
      project_path: ''
    };

    function wakePhysics(steps = 100) {
      state.physicsCooldown = Math.max(state.physicsCooldown || 0, steps);
      state.physicsSleeping = false;
      state.instanceDataDirty = true;
      state.cachedEdgePaths = null;
    }

    function invalidateEdgePathCache() {
      state.cachedEdgePaths = null;
      state.instanceDataDirty = true;
    }
    window.invalidateEdgePathCache = invalidateEdgePathCache;

    function precomputeDocMetadata(data) {
      if (!data) return;
      const nodes = data.nodes || [];
      const edges = data.edges || [];
      const nodeMap = new Map();
      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i];
        n._incidentEdges = [];
        nodeMap.set(n.id, n);
      }
      state.nodeMap = nodeMap;
      state.cachedEdgePaths = null;

      // Single-pass 10k+ Edge Pre-Indexer:
      // 1) Pre-resolves e._s and e._t references
      // 2) Builds O(1) node._incidentEdges adjacency lists for instant hover/selection
      // 3) Aggregates Inter-Cellular Synaptic Bundles (folderA <-> folderB) for 10k+ edge fluidity
      const docTargets = new Map();
      const bundleMap = new Map();
      const violationEdges = [];

      for (let i = 0; i < edges.length; i++) {
        const e = edges[i];
        e._idx = i;
        const s = nodeMap.get(e.source);
        const t = nodeMap.get(e.target);
        e._s = s || null;
        e._t = t || null;
        if (!s || !t) continue;

        if (
          e.relation === 'violates' ||
          e.filter_bits === 16 ||
          e.cve_id ||
          e.is_Constraint_Violation ||
          e.is_constraint_violation ||
          (e.relation && e.relation.toLowerCase().includes('violat'))
        ) {
          violationEdges.push(e);
        }

        s._incidentEdges.push(e);
        if (t !== s) t._incidentEdges.push(e);

        const sf = s.folder || 'root';
        const tf = t.folder || 'root';
        const isCross = sf !== tf;
        e._isCrossFolder = isCross;

        if (e.relation === 'documents') {
          let list = docTargets.get(e.source);
          if (!list) { list = []; docTargets.set(e.source, list); }
          list.push(t);
        } else if (isCross) {
          const key = sf < tf ? `${sf}|||${tf}` : `${tf}|||${sf}`;
          let b = bundleMap.get(key);
          if (!b) {
            b = {
              folderA: sf < tf ? sf : tf,
              folderB: sf < tf ? tf : sf,
              count: 0,
              importsCount: 0,
              submitsCount: 0,
              callsCount: 0,
              layers: new Set()
            };
            bundleMap.set(key, b);
          }
          b.count++;
          b.layers.add(s.layer);
          b.layers.add(t.layer);
          if (e.relation === 'imports') b.importsCount++;
          else if (e.relation === 'submits' || e.relation === 'network') b.submitsCount++;
          else b.callsCount++;
        }
      }
      state.folderBundles = Array.from(bundleMap.values());
      state.violationEdges = violationEdges;

      // Group nodes by folder: folder -> code node[]
      const folderNodes = new Map();
      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i];
        const f = n.folder || 'root';
        let list = folderNodes.get(f);
        if (!list) { list = []; folderNodes.set(f, list); }
        list.push(n);
      }

      // Precompute doc impact records
      const docImpactZones = [];
      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i];
        const isDoc = n.layer === 'docs' || n.symbol_type === 'document' || (n.file_path && n.file_path.endsWith('.md'));
        if (!isDoc) continue;

        const f = n.folder || 'root';
        const targets = docTargets.get(n.id) || [];
        const fNodes = (folderNodes.get(f) || []).filter(fn => fn.id !== n.id);
        const set = new Set([...targets, ...fNodes]);
        const allAssociated = Array.from(set);
        if (allAssociated.length === 0) continue;

        const keywordMatchCount = targets.length;
        const totalCodeNodes = fNodes.length;
        const coverageRatio = Math.min(1.0, keywordMatchCount / (totalCodeNodes || 1));

        docImpactZones.push({
          doc: n,
          folder: f,
          allAssociated,
          keywordMatchCount,
          totalCodeNodes,
          coverageRatio
        });
      }
      state.cachedDocImpactZones = docImpactZones;
      state.cachedFolderStats = null;
    }

    function truncateNodeName(name, maxLen = 18) {
      if (!name) return '';
      if (!maxLen || maxLen <= 0 || name.length <= maxLen) return name;
      return name.slice(0, Math.max(1, maxLen - 1)) + '…';
    }

    async function fetchDocContentOnDemand(docNode) {
      if (!docNode) return '';
      if (docNode.metadata && docNode.metadata.raw_content) {
        return docNode.metadata.raw_content;
      }
      const path = docNode.file_path || (docNode.metadata && docNode.metadata.file_path);
      if (path && state.docCache && state.docCache.has(path)) {
        return state.docCache.get(path);
      }
      if (!path) {
        return docNode.docstring || `# ${docNode.name}\nDocumentation`;
      }
      try {
        const res = await fetch(`/api/file_content?path=${encodeURIComponent(path)}`);
        if (res.ok) {
          const data = await res.json();
          if (data.status === 'ok' && data.content) {
            if (!state.docCache) state.docCache = new Map();
            state.docCache.set(path, data.content);
            if (!docNode.metadata) docNode.metadata = {};
            docNode.metadata.raw_content = data.content;
            return data.content;
          }
        }
      } catch (err) {
        console.warn('Erreur lazy-load doc:', err);
      }
      return docNode.docstring || `# ${docNode.name}\nDocumentation`;
    }

    function addToAiTrail(node, action) {
      if (!node) return;
      const last = state.aiTrail[state.aiTrail.length - 1];
      if (last && last.id === node.id) return;
      state.aiTrail.push({
        id: node.id,
        name: node.name,
        x: node.x,
        y: node.y,
        action: action || 'Navigation',
        timestamp: Date.now()
      });
      if (state.aiTrail.length > 12) {
        state.aiTrail.shift();
      }
    }

    const LAYER_COLORS = {
      ui: [0.98, 0.42, 0.75, 1.0],      // Neon Pink #fb7185
      action: [0.22, 0.78, 0.99, 1.0],  // Electric Cyan #38bdf8
      domain: [0.99, 0.76, 0.16, 1.0],  // Solar Gold #fbbf24
      database: [0.55, 0.58, 0.99, 1.0],// Cosmic Indigo #818cf8
      config: [0.38, 0.88, 0.98, 1.0],  // Celestial Ice Cyan
      utility: [0.75, 0.55, 0.99, 1.0], // Plasma Violet #c084fc
      docs: [0.10, 0.88, 0.58, 1.0]     // Radiant Emerald #10b981
    };

    // Deterministic Cosmic Palette for Folder Clusters (LoD 1)
    const COSMIC_FOLDER_HUES = [
      [0.22, 0.82, 0.99, 1.0], // Electric Cyan
      [0.98, 0.42, 0.75, 1.0], // Neon Pink
      [0.99, 0.78, 0.18, 1.0], // Solar Gold
      [0.55, 0.60, 0.99, 1.0], // Cosmic Indigo
      [0.10, 0.88, 0.58, 1.0], // Radiant Emerald
      [0.78, 0.50, 0.99, 1.0], // Plasma Violet
      [0.99, 0.45, 0.35, 1.0], // Nova Coral
      [0.18, 0.92, 0.82, 1.0], // Hyper Turquoise
      [0.99, 0.88, 0.25, 1.0]  // Pulsar Yellow
    ];

    function getFolderColor(folder) {
      if (!folder || folder === 'root') return [0.38, 0.88, 0.98, 1.0];
      if (!state.folderPalettes.has(folder)) {
        let hash = 0;
        for (let i = 0; i < folder.length; i++) hash = (hash * 31 + folder.charCodeAt(i)) & 0xffffffff;
        const idx = Math.abs(hash) % COSMIC_FOLDER_HUES.length;
        state.folderPalettes.set(folder, COSMIC_FOLDER_HUES[idx]);
      }
      return state.folderPalettes.get(folder);
    }

    // Clean, compact folder label formatter (respects the < 20-22 chars rule)
    function formatFolderLabel(folder) {
      if (!folder || folder === 'root') return 'root';
      const clean = folder.replace(/\\/g, '/');
      const parts = clean.split('/').filter(Boolean);
      if (parts.length <= 1) {
        return clean.length > 20 ? clean.slice(0, 18) + '…' : clean;
      }
      const last = parts[parts.length - 1];
      const prev = parts[parts.length - 2];
      const shortPath = `${prev}/${last}`;
      if (shortPath.length <= 22) return shortPath;
      return last.length > 20 ? last.slice(0, 18) + '…' : last;
    }

    function getHeatmapColor(normScore) {
      // Smooth gradient:
      // 0.0 - 0.4: Green (#10b981) -> Amber (#fbbf24)
      // 0.4 - 1.0: Amber (#fbbf24) -> Crimson Red (#ef4444)
      if (normScore <= 0.4) {
        const t = normScore / 0.4;
        return [
          0.10 + (0.99 - 0.10) * t,
          0.88 + (0.76 - 0.88) * t,
          0.58 + (0.16 - 0.58) * t,
          1.0
        ];
      } else {
        const t = (normScore - 0.4) / 0.6;
        return [
          0.99 + (0.95 - 0.99) * t,
          0.76 + (0.22 - 0.76) * t,
          0.16 + (0.22 - 0.16) * t,
          1.0
        ];
      }
    }

    function showToast(msg) {
      const t = document.getElementById('toast');
      t.innerText = msg;
      t.classList.add('show');
      setTimeout(() => t.classList.remove('show'), 2400);
    }

    // --- Procedural Cosmic Web Audio Synthesizer ---
    const AudioSys = {
      ctx: null,
      enabled: false,
      init() {
        if (!this.ctx && typeof AudioContext !== 'undefined') {
          this.ctx = new (window.AudioContext || window.webkitAudioContext)();
        }
        if (this.ctx && this.ctx.state === 'suspended') {
          this.ctx.resume();
        }
      },
      playTone(freq, type = 'sine', duration = 0.22, gainVal = 0.04) {
        if (!this.enabled || !this.ctx) return;
        try {
          const osc = this.ctx.createOscillator();
          const gain = this.ctx.createGain();
          osc.type = type;
          osc.frequency.setValueAtTime(freq, this.ctx.currentTime);
          gain.gain.setValueAtTime(gainVal, this.ctx.currentTime);
          gain.gain.exponentialRampToValueAtTime(0.0001, this.ctx.currentTime + duration);
          osc.connect(gain);
          gain.connect(this.ctx.destination);
          osc.start();
          osc.stop(this.ctx.currentTime + duration);
        } catch(e) {}
      },
      nodeSelect() {
        this.playTone(520, 'sine', 0.22, 0.05);
        setTimeout(() => this.playTone(780, 'triangle', 0.18, 0.03), 40);
      },
      beaconPulse() {
        this.playTone(880, 'sine', 0.35, 0.07);
        setTimeout(() => this.playTone(1320, 'sine', 0.25, 0.04), 70);
      },
      depthSwitch() {
        this.playTone(330, 'sine', 0.25, 0.06);
        setTimeout(() => this.playTone(660, 'sine', 0.25, 0.04), 80);
      },
      xraySweep() {
        this.playTone(440, 'triangle', 0.15, 0.05);
        setTimeout(() => this.playTone(880, 'sine', 0.20, 0.06), 60);
        setTimeout(() => this.playTone(1760, 'sine', 0.35, 0.04), 120);
      },
      supernovaBirth() {
        this.playTone(392, 'sine', 0.22, 0.06);
        setTimeout(() => this.playTone(523, 'triangle', 0.28, 0.08), 80);
        setTimeout(() => this.playTone(659, 'sine', 0.35, 0.08), 180);
        setTimeout(() => this.playTone(1046, 'sine', 0.50, 0.09), 320);
        setTimeout(() => this.playTone(1318, 'sine', 0.65, 0.06), 480);
      },
      deadCodeScan() {
        this.playTone(320, 'sawtooth', 0.16, 0.05);
        setTimeout(() => this.playTone(220, 'sawtooth', 0.24, 0.04), 70);
      },
      heatmapPulse() {
        this.playTone(420, 'sine', 0.16, 0.05);
        setTimeout(() => this.playTone(630, 'sine', 0.18, 0.05), 60);
        setTimeout(() => this.playTone(840, 'sine', 0.22, 0.04), 120);
      },
      promptCopied() {
        this.playTone(880, 'sine', 0.14, 0.06);
        setTimeout(() => this.playTone(1760, 'sine', 0.24, 0.06), 60);
      },
      waterBubblePop(seed = 1) {
        if (!this.enabled) return;
        this.init();
        if (!this.ctx) return;
        try {
          const baseFreq = 380 + (Math.sin(seed * 997.3) * 0.5 + 0.5) * 240;
          const osc = this.ctx.createOscillator();
          const gain = this.ctx.createGain();
          osc.type = 'sine';
          const t = this.ctx.currentTime;
          osc.frequency.setValueAtTime(baseFreq, t);
          osc.frequency.exponentialRampToValueAtTime(baseFreq * 1.55, t + 0.075);
          gain.gain.setValueAtTime(0.16, t);
          gain.gain.exponentialRampToValueAtTime(0.0001, t + 0.085);
          osc.connect(gain);
          gain.connect(this.ctx.destination);
          osc.start(t);
          osc.stop(t + 0.085);
        } catch(e) {}
      }
    };

    document.getElementById('btn-audio')?.addEventListener('click', () => {
      AudioSys.init();
      AudioSys.enabled = !AudioSys.enabled;
      const b = document.getElementById('btn-audio');
      if (b) {
        b.innerText = AudioSys.enabled ? '🔊 Audio ON' : '🔇 Audio OFF';
        b.classList.toggle('primary', AudioSys.enabled);
      }
      showToast(AudioSys.enabled ? 'Audio spatialisé actif' : 'Audio désactivé');
    });

const AetherState = state;
window.AetherState = state;
window.state = state;


// =========================================================================
// AETHERSCOPE V2 — CENTRALIZED API CLIENT (AetherAPI)
// Connects to all Axum backend endpoints in api_routes.rs
// =========================================================================
const AetherAPI = {
  async fetchGraph() {
    const res = await fetch('/api/graph');
    return res.json();
  },
  async fetchGraphVersion() {
    const res = await fetch('/api/graph/version');
    return res.json();
  },
  async fetchProjects() {
    const res = await fetch('/api/projects');
    return res.json();
  },
  async getWorkspace() {
    const res = await fetch('/api/workspace');
    return res.json();
  },
  async setWorkspace(workspace) {
    const res = await fetch('/api/workspace', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ workspace })
    });
    return res.json();
  },
  async pickWorkspace() {
    const res = await fetch('/api/dialog/pick_workspace', { method: 'POST' });
    return res.json();
  },
  async pickProject() {
    const res = await fetch('/api/dialog/pick_project', { method: 'POST' });
    return res.json();
  },
  async analyze(path, exclusions = null) {
    const activeExclusions = exclusions || state.settings?.exclusions || [];
    const res = await fetch('/api/analyze', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path, exclusions: activeExclusions })
    });
    return res.json();
  },
  async getSettings() {
    const res = await fetch('/api/settings');
    return res.json();
  },
  async saveSettings(settings) {
    const res = await fetch('/api/settings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings)
    });
    return res.json();
  },
  async initProject(path, stack) {
    const res = await fetch('/api/init_project', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path, stack })
    });
    return res.json();
  },
  async exportHtml(path) {
    const res = await fetch('/api/export', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path })
    });
    return res.json();
  },
  async saveDoc(projectPath, folder, fileName, content) {
    const res = await fetch('/api/save_doc', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        project_path: projectPath,
        folder,
        file_name: fileName,
        content
      })
    });
    return res.json();
  },
  async fetchFileContent(path) {
    const res = await fetch(`/api/file_content?path=${encodeURIComponent(path)}`);
    return res.json();
  },
  async sendTelemetry(target, action, details) {
    const res = await fetch('/api/ai/telemetry', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ target, action, details })
    });
    return res.json();
  },
  async getTelemetry() {
    const res = await fetch('/api/ai/telemetry');
    return res.json();
  },
  async fetchChatMessages() {
    const res = await fetch('/api/ai/chat');
    return res.json();
  },
  async sendChatMessage(text, sender = 'user', targetSymbol = null) {
    const res = await fetch('/api/ai/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text, sender, target_symbol: targetSymbol })
    });
    return res.json();
  },
  async clearChatMessages() {
    const res = await fetch('/api/ai/chat/clear', { method: 'POST' });
    return res.json();
  },
  async getAiStatus() {
    const res = await fetch('/api/ai/status');
    return res.json();
  },
  async cancelAi() {
    const res = await fetch('/api/ai/cancel', { method: 'POST' });
    return res.json();
  },
  async newAiSession() {
    const res = await fetch('/api/ai/new_session', { method: 'POST' });
    return res.json();
  },
  async fetchTasks() {
    try {
      const res = await fetch('/api/tasks');
      return await res.json();
    } catch (e) {
      console.warn('Erreur fetchTasks:', e);
      return { status: 'error', tasks: [] };
    }
  },
  async createTask(title, kind = 'function', targetSymbol = null, targetFile = null, priority = 'medium', initialReadme = null) {
    const res = await fetch('/api/tasks', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title,
        kind,
        target_symbol: targetSymbol,
        target_file: targetFile,
        author: 'user',
        priority,
        initial_readme: initialReadme
      })
    });
    return res.json();
  },
  async updateTask(taskId, status = null, progress = null, progressNote = null, fullReadme = null) {
    const res = await fetch('/api/tasks/update', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        task_id: taskId,
        status,
        progress,
        progress_note: progressNote,
        full_readme: fullReadme
      })
    });
    return res.json();
  },
  async deleteTask(taskId) {
    const res = await fetch('/api/tasks/delete', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ task_id: taskId })
    });
    return res.json();
  },
  async launchCli(tool, projectPath, prompt) {
    const res = await fetch('/api/ai/launch_cli', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        tool,
        project_path: projectPath,
        prompt
      })
    });
    return res.json();
  },
  async fetchSystemPrompt() {
    const res = await fetch('/api/ai/system_prompt');
    return res.json();
  },
  async createFile(projectPath, filePath, content) {
    const res = await fetch('/api/ai/create_file', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        project_path: projectPath,
        file_path: filePath,
        content
      })
    });
    return res.json();
  },
  async signContracts(projectPath, folder = null) {
    const res = await fetch('/api/contracts/sign', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        project_path: projectPath,
        folder: folder || null
      })
    });
    return res.json();
  },
  async aiTarget(symbolId, actionType = 'inspect') {
    const res = await fetch('/api/ai/target', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        symbol_id: symbolId,
        action_type: actionType
      })
    });
    return res.json();
  }
};

window.AetherAPI = AetherAPI;
