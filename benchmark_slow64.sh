#!/bin/bash
# Benchmark: Scacchista v0.4.0-rc vs slow64_linux

echo "======================================"
echo "  BENCHMARK: Scacchista vs slow64"
echo "======================================"
echo ""

# Test positions
declare -a POSITIONS=(
    "startpos"
    "fen rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"  # After 1.e4
    "fen r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3"  # After 1.e4 e5 2.Nf3 Nc6
)

declare -a POS_NAMES=(
    "Starting Position"
    "After 1.e4"
    "After 1.e4 e5 2.Nf3 Nc6"
)

DEPTHS=(6 8)

echo "=== ENGINE INFO ==="
echo "Scacchista: $(./target/release/scacchista --version 2>/dev/null || echo 'v0.4.0-rc')"
echo "slow64_linux: $(./slow64_linux --version 2>/dev/null || echo 'unknown')"
echo ""

for DEPTH in "${DEPTHS[@]}"; do
    echo ""
    echo "=========================================="
    echo "  DEPTH $DEPTH BENCHMARK"
    echo "=========================================="
    echo ""

    for i in "${!POSITIONS[@]}"; do
        POS="${POSITIONS[$i]}"
        NAME="${POS_NAMES[$i]}"

        echo "--- Position: $NAME ---"
        echo ""

        # Test Scacchista
        echo -n "Scacchista: "
        RESULT_SCACCHISTA=$(cat <<EOF | timeout 120 ./target/release/scacchista 2>/dev/null | grep "info depth $DEPTH"
uci
isready
position $POS
go depth $DEPTH
quit
EOF
)

        if [ -n "$RESULT_SCACCHISTA" ]; then
            TIME_SC=$(echo "$RESULT_SCACCHISTA" | grep -oP 'time \K\d+' | tail -1)
            SCORE_SC=$(echo "$RESULT_SCACCHISTA" | grep -oP 'score cp \K-?\d+' | tail -1)
            NODES_SC=$(echo "$RESULT_SCACCHISTA" | grep -oP 'nodes \K\d+' | tail -1)
            NPS_SC=$((NODES_SC * 1000 / TIME_SC))
            echo "Time=${TIME_SC}ms, Score=${SCORE_SC}cp, Nodes=${NODES_SC}, NPS=${NPS_SC}"
        else
            echo "TIMEOUT or ERROR"
        fi

        # Test slow64
        echo -n "slow64:     "
        RESULT_SLOW=$(cat <<EOF | timeout 120 ./slow64_linux 2>/dev/null | grep "info depth $DEPTH"
uci
isready
position $POS
go depth $DEPTH
quit
EOF
)

        if [ -n "$RESULT_SLOW" ]; then
            TIME_SL=$(echo "$RESULT_SLOW" | grep -oP 'time \K\d+' | tail -1)
            SCORE_SL=$(echo "$RESULT_SLOW" | grep -oP 'score cp \K-?\d+' | tail -1)
            NODES_SL=$(echo "$RESULT_SLOW" | grep -oP 'nodes \K\d+' | tail -1)
            NPS_SL=$((NODES_SL * 1000 / TIME_SL))
            echo "Time=${TIME_SL}ms, Score=${SCORE_SL}cp, Nodes=${NODES_SL}, NPS=${NPS_SL}"
        else
            echo "TIMEOUT or ERROR"
        fi

        # Comparison
        if [ -n "$TIME_SC" ] && [ -n "$TIME_SL" ]; then
            SPEEDUP=$(echo "scale=2; $TIME_SL / $TIME_SC" | bc)
            echo "=> Scacchista is ${SPEEDUP}x vs slow64"
        fi
        echo ""
    done
done

echo ""
echo "======================================"
echo "  TACTICAL TEST SUITE"
echo "======================================"
echo ""

# Tactical positions (mate in 2-3, forks, pins)
declare -a TACTICAL=(
    "fen r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4|Scholar's Mate Threat"
    "fen r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 0 5|Italian Opening"
)

for entry in "${TACTICAL[@]}"; do
    FEN=$(echo "$entry" | cut -d'|' -f1 | sed 's/fen //')
    NAME=$(echo "$entry" | cut -d'|' -f2)

    echo "--- Tactical: $NAME ---"

    # Scacchista
    BM_SC=$(cat <<EOF | timeout 30 ./target/release/scacchista 2>/dev/null | grep "bestmove" | head -1
uci
isready
position fen $FEN
go depth 8
quit
EOF
)
    echo "Scacchista: $(echo $BM_SC | awk '{print $2}')"

    # slow64
    BM_SL=$(cat <<EOF | timeout 30 ./slow64_linux 2>/dev/null | grep "bestmove" | head -1
uci
isready
position fen $FEN
go depth 8
quit
EOF
)
    echo "slow64:     $(echo $BM_SL | awk '{print $2}')"
    echo ""
done

echo "======================================"
echo "  BENCHMARK COMPLETE"
echo "======================================"
