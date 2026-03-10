
import chess
import chess.engine
import chess.pgn
import sys
import math

def calculate_elo_diff(score_fraction):
    if score_fraction <= 0 or score_fraction >= 1:
        return 0
    return -400 * math.log10(1 / score_fraction - 1)

def run_tournament(rounds=20, time_limit=10.0, increment=0.1):
    engine1_path = "./scacchista_v0.5"
    engine2_path = "./scacchista_v0.4"

    print(f"Starting tournament: {engine1_path} vs {engine2_path}")
    print(f"Rounds: {rounds}, Time Control: {time_limit}+{increment}")

    engine1_score = 0.0
    engine2_score = 0.0
    draws = 0

    for i in range(1, rounds + 1):
        board = chess.Board()
        
        # Alternating colors
        if i % 2 != 0:
            white_engine_path = engine1_path
            black_engine_path = engine2_path
            white_name = "v0.5"
            black_name = "v0.4"
        else:
            white_engine_path = engine2_path
            black_engine_path = engine1_path
            white_name = "v0.4"
            black_name = "v0.5"

        try:
            # Simple engine management for each game to avoid state issues
            engine_w = chess.engine.SimpleEngine.popen_uci(white_engine_path)
            engine_b = chess.engine.SimpleEngine.popen_uci(black_engine_path)
        except Exception as e:
            print(f"Error starting engines: {e}")
            return

        print(f"Game {i}/{rounds} ({white_name} vs {black_name})...", end="", flush=True)

        while not board.is_game_over():
            if board.turn == chess.WHITE:
                result = engine_w.play(board, chess.engine.Limit(time=time_limit, increment=increment))
                board.push(result.move)
            else:
                result = engine_b.play(board, chess.engine.Limit(time=time_limit, increment=increment))
                board.push(result.move)

        outcome = board.outcome()
        result_str = "1/2-1/2"
        
        if outcome.winner == chess.WHITE:
            result_str = "1-0"
            if white_name == "v0.5":
                engine1_score += 1
            else:
                engine2_score += 1
        elif outcome.winner == chess.BLACK:
            result_str = "0-1"
            if black_name == "v0.5":
                engine1_score += 1
            else:
                engine2_score += 1
        else:
            engine1_score += 0.5
            engine2_score += 0.5
            draws += 1

        print(f" {result_str} ({outcome.termination})")
        
        engine_w.quit()
        engine_b.quit()

    print("\n--- Final Results ---")
    print(f"v0.5 Score: {engine1_score}")
    print(f"v0.4 Score: {engine2_score}")
    print(f"Draws: {draws}")
    
    total_score = engine1_score + engine2_score
    if total_score > 0:
        fraction = engine1_score / total_score
        elo_diff = calculate_elo_diff(fraction)
        print(f"Elo Difference (v0.5 - v0.4): {elo_diff:+.1f}")

if __name__ == "__main__":
    run_tournament()
