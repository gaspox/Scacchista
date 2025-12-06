use scacchista::uci::{parse_uci_command, UciCommand};

#[test]
fn test_parse_full_go_command() {
    let line = "go wtime 300000 btime 300000 movetime 1000 depth 6 nodes 1000000 mate 3 movestogo 5 infinite ponder";
    match parse_uci_command(line) {
        UciCommand::Go {
            wtime,
            btime,
            movetime,
            depth,
            nodes,
            mate,
            movestogo,
            infinite,
            ponder,
            ..  // Ignore winc, binc (added in Bug #4 fix)
        } => {
            assert_eq!(wtime, Some(300000));
            assert_eq!(btime, Some(300000));
            assert_eq!(movetime, Some(1000));
            assert_eq!(depth, Some(6));
            assert_eq!(nodes, Some(1000000));
            assert_eq!(mate, Some(3));
            assert_eq!(movestogo, Some(5));
            assert!(infinite);
            assert!(ponder);
        }
        other => panic!("Expected Go command, got: {:?}", other),
    }
}
