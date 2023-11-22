use libc::RTMGRP_LINK;
use netlink_packet_core::constants::NLM_F_ACK;
use netlink_packet_core::constants::NLM_F_REQUEST;
use netlink_packet_core::NetlinkHeader;
use netlink_packet_core::NetlinkMessage;
use netlink_packet_core::NetlinkPayload;
use netlink_packet_route::LinkMessage;
use netlink_packet_route::RtnlMessage;
use netlink_packet_utils::DecodeError;
use netlink_sys::protocols::NETLINK_ROUTE;
use netlink_sys::Socket;
use netlink_sys::SocketAddr;
use std::ffi::CStr;
use std::sync::mpsc::Sender;
use std::thread;

use super::ProcessRequest;

pub(crate) fn netlink_link_monitor(
    proc_tx: Sender<ProcessRequest>,
) -> Result<thread::JoinHandle<()>, std::io::Error> {
    let mut socket = Socket::new(NETLINK_ROUTE)?;
    let addr = SocketAddr::new(0, RTMGRP_LINK as u32);
    socket.bind(&addr)?;

    let mut buf = vec![0; 8192];
    let mut off = 0;

    Ok(thread::spawn(move || {
        loop {
            let size = match socket.recv(&mut &mut buf[..], 0) {
                Ok(v) => v,
                Err(e) => {
                    log::error!(
                        "netlink link monitor: failed to receive from netlink socket: {e:?}"
                    );
                    continue;
                }
            };

            // there is no guarantee that a single receive call gives us only one netlink message
            // so we need to loop and try to deserialize multiple messages
            loop {
                let packet: NetlinkMessage<RtnlMessage> = match NetlinkMessage::deserialize(
                    &buf[off..],
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("netlink link monitor: received invalid netlink message, failed to deserialize: {e:?}");
                        break;
                    }
                };

                match packet.payload {
                    NetlinkPayload::InnerMessage(RtnlMessage::SetLink(v)) => {
                        let _ = proc_tx.send(ProcessRequest::NetlinkLinkChanged(convert(v)));
                    }
                    NetlinkPayload::InnerMessage(RtnlMessage::NewLink(v)) => {
                        let _ = proc_tx.send(ProcessRequest::NetlinkLinkChanged(convert(v)));
                    }
                    NetlinkPayload::InnerMessage(RtnlMessage::DelLink(v)) => {
                        let _ = proc_tx.send(ProcessRequest::NetlinkLinkRemoved(v.header.index));
                    }
                    NetlinkPayload::Error(e) => {
                        log::error!(
                        "netlink link monitor: received error message from netlink socket: {e:?}"
                    );
                    }
                    v => {
                        log::warn!("netlink link monitor: received unexpected message from netlink socket: {v:?}");
                    }
                }

                off += packet.header.length as usize;
                if off == size || packet.header.length == 0 {
                    off = 0;
                    break;
                }
            }
        }
    }))
}

fn convert(lm: LinkMessage) -> (u32, bool) {
    if lm.header.flags & libc::IFF_UP as u32 != 0 {
        (lm.header.index, true)
    } else {
        (lm.header.index, false)
    }
}

pub(crate) fn get_interface_name(index: u32) -> Result<String, std::io::Error> {
    let mut buf = [0u8; libc::IFNAMSIZ];
    let ret = unsafe { libc::if_indextoname(index, buf.as_mut_ptr() as *mut libc::c_char) };
    if ret.is_null() {
        return Err(std::io::Error::last_os_error());
    }
    let name = unsafe { CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
    Ok(name.to_string_lossy().into_owned())
}

pub(crate) fn get_interface_index(name: &str) -> Result<u32, std::io::Error> {
    let name = std::ffi::CString::new(name).unwrap();
    let ret = unsafe { libc::if_nametoindex(name.as_ptr()) };
    if ret <= 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(ret)
}

pub(crate) fn get_interfaces() -> Result<Vec<(u32, String)>, std::io::Error> {
    let mut ptr = unsafe { libc::if_nameindex() };
    if ptr.is_null() {
        return Err(std::io::Error::last_os_error());
    }
    let mut ret: Vec<(u32, String)> = Vec::new();
    loop {
        let if_nameindex = unsafe { *ptr };
        if if_nameindex.if_index == 0 {
            break;
        }
        let name = unsafe { CStr::from_ptr(if_nameindex.if_name) };
        let name = name.to_string_lossy().into_owned();
        ret.push((if_nameindex.if_index, name));
        ptr = unsafe { ptr.offset(1) };
    }

    // // iterate over the returned list and build the vector for return
    // let count = unsafe { attr.value.u32list.count };
    // let list = unsafe { attr.value.u32list.list };
    // let mut ret: Vec<u32> = Vec::with_capacity(count as usize);
    // for i in 0..count {
    //     let speed: u32 = unsafe { *list.offset(i as isize) };
    //     ret.push(speed);
    // }
    Ok(ret)
}

#[derive(Debug)]
pub(crate) enum SetLinkError {
    IOError(std::io::Error),
    NetlinkDecodeError(DecodeError),
    UnexpectedNetlinkMessage(u16),
    NetlinkError(i32),
}

impl std::error::Error for SetLinkError {}

impl std::fmt::Display for SetLinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetLinkError::IOError(e) => write!(f, "IO error: {}", e),
            SetLinkError::NetlinkDecodeError(e) => write!(f, "netlink decoding error: {}", e),
            SetLinkError::UnexpectedNetlinkMessage(v) => {
                write!(f, "unexpected netlink message received: {}", v)
            }
            SetLinkError::NetlinkError(v) => write!(f, "netlink error: {}", v),
        }
    }
}

impl From<std::io::Error> for SetLinkError {
    fn from(e: std::io::Error) -> Self {
        SetLinkError::IOError(e)
    }
}

impl From<DecodeError> for SetLinkError {
    fn from(e: DecodeError) -> Self {
        SetLinkError::NetlinkDecodeError(e)
    }
}

#[allow(dead_code)]
pub(crate) fn set_link_status(index: u32, oper_status: bool) -> Result<(), SetLinkError> {
    let mut socket = Socket::new(NETLINK_ROUTE)?;
    let sock_addr = socket.bind_auto()?;
    let port_number = sock_addr.port_number();
    socket.connect(&SocketAddr::new(0, 0))?;

    // GET LINK - get the link first before we modify it
    // build request
    let mut hdr = NetlinkHeader::default();
    hdr.flags = NLM_F_REQUEST;
    hdr.port_number = port_number;
    let mut lm = LinkMessage::default();
    lm.header.index = index;
    let mut req = NetlinkMessage::new(hdr, NetlinkPayload::from(RtnlMessage::GetLink(lm)));
    req.finalize();

    // serialize the request
    let mut buf = vec![0u8; req.header.length as usize];
    req.serialize(buf.as_mut_slice());

    // send the request
    let _ = socket.send(buf.as_slice(), 0)?;

    // receive the response
    let mut buf = vec![0u8; 8192];
    let _ = socket.recv(&mut &mut buf[..], 0)?;

    // deserialize the response
    // we are expecting an NLMSG_ERROR message without an error (which equals an ACK)
    let resp = <NetlinkMessage<RtnlMessage>>::deserialize(&buf.as_slice())?;
    let current_lm = match resp.payload {
        NetlinkPayload::InnerMessage(RtnlMessage::GetLink(v)) => v,
        NetlinkPayload::InnerMessage(RtnlMessage::NewLink(v)) => v,
        v => {
            println!("{v:?}");
            return Err(SetLinkError::UnexpectedNetlinkMessage(v.message_type()));
        }
    };

    // SET LINK - modify the link now
    // build our netlink request
    // we take the current link message and modify the flags
    // NOTE: clone() does not work, it will panic during serialization
    let mut hdr = NetlinkHeader::default();
    hdr.flags = NLM_F_REQUEST | NLM_F_ACK;
    hdr.port_number = port_number;
    let mut lm = LinkMessage::default();
    lm.header.interface_family = current_lm.header.interface_family;
    lm.header.index = index;
    lm.header.link_layer_type = current_lm.header.link_layer_type;
    lm.header.flags = current_lm.header.flags;
    lm.header.change_mask = current_lm.header.change_mask;
    if oper_status {
        lm.header.flags |= libc::IFF_UP as u32;
    } else {
        lm.header.flags &= !(libc::IFF_UP as u32);
    }
    let mut req = NetlinkMessage::new(hdr, NetlinkPayload::from(RtnlMessage::SetLink(lm)));
    req.finalize();

    // serialize the request
    let mut buf = vec![0u8; req.header.length as usize];
    req.serialize(buf.as_mut_slice());

    // send the request
    let _ = socket.send(buf.as_slice(), 0)?;

    // receive the response
    let mut buf = vec![0u8; 4096];
    let _ = socket.recv(&mut &mut buf[..], 0)?;

    // deserialize the response
    // we are expecting an NLMSG_ERROR message without an error (which equals an ACK)
    let resp = <NetlinkMessage<RtnlMessage>>::deserialize(&buf.as_slice())?;
    match resp.payload {
        NetlinkPayload::Error(err_msg) => match err_msg.code {
            None => Ok(()),
            Some(code) => Err(SetLinkError::NetlinkError(code.into())),
        },
        v => Err(SetLinkError::UnexpectedNetlinkMessage(v.message_type())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_interfaces() {
        let interfaces = get_interfaces().unwrap();
        println!("{:#?}", interfaces);
    }
}
