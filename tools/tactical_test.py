#!/usr/bin/env python3
"""Run Scacchista on EPD suites and report bestmove accuracy."""
import argparse
import os
import subprocess
import sys
import time

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SUITE_FILES = {
    "wac": "tools/test_suites/wac.epd",
    "bk": "tools/test_suites/bk.epd",
    "eigenmann": "tools/test_suites/eigenmann.epd",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Tactical suite regression tool")
    parser.add_argument("--suite", choices=SUITE_FILES.keys(), default="wac")
    parser.add_argument("--depth", type=int, default=6)
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--limit", type=int, default=None, help="Limit number of positions")
    parser.add_argument(
        "--engine",
        default=os.path.join(BASE_DIR, "target/release/scacchista"),
        help="Path to the Scacchista UCI binary",
    )
    return parser.parse_args()


def load_positions(path: str):
    with open(path, "r", encoding="utf-8") as fp:
        for raw in fp:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            parts = [segment.strip() for segment in line.split(";") if segment.strip()]
            if not parts:
                continue
            fen = parts[0]
            bm = None
            identifier = None
            for token in parts[1:]:
                if token.startswith("bm "):
                    bm_parts = token.split()
                    if len(bm_parts) >= 2:
                        bm = bm_parts[1]
                elif token.startswith("id "):
                    identifier = token[3:].strip().strip('"')
            if bm is None:
                continue
            yield fen, bm, identifier


def query_bestmove(engine_path: str, fen: str, depth: int, threads: int) -> str | None:
    commands = [
        "uci",
        f"setoption name Threads value {threads}",
        "isready",
        f"position fen {fen}",
        "isready",
        f"go depth {depth}",
        "quit",
    ]
    proc = subprocess.run(
        [engine_path],
        input="\n".join(commands) + "\n",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"Engine {engine_path} exited with code {proc.returncode}: {proc.stderr.strip()}"
        )
    for line in proc.stdout.splitlines():
        if line.startswith("bestmove"):
            parts = line.split()
            if len(parts) >= 2:
                return parts[1]
            return None
    return None


def run_suite(engine_path: str, suite_path: str, depth: int, threads: int, limit: int | None):
    positions = list(load_positions(suite_path))
    if limit is not None:
        positions = positions[:limit]
    if not positions:
        raise SystemExit("No positions found in suite")

    print(f"Running suite {os.path.basename(suite_path)} ({len(positions)} positions) at depth {depth}")
    mismatches = []
    solved = 0
    start = time.time()
    for idx, (fen, expected, identifier) in enumerate(positions, start=1):
        print(f"[{idx}/{len(positions)}] id={identifier or 'N/A'} expecting {expected}", flush=True)
        try:
            bestmove = query_bestmove(engine_path, fen, depth, threads)
        except RuntimeError as exc:
            print(f"Engine error on position {idx}: {exc}")
            bestmove = None
        if bestmove == expected:
            solved += 1
        else:
            mismatches.append((idx, identifier, expected, bestmove))
            print(
                f"Mismatch #{idx}: expected {expected}, got {bestmove}",
                flush=True,
            )
    duration = time.time() - start
    solved_ratio = (solved / len(positions)) * 100
    print("\n=== Summary ===")
    print(f"Suite: {os.path.basename(suite_path)}")
    print(f"Depth: {depth}")
    print(f"Positions: {len(positions)}")
    print(f"Matched: {solved}/{len(positions)} ({solved_ratio:.2f}%)")
    print(f"Duration: {duration:.1f}s ({duration/len(positions):.2f}s per position)")
    if mismatches:
        print("Mismatches details:")
        for idx, identifier, expected, best in mismatches:
            print(f"  #{idx} {identifier or 'N/A'}: expected {expected}, got {best}")


def main() -> None:
    args = parse_args()
    suite_path = SUITE_FILES[args.suite]
    if not os.path.exists(suite_path):
        raise SystemExit(f"Suite file not found: {suite_path}")
    if not os.path.exists(args.engine):
        raise SystemExit(f"Engine binary not found: {args.engine}")
    run_suite(args.engine, suite_path, args.depth, args.threads, args.limit)


if __name__ == "__main__":
    main()
