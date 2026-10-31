from typing import Dict, List, Optional
import uuid

STORE_ITEMS = [
    {
        "id": "rank_vip",
        "category": "grades",
        "name": "Grade VIP",
        "badge": "VIP",
        "badge_color": "emerald",
        "price": 4.99,
        "currency": "EUR",
        "icon": "👑",
        "description": "Le grade idéal pour commencer l'aventure avec confort et style.",
        "perks": [
            "Préfixe [VIP] vert dans le chat et le tablist",
            "Accès aux commandes /fly en lobby et /feed",
            "+2 Homes supplémentaires (/sethome)",
            "Kit VIP hebdomadaire (Armure fer P2 + 16 Pommes dorées)",
            "Accès prioritaire aux serveurs pleins"
        ]
    },
    {
        "id": "rank_aether",
        "category": "grades",
        "name": "Grade Aether (Héros)",
        "badge": "AETHER",
        "badge_color": "sky",
        "price": 14.99,
        "currency": "EUR",
        "icon": "🪐",
        "popular": True,
        "description": "Le grade prestigieux conçu pour les maîtres du monde Aether.",
        "perks": [
            "Préfixe [AETHER] cyan scintillant animé",
            "Commandes /fly partout en survie + /workbench + /anvil",
            "+10 Homes personnalisables",
            "Kit Aether légendaire (Armure diamant P4 + Pioche Fortune III)",
            "Accès exclusif au canal Discord VIP & Salons Vocaux",
            "2 Clés Mystiques offertes chaque mois"
        ]
    },
    {
        "id": "rank_titan",
        "category": "grades",
        "name": "Grade Titan Primordial",
        "badge": "TITAN",
        "badge_color": "amber",
        "price": 29.99,
        "currency": "EUR",
        "icon": "⚡",
        "description": "La puissance ultime avec tous les privilèges et cosmétiques débloqués.",
        "perks": [
            "Préfixe [TITAN] or flamboyant dans le chat",
            "Commandes /god en zone paisible + /nick personnalisé",
            "Homes illimités & Accès au coffre Ender géant (/enderchest)",
            "Kit Titan Netheirite enchanté complet toutes les 2 semaines",
            "Cape cosmétique cosmologique exclusive Aether",
            "Priorité absolue et multiplicateur x2 d'expérience"
        ]
    },
    {
        "id": "key_mystic_bundle",
        "category": "keys",
        "name": "Pack 5x Clés Célestes",
        "badge": "CLÉS",
        "badge_color": "purple",
        "price": 3.49,
        "currency": "EUR",
        "icon": "🗝️",
        "description": "Ouvrez la Caisse des Étoiles au spawn et tentez de remporter des artéfacts rares.",
        "perks": [
            "5 Clés utilisables instantanément au Spawn",
            "Butins possibles : Armes légendaires, Spawners, Tokens",
            "Taux de drop d'items mythiques augmenté de 15%"
        ]
    },
    {
        "id": "key_omega_bundle",
        "category": "keys",
        "name": "Pack 12x Clés Oméga (Max)",
        "badge": "MEILLEURE OFFRE",
        "badge_color": "rose",
        "price": 7.99,
        "currency": "EUR",
        "icon": "💎",
        "description": "Le pack ultime pour les chasseurs de trésors et pilleurs de donjons.",
        "perks": [
            "12 Clés de la Caisse Céleste",
            "Garantie d'au moins 1 item légendaire ou spawner rare",
            "+500 Aether Gems bonus offerts sur votre compte"
        ]
    },
    {
        "id": "cosmetic_wings",
        "category": "cosmetics",
        "name": "Ailes Quantiques Stellaires",
        "badge": "COSMÉTIQUE",
        "badge_color": "indigo",
        "price": 4.50,
        "currency": "EUR",
        "icon": "🪽",
        "description": "Paire d'ailes luminescentes animées flottant dans votre dos.",
        "perks": [
            "Visibilité par tous les joueurs sur le serveur (sans mods)",
            "Particules d'étoiles filantes lors des déplacements",
            "Désactivable à tout moment via /cosmetics"
        ]
    },
    {
        "id": "cosmetic_dragon_pet",
        "category": "cosmetics",
        "name": "Mini Dragon de l'Aether (Familier)",
        "badge": "PET",
        "badge_color": "violet",
        "price": 6.00,
        "currency": "EUR",
        "icon": "🐉",
        "description": "Un familier loyal qui vole à vos côtés et rugit joyeusement.",
        "perks": [
            "Familier miniature qui vous suit fidèlement",
            "Animations personnalisées et petits bruits de bébé dragon",
            "Nom personnalisable avec couleurs"
        ]
    }
]

ORDERS_HISTORY = []

def get_store_items(category: Optional[str] = None) -> List[Dict]:
    if not category or category == "all":
        return STORE_ITEMS
    return [item for item in STORE_ITEMS if item.get("category") == category]

def get_item_by_id(item_id: str) -> Optional[Dict]:
    for item in STORE_ITEMS:
        if item["id"] == item_id:
            return item
    return None

def process_checkout(username: str, cart_items: List[Dict]) -> Dict:
    total = sum(item["price"] * item.get("quantity", 1) for item in cart_items)
    order_id = f"CMD-{str(uuid.uuid4())[:8].upper()}"
    order = {
        "order_id": order_id,
        "username": username,
        "total": round(total, 2),
        "currency": "EUR",
        "products": cart_items,
        "items_count": len(cart_items),
        "status": "LIVRÉ",
        "command_to_run": [
            f"lp user {username} parent add {item['id'].replace('rank_', '')}" 
            if item['id'].startswith("rank_") else f"give {username} {item['id']}"
            for item in cart_items
        ]
    }
    ORDERS_HISTORY.insert(0, order)
    return order
