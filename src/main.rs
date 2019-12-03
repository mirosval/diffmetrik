use libc;
use std::convert::TryInto;
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

fn main() {
    let oid: Vec<i32> = vec![libc::CTL_NET, libc::PF_ROUTE, 0, 0, libc::NET_RT_IFLIST2, 0];
    let ctl = sysctl::Ctl { oid };

    dbg!(ctl.value_string().expect("aaa"));
    dbg!(ctl.info().expect("aaa"));
    let vval = ctl.value().expect("aaa");
    dbg!(&vval);
    let sval = std::mem::size_of::<libc::if_msghdr>();
    dbg!(&sval);
    if let sysctl::CtlValue::Node(nvec) = vval {
        let (aaa, bbb) = nvec[..sval].split_at(sval);
        dbg!(&aaa);
        let x: libc::if_msghdr = unsafe { std::mem::transmute_copy(&aaa[0]) };
        dbg!(&x.ifm_msglen);
        dbg!(x.ifm_type == libc::RTM_IFINFO2.try_into().unwrap());

        if x.ifm_type == libc::RTM_IFINFO2.try_into().unwrap() {
            let (aaa, bbb) = nvec.split_at(std::mem::size_of::<if_msghdr2>());
            dbg!(&aaa);
            let x: if_msghdr2 = unsafe { std::mem::transmute_copy(&aaa[0]) };
            dbg!(&x.ifm_data.ifi_type);
            dbg!(&x.ifm_data.ifi_obytes);
            dbg!(&x.ifm_data.ifi_ibytes);
            dbg!(&x.ifm_data.ifi_ipackets);
            dbg!(&x.ifm_data.ifi_opackets);
        }
    }
}
