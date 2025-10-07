# Scacchista

Un motore UCI di scacchi scritto in Rust che sfrutta shakmaty per la logica delle regole e una rappresentazione interna ad alte prestazioni per la ricerca.

Obiettivo
- Fornire un motore UCI modulare e performante in Rust, con valutazione manuale parametrica, supporto a personalità di gioco (Tal/Petrosian), esperienza persistente (Experience Book), book Polyglot e probing Syzygy.

Caratteristiche principali
- Rappresentazione ibrida: shakmaty per parsing/validazione, board interno bitboard per make/unmake
- Ricerca: iterative deepening + alpha-beta con TT, quiescence, null-move, LMR, killer/history
- Valutazione HCE: PSQT, pawn structure, king safety, mobility, tapered eval
- Experience Book persistente con aggiornamento post-partita ispirato a Q-learning
- UCI compliant con opzioni configurabili: Hash, Threads, SyzygyPath, BookFile, Style, UseExperienceBook
- Supporto Polyglot book e Syzygy EGTB via shakmaty-syzygy

Quick start
- Build: `cargo build`
- Run (binary): `cargo run -- --uci` (se il main espone il flag `--uci`), oppure `cargo run` e usa stdin per comandi UCI
- Test: `cargo test` (perft e unit tests)
- Lint / format: `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --all`

Docs
- Vedere documentation.md per il manuale tecnico completo.
- Vedere CLAUDE.md per istruzioni specifiche per Claude Code.

Contribuire
- Aprire issue per bug o proposte
- Seguire la roadmap in documentation.md per milestone

Licenza
- Nota: il progetto utilizza il crate `shakmaty` (GPL-3.0). Verificare le impicazioni di licenza prima di distribuire binari.
