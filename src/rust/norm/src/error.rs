use std::fmt;
use std::error::Error as StdError;

/// Error type for NORM operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// An invalid handle was provided to a NORM function
    InvalidHandle,

    /// An invalid network address was provided
    InvalidAddress,

    /// An invalid parameter was provided to a NORM function
    InvalidParameter,

    /// Memory allocation failed
    AllocationFailed,

    /// A socket-related error occurred
    SocketError(String),

    /// A file-related error occurred
    FileError(String),

    /// An operation timed out
    Timeout,

    /// A resource was not ready for the requested operation
    NotReady,

    /// An operation failed for an unspecified reason
    OperationFailed(String),

    /// A system error occurred (with errno)
    #[cfg(unix)]
    SystemError {
        /// The error message
        message: String,
        /// The errno value
        errno: i32,
    },

    /// A system error occurred (with Win32 error code)
    #[cfg(windows)]
    SystemError {
        /// The error message
        message: String,
        /// The Win32 error code
        error_code: u32,
    },

    /// A string conversion error occurred
    StringConversionError,

    /// A null pointer was encountered where a valid pointer was expected
    NullPointer,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidHandle => write!(f, "Invalid NORM handle"),
            Error::InvalidAddress => write!(f, "Invalid network address"),
            Error::InvalidParameter => write!(f, "Invalid parameter"),
            Error::AllocationFailed => write!(f, "Memory allocation failed"),
            Error::SocketError(s) => write!(f, "Socket error: {}", s),
            Error::FileError(s) => write!(f, "File error: {}", s),
            Error::Timeout => write!(f, "Operation timed out"),
            Error::NotReady => write!(f, "Resource not ready"),
            Error::OperationFailed(s) => write!(f, "Operation failed: {}", s),
            #[cfg(unix)]
            Error::SystemError { message, errno } => {
                write!(f, "System error: {} (errno: {})", message, errno)
            }
            #[cfg(windows)]
            Error::SystemError { message, error_code } => {
                write!(f, "System error: {} (error code: {})", message, error_code)
            }
            Error::StringConversionError => write!(f, "String conversion error"),
            Error::NullPointer => write!(f, "Null pointer encountered"),
        }
    }
}

impl StdError for Error {}

/// Result type for NORM operations
pub type Result<T> = std::result::Result<T, Error>;

/// Helper function to convert a C string to a Rust string result
///
/// # Safety
///
/// `c_str` must be null or point to a NUL-terminated string that stays valid
/// for the duration of the call. A non-null dangling pointer is undefined
/// behaviour, which is why this cannot be a safe function.
pub(crate) unsafe fn c_string_to_string(c_str: *const libc::c_char) -> Result<String> {
    if c_str.is_null() {
        return Err(Error::NullPointer);
    }

    // SAFETY: non-null, and the caller guarantees a valid NUL-terminated string.
    let c_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
    c_str.to_str()
        .map(|s| s.to_owned())
        .map_err(|_| Error::StringConversionError)
}

/// Helper function to convert a Rust string to a C string result
pub(crate) fn string_to_c_string(s: &str) -> Result<std::ffi::CString> {
    std::ffi::CString::new(s)
        .map_err(|_| Error::StringConversionError)
}

/// Convert a boolean result from NORM API to a Result
pub(crate) fn bool_result(success: bool, error_message: &str) -> Result<()> {
    if success {
        Ok(())
    } else {
        Err(Error::OperationFailed(error_message.to_string()))
    }
}

/// Helper to check if a handle is valid
///
/// # Safety
///
/// This function is unsafe because it may compare with extern static values
/// that are not controlled by the Rust type system.
pub(crate) unsafe fn check_handle<T>(handle: T, invalid: T) -> Result<T>
where
    T: PartialEq
{
    if handle == invalid {
        Err(Error::InvalidHandle)
    } else {
        Ok(handle)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_result_success() {
        assert!(bool_result(true, "Test error").is_ok());
    }

    #[test]
    fn test_bool_result_failure() {
        let result = bool_result(false, "Test error");
        assert!(result.is_err());
        match result {
            Err(Error::OperationFailed(msg)) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected OperationFailed error"),
        }
    }

    #[test]
    fn test_string_to_c_string() {
        let result = string_to_c_string("test");
        assert!(result.is_ok());

        let c_str = result.unwrap();
        assert_eq!(c_str.to_str().unwrap(), "test");
    }

    #[test]
    fn test_string_to_c_string_with_null() {
        // Build a string with an embedded null byte without a literal null in source
        let s = format!("test{}embedded", '\0');
        let result = string_to_c_string(&s);
        assert!(result.is_err());
        match result {
            Err(Error::StringConversionError) => {},
            _ => panic!("Expected StringConversionError"),
        }
    }

    #[test]
    fn test_error_display() {
        let err = Error::InvalidHandle;
        assert_eq!(err.to_string(), "Invalid NORM handle");

        let err = Error::OperationFailed("test failure".to_string());
        assert_eq!(err.to_string(), "Operation failed: test failure");

        let err = Error::FileError("file not found".to_string());
        assert_eq!(err.to_string(), "File error: file not found");
    }
}

