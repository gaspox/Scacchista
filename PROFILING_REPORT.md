# PROFILING REPORT - Scacchista Performance Analysis

**Data**: 2025-12-03
**Metodo**: Manual instrumentation + timing benchmarks

---

## 🎯 EXECUTIVE SUMMARY

**SCOPERTA CRITICA**: Il bottleneck NON è nella move generation!

**Performance Gap Identificato**:
- **Perft (move generation pura)**: 4.3M nodes/sec
- **Search (alpha-beta + eval)**: ~15k nodes/sec
- **Ratio**: **287x più lento durante search!**

**Conclusione**: Ottimizzare move generation avrà impatto limitato. Il vero bottleneck è nel **search loop** (evaluation, TT, move ordering, alpha-beta overhead).

---

## 📊 DATI MISURATI

### Benchmark 1: Funzioni Individuali

| Funzione | Tempo/chiamata | Chiamate/sec | Note |
|----------|----------------|--------------|------|
| `generate_moves()` | 3,235 ns | 309,119/s | Include legality check |
| `make_move() + unmake_move()` | **76 ns** | 13,157,895/s | MOLTO veloce! |
| `is_square_attacked()` | 31 ns | 32,258,065/s | Già ottimizzato |

**Osservazioni**:
- `make_move/unmake_move` sono ESTREMAMENTE veloci (76ns totali)
- Questo **invalida l'assunzione** che make/unmake siano 50-70% del tempo
- `is_square_attacked` è già molto veloce (31ns)

### Benchmark 2: Perft Performance

| Depth | Nodes | Tempo | Nodes/sec |
|-------|-------|-------|-----------|
| 5 | 4,865,609 | 1.13s | **4.3M** |

**Osservazioni**:
- Perft (solo move generation + make/unmake) raggiunge 4.3M nodes/sec
- Questo è **287x più veloce** del search (15k nodes/sec)
- Move generation NON è il bottleneck principale!

### Benchmark 3: Search Performance

| Test | Depth | Tempo | Est. Nodes | Est. Nodes/sec |
|------|-------|-------|------------|----------------|
| Startpos | 6 | 1.13s | ? | ~15k (da bench prev) |

**Problema**: Output UCI non include nodes count, solo time.

---

## 🔍 ANALISI BOTTLENECK

### ✅ NON SONO BOTTLENECK (validato)

1. **make_move/unmake_move**: Solo 76ns per coppia
   - Piano originale assumeva 50-70% del tempo
   - **ASSUNZIONE INVALIDATA**
   - Ottimizzare questi non aiuterà molto

2. **is_square_attacked**: Solo 31ns
   - Già molto veloce
   - Non vale la pena ottimizzare

3. **Move generation base**: 3.2μs per call
   - Perft raggiunge 4.3M nodes/sec
   - Se fosse il bottleneck, search andrebbe simile

### ❓ PROBABIL BOTTLENECK (da confermare)

1. **Evaluation Function** (NON MISURATO)
   - Chiamata ad ogni nodo
   - Potrebbe includere:
     - Material counting
     - PSQT lookup
     - Mobility calculation
     - King safety
     - Pawn structure analysis

2. **Transposition Table** (NON MISURATO)
   - Zobrist hash calculation (già incrementale)
   - TT probe (hash lookup)
   - TT store (scrittura)

3. **Move Ordering** (NON MISURATO)
   - MVV-LVA scoring
   - History heuristic lookup
   - Killer move check
   - Sorting overhead

4. **Search Overhead** (NON MISURATO)
   - Alpha-beta logic
   - Aspiration window management
   - Extension decisions
   - Pruning decisions (null-move, LMR, futility)

5. **Quiescence Search** (NON MISURATO)
   - Potrebbe essere chiamato molto spesso
   - Generate solo catture (più lento di quanto sembra?)

---

## 🚨 INVALIDAZIONE ASSUNZIONI PIANO ORIGINALE

| Assunzione Piano Originale | Validazione | Realtà Misurata |
|----------------------------|-------------|-----------------|
| make/unmake = 50-70% tempo | ❌ FALSA | <5% stimato |
| Heap allocations = 20-40% | ❓ Non misurato | Da verificare |
| piece_on() = 10-15% | ❓ Non misurato | Probabilmente <1% |
| Fast legality = 3-5x speedup | ❌ IMPROBABILE | Move gen già veloce |

**Impatto**:
- **Fase 3 (Fast Legality Check)**: Probabilmente inutile, make/unmake già velocissimi
- **Fase 2 (MoveList stack)**: Beneficio incerto senza misurare allocations
- **Fase 1 (Inline, Zobrist)**: Probabilmente marginale

---

## 📈 CALCOLO DEL VERO BOTTLENECK

### Breakdown Stimato (da validare)

Assumendo search @ 15k nodes/sec vs perft @ 4.3M nodes/sec:

```
Tempo totale per nodo nel search: 1,000,000 ns / 15,000 = 66,667 ns

Di cui:
- Move generation: ~3,235 ns (4.8%)
- Make/unmake: ~76 ns × 40 mosse = 3,040 ns (4.6%)
- is_square_attacked legality: ~31 ns × 40 = 1,240 ns (1.9%)

TOTALE Move Gen + Make/Unmake: ~7,515 ns (11.3%)

MANCA: 66,667 - 7,515 = 59,152 ns (88.7%!)
```

**Quindi l'88.7% del tempo è speso in QUALCOS'ALTRO!**

Probabili candidati per i 59,152 ns:
- Evaluation: ~20,000-30,000 ns? (30-45%)
- TT operations: ~10,000-15,000 ns? (15-23%)
- Move ordering: ~5,000-10,000 ns? (8-15%)
- Search logic: ~10,000-15,000 ns? (15-23%)
- Other: ~5,000 ns (8%)

---

## 🎯 RACCOMANDAZIONI BASATE SU DATI

### PRIORITÀ ALTA: Profilare il Search Loop

**Azione Immediata**: Instrumentare il search per misurare:

```rust
// In src/search/search.rs - negamax_pv()

let eval_time_start = Instant::now();
let score = self.evaluate();
EVAL_TIME.fetch_add(eval_time_start.elapsed().as_nanos(), Ordering::Relaxed);

let tt_time_start = Instant::now();
if let Some(entry) = self.tt.probe(zobrist) { ... }
TT_TIME.fetch_add(tt_time_start.elapsed().as_nanos(), Ordering::Relaxed);

// etc.
```

**Output atteso**:
```
=== SEARCH PROFILING ===
Evaluation: 35.2%
TT operations: 18.7%
Move ordering: 12.3%
Move generation: 11.3%
Search logic: 15.1%
Other: 7.4%
```

### PRIORITÀ MEDIA: Ottimizzazioni Basate su Nuovi Dati

**SE evaluation è >30%**:
1. Lazy evaluation (skip eval se alpha/beta cutoff precoce)
2. Incremental evaluation updates
3. Simplified eval a depth basso

**SE TT è >20%**:
1. Ottimizzare hash function
2. Reduce TT entry size
3. Better replacement scheme

**SE move ordering è >15%**:
1. Simplified MVV-LVA
2. Reduce history table lookups
3. Lazy sorting

### PRIORITÀ BASSA: Ottimizzazioni Move Generation

**Solo SE profiling conferma che è >20% del tempo**:
1. Stack-allocated MoveList (Fase 2 del piano originale)
2. Inline annotations (Fase 1)

**NON FARE**:
- ❌ Fast legality check (make/unmake già velocissimi!)
- ❌ Ottimizzare is_square_attacked (già 31ns)
- ❌ piece_on() mailbox (beneficio trascurabile)

---

## 🔬 NEXT STEPS

### Step 1: Instrumentare Search (1 ora)
- Aggiungere timing ad eval, TT, move ordering
- Compilare e run benchmark
- Generare breakdown percentuale

### Step 2: Analizzare Breakdown (30 min)
- Identificare top 3 bottleneck reali
- Validare/invalidare ipotesi

### Step 3: Strategia Rivista (30 min)
- Creare nuovo piano basato su dati REALI
- Eliminare ottimizzazioni inutili
- Aggiungere ottimizzazioni per veri bottleneck

### Step 4: Implementazione Mirata (variabile)
- Focus SOLO su top bottleneck verificati
- Misurare dopo ogni ottimizzazione
- Stop se non c'è improvement

---

## 📊 CONCLUSIONI

**Key Takeaway**: Il piano originale era basato su assunzioni errate.

**Verità Misurata**:
- ✅ Move generation è GIÀ veloce (4.3M nodes/sec in perft)
- ✅ Make/unmake sono GIÀ velocissimi (76ns)
- ❌ Il bottleneck è NEL SEARCH, non in move generation

**Decisione Strategica**:

**NON procedere con piano originale**. Invece:

1. **Completare profiling search** (1-2 ore)
2. **Identificare REALI bottleneck** (eval? TT? ordering?)
3. **Creare piano NUOVO** basato su dati
4. **Implementare solo ottimizzazioni validate**

**Stima Realistica**:

Se il bottleneck è nell'evaluation (probabile):
- Ottimizzare eval: possibile 2-3x speedup
- Lazy eval: possibile 1.5-2x
- Incremental eval: possibile 1.3-1.5x

**Target rivisto**: 30-60k nodes/sec (2-4x) tramite ottimizzazione search/eval, NON move generation.

---

**Next Action**: Instrumentare search loop per confermare ipotesi.
