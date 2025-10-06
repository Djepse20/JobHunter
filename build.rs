fn main() -> Result<(), dotenvy::Error> {
    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    dotenvy::dotenv()?;

    Ok(())
}
