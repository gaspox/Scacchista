# Roadmap v0.6.0 - Performance & Scaling

**Data:** 2026-04-30  
**Versione corrente:** v0.6.0-alpha.2  
**Prossima milestone:** v0.6.0 final / v0.6.1

---

## TL;DR

Dopo aver completato tutti i P0 items (SEE Cache, Razoring, Draw Detection, Test EPD), 
la v0.6.0 si concentra su **multi-threading** e **performance pura**.

---

## 🎯 Obiettivi v0.6.0

| Priorità | Feature | Impact | Stato |
|----------|---------|--------|-------|
| P0 | Lazy-SMP Diversity | +50-80% su 2 threads | ✅ Completato |
| P0 | Pawn Hash Table | +10-15% speed | ⏳ v0.6.1 |
| P1 | Magic Bitboards | 3-5x move gen | ✅ Completato (integrato in board.rs) |
| P1 | Endgame Recognition | +60-80 ELO | ✅ Completato (7 pattern aggiunti) |
| P2 | Advanced Pruning | +30-50 ELO | ⏳ v0.6.1 |
| — | Tapered Eval + Pawn Struct + Mobility | +100-150 ELO | ✅ Completato (Fase 3) |

---

## P0 - Lazy-SMP Diversity

**Problema attuale:** Solo 4% speedup su 2 threads

**Soluzioni:**
```rust
// 1. Per-worker history tables
thread_local! {
    static HISTORY: RefCell<[[i16; 64]; 12]> = ...
}

// 2. Different aspiration windows
let base_window = 25 + (thread_id * 5);

// 3. Random eval noise (±5cp)
let noise = (rand::random::<i8>() % 10) - 5;
```

---

## P0 - Pawn Hash Table

**Motivazione:** Valutazione pedoni costosa e ripetuta

**Implementazione:**
```rust
pub struct PawnHashEntry {
    key: u64,
    score: Score,
    passed_pawns: Bitboard,
    pawn_attacks: [Bitboard; 2],
}

static PAWN_HASH: [Mutex<PawnHashEntry>; 1024] = ...
```

---

## P1 - Magic Bitboards

**Stato attuale:** Sliding pieces usano loop

**Target:** Lookup O(1) per sliding moves

```rust
// Prima (loop)
for sq in 0..64 {
    if rook_attacks(sq, occupied) & target != 0 { ... }
}

// Dopo (magic)
let attacks = ROOK_MAGICS[sq].attacks[occupancy_to_index(occupied)];
```

---

## P1 - Endgame Recognition

**Manca:**
- KB+KN vs K = winning (but difficult)
- KNN vs K = draw (insufficient)
- KR+KP vs KR = Lucena/Philidor

**Implementazione:**
```rust
pub fn endgame_score(board: &Board) -> Option<Score> {
    match piece_count(board) {
        (0, 0, 1, 0, 0) => Some(king_and_rook_vs_king(board)),
        (0, 0, 0, 1, 0) => Some(king_and_queen_vs_king(board)),
        _ => None,
    }
}
```

---

## P2 - Advanced Pruning

### Singular Extensions
```rust
if depth >= 8 && !is_pv && tt_move.exists() {
    let singular_beta = tt_score - 2 * depth;
    // Search all moves except TT move
    // If all fail low below singular_beta, extend TT move
}
```

### Probcut
```rust
if depth >= 5 && !in_check {
    let probcut_beta = beta + 100;
    // Reduced depth search with probcut_beta
}
```

---

## 📊 ELO Projection

| Milestone | ELO Stimato | Cumulativo |
|-----------|-------------|------------|
| v0.5.3 (attuale) | 1700-1800 | - |
| + Lazy-SMP | +100 | 1800-1900 |
| + Pawn Hash | +30 | 1830-1930 |
| + Endgame | +80 | 1910-2010 |
| + Magic Bitboards | +50* | 1960-2060 |

*via greater search depth

**Target v0.6.0:** 2000 ELO

---

## 🗓️ Timeline Stimata

| Settimana | Focus | Deliverable |
|-----------|-------|-------------|
| 1 | Lazy-SMP + Pawn Hash | v0.6.0-alpha |
| 2 | Magic Bitboards | v0.6.0-beta |
| 3 | Endgame + Pruning | v0.6.0-rc |
| 4 | Testing + Polish | v0.6.0-final |

---

## ✅ Definition of Done

- [ ] 100% test suite pass
- [ ] 50%+ speedup on 4 threads
- [ ] ELO gain verified vs v0.5.3
- [ ] Documentation updated
- [ ] No regression in search quality

---

## 🔗 References

- [v0.5.3 Status](./FINAL_REPORT_v0.5.3.md)
- [Architecture](../docs/architecture/overview.md)
- [Performance Guide](../docs/reference/performance.md)

---

*"v0.6.0: Scaling to 2000 ELO through better parallelism and raw speed."*
