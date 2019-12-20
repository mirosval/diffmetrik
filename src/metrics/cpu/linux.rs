use super::error::CpuError;
use super::CPUMetrics;

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
