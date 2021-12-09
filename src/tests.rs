use super::*;
use std::collections::HashSet;

#[test]
fn output_equals_input() {
    /* Multiline strings are not indented correctly because the indentation would change the
     * string content. */
    let input = "
   0  +2
-2   0  -3  +3
   0           0
"
    .trim_matches('\n');
    assert_eq!(input, Board::parse(input).unwrap().write(false));
}

#[test]
fn parse_fails_on_invalid_board() {
    assert!(Board::parse("abcdefg").is_err());
}

#[test]
fn possible_moves_are_found() {
    let input = "
   0  +2
-2   0  -3  +3
   0           0
"
    .trim_matches('\n');
    let max_moves = [
        "
  +1  +1
-2   0  -3  +3
   0           0
"
        .trim_matches('\n'),
        "
   0  +1
-2   0  -3  +3
  +1           0
"
        .trim_matches('\n'),
        "
   0  +2
-2   0  -3  +2
   0          +1
"
        .trim_matches('\n'),
        "
   0  +2
-2   0  -3  +1
   0          +2
"
        .trim_matches('\n'),
    ];
    assert_eq!(
        Board::parse(input)
            .unwrap()
            .possible_moves(Player::Max)
            .collect::<HashSet<Board>>(),
        max_moves
            .iter()
            .map(|s| Board::parse(s).unwrap())
            .collect::<HashSet<Board>>()
    );
}

#[test]
fn win_evaluates_as_winners_advantage() {
    let max_wins = "
  +14 +1   0   0
-15 +1  -1   0
"
    .trim_matches('\n');
    assert!(Board::parse(max_wins).unwrap().heuristic_evaluate() > 0);
}

#[test]
fn end_with_equal_controlled_tiles_considers_field_size() {
    let max_has_greater_field = "
  +15 -1   0   0
-15 +1   0   0
"
    .trim_matches('\n');
    assert!(
        Board::parse(max_has_greater_field)
            .unwrap()
            .heuristic_evaluate()
            > 0
    );
}

#[test]
fn draw_evaluates_as_zero() {
    let draw = "
  +1   0  -1  +14
-14 +1   0  -1
"
    .trim_matches('\n');
    assert!(Board::parse(draw).unwrap().heuristic_evaluate() == 0);
}

#[test]
fn in_end_tile_count_weighs_more_than_field_size() {
    let min_wins = "
             0   0
  +8  -1   0  -1
-14 +8
"
    .trim_matches('\n');
    assert!(Board::parse(min_wins).unwrap().heuristic_evaluate() < 0);
}

#[test]
fn win_evaluates_higher_than_continuing_game() {
    let min_wins = "
     0
   0   0   0
     0   0
  -15
+16 -1   0   0   0   0   0   0   0   0
"
    .trim_matches('\n');
    let min_will_lose = "
     0
   0  -15  0
     0   0
  -1
+16  0   0   0   0   0   0   0   0   0
"
    .trim_matches('\n');
    assert!(
        Board::parse(min_wins).unwrap().heuristic_evaluate()
            < Board::parse(min_will_lose).unwrap().heuristic_evaluate()
    );
}