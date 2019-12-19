use human_format::{Formatter, Scales};
use libc;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::iter::FromIterator;
use std::time::Duration;
use sysctl::Sysctl;

#[repr(C)]
struct if_msghdr2 {
    ifm_msglen: u16,     /* to skip over non-understood messages */
    ifm_version: u8,     /* future binary compatability */
    ifm_type: u8,        /* message type */
    ifm_addrs: i32,      /* like rtm_addrs */
    ifm_flags: i32,      /* value of if_flags */
    ifm_index: u16,      /* index for associated ifp */
    ifm_snd_len: i32,    /* instantaneous length of send queue */
    ifm_snd_maxlen: i32, /* maximum length of send queue */
    ifm_snd_drops: i32,  /* number of drops in send queue */
    ifm_timer: i32,      /* time until if_watchdog called */
    ifm_data: if_data64, /* statistics and other data */
}

/*
 * Structure describing information about an interface
 * which may be of interest to management entities.
*/
#[repr(C)]
struct if_data64 {
    /* generic interface information */
    ifi_type: u8,      /* ethernet, tokenring, etc */
    ifi_typelen: u8,   /* Length of frame type id */
    ifi_physical: u8,  /* e.g., AUI, Thinnet, 10base-T, etc */
    ifi_addrlen: u8,   /* media address length */
    ifi_hdrlen: u8,    /* media header length */
    ifi_recvquota: u8, /* polling quota for receive intrs */
    ifi_xmitquota: u8, /* polling quota for xmit intrs */
    ifi_unused1: u8,   /* for future use */
    ifi_mtu: u32,      /* maximum transmission unit */
    ifi_metric: u32,   /* routing metric (external only) */
    ifi_baudrate: u64, /* linespeed */
    /* volatile statistics */
    ifi_ipackets: u64,             /* packets received on interface */
    ifi_ierrors: u64,              /* input errors on interface */
    ifi_opackets: u64,             /* packets sent on interface */
    ifi_oerrors: u64,              /* output errors on interface */
    ifi_collisions: u64,           /* collisions on csma interfaces */
    ifi_ibytes: u64,               /* total number of octets received */
    ifi_obytes: u64,               /* total number of octets sent */
    ifi_imcasts: u64,              /* packets received via multicast */
    ifi_omcasts: u64,              /* packets sent via multicast */
    ifi_iqdrops: u64,              /* dropped on input, this interface */
    ifi_noproto: u64,              /* destined for unsupported protocol */
    ifi_recvtiming: u32,           /* usec spent receiving when timing */
    ifi_xmittiming: u32,           /* usec spent xmitting when timing */
    ifi_lastchange: libc::timeval, /* time of last administrative change */
}

#[cfg(target_os = "macos")]
fn parse_msghdr(data: &[u8], offset: usize) -> (Option<if_msghdr2>, Option<usize>) {
    let if_msghdr_size = std::mem::size_of::<libc::if_msghdr>();
    let if_msghdr2_size = std::mem::size_of::<if_msghdr2>();
    let sval = offset + if_msghdr_size;
    if sval > data.len() {
        return (None, None);
    }
    //let (first, _) = data.split_at(sval);
    let sub = Vec::from_iter(data[offset..sval].iter().cloned());
    let msghdr: libc::if_msghdr = unsafe { std::mem::transmute_copy(&sub[0]) };
    let len: usize = msghdr.ifm_msglen.try_into().unwrap();
    let utype: u8 = libc::RTM_IFINFO2.try_into().unwrap();
    if msghdr.ifm_type == utype {
        let msghdr2 = Vec::from_iter(data[offset..offset + if_msghdr2_size].iter().cloned());
        let x: if_msghdr2 = unsafe { std::mem::transmute_copy(&msghdr2[0]) };
        // dbg!(&x.ifm_data.ifi_type);
        // dbg!(&x.ifm_data.ifi_obytes);
        // dbg!(&x.ifm_data.ifi_ibytes);
        // dbg!(&x.ifm_data.ifi_ipackets);
        // dbg!(&x.ifm_data.ifi_opackets);
        (Some(x), Some(offset + len))
    } else {
        (None, Some(offset + len))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkMetrics {
    pub total_ibytes: u64,
    pub total_obytes: u64,
}

impl NetworkMetrics {
    pub fn diff(&self, old: &NetworkMetrics, dtime: &Duration) -> NetworkMetricRate {
        let formatter = {
            let mut f = Formatter::new();
            f.with_scales(Scales::Binary());
            f.with_units("B");
            f
        };
        let ibyte_rate = (self.total_ibytes as f64 - old.total_ibytes as f64) / dtime.as_secs_f64();
        let obyte_rate = (self.total_obytes as f64 - old.total_obytes as f64) / dtime.as_secs_f64();
        NetworkMetricRate {
            ibyte_rate: format!("{}/s", formatter.format(ibyte_rate)),
            obyte_rate: format!("{}/s", formatter.format(obyte_rate)),
        }
    }
}

pub enum NetworkError {
    IO,
    CtlError,
    GetMetrics { message: String },
    ParseError,
}

#[cfg(target_os = "macos")]
pub fn get_network_metrics() -> Result<NetworkMetrics, NetworkError> {
    let oid: Vec<i32> = vec![libc::CTL_NET, libc::PF_ROUTE, 0, 0, libc::NET_RT_IFLIST2, 0];
    let ctl = sysctl::Ctl { oid };
    let vval = ctl.value().map_err(|_| NetworkError::CtlError)?;
    if let sysctl::CtlValue::Node(nvec) = vval {
        let mut next = Some(0);
        let mut total_ibytes: u64 = 0;
        let mut total_obytes: u64 = 0;
        loop {
            let (h1, n) = parse_msghdr(&nvec, next.unwrap());
            if let Some(h1) = h1 {
                if h1.ifm_data.ifi_type == 6 {
                    total_ibytes += h1.ifm_data.ifi_ibytes;
                    total_obytes += h1.ifm_data.ifi_obytes;
                }
            }
            next = n;
            if n.is_none() {
                break;
            }
        }
        // dbg!(&total_ibytes);
        // dbg!(&total_obytes);
        let metrics = NetworkMetrics {
            total_ibytes,
            total_obytes,
        };
        Ok(metrics)
    } else {
        Err(NetworkError::GetMetrics {
            message: "value retrieved from ctl was not a node".to_string(),
        })
    }
}

#[cfg(target_os = "linux")]
pub fn get_network_metrics() -> Result<NetworkMetrics, NetworkError> {
    let path = "/proc/net/dev";
    let proc = std::fs::read_to_string(path).map_err(|e| NetworkError::IO)?;
    parse_linux_proc_net_dev(&proc)
}

#[derive(Debug)]
pub struct NetworkMetricRate {
    pub ibyte_rate: String,
    pub obyte_rate: String,
}

#[derive(Debug)]
struct LinuxProcNetDevLine {
    iface: String,
    rx_bytes: u64,
    rx_packets: u64,
    rx_errors: u64,
    rx_dropped_missed: u64,
    rx_fifo_errors: u64,
    rx_length_over_ctc_frame_errors: u64,
    rx_compressed: u64,
    multicast: u64,
    tx_bytes: u64,
    tx_packets: u64,
    tx_errors: u64,
    tx_dropped: u64,
    tx_fifo_errors: u64,
    collisions: u64,
    tx_carrier_aborted_window_heartbeat_errors: u64,
    tx_compressed: u64,
}

impl LinuxProcNetDevLine {
    fn new(line: &str) -> Result<LinuxProcNetDevLine, NetworkError> {
        let msg = format!("failed to parse {}", &line);
        let line = line.split_whitespace().collect::<Vec<&str>>();
        let iface = line[0].to_string();
        let line: Vec<u64> = line
            .iter()
            .skip(1)
            .map(|el| el.parse::<u64>())
            .flatten()
            .collect::<Vec<u64>>();
        Ok(LinuxProcNetDevLine {
            iface,
            rx_bytes: line[0],
            rx_packets: line[1],
            rx_errors: line[2],
            rx_dropped_missed: line[3],
            rx_fifo_errors: line[4],
            rx_length_over_ctc_frame_errors: line[5],
            rx_compressed: line[6],
            multicast: line[7],
            tx_bytes: line[8],
            tx_packets: line[9],
            tx_errors: line[10],
            tx_dropped: line[11],
            tx_fifo_errors: line[12],
            collisions: line[13],
            tx_carrier_aborted_window_heartbeat_errors: line[14],
            tx_compressed: line[15],
        })
    }
}

fn parse_linux_proc_net_dev(s: &str) -> Result<NetworkMetrics, NetworkError> {
    let lines = s.lines().skip(1).collect::<Vec<&str>>();
    let lines = lines
        .iter()
        .map({ |line| line.trim() })
        .filter({ |line| line.starts_with("eth") })
        .map({ |line| LinuxProcNetDevLine::new(line) })
        .flatten()
        .collect::<Vec<LinuxProcNetDevLine>>();
    let total_ibytes = lines.iter().map(|line| line.rx_bytes).sum();
    let total_obytes = lines.iter().map(|line| line.tx_bytes).sum();
    Ok(NetworkMetrics {
        total_ibytes,
        total_obytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    static LINUX_PROC_NET_DEV: &str = include_str!("test/linux_proc_net_dev.txt");

    #[test]
    fn test_linux() {
        let m = parse_linux_proc_net_dev(LINUX_PROC_NET_DEV).ok().unwrap();
        assert_eq!(m.total_ibytes, 5610486);
        assert_eq!(m.total_obytes, 81092);
    }
}
