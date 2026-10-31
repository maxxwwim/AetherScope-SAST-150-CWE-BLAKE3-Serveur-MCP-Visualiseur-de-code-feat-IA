# 📜 Spécification du Manifest Aether Minecraft (v1.0)

## 1. Vue d'ensemble
Le **Manifest Aether** est le contrat de données immuable généré par l'orchestrateur Python Flask et consommé par le runner natif Rust (`aether-mc-runner`).
Il décrit exhaustivement :
- La version de Java et les flags de démarrage JVM (Aikar Garbage Collection).
- Le binaire serveur (Paper, Purpur, Fabric, Vanilla) et son checksum de sécurité.
- Les plugins / mods requis avec leurs URLs de téléchargement et chemins relatifs.
- Les fichiers de configuration essentiels (`eula.txt`, `server.properties`, etc.).

---

## 2. Schéma JSON

```json
{
  "$schema": "https://aether.local/schemas/mc-manifest-v1.json",
  "version": "1.0",
  "server_id": "srv_survival_01",
  "name": "Aether Craft Survival",
  "runtime": {
    "java_major_version": 21,
    "vendor": "adoptium",
    "heap_min": "4G",
    "heap_max": "4G",
    "jvm_flags": [
      "-XX:+UseG1GC",
      "-XX:+ParallelRefProcEnabled",
      "-XX:MaxGCPauseMillis=200",
      "-XX:+UnlockExperimentalVMOptions",
      "-XX:+DisableExplicitGC",
      "-XX:+AlwaysPreTouch",
      "-XX:G1NewSizePercent=30",
      "-XX:G1MaxNewSizePercent=40",
      "-XX:G1ReservePercent=20",
      "-Dterminal.jline=false",
      "-Dterminal.ansi=true"
    ]
  },
  "core": {
    "type": "paper",
    "mc_version": "1.21.1",
    "build": 118,
    "target_jar": "paper-server.jar",
    "download_url": "https://api.papermc.io/v2/projects/paper/versions/1.21.1/builds/118/downloads/paper-1.21.1-118.jar",
    "sha256": ""
  },
  "plugins_and_mods": [
    {
      "name": "Chunky.jar",
      "target_dir": "plugins",
      "download_url": "https://cdn.modrinth.com/data/fALzjamp/versions/R1Kxvd8Z/Chunky-1.4.28.jar",
      "sha256": ""
    }
  ],
  "configs": {
    "eula.txt": "eula=true\n",
    "server.properties": "server-port=25565\nmotd=§bAetherCraft §7Server\nonline-mode=true\ndifficulty=hard\npvp=true\nview-distance=10\nmax-players=20\n"
  }
}
```

---

## 3. Typage Rust (`serde::Deserialize`)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerManifest {
    pub version: String,
    pub server_id: String,
    pub name: String,
    pub runtime: RuntimeConfig,
    pub core: CoreConfig,
    #[serde(default)]
    pub plugins_and_mods: Vec<ArtifactConfig>,
    #[serde(default)]
    pub configs: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeConfig {
    pub java_major_version: u32,
    pub vendor: String,
    pub heap_min: String,
    pub heap_max: String,
    pub jvm_flags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoreConfig {
    pub r#type: String,
    pub mc_version: String,
    pub build: u64,
    pub target_jar: String,
    pub download_url: String,
    #[serde(default)]
    pub sha256: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactConfig {
    pub name: String,
    pub target_dir: String,
    pub download_url: String,
    #[serde(default)]
    pub sha256: String,
}
```
