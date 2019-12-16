use super::errors::Error;
use serde::{Deserialize, Serialize};
use sysctl::Sysctl;

type Result<T, E = Error> = std::result::Result<T, E>;

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

pub fn get_cpu_metrics() -> Result<CPUMetrics> {
    let ctl = sysctl::Ctl::new("vm.loadavg").map_err(super::errors::CtlErrorWrapper::new)?;
    let vval = ctl.value().map_err(super::errors::CtlErrorWrapper::new)?;
    if let sysctl::CtlValue::Struct(sval) = vval {
        let x: loadavg = unsafe { std::mem::transmute_copy(&sval[0]) };
        Ok(CPUMetrics {
            m1: x.ldavg[0] as f32 / x.fscale as f32,
            m5: x.ldavg[1] as f32 / x.fscale as f32,
            m15: x.ldavg[2] as f32 / x.fscale as f32,
        })
    } else {
        Err(Error::GetMetrics {
            message: "value retrieved from ctl was not a struct".to_string(),
        })
    }
}
