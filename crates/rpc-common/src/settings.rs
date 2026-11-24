//! `.env` loading.

/// Load `.env` into the process environment (missing file is ignored).
pub fn load_dotenv() {
    let _ = dotenvy::dotenv();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_is_idempotent_and_safe() {
        load_dotenv();
        load_dotenv();
    }
}
