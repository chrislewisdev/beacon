use aws_sdk_route53::error::{ProvideErrorMetadata, SdkError};

// Helper to easily map results into strings
pub trait ErrorContext<R> {
    fn context(self, c: &str) -> Result<R, String>;
}
impl<R, E> ErrorContext<R> for Result<R, E>
where
    E: ToString,
{
    fn context(self, c: &str) -> Result<R, String> {
        self.map_err(|e| format!("{}: {}", c, e.to_string()))
    }
}

// AWS SdkErrors returned by API requests don't provide useful messages from to_string().
// The message() function can be used instead for an informative error.
pub trait AwsErrorContext<R> {
    fn aws_context(self, c: &str) -> Result<R, String>;
}
impl<R, E> AwsErrorContext<R> for Result<R, SdkError<E>>
where
    E: ProvideErrorMetadata,
{
    fn aws_context(self, c: &str) -> Result<R, String> {
        self.map_err(|e| format!("{}: {}", c, e.message().unwrap_or("Unknown sdk error")))
    }
}
