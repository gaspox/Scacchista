# HANDOFF.md - Scacchista Chess Engine

**Data**: 2025-01-05
**Versione**: 0.2.1-beta
**Branch**: master
**Ultimo Commit**: perf: Optimize eval loop with direct bitboard iteration (+8-9% speedup)

---

## 📋 Panoramica Progetto

**Scacchista** è un motore di scacchi UCI scritto in Rust, progettato con focus su performance e correttezza. Implementa un'architettura engine classica con search alpha-beta, transposition table, e valutazione HCE (Hand-Crafted Evaluation).

### Caratteristiche Principali

- **UCI Protocol**: Completo con supporto per Hash, Threads, Style, Experience Book
- **Search Engine**: Alpha-beta con PVS, aspiration windows, quiescence, null-move, LMR, futility
- **Move Ordering**: TT moves, MVV-LVA+SEE, killer moves (2 slot/ply), history heuristic
- **Evaluation**: Material + PSQT + king safety + development penalty + pawn structure
- **Parallel Search**: Lazy-SMP broadcast architecture (shared TT, 2+ workers)
- **Time Management**: Allocazione intelligente con fallback strategies
- **Opening Book**: Polyglot format support
- **Endgame**: Syzygy tablebase probing (WDL/DTC)
- **Experience Book**: Self-play learning con Q-like backpropagation

### Stack Tecnologico

- **Linguaggio**: Rust (edition 2021)
- **Dipendenze Core**:
  - `shakmaty` / `shakmaty-uci` / `shakmaty-syzygy` (regole e validazione)
  - `serde` / `bincode` (persistenza)
  - Board interno custom con bitboards per performance

---

## 🎯 Sessione Corrente: Analisi e Ottimizzazioni

### Obiettivi

1. ✅ Mergiare feature/uci-phase3 in master
2. ✅ Analisi approfondita del motore (GrandMaster agent)
3. ✅ Fix bug TT sharing in lazy-SMP
4. ✅ Ottimizzazione eval loop con bitboard iteration

### Lavoro Svolto

#### 1. Merge feature/uci-phase3 → master

**Contenuto del merge**:
- Fix UCI `go depth N` timeout issue (3 nuovi test UCI)
- TT optimization: incremental zobrist + improved replacement scheme (~1.6% speedup)
- Lazy evaluation: implementata e testata, ma REVERTED per regressione -10%

**Performance cumulativa pre-sessione**:
- Depth 6: 1763ms → 559ms = **3.15x speedup**
- Depth 8: ~26s → 15.2s = **1.7x speedup**
- 80/80 test passing

**Commit**: `650db68` (Merge branch 'master' + feature/uci-phase3)

---

#### 2. Analisi GrandMaster: Audit Completo del Motore

Invocato `GrandMaster` agent per analisi tecnica approfondita. **Findings principali**:

##### 🔴 **Bug Critici**

1. **TT non condivisa in ThreadManager** (CRITICO)
   - Workers creavano TT locali 16MB invece di condividere la globale
   - `_tt_clone` clonata ma mai usata
   - Vanificava completamente benefici lazy-SMP

2. **Magic Bitboards non implementati** (nonostante CLAUDE.md li menzioni)
   - Generazione sliding pieces usa loop manuali O(7n) invece di O(1)
   - Opportunità: 3-5x speedup in move generation

3. **Eval loop inefficiente**
   - `piece_on(sq)` itera 12 bitboard per ogni casella → O(768) checks
   - Dovrebbe iterare sui bitboard → O(~32) operazioni

##### 📊 **Valutazione Architetturale**

| Aspetto | Valutazione | Note |
|---------|-------------|------|
| Modularità | ⭐⭐⭐⭐⭐ | Separazione netta board/search/eval/uci |
| Test Coverage | ⭐⭐⭐⭐ | 80 test, perft validation, tactical tests |
| Search Completeness | ⭐⭐⭐⭐ | Null-move, LMR, futility, aspiration windows |
| TT Implementation | ⭐⭐⭐ | Replacement scheme buono, ma single-bucket |
| Evaluation | ⭐⭐⭐ | HCE solid, mancano passed pawn, bishop pair |
| Parallel Search | ⭐⭐ | Architettura corretta ma bug + manca diversity |

##### 🎯 **Priority Roadmap**

**Quick Wins** (ROI alto, sforzo basso):
1. ✅ Fix TT sharing (~1 ora) → +30-50 ELO su multi-core
2. ✅ Eval bitboard iteration (~2 ore) → ~30-40% speedup stimato
3. ⏳ SEE cache array fisso (~30 min) → ~5-10% speedup
4. ⏳ Razoring (~1 ora) → ~2-3% speedup

**Medium-Term** (1-2 settimane):
5. ⏳ Magic Bitboards → 3-5x speedup move gen
6. ⏳ Passed Pawn detection → +30-50 ELO
7. ⏳ Tapered Evaluation (MG/EG) → +20-30 ELO

**Long-Term Strategic**:
8. ⏳ NNUE Integration → +200-400 ELO
9. ⏳ Syzygy Tablebase Integration (già in Cargo.toml)

**Stima ELO**: Attuale 1500-1800 → Potenziale 2200-2400 (senza NNUE)

---

#### 3. Quick Win #1: Fix TT Sharing + Broadcast Lazy-SMP

##### Problema

ThreadManager aveva bug critico:
```rust
let _tt_clone = tt.clone();  // Clonata ma mai usata!
let mut search = Search::new(board, 16, params).with_stop_flag(job_stop);
```

Ogni worker creava TT locale da 16MB, vanificando lazy-SMP.

##### Soluzione (2 commit)

**Commit 1**: `87bd947` - Enable true TT sharing
- Cambiato `Search::tt` da owned a `Arc<Mutex<TranspositionTable>>`
- Aggiunto `with_shared_tt()` builder method
- Aggiornati tutti 9 TT accesses con `lock().unwrap()` pattern
- ThreadManager passa shared TT ai worker

**Commit 2**: `06f0c82` - Broadcast lazy-SMP architecture
- Rimosso work-stealing queue
- Implementato broadcast model: tutti N worker cercano stessa posizione
- Sync con `job_available` atomic, `workers_done` counter, results vector
- Best result scelto tra tutti i worker

##### Risultati

**Test**: 80/80 passing, zero regressioni ✅

**Performance**:
- Depth 7: 1 thread 2.386s, 2 threads 2.283s = **+4.5% speedup**
- Depth 9: 1 thread 53.817s, 2 threads 53.879s = **0% speedup** ⚠️

**Root Cause**: Tutti i worker eseguono ricerca identica. Primo a finire vince, altri fermati senza contribuire. CPU utilization 200% ma zero wall-clock benefit.

**Manca**: Artificial diversity (per-worker history, aspiration width, random eval perturbations, ID depth offsets) per esplorare paths diversi.

**Status**: Infrastruttura broadcast corretta e testata, ma serve diversity layer per speedup reale. **Deferred** per focus su ottimizzazioni single-thread con ROI garantito.

---

#### 4. Quick Win #2: Eval Loop Optimization

##### Problema

Eval loop originale:
```rust
for sq in 0..64 {
    if let Some((kind, color)) = board.piece_on(sq) {
        // piece_on() itera 12 bitboard → 64*12 = 768 checks totali
    }
}
```

Estremamente inefficiente: ~2/3 delle caselle sono vuote.

##### Soluzione

**Commit**: `2b977cd` - Optimize eval loop with direct bitboard iteration

Bitboard iteration diretta:
```rust
for &kind in &piece_kinds {
    let material_value = ...;
    let psqt_table = ...;

    let mut white_bb = board.piece_bb(kind, Color::White);
    while white_bb != 0 {
        let sq = white_bb.trailing_zeros() as usize;
        white_bb &= white_bb - 1;  // Clear LSB
        white_score += material_value + psqt_table[sq];
    }
    // Same for black
}
```

Complessità: O(768) → O(~32) operazioni (media 3-5 pezzi per tipo * 12 bitboard).

##### Risultati

**Test**: 57/57 unit tests passing, 10/10 eval tests passing ✅

**Performance**:

| Depth | Prima | Dopo | Speedup |
|-------|-------|------|---------|
| 7     | 2.386s | 2.180s | **+9.5%** |
| 8     | ~15.2s | 14.148s | **+7.4%** |

**Speedup consistente 8-9%** across depths.

**Note**: GrandMaster stimava 30-40%, realtà è 8-9% perché:
1. Compiler Rust già ottimizzava `piece_on()` efficacemente
2. Eval non è l'unico bottleneck (TT, move gen, pruning contribuiscono)
3. CPU moderni gestiscono branch prediction bene in tight loops

Comunque **8-9% è un guadagno solido e misurabile**.

---

## 📊 Performance Complessiva Post-Sessione

### Benchmark Risultati

| Metrica | Baseline Iniziale | Post-Sessione | Guadagno Totale |
|---------|------------------|---------------|-----------------|
| **Depth 6** | 1763ms | ~500ms* | **~3.5x** |
| **Depth 7** | ~10s** | 2.180s | **~4.6x** |
| **Depth 8** | ~26s** | 14.148s | **~1.8x** |

*stima basata su speedup cumulativo (559ms pre-sessione * 0.92 post-eval-opt)
**stime basate su extrapolazione da depth 6

### Breakdown Ottimizzazioni

| Ottimizzazione | Speedup | Commit | Status |
|----------------|---------|--------|--------|
| Evaluate_fast (phase 2) | ~3.1x | f9be0ae (pre) | ✅ Deployed |
| TT incremental zobrist | ~1.6% | 4e50c11 (pre) | ✅ Deployed |
| Eval bitboard iteration | ~8-9% | 2b977cd | ✅ Deployed |
| TT sharing fix | 0% (needs diversity) | 87bd947 | ✅ Infra ready |
| Lazy eval threshold | -10% (reverted) | c56dfb5 | ❌ Not deployed |

### Test Coverage

- **Unit tests**: 57/57 passing
- **Integration tests**: 23/23 passing
- **Total**: 80/80 tests passing ✅
- **Perft validation**: Exact match a tutte le profondità ✅
- **Tactical tests**: 7/7 passing ✅

---

## 🗂️ Architettura Codebase

### Struttura Directory

```
Tal/
├── src/
│   ├── main.rs              # Entry point UCI
│   ├── lib.rs               # Module exports
│   ├── board.rs             # Bitboard representation + move gen
│   ├── zobrist.rs           # Zobrist hashing tables
│   ├── utils.rs             # Attack tables, magic bitboards stub
│   ├── eval.rs              # HCE evaluation (OTTIMIZZATO ✅)
│   ├── search/
│   │   ├── mod.rs
│   │   ├── search.rs        # Core alpha-beta + pruning
│   │   ├── tt.rs            # Transposition table (SHARED ✅)
│   │   ├── thread_mgr.rs    # Lazy-SMP manager (BROADCAST ✅)
│   │   ├── stats.rs         # Search statistics
│   │   └── params.rs        # Search parameters
│   ├── time/
│   │   └── mod.rs           # Time management
│   ├── uci/
│   │   ├── mod.rs
│   │   ├── loop.rs          # UCI main loop
│   │   ├── parser.rs        # Command parsing
│   │   └── options.rs       # UCI options
│   └── bin/
│       ├── perft.rs         # Move gen validation
│       ├── simple_search_test.rs
│       └── stress_search_test.rs
├── tests/                   # Integration tests
├── CLAUDE.md                # Project guide per Claude Code
├── AGENTS.md                # Agent specialization config
└── HANDOFF.md               # Questo documento

```

### File Chiave Modificati Oggi

1. **src/search/search.rs** (87bd947)
   - `tt: Arc<Mutex<TranspositionTable>>`
   - `with_shared_tt()` method
   - 9 TT accesses aggiornati con lock pattern

2. **src/search/thread_mgr.rs** (06f0c82, 87bd947)
   - Broadcast lazy-SMP architecture
   - job_available, workers_done, results vector
   - stop semantics migliorati

3. **src/eval.rs** (2b977cd)
   - Bitboard iteration diretta
   - O(768) → O(32) complessità

---

## 🐛 Known Issues

### 1. Lazy-SMP Non Scala (Needs Artificial Diversity)

**Problema**: Workers eseguono ricerca identica, nessuno speedup multi-thread.

**Root Cause**: Shared TT + nessuna diversità artificiale → convergenza immediata su PV identico.

**Soluzioni Possibili**:
- Per-worker history tables (non condivise)
- Different aspiration window widths per worker
- Helper threads a depth ID diversi
- Random eval perturbations (~5cp)
- Different move ordering seeds

**Effort**: Medium (1-2 giorni)
**Impatto**: +50-80% speedup su 2 threads, +100-150% su 4 threads (tipico lazy-SMP)

### 2. SEE Cache con HashMap (Overhead)

**Problema**: HashMap allocation/lookup per SEE caching.

**Fix**: Usare `[Option<i16>; 64]` array fisso.

**Effort**: 30 minuti
**Impatto**: ~5-10% speedup in move ordering

### 3. Magic Bitboards Non Implementati

**Problema**: Sliding piece generation usa loop manuali O(7n).

**Fix**: Implementare magic bitboards con lookup tables precomputed.

**Effort**: 1-2 settimane
**Impatto**: 3-5x speedup in move generation

---

## 🚀 Roadmap Prossimi Sviluppi

### Immediate (1-3 giorni)

1. **SEE Cache Array Fisso** (30 min)
   - Replace HashMap con `[Option<i16>; 64]`
   - Speedup: ~5-10%

2. **Razoring** (1 ora)
   - Quick pruning technique per nodes non-PV
   - Speedup: ~2-3%

3. **Lazy-SMP Diversity** (1-2 giorni)
   - Implement per-worker history tables
   - Random eval noise (~5cp)
   - Speedup: +50-80% su 2 threads

### Short-Term (1-2 settimane)

4. **Magic Bitboards** (1-2 settimane)
   - Precompute attack tables
   - Replace loop-based sliding generation
   - Speedup: 3-5x move gen

5. **Passed Pawn Evaluation** (2-3 giorni)
   - Detection con bitboard masks
   - Bonuses progressivi per rank
   - ELO gain: +30-50

6. **Tapered Evaluation** (2-3 giorni)
   - Separate MG/EG scores
   - Linear interpolation by phase
   - ELO gain: +20-30

7. **Bishop Pair Bonus** (1 ora)
   - Simple check: popcnt(bishop_bb) == 2
   - Bonus: ~30-50cp
   - ELO gain: +10-15

### Medium-Term (1-2 mesi)

8. **Pawn Hash Table** (1 settimana)
   - Cache pawn structure evaluation
   - Speedup: ~10-15% (pawn eval expensive)

9. **King Safety Improvements** (1 settimana)
   - Pawn storm detection
   - Attacking piece proximity
   - ELO gain: +20-30

10. **Multi-PV Support** (3-5 giorni)
    - UCI multipv option
    - Track top-N moves
    - Useful per analysis mode

### Long-Term Strategic (3-6 mesi)

11. **NNUE Integration** (2-3 mesi)
    - Stockfish NNUE eval via FFI, oppure
    - Custom NNUE training (HalfKP architecture)
    - ELO gain: +200-400

12. **Advanced Pruning** (1-2 mesi)
    - Singular Extensions
    - Multi-Cut
    - Probcut
    - ELO gain: +50-100 cumulativo

13. **Syzygy Tablebase Full Integration** (1 settimana)
    - Probe in root + search (già in Cargo.toml)
    - DTZ-aware search termination
    - Perfect play in <=6 pieces endgames

14. **Experience Book Improvements** (2-3 settimane)
    - Tuning Q-learning parameters
    - Forgetting mechanism per bad lines
    - Move confidence scoring

---

## 📚 Documentazione Riferimenti

### File Documentazione Progetto

- **CLAUDE.md**: Istruzioni per Claude Code (comandi, architettura, stile)
- **AGENTS.md**: Configurazione specialized agents (GrandMaster, StressStrategist, etc.)
- **README.md**: Overview pubblico del progetto
- **HANDOFF.md**: Questo documento

### Risorse Esterne

**Chess Programming**:
- Chess Programming Wiki: https://www.chessprogramming.org/
- Stockfish source: https://github.com/official-stockfish/Stockfish
- Lazy-SMP paper: Lazy SMP, Better Than Parallel Alpha-Beta (Brinkmann 2013)

**Rust Chess Engines**:
- Rustic: https://github.com/mvanthoor/rustic
- Vice (C, reference): https://github.com/bluefeversoft/vice

**Testing Suites**:
- WAC (Win At Chess): 300 tactical positions
- Bratko-Kopec: 24 strategic positions
- STS (Strategic Test Suite): 1500 positions categorized

---

## 🔧 Setup e Comandi

### Build

```bash
# Debug
cargo build

# Release (sempre per benchmark!)
cargo build --release
```

### Testing

```bash
# All tests
cargo test

# Specific test
cargo test test_tt_replacement

# Lib tests only
cargo test --lib

# Integration tests
cargo test --test '*'
```

### Benchmark

```bash
# Perft (move gen validation)
cargo run --release --bin perft -- --depth 6

# Search depth benchmark
time printf 'uci\nsetoption name Threads value 1\nposition startpos\ngo depth 7\nquit\n' | ./target/release/scacchista

# UCI interaction
./target/release/scacchista
# (then type UCI commands)
```

### Lint & Format

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

## 🎓 Lessons Learned

### 1. Data-Driven Optimization Works

- Profiling rivelò che eval era bottleneck (non move gen come ipotizzato)
- Lazy eval mostrava -10% regression su balanced positions → REVERTED
- Bitboard iteration: 8-9% reale vs 30-40% stimato → adjust expectations

**Takeaway**: Measure, test, validate. Revert when data doesn't support deployment.

### 2. Architecture Matters More Than Quick Hacks

- Broadcast lazy-SMP infrastructure corretta ma serve diversity layer
- Meglio architettura pulita + deferred than hack veloce
- TT sharing fix ha zero regression perché design corretto

**Takeaway**: Invest in clean architecture upfront, pay dividends later.

### 3. Test Coverage è Essenziale

- 80 tests caught regressions multiple volte
- Perft validation garantisce move gen correctness
- Tactical tests (7/7) assicurano no search bugs

**Takeaway**: Comprehensive testing enables confident refactoring.

### 4. Compiler Optimizations Can Surprise

- `piece_on()` già ottimizzato dal compiler → speedup minore dell'atteso
- Debug assertions (`debug_assert_eq!`) zero overhead in release
- Branch prediction moderna mitiga alcuni pattern anti-performance

**Takeaway**: Don't assume bottlenecks, profile first.

---

## 📞 Contatti e Ownership

**Progetto**: Scacchista UCI Chess Engine
**Owner**: Gaspare (@gaspox)
**Repository**: github.com:gaspox/Scacchista.git
**Branch**: master
**Ultima Modifica**: 2025-01-05
**Stato**: Active Development

**Development Environment**:
- OS: Linux 6.12.48-1-MANJARO
- Rust: Edition 2021
- IDE: Claude Code / Claude Agent SDK

---

## ✅ Checklist Pre-Handoff

- [x] Tutti i test passing (80/80)
- [x] Build release funzionante
- [x] Performance benchmarked e documentata
- [x] Commit messages descrittivi e completi
- [x] Known issues documentati
- [x] Roadmap prioritizzata
- [x] HANDOFF.md completo
- [x] Git history pulita (no WIP commits)
- [x] Branch feature mergiati in master
- [x] Push a remote completato

---

## 🎯 Quick Start per Nuovo Contributor

1. **Clone e Setup**:
   ```bash
   git clone github.com:gaspox/Scacchista.git
   cd Scacchista
   cargo build --release
   cargo test
   ```

2. **Leggi Documentazione**:
   - CLAUDE.md (project guide)
   - AGENTS.md (agent config)
   - Questo HANDOFF.md

3. **Run Benchmark Baseline**:
   ```bash
   time printf 'uci\nsetoption name Threads value 1\nposition startpos\ngo depth 7\nquit\n' | ./target/release/scacchista
   ```
   Expected: ~2.18s

4. **Pick Next Task**:
   - SEE cache array (30 min, easy)
   - Razoring (1 ora, easy)
   - Lazy-SMP diversity (1-2 giorni, medium)

5. **Development Workflow**:
   - Create feature branch
   - Implement + test
   - Benchmark before/after
   - Document in commit message
   - PR to master

---

**Fine HANDOFF.md**

Timestamp: 2025-01-05 23:59 UTC
Generated by: Claude Code Assistant
Review Status: Ready for Handoff ✅
