import random
from typing import Dict

SERVER_INFO = {
    "ip": "play.aethercraft.net",
    "port": 25565,
    "local_ip": "127.0.0.1:25565",
    "motd": "§b🪐 AetherCraft §7— Survie Hardcore & Donjons Stellaires [1.20 - 1.21.x]",
    "version": "Paper 1.21.1",
    "max_players": 250,
    "discord_url": "https://discord.gg/aethercraft",
    "runner_download_cmd": "cargo run -p aether-mc-runner -- install-and-run --manifest http://127.0.0.1:5000/api/v1/servers/default/manifest"
}

def get_live_server_status() -> Dict:
    """
    Retourne le statut en direct du serveur avec simulation d'activité réaliste.
    """
    online_count = random.randint(118, 142)
    ping_ms = random.randint(18, 34)
    return {
        "status": "online",
        "ip": SERVER_INFO["ip"],
        "port": SERVER_INFO["port"],
        "local_ip": SERVER_INFO["local_ip"],
        "motd": SERVER_INFO["motd"],
        "version": SERVER_INFO["version"],
        "players_online": online_count,
        "players_max": SERVER_INFO["max_players"],
        "ping_ms": ping_ms,
        "tps": 20.0,
        "runner_cmd": SERVER_INFO["runner_download_cmd"]
    }
