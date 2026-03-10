# BISECT Log - v0.5 Regression Hunt

## Metodologia
Branch: `v0.5-bisect` creato da `v0.4.1`

Ogni commit viene cherry-pickato e testato con:
1. Build release
2. Test posizione iniziale (depth 6, score e tempo)
3. Mini-torneo 5 partite vs v0.4.1
4. Risultato: ✅ SAFE / ⚠️ SOSPETTO / ❌ REGRESSIONE

## Commits da Analizzare

```
ff90a62 docs: Update AGENTS.md
723459e Improve draw detection and EP handling
4688a98 Add simple endgame recognition
591c267 Add tactical suite runner
8c030f2 Fix move generation bugs
fe35877 Phase 1: Lock-free TT, Delta Pruning, Bitboard Eval ⚠️
84d62d8 PVS at root ⚠️
d415a1e docs update
d5600f0 docs polish
2a46302 Fix CI: i16::MIN panic, TT bounds, Timeout ⚠️
06fec44 CI format
1efaa44 Docs benchmark
24e985f Phase 2.3: SEE Pruning ⚠️
45aa6d9 Docs
085e98c Phase 2.4: Countermove Heuristic
39d046c Docs cleanup
f9fa457 Docs
8a0bc35 Phase 3.2: Pawn Structure + Tapered Eval ⚠️⚠️
1892d2c Phase 3.3: Passed Pawns
09ca171 Phase 3.4: Bishop Pair
8a71dac Phase 3.4 details
```

---

## Risultati

### STEP 1: Commits 1-5 ✅ SAFE
**Commit**: `0f924bd` BISECT STEP 1
**Include**:
- ff90a62 docs: Update AGENTS.md
- 723459e Improve draw detection and EP handling
- 4688a98 Add simple endgame recognition
- 591c267 Add tactical suite runner
- 8c030f2 Fix move generation bugs and time management

**Test Posizione Iniziale**:
- Score: cp 50 ✅ (identico a v0.4.1)
- Tempo: 383ms ✅ (simile a v0.4.1: 393ms)
- Bestmove: g1f3 ✅

**Mini-Torneo (3 partite)**:
- v0.5_step1: 1 vittoria
- v0.4: 2 vittorie
- Risultato: Bilanciato, nessuna regressione evidente ✅

**Conclusione**: Questi 5 commit sono SAFE. Passiamo allo STEP 2.

---

### STEP 2: Commit fe35877 ❌ REGRESSIONE IDENTIFICATA + ✅ FIXATO!

**Problema Root Cause**: La lock-free TT implementata in fe35877 aveva race condition e collisioni hash eccessive.

**Fix applicato**: Sostituita lock-free TT con Mutex TT (interfaccia compatibile)
- File: `src/search/tt.rs`
- La nuova TT usa `Mutex<TranspositionTableInner>` internamente
- Mantiene la stessa interfaccia (senza `.lock().unwrap()` nelle chiamate)
- Thread-safe corretta, niente race condition

**Test dopo fix**:
- Score posizione iniziale: cp **50** ✅ (era -25)
- Tempo: **363ms** ✅
- Bestmove: **g1f3** ✅
- Test suite: Passa tutto ✅

**Risultato**: STEP 2 FIXATO!
**Commit**: Phase 1 Complete: Lock-free TT, Delta Pruning, Bitboard Eval
**Status**: ❌ REGRESSIONE GRAVE

**Test Posizione Iniziale**:
- Score: cp **-25** ❌ (prima era cp 50)
- Tempo: 493ms
- Bestmove: **e2e4** ❌ (prima era g1f3)

**Analisi**: Questo commit introduce una regressione MASSIVA. L'engine valuta la posizione iniziale come -25cp (pessimista) invece di +50cp.

**Sottocomponenti sospette**:
1. **Lock-free TT** - Cambio da Arc<Mutex<>> a Arc<> (race condition?)
2. **Delta Pruning** - Potrebbe essere troppo aggressivo
3. **Bitboard Eval** - Cambiamento nella logica di valutazione

**Decisione**: Bisogna analizzare più a fondo questo commit. Procediamo con la decomposizione.

