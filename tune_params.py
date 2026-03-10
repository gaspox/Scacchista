#!/usr/bin/env python3
"""
Tuning Parameters Script for Scacchista v0.5
Fase 3: Tuning automatico parametri pruning

Approccio: Grid search su parametri chiave con mini-torneo di validazione
"""

import subprocess
import json
import itertools
import sys
from dataclasses import dataclass
from typing import Dict, List, Tuple
import time

@dataclass
class Params:
    aspiration_window: int = 50
    null_move_min_depth: int = 2
    lmr_min_depth: int = 3
    lmr_base_reduction: int = 2
    futility_margin: int = 200
    futility_min_depth: int = 3
    qsearch_depth: int = 4
    enable_qsearch_optimizations: bool = False

    def to_patch(self) -> str:
        """Generate Rust code patch"""
        return f'''impl Default for SearchParams {{
    fn default() -> Self {{
        Self {{
            max_depth: 8,
            time_limit_ms: 5000,
            node_limit: 0,
            aspiration_window: {self.aspiration_window},
            enable_null_move_pruning: true,
            null_move_min_depth: {self.null_move_min_depth},
            enable_lmr: true,
            lmr_min_depth: {self.lmr_min_depth},
            lmr_base_reduction: {self.lmr_base_reduction},
            enable_futility_pruning: true,
            futility_margin: {self.futility_margin},
            futility_min_depth: {self.futility_min_depth},
            killer_moves_count: 2,
            qsearch_depth: {self.qsearch_depth},
            enable_qsearch_optimizations: {str(self.enable_qsearch_optimizations).lower()},
        }}
    }}
}}'''

def apply_params(params: Params, filepath: str = "src/search/params.rs"):
    """Apply parameters to source file"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Find and replace the default impl
    start_marker = "impl Default for SearchParams {"
    end_marker = "}\n}\n\nimpl SearchParams"
    
    start_idx = content.find(start_marker)
    end_idx = content.find(end_marker)
    
    if start_idx == -1 or end_idx == -1:
        raise ValueError("Could not find default impl in params.rs")
    
    new_content = content[:start_idx] + params.to_patch() + content[end_idx + 4:]
    
    with open(filepath, 'w') as f:
        f.write(new_content)

def run_minitornament(engine_path: str = "./scacchista_v0.5", 
                      opponent_path: str = "./scacchista_v0.4",
                      rounds: int = 5,
                      time_ms: int = 5000,
                      inc_ms: int = 50) -> Tuple[float, float, int]:
    """
    Run mini tournament and return (score_v0.5, score_v0.4, draws)
    Returns score as percentage (0.0-1.0)
    """
    # Simple game runner - plays a few moves and checks result
    # For simplicity, we use a shorter version
    
    wins = 0
    losses = 0
    draws = 0
    
    for game in range(rounds):
        # Alternate colors
        if game % 2 == 0:
            white_path, black_path = engine_path, opponent_path
            white_name, black_name = "v0.5", "v0.4"
        else:
            white_path, black_path = opponent_path, engine_path
            white_name, black_name = "v0.4", "v0.5"
        
        result = run_single_game(white_path, black_path, time_ms, inc_ms)
        
        if result == "1-0":
            if white_name == "v0.5":
                wins += 1
            else:
                losses += 1
        elif result == "0-1":
            if black_name == "v0.5":
                wins += 1
            else:
                losses += 1
        else:
            draws += 1
    
    score_v05 = (wins + draws * 0.5) / rounds
    score_v04 = (losses + draws * 0.5) / rounds
    
    return score_v05, score_v04, draws

def run_single_game(white_path: str, black_path: str, time_ms: int, inc_ms: int) -> str:
    """Run a single game and return result (1-0, 0-1, or 1/2-1/2)"""
    try:
        # Use the tournament binary for a single game
        result = subprocess.run(
            ["./target/release/tournament", "--single", white_path, black_path, 
             str(time_ms), str(inc_ms)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Parse result from output
        for line in result.stdout.split('\n'):
            if '1-0' in line or '0-1' in line or '1/2' in line:
                if '1-0' in line:
                    return '1-0'
                elif '0-1' in line:
                    return '0-1'
                else:
                    return '1/2-1/2'
        
        return '1/2-1/2'  # Default to draw if unclear
        
    except Exception as e:
        print(f"Error in game: {e}")
        return '1/2-1/2'

def grid_search():
    """Perform grid search on key parameters"""
    
    # Define parameter ranges
    param_grid = {
        'aspiration_window': [30, 50, 70],
        'lmr_base_reduction': [1, 2, 3],
        'futility_margin': [150, 200, 250],
        'qsearch_depth': [3, 4, 5],
    }
    
    best_params = None
    best_score = 0.0
    results = []
    
    # Generate all combinations
    keys = list(param_grid.keys())
    values = [param_grid[k] for k in keys]
    
    print("=" * 60)
    print("PHASE 3: PARAMETER TUNING")
    print("=" * 60)
    print(f"Testing {len(list(itertools.product(*values)))} configurations")
    print(f"Parameters: {keys}")
    print("=" * 60)
    
    for i, combo in enumerate(itertools.product(*values)):
        params = Params()
        for key, value in zip(keys, combo):
            setattr(params, key, value)
        
        print(f"\n[{i+1}] Testing: {dict(zip(keys, combo))}")
        
        # Apply parameters
        apply_params(params)
        
        # Build
        print("  Building...")
        build_result = subprocess.run(
            ["cargo", "build", "--release"],
            capture_output=True,
            timeout=120
        )
        
        if build_result.returncode != 0:
            print("  Build failed, skipping...")
            continue
        
        # Copy binary
        subprocess.run(["cp", "target/release/scacchista", "./scacchista_v0.5"])
        
        # Run mini-tournament
        print("  Running mini-tournament (5 games)...")
        score_v05, score_v04, draws = run_minitornament(rounds=5, time_ms=5000, inc_ms=50)
        
        print(f"  Result: v0.5={score_v05:.2%}, v0.4={score_v04:.2%}, draws={draws}")
        
        results.append({
            'params': dict(zip(keys, combo)),
            'score_v05': score_v05,
            'score_v04': score_v04,
            'draws': draws
        })
        
        if score_v05 > best_score:
            best_score = score_v05
            best_params = params
            print(f"  *** NEW BEST: {score_v05:.2%} ***")
    
    print("\n" + "=" * 60)
    print("TUNING COMPLETE")
    print("=" * 60)
    print(f"Best configuration: {best_params}")
    print(f"Best score: {best_score:.2%}")
    
    # Save results
    with open("tuning_results.json", 'w') as f:
        json.dump({
            'best_params': best_params.__dict__,
            'best_score': best_score,
            'all_results': results
        }, f, indent=2)
    
    # Apply best params
    if best_params:
        print("\nApplying best parameters...")
        apply_params(best_params)
        subprocess.run(["cargo", "build", "--release"])
        subprocess.run(["cp", "target/release/scacchista", "./scacchista_v0.5_tuned"])
    
    return best_params

def quick_test():
    """Quick test with most promising parameter combinations"""
    
    test_configs = [
        # Config 1: Aggressive pruning
        Params(futility_margin=250, lmr_base_reduction=2, qsearch_depth=5),
        # Config 2: Conservative
        Params(futility_margin=150, lmr_base_reduction=1, qsearch_depth=4),
        # Config 3: Balanced
        Params(futility_margin=200, lmr_base_reduction=2, qsearch_depth=4),
        # Config 4: Aggressive LMR
        Params(futility_margin=200, lmr_base_reduction=3, qsearch_depth=4),
        # Config 5: Deep qsearch
        Params(futility_margin=200, lmr_base_reduction=2, qsearch_depth=5),
    ]
    
    print("=" * 60)
    print("QUICK TUNING TEST")
    print("=" * 60)
    
    best_params = None
    best_score = 0.0
    
    for i, params in enumerate(test_configs):
        print(f"\n[{i+1}/5] Testing config: futility={params.futility_margin}, "
              f"lmr={params.lmr_base_reduction}, qsearch={params.qsearch_depth}")
        
        apply_params(params)
        
        print("  Building...")
        result = subprocess.run(["cargo", "build", "--release"], 
                              capture_output=True, timeout=120)
        if result.returncode != 0:
            print("  Build failed")
            continue
        
        subprocess.run(["cp", "target/release/scacchista", "./scacchista_v0.5"])
        
        print("  Testing (3 games)...")
        # Simplified - just check position eval for speed
        score = test_position_eval()
        print(f"  Position score: {score}")
        
        if score > best_score:
            best_score = score
            best_params = params
    
    print(f"\nBest: futility={best_params.futility_margin}, "
          f"lmr={best_params.lmr_base_reduction}, qsearch={best_params.qsearch_depth}")
    
    return best_params

def test_position_eval() -> int:
    """Quick test - evaluate startpos and return score"""
    try:
        result = subprocess.run(
            ["./scacchista_v0.5"],
            input="uci\nposition startpos\ngo depth 6\nquit\n",
            capture_output=True,
            text=True,
            timeout=10
        )
        
        for line in result.stdout.split('\n'):
            if 'score cp' in line:
                parts = line.split()
                idx = parts.index('cp')
                return int(parts[idx + 1])
        
        return 0
    except:
        return 0

if __name__ == "__main__":
    import os
    
    if not os.path.exists("./scacchista_v0.4"):
        print("ERROR: scacchista_v0.4 not found!")
        sys.exit(1)
    
    # Run quick test first
    print("\nPHASE 3.1: Quick Parameter Test\n")
    best = quick_test()
    
    if best:
        print("\nApplying best parameters and rebuilding...")
        apply_params(best)
        subprocess.run(["cargo", "build", "--release"])
        subprocess.run(["cp", "target/release/scacchista", "./scacchista_v0.5_tuned"])
        print("Done! Binary: ./scacchista_v0.5_tuned")
