use failure::Fail;
use snafu::Snafu;

#[derive(Debug)]
pub struct CtlErrorWrapper {
    message: String,
}

impl CtlErrorWrapper {
    pub fn new(e: sysctl::SysctlError) -> Self {
        CtlErrorWrapper {
            message: e.name().unwrap_or("Unknown SysCtl error").to_string(),
        }
    }
}

impl std::fmt::Display for CtlErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CtlErrorWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::convert::From<CtlErrorWrapper> for Error {
    fn from(e: CtlErrorWrapper) -> Error {
        Error::CtlError { message: e.message }
    }
}

#[derive(Debug, Snafu)]
pub struct PubTime(Error);

#[derive(Debug, Snafu)]
pub enum Error {
    Time {
        message: String,
        source: std::time::SystemTimeError,
    },
    GetMetrics {
        message: String,
    },
    CtlError {
        message: String,
    },
}
