#[derive(Debug)]
pub enum CpuError {
    #[allow(dead_code)]
    GetMetrics(String),
    #[allow(dead_code)]
    CtlError,
    #[allow(dead_code)]
    IO(std::io::Error),
    ParseError(std::num::ParseFloatError),
}

impl std::fmt::Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CpuError::CtlError => write!(f, "CtlError"),
            CpuError::GetMetrics(e) => write!(f, "{}", &e),
            CpuError::IO(e) => write!(f, "{}", &e),
            CpuError::ParseError(e) => write!(f, "{}", &e),
        }
    }
}

impl From<std::num::ParseFloatError> for CpuError {
    fn from(e: std::num::ParseFloatError) -> CpuError {
        CpuError::ParseError(e)
    }
}
