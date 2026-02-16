# Scacchista v0.5.0 - Performance & Strength Improvements

## Fase 1 — Performance ⚡
- [x] 1.1 Generazione catture dedicata (`generate_captures()`)
  - [x] Aggiungere `Board::generate_captures()` in `board.rs`
  - [x] Integrare in `qsearch()` in `search.rs`
  - [x] Test e benchmark
- [x] 1.2 Delta Pruning in Quiescence Search
- [x] 1.3 TT senza Mutex (lock-free)
- [x] 1.4 `evaluate_fast()` su bitboard

## Fase 2 — Forza Tattica 🗡️
  - [x] Implement PVS at root for first ply
  - [x] Fix CI Failures (Fixed i16::MIN panic, TT bounds bug, and Timeout -32000 bug)
- [x] 2.2 IIR (Internal Iterative Reduction)
- [x] 2.3 SEE Pruning in qsearch
- [x] 2.4 Countermove Heuristic

## Fase 3 — Valutazione Posizionale 🧠
- [ ] 3.1 Tapered Evaluation (MG/EG)
- [ ] 3.2 Mobilità
- [ ] 3.3 Coppia degli Alfieri
- [ ] 3.4 Torre su colonna aperta

## Fase 5 — Portfolio Polish 🎨
- [x] 5.1 Code Hygiene (fix clippy warnings)
- [x] 5.2 Visual Identity (Mermaid diagrams)
- [x] 5.3 Performance Graph (README)
- [x] 5.4 Demo Section (README)

