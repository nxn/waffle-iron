// Don't use regex to validate the input, the lib is way too big to include.
pub fn puzzle_str_to_bytes(puzzle: &str) -> Option<[u8; 81]> {
    let mut result = [0; 81];
    let bytes = puzzle.as_bytes();

    if bytes.len() != 81 {
        return None;
    }

    for i in 0..bytes.len() {
        let b = bytes[i];

        if b > 47 && b < 58 {
            result[i] = bytes[i] - 48;
        }
        else {
            return None;
        }
    }

    Some(result)
}