use super::error::CpuError;
use super::CPUMetrics;
use sysctl::Sysctl;

#[repr(C)]
#[derive(Debug)]
struct loadavg {
    ldavg: [u32; 3],
    fscale: i64,
}

#[cfg(target_os = "macos")]
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
