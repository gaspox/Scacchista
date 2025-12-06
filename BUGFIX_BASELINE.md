# Baseline Pre-Bugfix - v0.2.1

**Data**: 2025-12-06
**Branch**: bugfix/time-score-corruption
**Tag**: pre-bugfix

## Stato Attuale

### Bug Critici Attivi

**BUG #1: Time Expiration Score Corruption** ⚠️
- **Severity**: CRITICAL / SHOWSTOPPER
- **Impact**: Mosse suicide con time control
- **Test**: `cargo test --test time_control_bug test_time_expiration_no_fake_mate` → **FAILS**
- **Evidence**: Score=30000 (falso mate) viene restituito frequentemente
- **Partita test**: `prova.pgn` - motore perde in 19 mosse con mosse suicide

**BUG #2: Multi-threading Crash**
- **Severity**: HIGH
- **Impact**: Core dump con Threads > 1
- **Test**: Manual UCI test con `setoption name Threads value 2` → **CRASHES**

**BUG #3: UCI Depth Info Errato**
- **Severity**: LOW
- **Impact**: Mostra sempre "depth 3"
- **Test**: Visuale, non ha test automatico

**BUG #4: Time Management Ignora Incremento**
- **Severity**: MEDIUM
- **Impact**: Non usa winc/binc
- **Test**: Manual check con `go wtime X btime Y winc Z binc W`

### Performance Baseline

**Con depth fisso** (`go depth N`):
- Depth 6: ~500ms
- Depth 7: ~2.2s
- Depth 8: ~14.1s
- ELO stimato: **~1500-1600** ✅

**Con time control** (`go movetime X`):
- Mosse suicide frequenti
- Score 30000 (fake mate) comune
- ELO stimato: **~800** ⚠️

### Test Suite

**Unit tests**: 57/57 passing (ma non coprono time control)
**Integration tests**: 23/23 passing (ma non coprono time control)
**NEW**: `time_control_bug.rs` - 4 test, **tutti FAILING** (come atteso)

### Known Limitations

- Engine **NON GIOCABILE** con time control in partite reali
- Multi-threading **DISABILITATO** (crash garantito)
- Raccomandazione: Usare solo `go depth N` con `Threads=1`

## Piano di Fix

**Fase 1**: Fix score corruption (45 min) → v0.2.2-hotfix
**Fase 2**: Fix multi-threading (90 min)
**Fase 3**: Fix time increment (30 min)
**Fase 4**: Fix depth display (15 min)
**Total**: 4 ore → v0.3.0

## Success Metrics

| Metrica | Pre-Fix | Target Post-Fix |
|---------|---------|-----------------|
| test_time_expiration_no_fake_mate | **FAIL** | **PASS** |
| Mosse suicide (time control) | Frequenti | Zero |
| Crash (Threads>1) | Sempre | Zero |
| ELO (time control) | ~800 | ~1500 |

---

**Checkpoint**: Questo documento rappresenta lo stato prima dei fix.
Tutti i test attuali dovranno continuare a passare, più i nuovi test dovranno passare dopo i fix.
