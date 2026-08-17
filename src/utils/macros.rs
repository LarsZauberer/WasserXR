//! This module defines macros which are not exposed publically to the public API. These macros are
//! intended to be used internally for better code sanity.
//!
//! The macros described here are macros should have a global context and could be utilized
//! everywhere in WasserXR. They are not closely coupled to a certain module. If a macro is closely
//! coupled and has no global use case, it should be defined directly in the corresponding module
//! instead of here

/// This macro produces a simple standardized log message prefix. It should be used when inside of
/// an assert macro, which prints this message.
///
/// It takes another [&str] as an argument, to describe what went wrong. This message should help
/// the bug report, what happened.
macro_rules! invariant_msg {
    ($msg: expr) => {
        format!("WasserXR Invariant Violation! This is a Bug. Please report it to https://github.com/LarsZauberer/WasserXR/issues with the following message: {}", $msg)
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test_invariant_msg() {
        println!("{}", invariant_msg!("This is an intended invariant break!"));
    }
}
