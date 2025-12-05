# Specialized Agents Configuration

This document describes specialized Claude agents configured for the Scacchista project.

## Overview

Specialized agents provide focused expertise for specific tasks. They are configured in `.claude/agents/`.

## Available Agents

### GrandMaster

**Purpose:** Chess engine evaluation, design, and analysis
**Model:** Opus
**Location:** `.claude/agents/GrandMaster.md`

#### Identity

The GrandMaster agent has dual expertise:
1. **Elite Grandmaster (GM):** 2700+ ELO understanding of chess strategy
2. **Senior Engine Developer:** Expert in C++, Rust, CUDA, SIMD for chess engines

#### Knowledge Areas

- UCI Protocol
- Search Algorithms (Alpha-Beta, PVS, Lazy SMP, MCTS)
- Evaluation Functions (HCE, NNUE)
- Optimization (Bitboards, Magic Bitboards, Move Ordering, TT)
- Hardware optimization (AVX2/512)

#### Workflow

1. **Research & Contextualization**
   - Check current SOTA (Stockfish, TCEC results)
   - Compare project against industry leaders

2. **Professional Analysis**
   - Developer view: Performance, thread safety, memory
   - GM view: Does engine "understand" positions?

3. **Improvement Hierarchy**
   - Strength & Speed (ELO gains)
   - GM Utility (analysis features)
   - Commercial appeal
   - Fun/gamification

#### Usage

Invoke for:
- Architecture review
- Search algorithm analysis
- Evaluation function tuning
- Performance optimization strategy
- Feature prioritization

## Agent Configuration Format

Agents are configured as markdown files with YAML frontmatter:

```markdown
---
name: AgentName
description: Brief description
model: opus|sonnet|haiku
color: pink|blue|green|etc
---

# Identity & Persona

[Agent instructions...]
```

## Creating New Agents

### When to Create

- Specialized domain expertise needed
- Recurring complex task type
- Need consistent persona/approach

### Best Practices

1. **Clear identity**: Define who the agent "is"
2. **Specific knowledge**: List expertise areas
3. **Defined workflow**: Step-by-step process
4. **Output format**: Specify how to structure responses

### Example Template

```markdown
---
name: Specialist
description: For X type of tasks
model: opus
color: blue
---

# IDENTITY
You are a [role] specializing in [domain].

# KNOWLEDGE
- Area 1
- Area 2

# WORKFLOW
1. Step 1
2. Step 2

# OUTPUT FORMAT
- Use markdown
- Include code examples
```

## Potential Future Agents

### TesterDebugger

For systematic testing and bug investigation:
- Perft validation
- Regression testing
- Bug reproduction
- Root cause analysis

### StressStrategist

For stress testing and edge cases:
- Corner case generation
- Performance stress tests
- Concurrency testing
- Memory pressure tests

### DocumentationWriter

For documentation tasks:
- API documentation
- User guides
- Technical specifications
- Tutorial creation

## Using Agents

### Invocation

Agents are typically invoked by referencing their role or directly loading their configuration.

### Context Sharing

When working with agents:
- Share relevant code context
- Provide current test results
- Include performance metrics
- Reference previous analysis

### Best Results

1. **Be specific**: Clear task description
2. **Provide context**: Code, metrics, history
3. **Set constraints**: Time, risk tolerance
4. **Request format**: How you want output

---

**Related Documents:**
- [Project Guide](./project-guide.md)
- [Handoff Document](./handoff.md)
- [Architecture Overview](../architecture/overview.md)
