# documentation.md

Questo documento è il manuale tecnico dettagliato del motore UCI in Rust che stiamo progettando, basato sulle linee guida del progetto (shakmaty, shakmaty-uci, shakmaty-syzygy). È pensato come riferimento operativo per sviluppatori che vogliono comprendere, estendere e ottimizzare il motore.

Indice
1. Scopo
2. Panoramica architetturale (big picture)
3. Layout consigliato del repository
4. Dipendenze principali e licenze
5. Rappresentazione della scacchiera e generazione mosse
6. Rappresentazione delle mosse e stack di undo
7. Zobrist hashing e gestione della cronologia
8. Tabella di trasposizione (TT): formato e strategia di replacement
9. Motore di ricerca: algoritmo, pseudo-codice e ottimizzazioni
10. Quiescence, Null-Move e LMR
11. Ordinamento delle mosse e heuristics (MVV-LVA, killer, history, experience)
12. Valutazione (HCE) dettagliata: MG/EG, PSQT, pawn structure, king safety, mobility
13. Pawn hash table e caching
14. Opening book (Polyglot) e Syzygy EGTB
15. Experience Book (apprendimento continuo) — struttura, serializzazione, aggiornamento
16. Paralellismo: Lazy-SMP e condivisione TT
17. Gestione tempo e opzioni UCI (esempi)
18. Testing: perft, unit tests, bench
19. Profiling e ottimizzazione (compilazione, LTO, target-native)
20. CI / GitHub Actions (esempio)
21. Esempi di API e snippet utili
22. Note sulla licenza e distribuzione
23. Roadmap tecnica e milestones consigliate

1. Scopo
Questo documento descrive l'architettura interna, gli algoritmi e i dati necessari per implementare un motore di scacchi UCI robusto e performante in Rust. Fornisce specifiche sufficienti per implementare ogni componente critico: board, movegen, make/unmake, hashing, TT, ricerca, valutazione, opening book, probing tablebase e sistema di apprendimento persistente.

2. Panoramica architetturale (big picture)
- Interazione UCI: stdin/stdout loop che riceve comandi UCI (shakmaty-uci consigliato) e risponde con id/option/uciok/readyok/bestmove. Opzioni runtime: Hash, Threads, SyzygyPath, BookFile, Style, UseExperienceBook, MoveOverhead, MultiPV.
- Validazione e parsing delle mosse/FEN: shakmaty fornisce parsing, validazione e API Position/Chess; tuttavia la ricerca utilizza una rappresentazione interna mutabile e più performante (bitboard/make-unmake) per evitare clone-cost.
- Core di ricerca: iterative-deepening + alpha-beta (PVS / aspiration windows), migliorato con TT, quiescence search, null-move pruning, LMR, futility, killer/history heuristics.
- Valutazione: funzione hand-crafted (HCE) a due componenti (middlegame, endgame) con Tapered Eval e struttura EvalParams parametrizzabile per personalità di gioco.
- Opening/Endgame: lettura di libri Polyglot e probing Syzygy tramite razionali bindings (shakmaty-syzygy).
- Experience Book: tabella persistente che apprende dalle self-play e guida ordering/valutazione.
- Parallelismo: lazy-SMP con TT condiviso, sincronizzazione leggera e meccanismi di terminazione cooperativa.

3. Layout consigliato del repository
Suggerimento di layout (nome file/cartelle consigliati):

- Cargo.toml
- src/
  - main.rs               # bin: uci loop / CLI entrypoint
  - uci.rs                # parsing e gestione opzioni UCI (shakmaty-uci wrapper)
  - engine/
    - mod.rs
    - board.rs            # rappresentazione bitboard interna, make/unmake
    - move_encode.rs      # codifica/decodifica movimenti (u32/u16)
    - zobrist.rs          # tabelle Zobrist e helper
    - tt.rs               # tabella di trasposizione
    - search.rs           # entrypoint per la ricerca (iterative deepening) e worker threads
    - search_core.rs      # alpha-beta / pvsearch / quiescence
    - eval.rs             # funzione di valutazione HCE + EvalParams
    - pawn_hash.rs        # pawn hash e cache
    - book.rs             # Polyglot reader
    - syzygy.rs           # tablebase probe (shakmaty-syzygy)
    - experience.rs       # Experience Book persistente
    - perft.rs            # perft utilities
  - utils/
    - bitboards.rs        # helper bitboard e magic attacks
    - config.rs           # parsing config / engine options
- tests/                  # integration tests (perft, etc.)
- benches/                # benchmarks
- docs/documentation.md   # (questo file)

Nota: Non è necessario seguire esattamente i nomi; sono suggerimenti per mantenere chiarezza e separazione delle responsabilità.

4. Dipendenze principali e licenze
- shakmaty (GPL-3.0): usa shakmaty per parsing, validazione, Zobrist, e per interfacciarsi con notazioni FEN/SAN/UCI. Attenzione: la licenza GPL-3 può imporre vincoli distributivi sul prodotto finale.
- shakmaty-uci: parser UCI consigliato
- shakmaty-syzygy: probe Syzygy EGTB
- serde + bincode: serializzazione Experience Book e altre strutture persistenti
- rayon o crossbeam: utilità parallele; rayon utile per task parallel iterators, crossbeam per canali/gestione thread
- rand: apertura book randomness e scelte non deterministiche
- tracing / pretty_env_logger: logging

5. Rappresentazione della scacchiera e generazione mosse
Proposta di rappresentazione interna (bitboard-first):

- Indici quadrati: 0..63 (A1=0, B1=1, …, H8=63) — usare mapping coerente con shakmaty se si interscambia dati.
- Bitboards per tipo di pezzo e colore: piece_bb: [u64; 12] (0..5 = white P,N,B,R,Q,K ; 6..11 = black)
- Occupancy:
  - occ_side: [u64; 2]  // [white_occ, black_occ]
  - occ: u64 = occ_side[0] | occ_side[1]
- Stato addizionale:
  - side_to_move: Color (0=White,1=Black)
  - castling_rights: u8 (bits KQkq)
  - ep_square: Option<u8> (square index or None)
  - halfmove_clock: u16
  - fullmove_number: u16
  - zobrist_key: u64 (incremental)

Esempio struct suggerito (Rust):

pub struct Board {
    piece_bb: [u64; 12],
    white_occ: u64,
    black_occ: u64,
    occ: u64,
    side: Color,
    castling: u8,
    ep: Option<u8>,
    halfmove: u16,
    fullmove: u16,
    zobrist: u64,
}

Move generation:
- Usare tabelle di attacco precomputate (magic bitboards) per scacchi veloci.
- Generare pseudo-legal moves velocemente e filtrare le mosse che lasciano il re sotto scacco.
- Allinearsi con shakmaty per la rappresentazione esterna (radice): usare shakmaty per convalidare la lista radice e sincronizzare il motore interno dopo position/moves.

6. Rappresentazione delle mosse e stack di undo
Codifica compatta delle mosse (32-bit recommended):
- from: 6 bit
- to: 6 bit
- piece: 4 bit
- captured: 4 bit
- promotion: 4 bit
- flags: 8 bit (es. en-passant, short/long castle)

Esempio:

pub type Move = u32;

// helper macros per pack/unpack: from_sq(move), to_sq(move), promotion(move) ...

Undo stack entry (per ply):

pub struct Undo {
    move_played: Move,
    captured_piece: Option<Piece>,
    castling: u8,
    ep: Option<u8>,
    halfmove: u16,
    zobrist: u64, // previous zobrist
}

Make/unmake:
- Fare make_move(board, mv) che aggiorni bitboards, occ, zobrist e push su stack un Undo.
- Unmake deve ripristinare lo stato da Undo in O(1).

7. Zobrist hashing e gestione della cronologia
- Tabelle:
  - piece_square[12][64] : u64
  - side_to_move_key : u64
  - castling_keys[16] : u64 (o singoli bit per KQkq)
  - en_passant_file[8] : u64 (o None se no ep)
- Calcolo: zobrist = XOR degli elementi pertinenti
- Aggiornamento: incremental XOR on make/unmake
- Cronologia per ripetizioni: mantenere stack<Vec<u64>> o conteggio occorrenze HashMap<u64, u16>

8. Tabella di trasposizione (TT): formato e strategia di replacement
Entry tipico:

#[repr(C)]
pub struct TTEntry {
    key: u64,
    value: i32,
    depth: i8,
    flag: u8, // EXACT=0, LOWER=1, UPPER=2
    best_move: u32,
    age: u8,
}

- Organizzazione: array di cluster (2 o 4 entries per bucket) per ridurre collisioni.
- Replacement policy: preferire entry più profonde; se stessa profondità, usare age-based replacement; sempre preferire scrivere se key==entry.key.
- Dimensione: default 64 MB; calcolo entries = (MB * 1024 * 1024) / size_of_entry

9. Motore di ricerca: algoritmo, pseudo-codice e ottimizzazioni
High-level: iterative_deepening(root_pos)
- Per depth = 1..max_depth:
  - call search(root, depth, -INF, +INF, ply=0)
  - store PV
  - adjust aspiration window for next depth

Pseudo-codice (semplificato):

fn search(board: &mut Board, depth: i32, mut alpha: i32, beta: i32, ply: usize) -> i32 {
    if depth <= 0 { return quiescence(board, alpha, beta); }
    // TT probe
    if let Some(tt) = tt_probe(board.zobrist) {
        if tt.depth >= depth { // usable
            match tt.flag {
                EXACT => return tt.value,
                LOWER => alpha = alpha.max(tt.value),
                UPPER => beta = beta.min(tt.value),
            }
            if alpha >= beta { return tt.value; }
        }
    }
    // generate moves and order
    let mut moves = generate_moves(board);
    order_moves(&mut moves, &board, &tt_move);
    // Principal Variation Search (PVS) loop
    let mut best = -INF;
    for (i, mv) in moves.iter().enumerate() {
        let mut undo = board.make_move(*mv);
        let score = -search(board, depth-1, -beta, -alpha, ply+1);
        board.unmake_move(undo);
        if score >= beta {
            // store in TT as lower bound
            tt_store(board.zobrist, score, depth, LOWER, *mv);
            return beta;
        }
        if score > best { best = score; if score > alpha { alpha = score; best_move = *mv; } }
    }
    // store EXACT/UPPER accordingly
    tt_store(board.zobrist, best, depth, EXACT, best_move);
    return best;
}

- PVSearch: search narrow window after first child as -alpha-1..-alpha for faster pruning (PVS) and full window for best move.
- Aspiration windows: start with small window around last score, widen on failure.

10. Quiescence, Null-Move e LMR
Quiescence:
- Explore captures (MVV-LVA ordered), promotions and possibly checks until position is ‘quiet’.
- Use stand-pat evaluation and alpha-beta in quiescence.

Null-move pruning:
- If not in PV and not in zugzwang-ish (e.g., material low), do make null (flip side) with reduction R and call search(depth - 1 - R,-beta,-beta+1). If returns >= beta, prune.
- Do NOT apply null-move when in reduced-material endgames / zugzwang risk.

Late Move Reductions (LMR):
- For quiet moves after a few first moves, reduce depth by a function R = f(depth, move_index).
- If reduced search returns > alpha, re-search at full depth.

11. Ordinamento delle mosse e heuristics
Ordering pipeline (radice -> leaf):
1. TT move
2. Captures ordered MVV-LVA (+SEE)
3. Promotions
4. Moves that give check
5. Killer moves
6. History heuristic score
7. Experience Book boost (if present)

Strutture:
- killer_moves[MaxDepth][2]
- history_table[Piece][Square] -> u32 incrementato quando la mossa migliora alpha

12. Valutazione (HCE) dettagliata
Unità: centipawn (100 = valore di un pedone)

Componenti:
- Materiale: valori base (es. Pawn=100, Knight=320, Bishop=330, Rook=500, Queen=900)
- Piece-Square Tables (PSQT) 64-entry per pezzo e per fase (MG/EG)
- Tapered Eval: mg_score, eg_score e fase di gioco (phase) -> eval = (mg*phase + eg*(max_phase-phase)) / max_phase
- Pawn structure: isolated, doubled, backward, passed, outposts
- King safety: ring attorno al re (inner ring / outer ring), conteggio attaccanti con pesi non lineari
- Mobility: conteggio pseudo-legal moves con mappatura non lineare su bonus
- Space / Piece activity / Rook on open files
- Pawn Hash: caching valutazione struttura pedoni

Esempio EvalParams (Rust):

pub struct EvalParams {
    pub piece_value: [i32; 6],
    pub pst_mg: [[i16;64];6],
    pub pst_eg: [[i16;64];6],
    pub pawn_penalties: PawnWeights,
    pub king_safety_weights: KingSafetyWeights,
    pub mobility_weights: MobilityWeights,
    // ... altri parametri
}

13. Pawn hash table e caching
- Key calcolata solo da pawn bitboards + side; dimensione ridotta e accesso veloce.
- Contenuto: struct PawnEntry { key: u64, score_mg: i32, score_eg: i32, used_at: u32 }
- Uso: quando in search/eval si incontra una posizione, consultare pawn hash prima di ricomputare termini costosi.

14. Opening book (Polyglot) e Syzygy EGTB
- Polyglot: leggere .bin e scegliere entry con peso (nash) o randomico. Se ci sono multiple entry per key, scegliere in base al peso.
- Syzygy (shakmaty-syzygy): probe per posizione data e numero di pezzi; se WDL è risolto, si può scegliere la mossa che porta all'outcome migliore. Configurare SyzygyPath come opzione UCI.

15. Experience Book (apprendimento continuo)
Struttura:

pub struct ExperienceEntry {
    pub score: i32,
    pub games_played: u32,
    pub wins: u32,
    pub draws: u32,
    pub avg_depth: u16,
}

Persistenza: serde + bincode per serializzare HashMap<u64, ExperienceEntry> in file compressato.

Flusso di aggiornamento (post-partita):
- Durante self-play registrare sequence Vec<(zobrist: u64, score_at_play: i32)> per ogni mossa.
- All'end partita, se risultato != draw, effettuare back-propagation usando formula:

St' = (1 - α) * St + α * (γ * St+1')

con α (learning rate) tipicamente 0.5 e γ (discount) 0.99. Per l'ultimo nodo St+1' usare risultato finale (es. +10000 / -10000 / 0).

Uso in ricerca:
- Per ordering: se la posizione figlia è nella Experience Book, aumentare la priorità della mossa.
- Per valutazione: blending: final_eval = (w_e * exp_score) + ((1 - w_e) * hce_score), dove w_e dipende da games_played (più è popolare la posizione, più alta la fiducia).

Conservazione/IO:
- Caricamento on startup (lazy load se grande), scrittura periodica (ogni N partite) o all'shutdown.
- Protezione: scrivere in file temporaneo e atomically rename per evitare corruption.

16. Paralellismo: Lazy-SMP e condivisione TT
- Modalità Lazy-SMP: lanciare N worker threads che ognuno esegue iterative deepening indipendente con differente move-order randomized o con priorità diverse; tutti condividono la stessa TT.
- TT condiviso: usare struttura lock-free o RwLock per bucket; preferire operazioni atomiche per scrivere entry (scrittura possibile senza lock per sovrascrivere entry) e usare replacement policy semplice per evitare lunghe contese.
- Stop/kill: canale atomico che indica a tutti i thread di fermarsi, e barrier per join.

17. Gestione tempo e opzioni UCI (esempi)
- Gestione di base: se go wtime btime winc binc movestogo
- Algoritmo rapido: allocare fraction = 0.95 del tempo disponibile per la mossa corrente; tenere un buffer MoveOverhead (ms) per latenza UI.
- Opzioni UCI da esporre (sintesi):
  - Hash (spin) default 64
  - Threads (spin) default 1
  - SyzygyPath (string)
  - BookFile (string)
  - Style (combo) [Normal, Tal, Petrosian]
  - UseExperienceBook (check) default true
  - MoveOverhead (spin ms) default 80

Esempio output all'avvio (in risposta a `uci`):

id name MyRustEngine
id author YourName
option name Hash type spin default 64 min 1 max 32768
option name Threads type spin default 1 min 1 max 256
option name Style type combo default Normal var Normal var Tal var Petrosian
option name UseExperienceBook type check default true
uciok

18. Testing: perft, unit tests, bench
- Perft: comando `perft(fen, depth)` che conta numero di nodi per profondità; salvare divide output (mossa -> nodes). Run perft su posizione iniziale depth 5/6 per smoke tests.
- Unit tests: test per make/unmake consistency, FEN round-trip, move generation count, zobrist invariants (make/unmake restores hash).
- Benchmarking: usare crates come criterion o bench harness; misurare nps (nodes per second), hash hitrate, average depth.

19. Profiling e ottimizzazione
- Compilare release con:
  RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C codegen-units=1" cargo build --release
- LTO e PGO (se necessario) per hot loops
- CPU sampling: perf, flamegraph. Strumenti Rust: cargo-flamegraph, pprof

20. CI / GitHub Actions (esempio)
Workflow minimal:
- Checkout
- Setup Rust (stable)
- cargo fmt -- --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test
- Run quick perft smoke test (perft depth 4) as integration

21. Esempi di API e snippet utili
Zobrist XOR update (esempio semplificato):

fn update_zobrist_piece(zobrist: &mut u64, piece: usize, sq: usize, zobrist_table: &[[u64;64];12]) {
    *zobrist ^= zobrist_table[piece][sq];
}

TT probe/store (semplificato):

fn tt_probe(hash: u64) -> Option<TTEntry> { /* bucket lookup */ }
fn tt_store(hash: u64, entry: TTEntry) { /* replacement policy */ }

Experience update (pseudocode):

for pos in reversed(game_positions) {
    let next_score = next_entry.score_prime;
    let s = experience.get_mut(pos.key).unwrap_or_default();
    s.score = ((1.0 - alpha) * s.score as f64 + alpha * (gamma * next_score as f64)) as i32;
    s.games_played += 1;
}

22. Note sulla licenza e distribuzione
- shakmaty è GPL-3: revisionare implicazioni legali prima di distribuire binari. Se si vuole evitare viralità GPL, considerare alternative non-GPL o isolare l'uso di shakmaty in strumenti separati (es. tool di test) senza incorporarlo nel binario finale.

23. Roadmap tecnica e milestones consigliate
- M1: Implementare board interno, make/unmake, e perft (validazione)
- M2: Implementare alphabeta baseline + quiescence + TT
- M3: Implementare evaluation HCE con PSQT e pawn hash
- M4: Aggiungere move ordering, killer/history, LMR, null-move
- M5: Implementare UCI loop e opzioni, expose Style/UseExperienceBook
- M6: Implementare opening book Polyglot e Syzygy probe
- M7: Implementare Experience Book e self-play pipeline
- M8: Profiling, parallel search (Lazy-SMP) e ottimizzazione release

Appendice: formule / valori default
- Piece values: P=100,N=320,B=330,R=500,Q=900
- Learning rate α default = 0.5
- Discount γ default = 0.99
- Default hash: 64 MB
- Default TT entry size: ~24 bytes (dipende da alignment)

Fine del documento. Per proseguire posso generare:
- file di esempio src/board.rs con implementazione iniziale di bitboards
- file perft CLI con test e script CI
- script di tuning Texel (pipeline)

