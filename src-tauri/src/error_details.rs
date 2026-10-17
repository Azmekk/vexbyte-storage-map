use std::any::Any;
use std::error::Error;

/// The error's message followed by the messages of its sources, for the `details` field the
/// frontend shows. The library errors keep the operating system message in their sources.
pub fn describe_error_with_sources(error: &dyn Error) -> String {
    let mut description = error.to_string();
    let mut source = error.source();

    while let Some(cause) = source {
        let cause_text = cause.to_string();
        if !description.ends_with(&cause_text) {
            description.push_str(": ");
            description.push_str(&cause_text);
        }
        source = cause.source();
    }

    description
}

/// The message of a caught panic, when it carried one.
pub fn describe_panic(panic_payload: &(dyn Any + Send)) -> String {
    if let Some(message) = panic_payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = panic_payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "the background thread panicked".to_owned()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("could not read the folder")]
    struct OuterError {
        source: std::io::Error,
    }

    #[test]
    fn appends_every_source() {
        let error = OuterError {
            source: std::io::Error::other("access is denied"),
        };

        assert_eq!(
            describe_error_with_sources(&error),
            "could not read the folder: access is denied"
        );
    }
}
