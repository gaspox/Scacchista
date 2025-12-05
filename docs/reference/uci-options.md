# UCI Options Reference

This document lists all UCI options supported by Scacchista.

## Overview

UCI options are configured using the `setoption` command:

```
setoption name <option_name> value <value>
```

## Available Options

### Hash

Transposition table size in megabytes.

| Property | Value |
|----------|-------|
| Type | spin |
| Default | 64 |
| Min | 1 |
| Max | 32768 |

```
setoption name Hash value 256
```

**Notes:**
- Larger hash improves search quality
- Memory usage equals this value
- Clear with `ucinewgame` when changing

### Threads

Number of search threads.

| Property | Value |
|----------|-------|
| Type | spin |
| Default | 1 |
| Min | 1 |
| Max | 256 |

```
setoption name Threads value 4
```

**Notes:**
- More threads can improve search speed
- Current Lazy-SMP implementation has limited scaling
- Diminishing returns beyond CPU core count

### Style

Playing style personality.

| Property | Value |
|----------|-------|
| Type | combo |
| Default | Normal |
| Values | Normal, Tal, Petrosian |

```
setoption name Style value Tal
```

**Personalities:**
- **Normal**: Balanced play
- **Tal**: Aggressive, tactical (higher piece activity weights)
- **Petrosian**: Positional, solid (higher king safety weights)

### SyzygyPath

Path to Syzygy endgame tablebases.

| Property | Value |
|----------|-------|
| Type | string |
| Default | (empty) |

```
setoption name SyzygyPath value /path/to/syzygy
```

**Notes:**
- Enables perfect endgame play with <= 6 pieces
- Requires Syzygy files (WDL/DTZ)
- Multiple paths separated by `:` (Linux) or `;` (Windows)

### BookFile

Path to Polyglot opening book.

| Property | Value |
|----------|-------|
| Type | string |
| Default | (empty) |

```
setoption name BookFile value /path/to/book.bin
```

**Notes:**
- Polyglot format (.bin files)
- Used for opening moves
- Disabled if path is empty

### UseExperienceBook

Enable/disable experience book learning.

| Property | Value |
|----------|-------|
| Type | check |
| Default | true |

```
setoption name UseExperienceBook value false
```

**Notes:**
- Experience book learns from played games
- Uses Q-learning style updates
- Persisted between sessions

### MoveOverhead

Time buffer for move transmission (milliseconds).

| Property | Value |
|----------|-------|
| Type | spin |
| Default | 80 |
| Min | 0 |
| Max | 5000 |

```
setoption name MoveOverhead value 100
```

**Notes:**
- Compensates for GUI/network latency
- Subtracted from allocated time
- Increase if experiencing time losses

### MultiPV

Number of principal variations to output.

| Property | Value |
|----------|-------|
| Type | spin |
| Default | 1 |
| Min | 1 |
| Max | 500 |

```
setoption name MultiPV value 3
```

**Notes:**
- Useful for analysis mode
- Higher values slow search
- Each PV shown in info output

## UCI Protocol

### Initialization

```
uci
# Engine responds with id and options
# ...
uciok

isready
readyok
```

### Setting Options

```
setoption name Hash value 128
setoption name Threads value 2
setoption name Style value Tal
```

### Starting a Game

```
ucinewgame
position startpos moves e2e4 e7e5
go wtime 300000 btime 300000
```

### Stopping Search

```
stop
```

### Quitting

```
quit
```

## Example Session

```
uci
id name Scacchista
id author Gaspare

option name Hash type spin default 64 min 1 max 32768
option name Threads type spin default 1 min 1 max 256
option name Style type combo default Normal var Normal var Tal var Petrosian
option name SyzygyPath type string default
option name BookFile type string default
option name UseExperienceBook type check default true
option name MoveOverhead type spin default 80 min 0 max 5000
uciok

isready
readyok

setoption name Hash value 128
setoption name Threads value 2

ucinewgame

position startpos
go depth 10
info depth 1 score cp 20 pv g1f3
info depth 2 score cp 0 pv g1f3 g8f6
...
info depth 10 score cp 25 nodes 145623 nps 48541 pv g1f3 g8f6 ...
bestmove g1f3

quit
```

## Default Configuration

For general use:

```
setoption name Hash value 256
setoption name Threads value 1
setoption name Style value Normal
setoption name MoveOverhead value 80
```

For analysis:

```
setoption name Hash value 1024
setoption name Threads value 4
setoption name MultiPV value 3
```

## Troubleshooting

### Option Not Applied

- Set options **before** `ucinewgame`
- Some options require restart
- Check option name spelling (case-sensitive)

### Time Management Issues

- Increase `MoveOverhead` if timing out
- Check GUI time settings match engine

### Memory Issues

- Reduce `Hash` if running out of memory
- Each MB of hash requires 1 MB RAM

---

**Related Documents:**
- [Development Setup](../development/setup.md)
- [Architecture Overview](../architecture/overview.md)
