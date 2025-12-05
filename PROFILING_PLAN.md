# FASE 0: PROFILING PLAN - Scacchista Performance Analysis

**Data**: 2025-12-03
**Obiettivo**: Identificare i REALI bottleneck tramite profiling prima di ottimizzare

---

## 🎯 DOMANDE DA RISPONDERE

### 1. Qual è la distribuzione del tempo CPU?
- % in `make_move` / `unmake_move`
- % in `generate_moves` / `generate_pseudo_moves`
- % in `is_square_attacked`
- % in evaluation function
- % in TT lookup/probe
- % in altre funzioni

### 2. Quali funzioni sono più "hot"?
Top 10 funzioni per:
- Tempo totale (inclusive time)
- Tempo proprio (exclusive time)
- Numero di chiamate

### 3. Dove sono i memory bottleneck?
- Numero allocazioni heap
- Cache miss rate (se perf disponibile)
- Memory bandwidth usage

### 4. Validazione assunzioni
- ✅/❌ make/unmake è 50-70% del tempo?
- ✅/❌ Allocazioni heap sono 20-40% overhead?
- ✅/❌ piece_on() è 10-15% overhead?

---

## 🔧 STRUMENTI DI PROFILING

### Opzione A: cargo-flamegraph (Visual Flame Graph)
```bash
# Installazione
cargo install flamegraph

# Profiling perft depth 5
sudo cargo flamegraph --bin perft -- --depth 5

# Genera flamegraph.svg visualizzabile nel browser
```

**Pro**: Visualizzazione chiara, facile da interpretare
**Contro**: Richiede sudo, potrebbe non funzionare su tutte le distro

### Opzione B: perf (Linux Performance Tools)
```bash
# Build release
cargo build --release

# Profiling con perf
perf record --call-graph=dwarf ./target/release/perft --depth 5

# Analisi
perf report
```

**Pro**: Tool standard Linux, molto dettagliato
**Contro**: Output meno user-friendly

### Opzione C: Manual Instrumentation
```rust
// Aggiungere timing manuale alle funzioni critiche
use std::time::Instant;

pub fn generate_moves(&mut self) -> Vec<Move> {
    let start = Instant::now();
    // ... codice ...
    println!("generate_moves: {:?}", start.elapsed());
}
```

**Pro**: Non richiede tool esterni, molto specifico
**Contro**: Invasivo, richiede modifica codice

---

## 📊 BENCHMARK SUITE PER PROFILING

### Test 1: Perft Depth 5 (Baseline)
```bash
cargo flamegraph --bin perft -- --depth 5
```
**Posizione**: startpos
**Nodi attesi**: 4,865,609
**Tempo atteso**: ~300ms @ 15k nodes/sec
**Obiettivo**: Profilo generale move generation

### Test 2: Perft Depth 4 (Kiwipete)
```bash
cargo flamegraph --bin perft -- --depth 4 \
  --fen "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"
```
**Nodi attesi**: 4,085,603
**Obiettivo**: Posizione complessa con castling, pin, check

### Test 3: Search Depth 6
```bash
# Profiling su search (non solo move gen)
cargo flamegraph -- <<EOF
uci
position startpos
go depth 6
quit
EOF
```
**Obiettivo**: Profilo completo search + eval + TT

### Test 4: Quiescence Search Heavy
```bash
# Posizione tattica con molte catture
cargo flamegraph -- <<EOF
uci
position fen r1bq1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 1
go depth 6
quit
EOF
```
**Obiettivo**: Stress test quiescence search

---

## 📈 METRICHE DA RACCOGLIERE

### Per ogni funzione critica:

| Funzione | Self Time | Total Time | Calls | Avg Time/Call |
|----------|-----------|------------|-------|---------------|
| generate_moves | ? | ? | ? | ? |
| make_move | ? | ? | ? | ? |
| unmake_move | ? | ? | ? | ? |
| is_square_attacked | ? | ? | ? | ? |
| generate_pseudo_moves | ? | ? | ? | ? |
| generate_pawn_pseudos | ? | ? | ? | ? |
| piece_on | ? | ? | ? | ? |
| TT::probe | ? | ? | ? | ? |
| evaluate | ? | ? | ? | ? |

### Metriche aggregate:

- **Move generation total**: (generate_moves + generate_pseudo + legality check)
- **Board manipulation**: (make_move + unmake_move)
- **Attack detection**: (is_square_attacked + helpers)
- **Search overhead**: (TT + ordering + extensions)
- **Evaluation**: (eval functions)
- **Other**: Tutto il resto

**Target distribuzione attesa**:
```
Move generation: 40-50%
Board manipulation: 30-40%
Attack detection: 10-20%
Search overhead: 5-10%
Evaluation: 5-10%
Other: <5%
```

---

## 🔍 ANALISI PATTERN DA CERCARE

### 1. Funzioni "Unexpectedly Hot"
Funzioni che consumano molto tempo ma non dovrebbero:
- `init_zobrist()` chiamato ripetutamente
- `Vec::push()` con reallocazioni
- Clone/Copy non necessari

### 2. Deep Call Stacks
Funzioni chiamate da molti livelli di profondità:
- Overhead di chiamate funzione
- Candidati per inlining

### 3. Memory Allocation Hotspots
```
Vec::with_capacity -> allocate
Vec::push -> realloc
```

### 4. Cache Miss Indicators
(se usando perf)
```bash
perf stat -e cache-misses,cache-references ./target/release/perft --depth 5
```

---

## 📋 PROFILING WORKFLOW

### Step 1: Baseline Measurement (30 min)
```bash
# Build release
cargo build --release

# Perft baseline
time ./target/release/perft --depth 5

# Output: 4865609 nodes, X seconds, Y nodes/sec
```

### Step 2: Flamegraph Generation (30 min)
```bash
# Run profiling
sudo cargo flamegraph --bin perft -- --depth 5

# Aprire flamegraph.svg in browser
firefox flamegraph.svg

# Screenshot per documentazione
```

### Step 3: Identify Top Bottlenecks (30 min)
- Annotare le 5 funzioni più "wide" nel flamegraph
- Annotare % tempo per ciascuna
- Verificare se match con assunzioni

### Step 4: Deep Dive Suspect Functions (1 ora)
Per ogni bottleneck identificato:
- Leggere il codice sorgente
- Identificare possibili ottimizzazioni
- Stimare impatto se ottimizzata

### Step 5: Profiling Report (30 min)
Creare documento con:
- Top 5 bottleneck
- Validazione/invalidazione assunzioni
- Raccomandazioni di ottimizzazione
- Priorità (HIGH/MEDIUM/LOW)

---

## ✅ DELIVERABLE

Al termine della Fase 0, produrre:

### 1. Flamegraph SVG
- `flamegraph_perft_d5.svg`
- `flamegraph_search_d6.svg`

### 2. Profiling Report (PROFILING_REPORT.md)
```markdown
# Top 5 Bottleneck

1. **Function Name** (XX.X% total time)
   - Self time: YY.Y%
   - Calls: NNNN
   - Avg time/call: MM μs
   - **Optimization opportunity**: [description]
   - **Expected impact**: X.Xx speedup

2. [...]
```

### 3. Updated Optimization Strategy
- Rivedere piano originale alla luce dei dati
- Riprioritizzare ottimizzazioni
- Eliminare ottimizzazioni non necessarie

---

## 🚦 GO/NO-GO CRITERIA

Prima di procedere con implementazione:

**GO** se:
- [ ] Profiling identifica almeno 3 bottleneck chiari (>5% ciascuno)
- [ ] Top bottleneck ha ottimizzazione chiara con impatto stimato >1.3x
- [ ] Assunzioni del piano originale sono >60% validate

**REVISIT STRATEGY** se:
- [ ] Nessun bottleneck chiaro (distribuzione uniforme)
- [ ] Top bottleneck è in libreria esterna (shakmaty)
- [ ] Assunzioni del piano originale sono <40% validate

**ABORT** (investigare alternative) se:
- [ ] Il vero bottleneck è architetturale (es. design fondamentalmente inefficiente)
- [ ] Miglioramenti stimati sono <1.2x anche con ottimizzazioni aggressive

---

## 📝 NOTES

- Profiling deve essere su **build release** (altrimenti risultati non rappresentativi)
- Evitare posizioni troppo shallow (depth < 4) - non stressano abbastanza
- Evitare posizioni troppo deep (depth > 6) - profiling troppo lungo
- Salvare output grezzo per future reference
- Confrontare profili tra perft e search (potrebbero essere molto diversi)

---

**Next Step**: Installare flamegraph e generare primo profiling su perft depth 5
