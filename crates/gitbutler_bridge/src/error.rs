use anyhow::Result;

/// Maps a GitButler error code to an anyhow::Error with appropriate context.
///
/// Once `but_error` is available as a dependency, this will convert
/// `but_error::Code` variants into structured anyhow errors that Zed
/// can present to the user.
pub fn map_gitbutler_error(message: &str) -> anyhow::Error {
    anyhow::anyhow!("gitbutler: {}", message)
}

/// Wraps a fallible GitButler operation, converting its error type to anyhow.
pub fn wrap_result<T, E: std::fmt::Display>(result: Result<T, E>) -> Result<T> {
    result.map_err(|error| anyhow::anyhow!("gitbutler: {}", error))
}
