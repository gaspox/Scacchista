# Bug Fix Summary - Tactical Blindness Issue

## Data
19 Ottobre 2025

## Problema Originale
L'engine Scacchista mostrava "cecità tattica" critica:
- Dopo aver perso un cavallo: mostrava score **+1120** (doveva essere negativo!)
- Dopo aver perso la Donna: mostrava score **+20800** (doveva essere fortemente negativo!)
- Faceva mosse insensate lasciando pezzi in presa
- Esempio: nella partita test_1.pgn, giocava 7.h3 lasciando la Donna in presa su d4

## Root Cause Analysis
Il bug era una **negazione mancante** nella chiamata a `qsearch()` dalla funzione `iddfs()` quando `depth <= qsearch_depth`.

### Codice Buggato (src/search/search.rs:270-276)
```rust
let (score, _node_type) = if depth <= self.params.qsearch_depth {
    // ❌ BUG: Manca la negazione!
    (
        self.qsearch(alpha, beta, self.params.qsearch_depth),  // Nessuna negazione
        NodeType::Exact,
    )
} else {
    let score = -self.negamax_pv(depth - 1, -beta, -alpha, 0);  // Negato correttamente
    // ...
}
```

### Perché il Bug Era Critico
Nel negamax, dopo aver fatto una mossa (`make_move`), si passa alla prospettiva dell'avversario. La chiamata ricorsiva deve quindi:
1. **Negare alpha/beta**: `-beta, -alpha` (finestra invertita)
2. **Negare il risultato**: `-search(...)` (prospettiva invertita)

Il codice originale faceva correttamente la negazione per `negamax_pv()`, ma NON per `qsearch()`.

Dato che `qsearch_depth` di default è **6**, tutte le ricerche a depth ≤ 6 usavano il path buggato!

## Fix Implementato
### src/search/search.rs:274
```rust
let (score, _node_type) = if depth <= self.params.qsearch_depth {
    // ✅ FIX: Aggiunte negazione score e inversione alpha/beta
    (
        -self.qsearch(-beta, -alpha, self.params.qsearch_depth),
        NodeType::Exact,
    )
} else {
    let score = -self.negamax_pv(depth - 1, -beta, -alpha, 0);
    // ...
}
```

## Convenzione Negamax
L'engine usa la convenzione **side-to-move**: le funzioni di valutazione ritornano score dalla **prospettiva di chi muove** (positivo = buono per chi muove).

### src/search/search.rs:865-872 (Confermato Corretto)
```rust
// material_eval() ritorna da prospettiva side-to-move
if self.board.side == Color::Black {
    -material_score  // Nega per il Nero
} else {
    material_score   // Diretto per il Bianco
}
```

Questa convenzione richiede che:
- Le valutazioni siano **sempre** dalla prospettiva di chi muove
- Le chiamate ricorsive neghino: `score = -search(...)`
- Alpha/beta siano invertiti: `search(-beta, -alpha, ...)`

## Risultati Dopo il Fix

### Prima del Fix
```
Position dopo 4...dxc3 (Nero cattura cavallo):
score cp 1120  ❌ (doveva essere negativo!)
bestmove d2c3

Position dopo 7...Nxd4 (Nero cattura Donna):
score cp 20800  ❌ (doveva essere ~-600!)
```

### Dopo il Fix
```
Position dopo 4...dxc3 (Nero cattura cavallo):
score cp -320  ✅ (corretto: Bianco ha perso il cavallo)
bestmove d2d3

Position dopo 7...Nxd4 (Nero cattura Donna):
score cp -1220  ✅ (corretto: Bianco ha perso la Donna)
```

## Test Suite Creata
Aggiunta `tests/tactical.rs` con 7 test per prevenire regressioni:
1. `test_material_after_knight_loss` - Verifica segno corretto dopo perdita cavallo
2. `test_material_after_queen_loss` - Verifica segno corretto dopo perdita Donna
3. `test_starting_position_balanced` - Startpos ha score ~0
4. `test_white_up_pawn` - Bianco avanti pedone → score positivo
5. `test_black_up_pawn` - Nero avanti pedone → score negativo
6. `test_captures_hanging_piece` - Engine cattura pezzi appesi
7. `test_negamax_sign_consistency` - Verifica coerenza negamax

**Tutti i test passano** ✅ (42 test unitari + 7 test tattici + 1 test perft)

## File Modificati
- `src/search/search.rs:274` - Fix critico: aggiunta negazione nella chiamata qsearch da iddfs
- `tests/tactical.rs` - Nuova test suite tattica (153 righe)
- `tests/test_material_eval_direct.rs` - Test di regressione diretto (42 righe)

## Impatto
- ✅ Engine ora valuta correttamente il materiale
- ✅ Vede quando è in vantaggio o svantaggio
- ✅ Non lascia più pezzi in presa senza motivo
- ⚠️ A depth 6-8 potrebbe ancora non vedere tutte le tattiche (problema di search horizon/pruning, non di valutazione)

## Testing
Per testare il fix:
```bash
# Rebuild
cargo build --release

# Run test suite
cargo test

# Test posizione tattica originale
./target/release/scacchista
position startpos moves b2b3 d7d5 c2c4 c7c5 f2f3 d5d4 b1c3 d4c3 d2c3 d8a5 d1d4 b8c6
go depth 6
# Atteso: score negativo, bestmove salva la Donna
```

## Note
Questo fix risolve il bug SHOWSTOPPER che rendeva l'engine completamente ingiocabile. L'engine è ora pronto per test con Lucas Chess.
