import requests
from typing import Dict, Any, Optional

PAPER_API_BASE = "https://api.papermc.io/v2"

def get_paper_build_info(version: str = "1.21.1") -> Dict[str, Any]:
    """
    Récupère le dernier build PaperMC pour la version spécifiée via l'API officielle.
    En cas d'échec réseau, renvoie une structure de repli valide.
    """
    try:
        url = f"{PAPER_API_BASE}/projects/paper/versions/{version}"
        res = requests.get(url, timeout=5)
        if res.status_code == 200:
            data = res.json()
            builds = data.get("builds", [])
            if builds:
                latest_build = max(builds)
                build_url = f"{PAPER_API_BASE}/projects/paper/versions/{version}/builds/{latest_build}"
                b_res = requests.get(build_url, timeout=5)
                if b_res.status_code == 200:
                    b_data = b_res.json()
                    dl_name = b_data["downloads"]["application"]["name"]
                    sha256 = b_data["downloads"]["application"].get("sha256", "")
                    return {
                        "build": latest_build,
                        "download_url": f"{build_url}/downloads/{dl_name}",
                        "target_jar": "paper-server.jar",
                        "sha256": sha256
                    }
    except Exception as e:
        print(f"[WARN] PaperMC API non joignable ({e}), utilisation du fallback")

    # Fallback sécurisé
    return {
        "build": 118,
        "download_url": f"https://api.papermc.io/v2/projects/paper/versions/{version}/builds/118/downloads/paper-{version}-118.jar",
        "target_jar": "paper-server.jar",
        "sha256": ""
    }

def get_purpur_build_info(version: str = "1.21.1") -> Dict[str, Any]:
    """Récupère les informations de téléchargement de PurpurMC."""
    return {
        "build": 2300,
        "download_url": f"https://api.purpurmc.org/v2/purpur/{version}/latest/download",
        "target_jar": "purpur-server.jar",
        "sha256": ""
    }

def get_plugin_presets(plugin_keys: list[str]) -> list[Dict[str, str]]:
    """Catalogue de plugins pré-vérifiés et optimisés pour serveur Aether."""
    CATALOG = {
        "chunky": {
            "name": "Chunky-1.4.28.jar",
            "target_dir": "plugins",
            "download_url": "https://cdn.modrinth.com/data/fALzjamp/versions/R1Kxvd8Z/Chunky-1.4.28.jar",
            "sha256": ""
        },
        "luckperms": {
            "name": "LuckPerms-Bukkit-5.4.140.jar",
            "target_dir": "plugins",
            "download_url": "https://download.luckperms.net/1556/bukkit/loader/LuckPerms-Bukkit-5.4.140.jar",
            "sha256": ""
        },
        "spark": {
            "name": "spark-bukkit.jar",
            "target_dir": "plugins",
            "download_url": "https://sparkcdn.lucko.me/download/bukkit",
            "sha256": ""
        }
    }
    plugins = []
    for key in plugin_keys:
        k = key.lower().strip()
        if k in CATALOG:
            plugins.append(CATALOG[k])
    return plugins
