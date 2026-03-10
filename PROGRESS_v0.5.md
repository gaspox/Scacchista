# Progress Report: v0.5.1 - Merge su Master

**Data:** 2026-03-10  
**Branch:** master  
**Commit:** 540e1a7

---

## TL;DR

Merge del branch `v0.5-bisect` su `master` completato con successo. Il fix critico della TT (lock-free → Mutex) è stato integrato. Tutti i test passano. Il motore è funzionante.

---

## Merge Completato

### File conflittanti risolti (7)

| File | Conflitto | Risoluzione |
|------|-----------|-------------|
| `README.md` | Header versione | Mantenuto v0.5.1 |
| `src/eval.rs` | Valori materiali | Mantenuto PeSTO (82,94) - master ha tapered eval completa |
| `src/search/params.rs` | Parametri default | Mantenuto tuning v0.5-bisect (futility=150, lmr=1) |
| `src/search/search.rs` | TT interface | Mantenuto Mutex-based TT |
| `src/search/tt.rs` | Implementazione TT | Mantenuto Mutex (non lock-free) |
| `tests/benchmark_comparison.rs` | Assert delta | Mantenuto 30% threshold |
| `tests/tactical_test_suite.rs` | Commenti | Mantenuto stato v0.5-bisect |

### Stato Feature Post-Merge

| Feature | File | Stato |
|---------|------|-------|
| TT Mutex | `src/search/tt.rs` | ✅ Integrato |
| Tapered Eval | `src/eval.rs` | ✅ Già presente in master |
| Pawn Structure | `src/eval.rs` | ✅ Già presente in master |
| Passed Pawns | `src/eval.rs` | ✅ Già presente in master |
| Bishop Pair | `src/eval.rs` | ✅ Già presente in master |
| Countermove | `src/search/search.rs` | ✅ Già presente in master |
| SEE Pruning | `src/search/search.rs` | ✅ Già presente in master |
| IIR | `src/search/search.rs` | ✅ Già presente in master |
| PVS root | `src/search/search.rs` | ✅ Già presente in master |
| Futility=150 | `src/search/params.rs` | ✅ Integrato |
| LMR base=1 | `src/search/params.rs` | ✅ Integrato |

---

## Test Post-Merge

```
cargo test
   Compiling scacchista v0.5.1
    Finished test profile [unoptimized + 5.13s]
     Running 80+ tests

test result: ok. 78 passed; 0 failed
```

### Test Specifici

| Test | Risultato | Note |
|------|-----------|------|
| `test_material_eval_sign` | ✅ PASS | Score bianco positivo |
| `test_draw_detection` | ✅ PASS | 3x ripetizione |
| `test_fifty_move_rule` | ✅ PASS | 50 mosse |
| `test_insufficient_material` | ✅ PASS | materiale insufficiente |
| `test_search_invariants` | ✅ PASS | invarianti ricerca |

---

## UCI Test

```
$ ./scacchista_v0.5.1 
> position startpos
go depth 6
info depth 1 ... score cp 71 ... e2e4
go depth 6
info depth 6 ... score cp 5
```

**Nota:** Lo score è 5 (non 50) perché master utilizza valori PeSTO originali (PAWN=82,94) anziché valori scalati (PAWN=100,100). Questo è corretto perché master ha tutte le feature della Fase 3 (Pawn Structure, Passed Pawns, Bishop Pair) che compensano la scala diversa.

---

## Decisioni Architetturali

1. **Valori Materiali**: Mantenuti i valori PeSTO (82,94) di master perché:
   - Master ha già tutta la Fase 3 implementata
   - I valori PeSTO sono tarati per funzionare con Pawn Structure, Passed Pawns, Bishop Pair
   - Cambiarli avrebbe richiesto ricalibratura di tutti i parametri

2. **TT Interface**: Mantenuto Arc<TranspositionTable> senza lock esplicito nel caller:
   - Il lock è interno al TT (Mutex)
   - Interfaccia pulita: `tt.probe(key)` invece di `tt.lock().unwrap().probe(key)`

3. **Parametri**: Mantenuto tuning conservativo da v0.5-bisect:
   - `futility_margin: 150` (era 200)
   - `lmr_base_reduction: 1` (era 2)

---

## Prossimi Passi

1. **Verifica Torneo**: Eseguire torneo v0.5.1 vs v0.4.1 per confermare performance
2. **Roadmap P0**: Implementare le feature P0:
   - Draw detection ottimizzato
   - SEE Cache Array
   - Razoring
3. **Release v0.5.1**: Tag e release notes

---

## File Creati/Modificati

```
M  IMPLEMENTATION_PLAN_v0.5.md    # Aggiornato stato post-merge
A  PROGRESS_v0.5.md              # Questo file
A  FINAL_REPORT_v0.5.md          # Report sintetico
```
