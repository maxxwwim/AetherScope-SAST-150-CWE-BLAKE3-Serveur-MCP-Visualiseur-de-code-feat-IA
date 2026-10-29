use crate::bubble::{Point2D, SymbolBubble};
use std::collections::HashMap;

const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653; // ~137.507764 degrees

pub fn compute_all_layouts(nodes: &mut [SymbolBubble]) {
    if nodes.is_empty() {
        return;
    }

    compute_obsidian_layout(nodes);
    compute_system_layout(nodes);
    compute_flow_layout(nodes);
    compute_classes_layout(nodes);

    // Initial default position = Obsidian
    for n in nodes.iter_mut() {
        if let Some(obs) = n.layouts.obsidian {
            n.x = obs.x;
            n.y = obs.y;
        }
    }
}

/// 1. Z-1 : Obsidian Galaxy (Clusters nébuleux organiques en spirale dorée de Fermat)
pub fn compute_obsidian_layout(nodes: &mut [SymbolBubble]) {
    let mut dir_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, n) in nodes.iter().enumerate() {
        let parts: Vec<&str> = n.file_path.split('/').collect();
        let dir = if parts.len() > 1 {
            parts[..parts.len() - 1].join("/")
        } else {
            "root".to_string()
        };
        dir_groups.entry(dir).or_default().push(idx);
    }

    let mut sorted_dirs: Vec<(String, Vec<usize>)> = dir_groups.into_iter().collect();
    sorted_dirs.sort_by(|a, b| a.0.cmp(&b.0));

    let num_dirs = sorted_dirs.len();
    // Vastly increased spacing between directory nebulas so each cell has its own clear celestial territory
    let cluster_spacing = if num_dirs > 30 { 440.0 } else { 520.0 };

    for (dir_idx, (_d_name, indices)) in sorted_dirs.into_iter().enumerate() {
        let dir_dist = (dir_idx as f64 + 0.6).sqrt() * cluster_spacing;
        let dir_angle = dir_idx as f64 * GOLDEN_ANGLE;
        let dir_cx = dir_angle.cos() * dir_dist;
        let dir_cy = dir_angle.sin() * dir_dist;

        let n_len = indices.len();
        // Wider intra-cellular radius so functions and files breathe freely without stacking
        let cluster_radius = ((n_len as f64).sqrt() * 62.0 + 55.0).clamp(90.0, 580.0);

        for (i, &n_idx) in indices.iter().enumerate() {
            let node_dist = ((i as f64 + 0.5) / n_len.max(1) as f64).sqrt() * cluster_radius;
            let node_angle = i as f64 * GOLDEN_ANGLE;

            nodes[n_idx].layouts.obsidian = Some(Point2D::new(
                (dir_cx + node_angle.cos() * node_dist).round(),
                (dir_cy + node_angle.sin() * node_dist).round(),
            ));
        }
    }
}

/// 2. Z-2 : Architecture Système (Constellations équilibrées par couches hiérarchiques)
pub fn compute_system_layout(nodes: &mut [SymbolBubble]) {
    let layer_tiers: HashMap<&str, f64> = [
        ("docs", -1050.0),
        ("ui", -760.0),
        ("action", -300.0),
        ("domain", 260.0),
        ("database", 820.0),
        ("config", -530.0),
        ("utility", 540.0),
    ]
    .into_iter()
    .collect();

    let mut layer_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, n) in nodes.iter().enumerate() {
        layer_groups.entry(n.layer.clone()).or_default().push(idx);
    }

    for (layer, indices) in layer_groups {
        let base_x = layer_tiers.get(layer.as_str()).copied().unwrap_or(0.0);
        let count = indices.len();
        let sub_cols = ((count as f64).sqrt() * 0.7).ceil().max(1.0) as usize;

        for (idx, &n_idx) in indices.iter().enumerate() {
            let col = idx % sub_cols;
            let row = idx / sub_cols;
            let total_rows = (count + sub_cols - 1) / sub_cols;

            let x = base_x + (col as f64 - (sub_cols - 1) as f64 / 2.0) * 95.0;
            let y = (row as f64 - total_rows as f64 / 2.0) * 105.0;

            nodes[n_idx].layouts.system = Some(Point2D::new(x.round(), y.round()));
        }
    }
}

/// 3. Z-3 : Flux d'Exécution DAG (Horizontal par étapes de pipeline)
pub fn compute_flow_layout(nodes: &mut [SymbolBubble]) {
    let mut stage_groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, n) in nodes.iter().enumerate() {
        stage_groups.entry(n.stage_idx).or_default().push(idx);
    }

    for stage_idx in 0..4 {
        if let Some(indices) = stage_groups.get(&stage_idx) {
            let stage_x = (stage_idx as f64 - 1.5) * 560.0;
            let count = indices.len();
            let sub_cols = ((count as f64).sqrt() * 0.6).ceil().max(1.0) as usize;

            for (idx, &n_idx) in indices.iter().enumerate() {
                let col = idx % sub_cols;
                let row = idx / sub_cols;
                let total_rows = (count + sub_cols - 1) / sub_cols;

                let x = stage_x + (col as f64 - (sub_cols - 1) as f64 / 2.0) * 90.0;
                let y = (row as f64 - total_rows as f64 / 2.0) * 100.0;

                nodes[n_idx].layouts.flow = Some(Point2D::new(x.round(), y.round()));
            }
        }
    }
}

/// 4. Z-4 : Classes & Composants UML (Grille planétaire matricielle)
pub fn compute_classes_layout(nodes: &mut [SymbolBubble]) {
    let total_nodes = nodes.len();
    if total_nodes == 0 {
        return;
    }
    let cols = ((total_nodes as f64 * 1.4).sqrt().ceil() as usize).max(2);
    let spacing_x = 340.0;
    let spacing_y = 160.0;

    for idx in 0..total_nodes {
        let r = idx / cols;
        let c = idx % cols;
        let x = (c as f64 - cols as f64 / 2.0) * spacing_x;
        let y = (r as f64 - (total_nodes / cols) as f64 / 2.0) * spacing_y;

        nodes[idx].layouts.classes = Some(Point2D::new(x.round(), y.round()));
    }
}
