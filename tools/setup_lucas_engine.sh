#!/usr/bin/env bash
set -euo pipefail

# Script di supporto per testare Scacchista con Lucas Chess su Linux.
# Cosa fa:
#  - compila la build release con `cargo build --release`
#  - copia il binario compilato in una posizione comoda che Lucas Chess può usare
#  - (opzionale) lancia Lucas Chess se è presente nel PATH

echo "[scacchista] Building release..."
cargo build --release

BIN="target/release/scacchista"
if [ ! -f "$BIN" ]; then
    echo "Binary not found at $BIN"
    echo "Controlla il nome del binario prodotto in target/release/"
    exit 1
fi

chmod +x "$BIN" || true

# Posizione dove copiare il binario per Lucas Chess (cartella locale per l'utente)
DEST_DIR="$HOME/.local/share/lucaschess/engines"
DEST="$DEST_DIR/scacchista"

mkdir -p "$DEST_DIR"
cp -f "$BIN" "$DEST"
chmod +x "$DEST"

echo "Installed engine to: $DEST"

# Se Lucas Chess è installato e disponibile nel PATH, proviamo a lanciarlo
if command -v LucasChess >/dev/null 2>&1 || command -v lucaschess >/dev/null 2>&1; then
    LC_CMD="$(command -v LucasChess || command -v lucaschess)"
    echo "Launching Lucas Chess ($LC_CMD)..."
    "$LC_CMD" &
    sleep 1
    echo "Attendi l'apertura di Lucas Chess. Poi vai su: Engines -> Manage engines -> Add UCI engine -> punta a: $DEST"
else
    echo "Lucas Chess non trovato nel PATH."
    echo "Apri Lucas Chess manualmente e aggiungi l'engine UCI selezionando: $DEST"i

cat <<'EOF'
Suggerimenti:
- In Lucas Chess, quando aggiungi l'engine scegli UCI e poi seleziona il binario copiato.
- Imposta opzioni (Threads, Hash) nella UI di Lucas Chess per misurare scaling.
- Per test automatizzati usa cutechess-cli invece della GUI se vuoi partite da terminale.
EOF
