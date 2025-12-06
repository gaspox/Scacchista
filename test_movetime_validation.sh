#!/bin/bash
# Validation test for Bug #1 fix
# Run 10 iterations with movetime and check for score corruption

echo "Testing 10 iterations with movetime=100ms on position where bug occurred..."
echo "Position: Black queen hanging on e6 (after 1.d4 Na6 2.c4 d6 3.e4 Qd7 4.Nf3 Qe6 5.Nc3 Nf6 6.d5)"
echo ""

CORRUPTION_COUNT=0

for i in 1 2 3 4 5 6 7 8 9 10; do
  echo -n "Iteration $i: "
  RESULT=$(printf 'uci\nisready\nposition fen r1b1kb1r/pp2pppp/n2qnn2/2pP4/2P1P3/2N2N2/PP3PPP/R1BQKB1R b KQkq - 0 6\ngo movetime 100\nquit\n' | ./target/release/scacchista 2>/dev/null | grep "info depth")

  SCORE=$(echo "$RESULT" | grep -oP 'score cp \K-?\d+')
  BESTMOVE=$(printf 'uci\nisready\nposition fen r1b1kb1r/pp2pppp/n2qnn2/2pP4/2P1P3/2N2N2/PP3PPP/R1BQKB1R b KQkq - 0 6\ngo movetime 100\nquit\n' | ./target/release/scacchista 2>/dev/null | grep "bestmove" | awk '{print $2}')

  # Check for corruption (score >= 29000 or <= -29000)
  if [ "$SCORE" -ge 29000 ] 2>/dev/null || [ "$SCORE" -le -29000 ] 2>/dev/null; then
    echo "❌ CORRUPTED! Score=$SCORE, Move=$BESTMOVE"
    CORRUPTION_COUNT=$((CORRUPTION_COUNT + 1))
  else
    echo "✅ OK - Score=$SCORE, Move=$BESTMOVE"
  fi
done

echo ""
if [ $CORRUPTION_COUNT -eq 0 ]; then
  echo "✅ SUCCESS: 0/10 iterations had score corruption"
  echo "Bug #1 fix is working correctly!"
  exit 0
else
  echo "❌ FAILURE: $CORRUPTION_COUNT/10 iterations had score corruption"
  echo "Bug #1 fix may not be working properly"
  exit 1
fi
