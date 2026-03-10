# Piano di Implementazione v0.5.x - Consolidato

> Documento unificato che raccoglie implementation_plan.md, task.md e walkthrough.md
> Ultimo aggiornamento: 2026-03-10

---

## Stato Attuale (Post-Fix v0.5.1)

### ✅ COMPLETATO

| Fase | Feature | Commit | Status |
|------|---------|--------|--------|
| **Fase 1** | Performance Optimizations | fe35877 | ✅ Fixato TT (Mutex) |
| 1.1 | Capture Generation | fe35877 | ✅ |
| 1.2 | Delta Pruning | fe35877 | ✅ (disabilitato in v0.5.1) |
| 1.3 | ~~Lock-free TT~~ | ~~fe35877~~ | ❌ **Buggy** - Sostituito con Mutex |
| 1.4 | Bitboard Eval | fe35877 | ✅ |
| **Fase 2** | Tattica | 84d62d8, 24e985f, 085e98c | ✅ |
| 2.1 | PVS at root | 84d62d8 | ✅ |
| 2.2 | IIR | 84d62d8 | ✅ |
| 2.3 | SEE Pruning | 24e985f | ✅ |
| 2.4 | Countermove Heuristic | 085e98c | ✅ (+19% NPS) |
| **Fase 3** | Valutazione | 8a0bc35, 1892d2c, 09ca171 | ✅ |
| 3.1 | Tapered Eval | 8a0bc35 | ✅ (valori fixati in v0.5.1) |
| 3.2 | Pawn Structure | 8a0bc35 | ✅ |
| 3.3 | Passed Pawns | 1892d2c | ✅ |
| 3.4 | Bishop Pair | 09ca171 | ✅ |

### 🔄 FIX APPLICATI (v0.5.1)

| Problema | Fix | File |
|----------|-----|------|
| TT race condition | Mutex TT | src/search/tt.rs |
| Valori materiali errati | Scala v0.4.1 | src/eval.rs |
| Pruning troppo aggressivo | tuning conservativo | src/search/params.rs |

---

## Roadmap Futura

### 🔴 P0 - Quick Wins (Alta priorità, basso sforzo)

#### 1. Draw Detection Completo
**Status**: Parzialmente implementato (commit 723459e)  
**Manca**: Integrazione completa in search  
**Impact**: +80-100 ELO  
**Effort**: 2-3 ore

```rust
// TODO in src/search/search.rs
// - Threefold repetition check
// - Fifty-move rule  
// - Insufficient material detection
```

#### 2. Endgame Recognition
**Status**: Parzialmente implementato (commit 4688a98)  
**Manca**: KB+KN vs K, KNN vs K, etc.  
**Impact**: +60-80 ELO  
**Effort**: 1-2 ore

#### 3. SEE Cache Array
**Impact**: +5-10% speed  
**Effort**: 30 minuti  
**File**: src/search/search.rs

```rust
// Replace HashMap with fixed array
// Current: let see_cache: HashMap<Move, i16>
// Proposed: let see_cache: [Option<i16>; 64]
```

#### 4. Razoring
**Impact**: +2-3% speed  
**Effort**: 1 ora

```rust
if depth <= 2 && !in_check && static_eval + margin < alpha {
    return static_eval;
}
```

### 🟡 P1 - Medium Effort, High Impact

#### 5. Lazy-SMP Diversity
**Impact**: +50-80% speedup on 2 threads  
**Effort**: 1-2 giorni  
**File**: src/search/thread_mgr.rs

Opzioni:
- Per-worker history tables
- Different aspiration windows per thread
- Random eval noise

#### 6. Magic Bitboards
**Impact**: 3-5x speedup move generation  
**Effort**: 1-2 settimane  
**File**: src/board.rs

Replace loop-based sliding piece generation with magic bitboard lookups.

#### 7. Pawn Hash Table
**Impact**: +10-15% speed  
**Effort**: 1 settimana  
**File**: src/eval.rs

Cache pawn structure evaluation separately.

### 🟢 P2 - Long Term

#### 8. NNUE Integration
**Impact**: +200-400 ELO  
**Effort**: 2-3 mesi  
Options:
- Use Stockfish NNUE via FFI
- Train custom NNUE (HalfKP architecture)

#### 9. Advanced Pruning
**Impact**: +50-100 ELO cumulative  
**Effort**: 1-2 mesi
- Singular Extensions
- Multi-Cut
- Probcut

#### 10. Syzygy Full Integration
**Impact**: Perfect endgame play  
**Effort**: 1 settimana  
**File**: Già in Cargo.toml
- WDL/DTZ probing
- Root move filtering
- Search termination

---

## Metriche Target

| Milestone | ELO Stimato | Note |
|-----------|-------------|------|
| **v0.5.1 (attuale)** | ~1600-1700 | Post-fix |
| + Draw detection | ~1700-1800 | +100 ELO |
| + Lazy-SMP diversity | ~1800-1900 | +100 ELO |
| + NNUE | ~2100-2300 | +300 ELO |

**Conservative Target**: 2000 ELO  
**Optimistic Target**: 2400 ELO (with NNUE)

---

## Note di Implementazione

### Valutazione PeSTO (Fase 3.1)

Implementazione originale usava valori PeSTO:
```rust
// Originale (buggy)
const PAWN_VALUE: Score = s(82, 94);     // troppo basso
const KNIGHT_VALUE: Score = s(337, 281); // scala diversa
```

Fix v0.5.1 - Valori scalati a v0.4.1:
```rust
// Fix
const PAWN_VALUE: Score = s(100, 100);   // unified
const KNIGHT_VALUE: Score = s(320, 320); // unified
```

### TT Lock-free → Mutex

Originale aveva race condition con:
- `Ordering::Relaxed` (inconsistenza memoria)
- 16-bit hash verification (collisioni)
- Store non atomico

Fix v0.5.1:
```rust
// Interfaccia compatibile
pub struct TranspositionTable {
    inner: Mutex<TranspositionTableInner>,
}
```

---

## Task Checklist

### P0 - Prossimi Passi
- [ ] Draw Detection: integrazione completa in search
- [ ] Endgame Recognition: KB+KN vs K, KNN vs K
- [ ] SEE Cache Array: implementazione 30min
- [ ] Razoring: implementazione 1h

### P1 - Medium Term
- [ ] Lazy-SMP Diversity: per-worker tables
- [ ] Magic Bitboards: sliding pieces
- [ ] Pawn Hash Table: cache structure

### P2 - Long Term
- [ ] NNUE Integration
- [ ] Advanced Pruning (Singular Extensions, etc.)
- [ ] Syzygy Full Integration

---

**Documento consolidato da**: implementation_plan.md, task.md, walkthrough.md  
**Data consolidamento**: 2026-03-10
