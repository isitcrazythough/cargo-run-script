use std::fmt;
use std::process::ExitCode;
use std::process::ExitStatus;
use std::process::Termination;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ErrorType {
    NoScriptName = 1,
    InvalidScriptName,
    ScriptFailed,
    ScriptFailedWithSignal,
    NoScriptInfo,
    NoToml,
}

pub struct Error {
    pub error_type: ErrorType,
    pub reason: String,
}

impl Error {
    pub fn new(code: ErrorType, reason: impl Into<String>) -> Self {
        Self {
            error_type: code,
            reason: reason.into(),
        }
    }

    pub fn parse_exit_status(status: ExitStatus) -> Result<(), Self> {
        match status.code() {
            Some(code) => match code {
                0 => Ok(()),
                _ => Err(Self {
                    error_type: ErrorType::ScriptFailed,
                    reason: "script failed".into(),
                }),
            },
            None => Err(Self {
                error_type: ErrorType::ScriptFailedWithSignal,
                reason: "exited with signal".into(),
            }),
        }
    }
}

impl Termination for Error {
    fn report(self) -> ExitCode {
        ExitCode::from(self.error_type as u8)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error [{}]: {}", self.error_type as u8, self.reason)
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.error_type as u8 == other.error_type as u8
    }
}