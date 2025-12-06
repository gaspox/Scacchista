# Test Pratici v0.4.0-rc

**Data**: 2025-12-06
**Build**: Scacchista v0.4.0-rc (commit d356c44)
**Miglioramenti**: King Safety, Check Extensions, Pawn Bonuses

---

## Test 1: Posizione Critica - Mossa 13 (prova_2.pgn)

### Setup

**Posizione dopo 12...O-O:**
```
FEN: r1bqk2r/pp2bppp/1np2n2/2pP4/3P4/1BN2N2/PP3PPP/R1BQ1RK1 w - - 0 13
```

**Mosse fino a questa posizione:**
```
1.Nf3 d5 2.d3 c5 3.Bf4 Nf6 4.Bxb8 Rxb8 5.Nc3 b6
6.d4 Bb7 7.e4 e6 8.Bb5+ Nd7 9.exd5 exd5 10.Qe2+ Be7
11.Bxd7+ Qxd7 12.b3 O-O
```

**Problema nella partita originale:**
- 13.Qxe7?? Qxe7+ 14.Kf1 → Bianco perde diritto arrocco in apertura

**Obiettivo del test:**
Verificare se i miglioramenti al king safety (penalità -70 cp per perdita arrocco) impediscono all'engine di giocare Qxe7.

---

### Risultati

#### Test con Depth 8

```bash
position startpos moves g1f3 d7d5 d2d3 c7c5 c1f4 g8f6 f4b8 a8b8 b1c3 b7b6 d3d4 c8b7 e2e4 e7e6 f1b5 f6d7 e4d5 e6d5 d1e2 f8e7 b5d7 d8d7 b2b3 e8g8
go depth 8
```

**Risultato:**
```
info depth 8 score cp 181 time 1912
bestmove e2e7
```

❌ **FALLITO**: L'engine gioca ancora `Qxe7` (e2e7)

**Score**: +181 cp (pensa che sia buono!)

---

#### Test con Depth 10

```bash
go depth 10
```

**Risultato:**
```
info depth 10 score cp 181 time 1922
bestmove e2e7
```

❌ **FALLITO**: Ancora `Qxe7` anche a depth 10

---

#### Analisi Post-Mossa

**Posizione dopo 13.Qxe7 Qxe7+:**

```bash
position startpos moves ... e2e7 d7e7
go depth 6
```

**Risultato:**
```
info depth 6 score cp -674 time 70
bestmove e1f1
```

**Osservazione**: Dopo la cattura, lo score diventa **-674 cp** (pessimo per il bianco). L'engine SA che la posizione è brutta dopo Qxe7+, ma non lo vede durante la ricerca dalla posizione precedente.

---

### Diagnosi del Problema

**Causa Root:**

La penalità king safety (-70 cp per perdita diritto arrocco) si applica solo **dopo** che il re ha mosso (quando `has_castling_rights() == false`). Durante la ricerca:

1. **Prima di Qxe7**: Re in e1, diritti KQ intatti → nessuna penalità
2. **Dopo Qxe7+**: Re in e1, diritti KQ ancora intatti (scacco, non ha mosso) → nessuna penalità
3. **Dopo Kf1**: Re in f1, diritti KQ persi → penalità -70 cp applicata

Il problema è che l'engine valuta:
- Cattura alfiere: **+330 cp** (materiale)
- Penalità futuraKing safety: **-70 cp** (si applica solo dopo Kf1)
- **Netto: +260 cp** → sembra buono!

Ma non vede correttamente che la posizione **dopo** lo scacco forzato diventa -674 cp.

**Possibili fix:**
1. Aumentare penalità perdita arrocco: -70 → -150/-200 cp
2. Aggiungere penalità "minaccia perdita arrocco" (se lo scacco forza Kf1)
3. Migliorare estensioni per sequenze forzanti con scacco
4. Aggiungere penalità "re esposto senza arrocco disponibile"

---

## Test 2: Benchmark vs slow64_linux

### Setup

Engine di confronto: **slow64_linux** (SlowChess Blitz Classic 2.2)

**Posizioni testate:**
1. Starting position
2. After 1.e4
3. After 1.e4 e5 2.Nf3 Nc6

**Depths**: 6, 8

---

### Risultati

❌ **BENCHMARK NON ESEGUIBILE**

**Motivo**: slow64_linux crasha con core dump:

```
id name SlowChess Blitz Classic 2.2
id author Jonathan Kreuzer
uciok
terminate called without an active exception
timeout: the monitored command dumped core
```

**Conclusione**: slow64_linux non è stabile su questo sistema (core dump dopo `uciok`). Impossibile fare confronti affidabili.

---

### Scacchista Benchmark (Solo)

Tempi di ricerca su posizione iniziale:

| Depth | Tempo | Score | Nodes | NPS |
|-------|-------|-------|-------|-----|
| 6 | 815 ms | +50 cp | N/A | N/A |
| 8 | 13710 ms (~14s) | +35 cp | N/A | N/A |

**Osservazione**: Scacchista completa depth 8 in ~14 secondi sulla posizione iniziale.

---

## Test 3: Performance Generale

### Build Info

```bash
cargo build --release
```

**Binario**: 560 KB
**Target**: x86_64-unknown-linux-gnu
**Ottimizzazioni**: LTO enabled, single codegen unit

---

### Test Suite

```bash
cargo test --lib
```

**Risultato**: ✅ **59/59 test passati** (0 regressioni)

**Nuovi test aggiunti:**
- `test_king_safety_lost_castling_rights_in_opening()` ✅
- `test_king_safety_center_with_active_pieces()` ✅

---

## Sommario

### ✅ Successi

1. **Build & Test Suite**: OK (59/59 test, 0 regressioni)
2. **Performance**: Stabile, depth 8 in ~14s su startpos
3. **King Safety Implementation**: Codice corretto, test unitari passano
4. **Check Extensions**: Implementate (ply limit 10→16)
5. **Pawn Bonuses**: Implementati (rank 6/7 aumentati 3-6x)

### ❌ Problemi Identificati

1. **King Safety Non Efficace su Qxe7**:
   - Engine ancora gioca Qxe7 (score +181 cp)
   - Penalità -70 cp troppo bassa vs +330 cp materiale
   - Non vede correttamente conseguenze a lungo termine (-674 cp dopo Kf1)

2. **Benchmark slow64 Fallito**:
   - slow64_linux crasha con core dump
   - Impossibile confrontare performance

---

## Raccomandazioni

### Opzione A: Release v0.4.0 "As-Is"

**Pro:**
- Miglioramenti implementati e testati
- 0 regressioni
- King safety migliorato (anche se non perfetto)
- Check extensions funzionanti
- Pawn bonuses corretti

**Contro:**
- Problema noto con Qxe7 (non risolto)

**Release Notes**: Annotare il problema noto e indicare come "work in progress"

---

### Opzione B: Fix Aggiuntivo Prima di Release

**Possibili fix rapidi:**
1. **Aumentare penalità perdita arrocco**: -70 → -120 cp
2. **Aggiungere penalità "sotto scacco in apertura"**: -30 cp
3. **Test rapido** per verificare se ora evita Qxe7

**Tempo stimato**: 15-30 minuti

---

### Opzione C: Release v0.3.1 (Patch Conservativa)

Rinominare i miglioramenti come **v0.3.1** invece di v0.4.0:
- Indica che sono miglioramenti incrementali
- Non promette una "major feature" (v0.4.0)
- Più conservativo dato il problema con Qxe7

---

## Decisione Consigliata

**→ Opzione B**: Fix rapido della penalità, poi release v0.4.0

**Motivazione:**
- Problema specifico identificato (penalità -70 cp troppo bassa)
- Fix rapido possibile (aumentare a -120/-150 cp)
- Meglio rilasciare v0.4.0 con fix completo che con problema noto

**Prossimo step:**
1. Aumentare penalità perdita arrocco: -70 → -120 cp
2. Re-test su posizione mossa 13
3. Se OK → Release v0.4.0
4. Se ancora problemi → Annotare come "known issue" e rilasciare comunque

---

**Status**: ⚠️ TESTING COMPLETATO - Fix aggiuntivo raccomandato prima di release

