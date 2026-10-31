import os
import uuid
import re
from flask import Flask, render_template, request, jsonify, session, abort
from services.manifest_generator import build_server_manifest
from services.store_service import get_store_items, get_item_by_id, process_checkout, ORDERS_HISTORY
from services.player_service import get_leaderboards, get_player_profile
from services.server_status_service import get_live_server_status

app = Flask(__name__)
app.secret_key = "aethercraft_secret_session_key_2026"

# Cache en mémoire des manifests générés
MANIFEST_STORE = {}

def slugify(text: str) -> str:
    text = text.lower()
    text = re.sub(r'[^a-z0-9]+', '_', text).strip('_')
    return text or "server"

@app.context_processor
def inject_global_status():
    """Injecte le statut du serveur dans tous les templates pour la navbar et le footer."""
    return {
        "server_status": get_live_server_status()
    }

# ========================================================
# 1. VITRINE & ACCUEIL JOUEURS
# ========================================================

@app.route("/")
def index():
    """Page d'accueil vitrine du serveur."""
    return render_template("index.html")

# ========================================================
# 2. BOUTIQUE EN LIGNE (GRADES & ITEMS)
# ========================================================

@app.route("/boutique")
def shop():
    cat = request.args.get("cat", "all")
    items = get_store_items(cat)
    cart = session.get("cart", [])
    cart_total = sum(i["price"] for i in cart)
    return render_template("shop.html", items=items, current_cat=cat, cart=cart, cart_total=cart_total)

@app.route("/api/store/cart/add", methods=["POST"])
def add_to_cart():
    item_id = request.form.get("item_id")
    item = get_item_by_id(item_id)
    cart = session.get("cart", [])
    if item:
        cart.append(item)
        session["cart"] = cart
    cart_total = sum(i["price"] for i in cart)
    return render_template("partials/cart.html", cart=cart, cart_total=cart_total)

@app.route("/api/store/cart/remove", methods=["POST"])
def remove_from_cart():
    item_id = request.form.get("item_id")
    cart = session.get("cart", [])
    for idx, item in enumerate(cart):
        if item["id"] == item_id:
            cart.pop(idx)
            break
    session["cart"] = cart
    cart_total = sum(i["price"] for i in cart)
    return render_template("partials/cart.html", cart=cart, cart_total=cart_total)

@app.route("/api/store/cart/clear", methods=["POST"])
def clear_cart():
    session["cart"] = []
    return render_template("partials/cart.html", cart=[], cart_total=0.0)

@app.route("/api/store/checkout", methods=["POST"])
def checkout():
    username = request.form.get("username", "Steve").strip()
    cart = session.get("cart", [])
    if not cart:
        return render_template("partials/cart.html", cart=[], cart_total=0.0)
    
    order = process_checkout(username, cart)
    session["cart"] = []  # Vide le panier après validation
    return render_template("partials/cart.html", cart=[], cart_total=0.0, order_success=order)

# ========================================================
# 3. JOUEURS & CLASSEMENTS
# ========================================================

@app.route("/joueurs")
def players():
    leaderboards = get_leaderboards()
    default_player = get_player_profile("AetherKnight")
    return render_template("players.html", leaderboards=leaderboards, player=default_player)

@app.route("/api/players/search", methods=["POST"])
def search_player():
    username = request.form.get("username", "").strip()
    if not username:
        username = "Steve"
    player = get_player_profile(username)
    return render_template("partials/player_card.html", player=player)

# ========================================================
# 4. COCKPIT CRÉATEUR & ADMINISTRATION
# ========================================================

@app.route("/admin")
def admin_cockpit():
    return render_template("admin.html", orders=ORDERS_HISTORY)

@app.route("/api/v1/servers/generate", methods=["POST"])
def generate_server():
    name = request.form.get("name", "Serveur Minecraft").strip()
    core_type = request.form.get("core_type", "paper")
    mc_version = request.form.get("mc_version", "1.21.1")
    ram_gb = int(request.form.get("ram_gb", 4))
    port = int(request.form.get("port", 25565))
    difficulty = request.form.get("difficulty", "normal")
    plugins = request.form.getlist("plugins")

    server_id = f"srv_{slugify(name)}_{str(uuid.uuid4())[:6]}"

    # Construction du Manifest
    manifest = build_server_manifest(
        server_id=server_id,
        name=name,
        core_type=core_type,
        mc_version=mc_version,
        ram_gb=ram_gb,
        plugins=plugins,
        port=port,
        difficulty=difficulty
    )
    MANIFEST_STORE[server_id] = manifest

    manifest_url = f"http://127.0.0.1:5000/api/v1/servers/{server_id}/manifest"
    cli_cmd = f"cargo run -p aether-mc-runner -- install-and-run --manifest {manifest_url}"

    html = f"""
    <div class="w-full text-left space-y-4 animate-fade-in bg-obsidian-950 p-5 rounded-2xl border border-amber-500/40 shadow-xl">
      <div class="flex items-center justify-between border-b border-white/10 pb-3">
        <div>
          <span class="text-[10px] uppercase tracking-wider text-amber-400 font-bold font-gaming">Manifest Prêt Pour Déploiement</span>
          <h3 class="text-base font-bold text-white font-gaming">{manifest['name']}</h3>
        </div>
        <span class="px-2.5 py-1 text-xs rounded-full bg-amber-950 text-amber-300 border border-amber-800 font-mono">
          {manifest['core']['type'].upper()} {manifest['core']['mc_version']} (Build {manifest['core']['build']})
        </span>
      </div>

      <div class="grid grid-cols-2 gap-2 text-xs">
        <div class="bg-black/50 p-2.5 rounded-xl border border-white/5">
          <span class="text-slate-400 block text-[10px] font-gaming">RAM & Garbage Collector</span>
          <span class="text-white font-semibold">{manifest['runtime']['heap_max']} (Aikar Flags G1GC)</span>
        </div>
        <div class="bg-black/50 p-2.5 rounded-xl border border-white/5">
          <span class="text-slate-400 block text-[10px] font-gaming">Plugins Injectés</span>
          <span class="text-white font-semibold">{len(manifest['plugins_and_mods'])} plugin(s)</span>
        </div>
      </div>

      <div class="space-y-1">
        <label class="text-[11px] font-semibold text-slate-300 font-gaming">URL du Manifest JSON :</label>
        <div class="flex items-center gap-2">
          <input type="text" readonly value="{manifest_url}" 
                 class="w-full text-xs bg-black/60 border border-white/10 rounded-xl p-2.5 font-mono text-amber-300">
          <a href="{manifest_url}" target="_blank" 
             class="px-3.5 py-2.5 bg-white/10 hover:bg-white/20 text-xs rounded-xl text-slate-200 border border-white/10 transition font-gaming font-bold">
            Consulter
          </a>
        </div>
      </div>

      <div class="space-y-1 pt-1">
        <label class="text-[11px] font-semibold text-slate-300 font-gaming">🚀 Commande d'Exécution Client Rust :</label>
        <div class="relative bg-black/70 p-3 rounded-xl border border-white/10 font-mono text-xs text-emerald-400 break-all select-all">
          {cli_cmd}
        </div>
      </div>
    </div>
    """
    return html

@app.route("/api/v1/servers/<server_id>/manifest", methods=["GET"])
def get_manifest(server_id):
    manifest = MANIFEST_STORE.get(server_id)
    if not manifest:
        manifest = build_server_manifest(
            server_id=server_id,
            name=f"Instance {server_id}",
            core_type="paper",
            mc_version="1.21.1",
            ram_gb=4
        )
        MANIFEST_STORE[server_id] = manifest
    return jsonify(manifest)

@app.route("/api/v1/server/ping", methods=["GET"])
def server_ping():
    return jsonify(get_live_server_status())

@app.route("/api/v1/health", methods=["GET"])
def health():
    return jsonify({"status": "ok", "service": "aether-mc-control-plane", "modules": ["store", "players", "server_creator"]})

if __name__ == "__main__":
    print("[*] Aether Community & Server Platform demarre sur http://127.0.0.1:5000")
    app.run(host="0.0.0.0", port=5000, debug=False)
