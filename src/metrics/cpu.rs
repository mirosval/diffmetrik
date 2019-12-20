use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use sysctl::Sysctl;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CPUMetrics {
    pub m1: f32,
    pub m5: f32,
    pub m15: f32,
}

#[repr(C)]
#[derive(Debug)]
struct loadavg {
    ldavg: [u32; 3],
    fscale: i64,
}

#[cfg(not(target_os = "linux"))]
pub fn get_cpu_metrics() -> Result<CPUMetrics, CpuError> {
    let ctl = sysctl::Ctl::new("vm.loadavg").map_err(|_| CpuError::CtlError)?;
    let vval = ctl.value().map_err(|_| CpuError::CtlError)?;
    if let sysctl::CtlValue::Struct(sval) = vval {
        let x: loadavg = unsafe { std::mem::transmute_copy(&sval[0]) };
        Ok(CPUMetrics {
            m1: x.ldavg[0] as f32 / x.fscale as f32,
            m5: x.ldavg[1] as f32 / x.fscale as f32,
            m15: x.ldavg[2] as f32 / x.fscale as f32,
        })
    } else {
        Err(CpuError::GetMetrics(
            "value retrieved from ctl was not a struct".to_string(),
        ))
    }
}

#[cfg(target_os = "linux")]
pub fn get_cpu_metrics() -> Result<CPUMetrics, CpuError> {
    let text = std::fs::read_to_string("/proc/loadavg").map_err(|e| CpuError::IO(e))?;
    let parsed = text
        .split(' ')
        .take(3)
        .map(|n| n.parse::<f32>())
        .collect::<Result<Vec<f32>, std::num::ParseFloatError>>()?;
    Ok(CPUMetrics {
        m1: parsed[0],
        m5: parsed[1],
        m15: parsed[2],
    })
}
