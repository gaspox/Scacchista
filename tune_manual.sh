#!/bin/bash
# Tuning manuale parametri - Test 3 configurazioni

set -e

echo "============================================"
echo "FASE 3: TUNING MANUALE PARAMETRI"
echo "============================================"
echo ""

# Salva configurazione originale
cp src/search/params.rs src/search/params.rs.bak

# Configurazione 1: Conservativa (futility_margin=150, lmr_base_reduction=1)
echo "=== Config 1: Conservative ==="
echo "futility_margin=150, lmr_base_reduction=1, qsearch_depth=4"
sed -i 's/lmr_base_reduction: 2,/lmr_base_reduction: 1,/g' src/search/params.rs
sed -i 's/futility_margin: 200,/futility_margin: 150,/g' src/search/params.rs
cargo build --release 2>&1 | tail -3
cp target/release/scacchista scacchista_v0.5_config1

# Test posizione iniziale
echo "Test posizione iniziale:"
timeout 10 ./scacchista_v0.5_config1 <<'TEST' | grep "score cp"
uci
position startpos
go depth 6
TEST

# Ripristina originale
cp src/search/params.rs.bak src/search/params.rs

# Configurazione 2: Agressiva (futility_margin=250, lmr_base_reduction=3)
echo ""
echo "=== Config 2: Aggressive ==="
echo "futility_margin=250, lmr_base_reduction=3, qsearch_depth=5"
sed -i 's/lmr_base_reduction: 2,/lmr_base_reduction: 3,/g' src/search/params.rs
sed -i 's/futility_margin: 200,/futility_margin: 250,/g' src/search/params.rs
sed -i 's/qsearch_depth: 4,/qsearch_depth: 5,/g' src/search/params.rs
cargo build --release 2>&1 | tail -3
cp target/release/scacchista scacchista_v0.5_config2

echo "Test posizione iniziale:"
timeout 10 ./scacchista_v0.5_config2 <<'TEST' | grep "score cp"
uci
position startpos
go depth 6
TEST

# Ripristina originale
cp src/search/params.rs.bak src/search/params.rs

# Configurazione 3: Balanced con qsearch_optimizations ri-abilitato
echo ""
echo "=== Config 3: Delta Pruning Fix Test ==="
echo "futility_margin=180, enable_qsearch_optimizations=true"
sed -i 's/futility_margin: 200,/futility_margin: 180,/g' src/search/params.rs
sed -i 's/enable_qsearch_optimizations: false/enable_qsearch_optimizations: true/g' src/search/params.rs
cargo build --release 2>&1 | tail -3
cp target/release/scacchista scacchista_v0.5_config3

echo "Test posizione iniziale:"
timeout 10 ./scacchista_v0.5_config3 <<'TEST' | grep "score cp"
uci
position startpos
go depth 6
TEST

# Ripristina originale finale
cp src/search/params.rs.bak src/search/params.rs

echo ""
echo "============================================"
echo "Tuning completato. Binari creati:"
echo "  - scacchista_v0.5_config1 (conservativo)"
echo "  - scacchista_v0.5_config2 (aggressivo)"
echo "  - scacchista_v0.5_config3 (delta pruning on)"
echo "============================================"
