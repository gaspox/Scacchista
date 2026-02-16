# Piano di Miglioramento Scacchista v0.5.0

Obiettivo: rendere il motore **più forte** (miglior gioco tattico e posizionale) e **più responsive** (nodi/secondo più alti, gestione tempo intelligente).

I miglioramenti sono ordinati per **impatto/sforzo**, dal più vantaggioso al più complesso.

## Stato Attuale (v0.5.0-dev) ✅
- **Fase 1 (Performance)**: Completata (Delta Pruning, TT Lock-free, Eval Bitboard).
- **Fase 2.1 (PVS Root)**: Completata (incluso fix critici CI/Panic).
- **Prossimo Step**: Fase 2.2 (Internal Iterative Reduction).

---

## Fase 1 — Performance (Responsività) ⚡

Aumento immediato di nodi/s senza cambiare la logica di gioco.

> [!IMPORTANT]
> Queste ottimizzazioni sono prerequisiti: più nodi/s → depth più alta → motore più forte.

### Phase 1.1: Capture Generation (Dedicated) [COMPLETED]
#### [Modify] `src/board.rs`
- Add `generate_captures()` function
- Add helper functions for each piece type
- Optimize `qsearch()` to use this instead of full generation

### Phase 1.2: Delta Pruning [COMPLETED]
#### [Modify] `src/search/search.rs`
- Add delta pruning logic in `qsearch()` loop
- Skip captures where `stand_pat + victim_val + margin < alpha`

### Phase 1.3: Lock-free TT [COMPLETED]
#### [Modify] `src/search/tt.rs`
- Change storage to `Vec<AtomicU64>`
- Implement atomic pack/unpack logic
- Remove Mutex usage in `search.rs` and `thread_mgr.rs`

### Phase 1.4: Evaluation Optimization [COMPLETED]
#### [Modify] `src/eval.rs`
- Replace square-centric evaluation loop with bitboard operations
- Use popcount and bit manipulation for material counting
- Optimize mobility calculation using attack bitboards

---

## Fase 2 — Forza Tattica 🗡️

Miglioramento diretto della capacità di trovare combinazioni e mosse tattiche.

### 2.1 PVS (Principal Variation Search) corretto al root [COMPLETED]

#### [MODIFY] [search.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/search/search.rs#L347-L430)

Al root (`iddfs`), la prima mossa viene cercata con finestra piena, le successive con null-window `(-alpha-1, -alpha)` e re-search solo se fail-high. Attualmente tutte le mosse root usano finestra piena → spreco. Guadagno: **10-20% cutoff** in più.

> **CI Fixes (Fase 2.1.1)**: Risolti panic `i16::MIN`, bug TT bounds, e logic timeout score `-32000`. Test suite 81/81 passati.

### 2.2 IIR (Internal Iterative Reduction) [COMPLETED]

#### [MODIFY] [search.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/search/search.rs#L432-L818)

Se non c'è TT move al nodo PV con depth ≥ 4, ridurre depth di 1. Più semplice della vecchia IID e molto efficace. Guadagno: **5-10% nodi**.
- **Benchmark Update**: NPS 723k (+19% vs baseline), nodi corretti post-fix TT.
- **Prossimo Step**: Fase 2.4 (Countermove Heuristic). [TODO]

### 2.3 SEE Pruning per catture perdenti in qsearch [COMPLETED]

#### [MODIFY] [search.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/search/search.rs)

Non cercare catture su case difese dove *nessuna* cattura è vantaggiosa (Target-based pruning).
- **Fix**: Correzione logica SEE (backprop, pawn attacks).
- **Benchmark**: 5x speedup (25s -> 5s), nodi ridotti da 18.7M a 3.8M con NPS stabile (761k). Engine tatticamente più solido.

### 2.4 Countermove Heuristic [COMPLETED]

#### [MODIFY] [search.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/search/search.rs)

Aggiungere array `countermoves[piece][to_sq]`: quando una mossa produce beta-cutoff, memorizzare come "risposta" alla mossa precedente. Usato nel move ordering dopo killer moves. Costo: ~200 righe. Guadagno: **5-10 Elo**.
- **Benchmark**: Countermove Effectiveness 25.8% (cutoffs indotti).

---

## Fase 3 — Valutazione Posizionale 🧠 [TODO]

Eval più accurata = scelte strategiche migliori. Impatto: **30-80 Elo**.

### 3.1 Tapered Evaluation (MG/EG)

#### [MODIFY] [eval.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/eval.rs)

Implementare PSQT separate per middlegame ed endgame. Calcolare `phase` dalla quantità di materiale. Score finale = interpolazione `(mg_score * phase + eg_score * (256 - phase)) / 256`. Questo è il singolo miglioramento eval più impattante (~**30-40 Elo**).

```rust
// Esempio concettuale
let phase = compute_phase(&board); // 256=MG puro, 0=EG puro
let mg = white_mg - black_mg;
let eg = white_eg - black_eg;
let score = (mg * phase + eg * (256 - phase)) / 256;
```

### 3.2 Mobilità

#### [MODIFY] [eval.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/eval.rs)

Contare le caselle raggiungibili da ogni pezzo (escludendo caselle bloccate da pedoni amici). Bonus per alta mobilità, penalità per pezzi bloccati. Focus su: Cavallo, Alfiere, Torre, Donna. Impatto: ~**15-20 Elo**.

### 3.3 Coppia degli Alfieri

#### [MODIFY] [eval.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/eval.rs)

Bonus di ~30-50 cp quando un lato ha entrambi gli alfieri. Semplice ma efficace. Impatto: ~**5 Elo**.

### 3.4 Torre su colonna aperta/semiaperta

#### [MODIFY] [eval.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/eval.rs)

Bonus per torre su colonna senza pedoni amici (semiaperta: +10 cp) o senza pedoni di nessun colore (aperta: +20 cp). Impatto: ~**5-10 Elo**.

---

## Fase 4 — Time Management Intelligente ⏱️ [TODO]

### 4.1 Soft/Hard Time Limit

#### [MODIFY] [time/mod.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/time/mod.rs)
#### [MODIFY] [search.rs](file:///home/gaspare/Documenti/TAL/Scacchista/src/search/search.rs)

- **Soft limit** (~50% del tempo allocato): non iniziare nuova iterazione ID se superato
- **Hard limit** (~200% del tempo allocato): abort immediato della ricerca

Attualmente c'è un solo limite → il motore o spreca tempo o lo finisce improvvisamente.

### 4.2 Score instability → tempo extra

Quando il best move cambia tra iterazioni consecutive, allocare il 50% di tempo extra. Evita di giocare mosse dubbie con poca fiducia.

---

## Fase 5 — Avanzato (Futuro) 🔮

| Intervento | Difficoltà | Impatto |
|---|---|---|
| NNUE Evaluation | Alta | +200 Elo |
| Syzygy Tablebases | Media | +20 Elo (finali) |
| Multi-PV per analisi | Bassa | UX |
| Pondering (think on opponent time) | Media | +30 Elo (tempo) |
| Opening Book | Bassa | +10 Elo (apertura) |

---

## Priorità di Implementazione Suggerita

```mermaid
graph LR
    A["1.1 generate_captures()"] --> B["1.2 Delta Pruning"]
    B --> C["1.4 evaluate_fast bitboard"]
    C --> D["2.1 PVS root"]
    D --> E["3.1 Tapered Eval"]
    E --> F["3.2 Mobilità"]
    F --> G["4.1 Soft/Hard Time"]
    G --> H["1.3 TT lock-free"]
    H --> I["2.2 IIR"]
    I --> J["2.4 Countermove"]
```

**Stima totale**: Fasi 1-4 → **+80-150 Elo** rispetto all'attuale v0.4.1.

## Verification Plan

### Automated Tests
- `cargo test` dopo ogni modifica
- Perft tests invariati (move generation non cambia nella Fase 1-3)
- Benchmark nodes/second prima e dopo ogni ottimizzazione performance
- WAC test suite (attualmente 0/2) — obiettivo post-Fase 2: ≥1/2

### Manual Verification
- Partite self-play (v0.5.0 vs v0.4.1) con `cutechess-cli`, ~100 partite a TC 10+0.1
- Confronto profondità raggiunta a parità di tempo in posizioni standard
