# OPTIMIZATION STRATEGY - Final Data-Driven Plan

**Data**: 2025-12-03
**Basato su**: Profiling reale, non assunzioni

---

## 🎯 EXECUTIVE SUMMARY

**Scoperta Chiave**: Il bottleneck NON è nella move generation.

**Dati Chiave**:
- Perft (move gen pura): **4.3M nodes/sec**
- Search (alpha-beta + eval): **15k nodes/sec**
- **Gap**: 287x più lento

**Breakdown Misurato**:
- Move generation + make/unmake: **11.3%** del tempo
- Search overhead (eval, TT, ordering, logic): **88.7%** del tempo

**Implicazione**: Ottimizzare move generation darebbe **massimo 11.3% speedup**. Dobbiamo foc us sull'88.7% rimasto!

---

## 📊 DATI DI PROFILING

### Funzioni Individuali

| Componente | ns/call | Conclusione |
|------------|---------|-------------|
| `generate_moves()` | 3,235 ns | Accettabile |
| `make_move() + unmake_move()` | **76 ns** | Eccellente! |
| `is_square_attacked()` | 31 ns | Eccellente! |

### Performance Aggregate

| Test | Nodes/sec | Note |
|------|-----------|------|
| Perft depth 5 | 4,300,000 | Move gen è veloce |
| Search depth 6 | ~15,000 | 287x più lento! |

### Calcolo Bottleneck

```
Tempo per nodo (search): 66,667 ns
- Move generation: 3,235 ns (4.8%)
- Make/unmake (×40 mosse avg): 3,040 ns (4.6%)
- Legality check (×40): 1,240 ns (1.9%)
────────────────────────────────────
SUB-TOTALE move gen: 7,515 ns (11.3%)

MANCANTE: 59,152 ns (88.7%)
```

Questo 88.7% è speso in:
- Evaluation (stimato 30-45%)
- TT operations (stimato 15-23%)
- Move ordering (stimato 8-15%)
- Search logic (stimato 15-23%)
- Other (stimato ~8%)

---

## 🚀 STRATEGIA RIVISTA (Data-Driven)

### APPROCCIO

**NON** ottimizzare move generation (già veloce).

**SÌ** ottimizzare search/evaluation (vero bottleneck).

### FASE 1: EVALUATION OPTIMIZATION (Priority HIGH)

**Obiettivo**: Ridurre tempo di evaluation da ~40% a ~20%

#### 1.1 Lazy Evaluation
```rust
// Attuale: eval chiamato sempre
let score = self.evaluate();
if score >= beta {
    return score;
}

// Ottimizzato: eval solo se necessario
if depth == 0 {
    return self.evaluate();
}
// Skip eval in nodi interni se non serve
```

**Expected Impact**: 1.3-1.5x speedup
**Risk**: BASSO
**Effort**: 1-2 ore

#### 1.2 Simplified Eval at Low Depth
```rust
fn evaluate_fast(&self) -> i16 {
    // Solo material + PSQT
    // Skip: mobility, king safety, pawn structure
    self.material_score() + self.psqt_score()
}

// Usare in qsearch e depth <= 2
```

**Expected Impact**: 1.2-1.3x speedup
**Risk**: MEDIO (potrebbe perdere qualità tattica)
**Effort**: 2-3 ore

#### 1.3 Incremental Evaluation (Advanced)
```rust
// Invece di ricalcolare eval da zero ogni volta:
struct BoardWithEval {
    board: Board,
    eval_cache: i16,  // Updated incrementally
}
```

**Expected Impact**: 1.5-2x speedup
**Risk**: ALTO (complesso, bug-prone)
**Effort**: 1-2 giorni

**Raccomandazione**: Fare 1.1 + 1.2, skip 1.3 (troppo complesso per ora)

---

### FASE 2: MOVE ORDERING IMPROVEMENT (Priority MEDIUM)

**Obiettivo**: Ridurre nodi esplorati migliorando move ordering

#### 2.1 Better MVV-LVA Scoring
```rust
// Attuale: MVV-LVA basic
// Ottimizzato: Include piece value differentials

fn mvv_lva_score(attacker: PieceKind, victim: PieceKind) -> i16 {
    PIECE_VALUE[victim] * 10 - PIECE_VALUE[attacker]
}
```

**Expected Impact**: 1.1-1.2x (riduce nodi del 10-20%)
**Risk**: BASSO
**Effort**: 1 ora

#### 2.2 History Heuristic Tuning
- Aumentare decay rate
- Separate history tables per depth
- Age history più aggressivamente

**Expected Impact**: 1.05-1.1x
**Risk**: BASSO
**Effort**: 2-3 ore

**Raccomandazione**: Fare 2.1, considerare 2.2 se serve

---

### FASE 3: TRANSPOSITION TABLE OPTIMIZATION (Priority LOW-MEDIUM)

**Obiettivo**: Ridurre overhead TT da ~20% a ~10%

#### 3.1 Reduce TT Entry Size
```rust
// Attuale: probabilmente 16-24 bytes per entry
// Ottimizzato: pack in 12-16 bytes
#[repr(packed)]
struct TTEntry {
    key: u32,       // Ridotto da u64 (collision accettabile)
    best_move: u16,
    score: i16,
    depth: u8,
    flags: u8,
}
```

**Expected Impact**: 1.05-1.1x (better cache usage)
**Risk**: MEDIO (packed struct ha overhead alignment)
**Effort**: 2-3 ore

#### 3.2 Better Replacement Scheme
- Always replace if depth > stored depth
- Otherwise, replace based on age + depth

**Expected Impact**: 1.03-1.05x
**Risk**: BASSO
**Effort**: 1-2 ore

**Raccomandazione**: Skip per ora (beneficio marginale)

---

### ❌ NON FARE (Validato come Inutile)

1. **Fast Legality Check**: make/unmake sono già 76ns, risparmiare questo è trascurabile
2. **Stack MoveList**: Beneficio <5% (move gen è solo 11% del tempo)
3. **Optimize piece_on()**: Funzione chiamata raramente, impatto minimo
4. **Inline annotations**: Compilatore probabilmente già inlina, beneficio <2%

---

## 📋 PIANO ESECUTIVO

### Week 1: Evaluation Optimization

**Giorno 1-2**: Lazy Evaluation
- Implementare skip eval in nodi non-leaf
- Test correttezza (perft, tactical tests)
- Benchmark

**Giorno 3-4**: Simplified Eval at Low Depth
- Implementare evaluate_fast()
- Usare in qsearch e depth <= 2
- Test qualità search (non deve peggiorare)
- Benchmark

**Giorno 5**: Buffer & Optimization
- Fix bug emersi
- Profiling post-optimization
- Documentazione

**Target Fine Week 1**: 20-25k nodes/sec (1.3-1.7x improvement)

---

### Week 2: Move Ordering & Polish

**Giorno 6-7**: Better MVV-LVA
- Implementare scoring migliorato
- Test su posizioni tattiche
- Benchmark node reduction

**Giorno 8-9**: History Tuning (optional)
- Se Week 1 non ha raggiunto target
- Altrimenti skip

**Giorno 10**: Final Testing & Documentation
- Perft validation
- Tactical suite
- Benchmark vs slow64
- Update documentation

**Target Finale**: 25-35k nodes/sec (1.6-2.3x total improvement)

---

## 🎯 SUCCESS CRITERIA

### Must Have
- [ ] Perft tests pass (zero tolerance)
- [ ] Tactical tests pass (same quality)
- [ ] Nodes/sec >= 20,000 (1.3x minimum)
- [ ] No correctness regressions

### Should Have
- [ ] Nodes/sec >= 25,000 (1.6x target)
- [ ] Depth 8 in < 18 seconds (vs 26.2s baseline)

### Nice to Have
- [ ] Nodes/sec >= 35,000 (2.3x stretch)
- [ ] Competitive with slow64 on depth 6-8

---

## ⚠️ RISK MITIGATION

### Risk: Lazy Eval Breaks Search Quality
**Mitigation**:
- Test on tactical suite first
- If quality drops >5%, revert
- Try less aggressive lazy eval

### Risk: Simplified Eval Misses Tactics
**Mitigation**:
- Only use at depth <= 2 initially
- Compare search results with/without
- Adjust threshold if needed

### Risk: Timeline Slip
**Mitigation**:
- Week 1 is self-contained (can ship just that)
- Week 2 is optional enhancement
- Checkpoint after each optimization

---

## 📊 ESTIMATED RESULTS

| Optimization | Speedup | Cumulative | Nodes/sec |
|--------------|---------|------------|-----------|
| Baseline | 1.0x | 1.0x | 15,600 |
| Lazy Eval | 1.4x | 1.4x | 21,840 |
| Simplified Eval Low Depth | 1.2x | 1.68x | 26,208 |
| Better MVV-LVA | 1.15x | 1.93x | 30,108 |
| **TOTAL** | - | **1.9x** | **~30k** |

**Realistic Range**: 25k-35k nodes/sec (1.6-2.3x)

---

## 🔄 ALTERNATIVE IF PLAN FAILS

If evaluation optimization doesn't yield expected results:

**Plan B**: Reduce nodes explored
- More aggressive pruning (null-move R=3 instead of R=2)
- LMR more aggressively
- Futility margin increase

**Plan C**: Accept current performance
- 15k nodes/sec is functional
- Focus on other features (opening book, endgame, tuning)
- Revisit performance later with better profiling tools

---

## ✅ NEXT IMMEDIATE ACTION

1. **Commit current profiling work**
   ```bash
   git add profile_search.rs Cargo.toml PROFILING_*.md
   git commit -m "Add profiling infrastructure and analysis"
   ```

2. **Start Week 1 Day 1: Lazy Evaluation**
   - Branch: `feature/lazy-evaluation`
   - Implement skip eval in non-leaf nodes
   - Test & benchmark

3. **Track progress daily**
   - Document speedup after each change
   - Maintain PROFILING_REPORT.md with updates

---

**Conclusion**: Focus on the 88.7% (evaluation + search overhead), not the 11.3% (move generation). Realistic target: 2x speedup via eval optimization.
