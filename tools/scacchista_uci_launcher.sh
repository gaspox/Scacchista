#!/usr/bin/env bash
# UCI launcher wrapper for Scacchista
# Uso: copia questo script nella cartella dove vuoi l'engine e modifica ENGINE_BIN se necessario
# Poi in Lucas Chess punta a questo script come 'engine' (assicurati che sia eseguibile)

set -euo pipefail

# Percorso predefinito del binario (modifica se hai un nome/posizione diversa)
ENGINE_BIN="/usr/local/bin/scacchista" # default target path, override by passing path as first arg to the script (or edit)
# If you built from this repo, use: $(pwd)/target/release/scacchista instead
# ENGINE_BIN fallback to local build if present
if [ -x "$(pwd)/target/release/scacchista" ] && [ -z "$1" ]; then
    ENGINE_BIN="$(pwd)/target/release/scacchista"
fi

# If user provided a path via first argument, override ENGINE_BIN
if [ $# -ge 1 ] && [ -x "$1" ]; then
    ENGINE_BIN="$1"
fi

# allow passing additional UCI args (kept for compatibility)

# Se vuoi usare un binario alternativo, puoi passarlo come primo argomento
if [ $# -ge 1 ]; then
    ENGINE_BIN="$1"
fi

if [ ! -x "$ENGINE_BIN" ]; then
    echo "Errore: binario engine non trovato o non eseguibile: $ENGINE_BIN" >&2
    exit 2
fi

# Lucas Chess (o qualsiasi GUI) lancerà questo script e si aspetta il protocollo UCI su stdin/stdout
# Qui semplicemente eseguiamo il binario reale e inoltriamo stdin/stdout
exec "$ENGINE_BIN" "$@"
