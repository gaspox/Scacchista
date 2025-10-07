use scacchista::uci::parse_uci_command;

#[test]
fn test_parse_full_go_command() {
    let line = "go wtime 300000 btime 300000 movetime 1000 depth 6 nodes 1000000 mate 3 movestogo 5 infinite ponder";
    match parse_uci_command(line) {
        UciCommand::Go {
            wtime,
            btime,
            movetime,
        },
        _ => panic!("Expected Go command"),
    30→    } else {
    31→        UciCommand::Unknown("".to_string())}
}