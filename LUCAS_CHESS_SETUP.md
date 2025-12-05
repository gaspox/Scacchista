# Setup Scacchista con Lucas Chess

Guida rapida per configurare Scacchista come engine UCI in Lucas Chess.

## Prerequisiti

1. **Build dell'engine in modalità release**:
   ```bash
   cargo build --release
   ```

2. **Verifica che funzioni**:
   ```bash
   ./scacchista-uci.sh
   # Dovrebbe mostrare "id name Scacchista" e opzioni UCI
   # Premi Ctrl+C per uscire
   ```

## Configurazione Lucas Chess

### 1. Apri Lucas Chess

### 2. Aggiungi Engine Esterno

- **Menu**: `Tools` → `Engines` → `New engine`
- Oppure: `Strumenti` → `Motori` → `Nuovo motore` (versione italiana)

### 3. Configura l'Engine

**Parametri da inserire**:

| Campo | Valore |
|-------|--------|
| **Name** | `Scacchista` |
| **Type** | `UCI` |
| **Command** | `/home/gaspare/Documenti/Tal/scacchista-uci.sh` |
| **Working Directory** | `/home/gaspare/Documenti/Tal` |
| **Arguments** | *(lasciare vuoto)* |

> **Nota**: Sostituisci `/home/gaspare/Documenti/Tal` con il percorso effettivo del repository se diverso.

### 4. Opzioni UCI Raccomandate

Dopo aver aggiunto l'engine, configura le opzioni:

| Opzione | Valore Raccomandato | Note |
|---------|---------------------|------|
| **Hash** | `64-128 MB` | Più memoria = migliore TT coverage |
| **Threads** | `2-4` | Usa 2-4 threads su CPU multi-core (lazy-SMP ha scaling limitato) |
| **Style** | `Normal` | Oppure `Tal` (aggressivo) o `Petrosian` (posizionale) |
| **UseExperienceBook** | `true` | Abilita learning da partite giocate |

### 5. Test dell'Engine

1. Crea una nuova partita contro Scacchista
2. Imposta tempo di riflessione (es: 5 secondi/mossa o depth 7-8)
3. Gioca alcune mosse per verificare che risponda correttamente

## Livelli di Forza Approssimativi

Configurazioni per diversi livelli di gioco:

### Principiante (~1200-1400 ELO)
- Depth: 4-5
- Time: 1-2 secondi/mossa
- Threads: 1

### Intermedio (~1500-1600 ELO) - **Attuale Forza**
- Depth: 6-7
- Time: 3-5 secondi/mossa
- Threads: 2
- Hash: 64 MB

### Avanzato (~1700-1800 ELO)
- Depth: 8-9
- Time: 10-15 secondi/mossa
- Threads: 4
- Hash: 128 MB

## Troubleshooting

### Engine non si avvia

**Problema**: Lucas Chess mostra errore "Engine failed to start"

**Soluzioni**:
1. Verifica che il percorso in `Command` sia corretto
2. Assicurati che `scacchista-uci.sh` sia eseguibile:
   ```bash
   chmod +x scacchista-uci.sh
   ```
3. Verifica che il binary release esista:
   ```bash
   ls -lh target/release/scacchista
   ```
4. Prova a eseguire manualmente lo script:
   ```bash
   ./scacchista-uci.sh
   # Dovrebbe stampare info UCI
   ```

### Engine si blocca o non risponde

**Problema**: L'engine sembra non muovere o va in timeout

**Soluzioni**:
1. Controlla i log di Lucas Chess (di solito in `~/.local/share/lucaschess/`)
2. Riduci depth o tempo di riflessione
3. Verifica che non ci siano processi zombie:
   ```bash
   ps aux | grep scacchista
   killall scacchista  # se necessario
   ```

### Prestazioni lente

**Problema**: L'engine impiega troppo tempo a muovere

**Soluzioni**:
1. Assicurati di aver buildato in `--release` (non debug!)
2. Riduci depth target (7-8 è ragionevole per gioco rapido)
3. Aumenta Hash size (più TT = meno ricalcoli)

## Comandi UCI Utili

Per testare manualmente l'engine:

```bash
./scacchista-uci.sh
```

Poi digita:
```
uci                    # Mostra opzioni engine
isready               # Verifica che sia pronto
position startpos     # Imposta posizione iniziale
go depth 6            # Cerca a profondità 6
quit                  # Esci
```

## Performance Attese

Con configurazione "Intermedio" (depth 7, 2 threads):
- **Tempo medio/mossa**: ~2-3 secondi
- **Nodi valutati**: ~500k-1M nodes
- **Velocità**: ~250-350 knodes/sec
- **TT fill**: ~5-15%

## Note su Stili di Gioco

- **Normal**: Bilanciato tra tattica e posizione
- **Tal**: Aggressivo, preferisce attacchi e sacrifici
- **Petrosian**: Solido e posizionale, preferisce strutture sicure

## Aggiornare l'Engine

Dopo modifiche al codice:

```bash
# 1. Rebuild
cargo build --release

# 2. Restart Lucas Chess (o ricarica l'engine dalle impostazioni)
```

Non serve riconfigurare, Lucas Chess userà automaticamente il nuovo binary.

---

**Versione**: 0.2.1-beta
**Data**: Dicembre 2025
**Performance**: ~1500-1600 ELO (stima)
