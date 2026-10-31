from typing import Dict, List, Optional
import datetime

# Joueurs d'exemple pour le classement
MOCK_PLAYERS = [
    {
        "username": "AetherKnight",
        "rank": "TITAN",
        "rank_color": "amber",
        "hours_played": 342,
        "balance": 184500,
        "kills": 892,
        "deaths": 114,
        "is_online": True,
        "guild": "Légion Stellaire",
        "joined": "2026-03-15"
    },
    {
        "username": "PixelSorceress",
        "rank": "AETHER",
        "rank_color": "sky",
        "hours_played": 289,
        "balance": 142000,
        "kills": 430,
        "deaths": 62,
        "is_online": True,
        "guild": "Gardiens d'Aether",
        "joined": "2026-04-02"
    },
    {
        "username": "VortexCrafter",
        "rank": "VIP",
        "rank_color": "emerald",
        "hours_played": 195,
        "balance": 98400,
        "kills": 312,
        "deaths": 89,
        "is_online": False,
        "guild": "Forge Quantique",
        "joined": "2026-05-18"
    },
    {
        "username": "ShadowRanger",
        "rank": "JOUEUR",
        "rank_color": "slate",
        "hours_played": 87,
        "balance": 34100,
        "kills": 154,
        "deaths": 73,
        "is_online": True,
        "guild": "Solitaire",
        "joined": "2026-08-10"
    },
    {
        "username": "Notch",
        "rank": "CRÉATEUR",
        "rank_color": "rose",
        "hours_played": 999,
        "balance": 999999,
        "kills": 1337,
        "deaths": 0,
        "is_online": False,
        "guild": "Mojang",
        "joined": "2026-01-01"
    }
]

def get_leaderboards() -> Dict[str, List[Dict]]:
    """Classements des meilleurs joueurs par richesse et temps de jeu."""
    by_balance = sorted(MOCK_PLAYERS, key=lambda x: x["balance"], reverse=True)
    by_hours = sorted(MOCK_PLAYERS, key=lambda x: x["hours_played"], reverse=True)
    return {
        "top_rich": by_balance[:5],
        "top_hours": by_hours[:5]
    }

def get_player_profile(username: str) -> Dict:
    """Retourne la fiche du joueur ou génère dynamiquement ses stats."""
    for p in MOCK_PLAYERS:
        if p["username"].lower() == username.lower():
            return {
                **p,
                "avatar_url": f"https://mc-heads.net/avatar/{p['username']}/96",
                "body_url": f"https://mc-heads.net/body/{p['username']}/right"
            }

    # Si c'est un joueur recherché non présent dans les mocks, on génère une fiche dynamique
    # avec la vraie texture de skin Minecraft via l'API mc-heads
    return {
        "username": username,
        "rank": "JOUEUR",
        "rank_color": "slate",
        "hours_played": 12,
        "balance": 2500,
        "kills": 18,
        "deaths": 4,
        "is_online": False,
        "guild": "Nouvel Aventurier",
        "joined": "2026-10-30",
        "avatar_url": f"https://mc-heads.net/avatar/{username}/96",
        "body_url": f"https://mc-heads.net/body/{username}/right"
    }
