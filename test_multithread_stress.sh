#!/bin/bash
# Stress test for multi-threading (Bug #2 validation)

echo "=== Multi-threading Stress Test ==="
echo "Testing engine with various thread counts and search depths"
echo ""

SUCCESS=0
FAIL=0

# Test configurations: threads x depth
CONFIGS=(
  "1:6"
  "2:6"
  "4:6"
  "2:7"
  "4:7"
  "8:6"
)

for config in "${CONFIGS[@]}"; do
  IFS=':' read -r threads depth <<< "$config"

  echo -n "Testing Threads=$threads Depth=$depth ... "

  OUTPUT=$(timeout 30 bash -c "printf 'uci\nsetoption name Threads value $threads\nisready\nposition startpos\ngo depth $depth\nquit\n' | ./target/release/scacchista 2>&1")
  EXIT_CODE=$?

  if [ $EXIT_CODE -eq 0 ]; then
    BESTMOVE=$(echo "$OUTPUT" | grep "bestmove" | head -1)
    if [ -n "$BESTMOVE" ]; then
      echo "✅ OK - $BESTMOVE"
      SUCCESS=$((SUCCESS + 1))
    else
      echo "❌ FAIL - No bestmove output"
      FAIL=$((FAIL + 1))
    fi
  elif [ $EXIT_CODE -eq 124 ]; then
    echo "❌ TIMEOUT (30s)"
    FAIL=$((FAIL + 1))
  else
    echo "❌ CRASH (exit code $EXIT_CODE)"
    FAIL=$((FAIL + 1))
  fi
done

echo ""
echo "=== Async Search Test (go infinite + stop) ==="

for threads in 1 2 4 8; do
  echo -n "Testing Threads=$threads (go infinite) ... "

  OUTPUT=$(timeout 10 bash -c "{
    printf 'uci\nsetoption name Threads value $threads\nisready\nposition startpos\ngo infinite\n'
    sleep 2
    printf 'stop\nquit\n'
  } | ./target/release/scacchista 2>&1")
  EXIT_CODE=$?

  if [ $EXIT_CODE -eq 0 ]; then
    BESTMOVE=$(echo "$OUTPUT" | grep "bestmove" | head -1)
    if [ -n "$BESTMOVE" ]; then
      echo "✅ OK - $BESTMOVE"
      SUCCESS=$((SUCCESS + 1))
    else
      echo "❌ FAIL - No bestmove"
      FAIL=$((FAIL + 1))
    fi
  else
    echo "❌ CRASH (exit code $EXIT_CODE)"
    FAIL=$((FAIL + 1))
  fi
done

echo ""
echo "=== Results ==="
echo "Success: $SUCCESS"
echo "Failures: $FAIL"

if [ $FAIL -eq 0 ]; then
  echo "✅ All multi-threading tests PASSED!"
  exit 0
else
  echo "❌ Some tests FAILED"
  exit 1
fi
