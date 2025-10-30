use std::env;
use std::sync::LazyLock;

const DEFAULT_DIFFICULTY_PREFIX: &'static str = "000";

pub static DIFFICULTY_PREFIX: LazyLock<String> = LazyLock::new(|| {
    let difficulty_prefix =
        env::var("DIFFICULTY_PREFIX").unwrap_or(DEFAULT_DIFFICULTY_PREFIX.into());

    if is_valid_difficulty_prefix(&difficulty_prefix) {
        difficulty_prefix
    } else {
        DEFAULT_DIFFICULTY_PREFIX.into()
    }
});

fn is_valid_difficulty_prefix(difficulty_prefix: &str) -> bool {
    let mut valid = true;

    difficulty_prefix.chars().for_each(|c| {
        if c != '0' {
            valid = false;
        }
    });

    valid
}
