use std::io::Cursor;
use std::io::Read;
use std::net::IpAddr;

#[derive(Debug)]
pub enum BindError {
    Socket(std::io::Error),
    Setsockopt(libc::c_int, std::io::Error),
    Bind(std::io::Error),
}

pub struct LLDPSocket {
    sockfd: i32,
}

impl LLDPSocket {
    pub fn new(if_index: i32) -> Result<Self, BindError> {
        // create a raw packet socket
        let sockfd =
            unsafe { libc::socket(libc::AF_PACKET, libc::SOCK_RAW, (ETH_P_LLDP as i32).to_be()) };
        if sockfd < 0 {
            return Err(BindError::Socket(std::io::Error::last_os_error()));
        }

        // ensure to remove promiscuous mode to receive all packets
        let opt = libc::packet_mreq {
            mr_ifindex: if_index,
            mr_type: libc::PACKET_MR_PROMISC as u16,
            mr_alen: 0,
            mr_address: [0; 8],
        };
        let opt_ptr = &opt as *const libc::packet_mreq as *const libc::c_void;
        let ret = unsafe {
            libc::setsockopt(
                sockfd,
                libc::SOL_PACKET,
                libc::PACKET_DROP_MEMBERSHIP,
                opt_ptr,
                std::mem::size_of::<libc::packet_mreq>() as libc::socklen_t,
            )
        };
        if ret < 0 {
            return Err(BindError::Setsockopt(
                libc::PACKET_DROP_MEMBERSHIP,
                std::io::Error::last_os_error(),
            ));
        }

        // but set the socket option to receive all multicast packets
        let opt = libc::packet_mreq {
            mr_ifindex: if_index,
            mr_type: libc::PACKET_MR_ALLMULTI as u16,
            mr_alen: 0,
            mr_address: [0; 8],
        };
        let opt_ptr = &opt as *const libc::packet_mreq as *const libc::c_void;
        let ret = unsafe {
            libc::setsockopt(
                sockfd,
                libc::SOL_PACKET,
                libc::PACKET_ADD_MEMBERSHIP,
                opt_ptr,
                std::mem::size_of::<libc::packet_mreq>() as libc::socklen_t,
            )
        };
        if ret < 0 {
            return Err(BindError::Setsockopt(
                libc::PACKET_ADD_MEMBERSHIP,
                std::io::Error::last_os_error(),
            ));
        }

        // the socket address that we are going to use when we bind to a specific interface
        // integers are expected to be set in network byte order
        let bind_sa: libc::sockaddr_ll = libc::sockaddr_ll {
            // sll_family: (libc::AF_PACKET as u16).to_be(),
            // sll_protocol: ETH_P_LLDP.to_be(),
            sll_family: (libc::AF_PACKET as u16),
            sll_protocol: ETH_P_LLDP.to_be(),
            sll_ifindex: if_index,         // 0 stands for all interfaces
            sll_hatype: 0,                 // not required (not used?) for bind
            sll_pkttype: PACKET_MULTICAST, // not required (not used?) for bind
            sll_halen: 0,                  // not used for bind
            sll_addr: [0; 8],              // not used for bind
        };
        /*
                   sll_hatype: 0,         // not required (not used?) for bind
           sll_pkttype: PACKET_MULTICAST.to_be(), // not required (not used?) for bind
           sll_halen: 0,          // not used for bind
           sll_addr: [0; 8],      // not used for bind
        */
        let bind_sa_ptr = &bind_sa as *const libc::sockaddr_ll as *const libc::sockaddr;

        // now bind the socket to the specific interface
        let ret = unsafe {
            libc::bind(
                sockfd,
                bind_sa_ptr,
                std::mem::size_of::<libc::sockaddr_ll>() as libc::socklen_t,
            )
        };
        if ret < 0 {
            return Err(BindError::Bind(std::io::Error::last_os_error()));
        }

        Ok(LLDPSocket { sockfd: sockfd })
    }

    pub fn close(self) {
        // close() should never be retried on error
        // so this call is fine like that
        let ret = unsafe { libc::close(self.sockfd) };
        if ret < 0 {
            log::debug!(
                "error closing socket {}: {}",
                self.sockfd,
                std::io::Error::last_os_error()
            );
        }
    }

    pub fn recv_packet(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = [0u8; 65536];
        let ret = unsafe {
            libc::recvfrom(
                self.sockfd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let length = ret as usize;
        Ok(buffer[..length].to_vec())
    }
}

/// from <linux/if_packet.h>
const PACKET_MULTICAST: u8 = 2;
const ETH_P_LLDP: u16 = 0x88cc;

const LLDP_TLV_TYPE_SYSTEM_NAME: u8 = 5;
const LLDP_TLV_TYPE_SYSTEM_DESCRIPTION: u8 = 6;
const LLDP_TLV_TYPE_MGMT_ADDRESS: u8 = 8;
const LLDP_TLV_TYPE_VENDOR_SPECIFIC: u8 = 127;

/// +--------+--------+-------------+
/// |TLV Type|  len   | system name |
/// |   =5   |        |    chars    |
/// |(7 bits)|(9 bits)| (len octet) |
/// +--------+--------+-------------+
///              len  <------------->
pub struct LLDPTLVSystemName {
    pub name: String,
}

impl TryFrom<&LLDPTLV> for LLDPTLVSystemName {
    type Error = String;

    fn try_from(tlv: &LLDPTLV) -> Result<Self, Self::Error> {
        if tlv.typ != LLDP_TLV_TYPE_SYSTEM_NAME {
            return Err("not a system name TLV".to_string());
        }

        let name = String::from_utf8_lossy(&tlv.value).to_string();

        Ok(Self { name })
    }
}

/// +--------+--------+--------------------+
/// |TLV Type|  len   | system description |
/// |   =6   |        |       chars        |
/// |(7 bits)|(9 bits)|    (len octet)     |
/// +--------+--------+--------------------+
///              len  <-------------------->
pub struct LLDPTLVSystemDescription {
    pub description: String,
}

impl TryFrom<&LLDPTLV> for LLDPTLVSystemDescription {
    type Error = String;

    fn try_from(tlv: &LLDPTLV) -> Result<Self, Self::Error> {
        if tlv.typ != LLDP_TLV_TYPE_SYSTEM_DESCRIPTION {
            return Err("not a system description TLV".to_string());
        }

        let description = String::from_utf8_lossy(&tlv.value).to_string();

        Ok(Self { description })
    }
}

/// +--------+--------+---------------+---------------------+-------------+-----------------------------------|
/// |TLV Type|  len   | mgmt addr len |   mgmt addr subtyp  |  mgmt addr  | ... we ignore the interface stuff |
/// |   =8   |        |               | =1 (IPv4) =2 (IPv6) |             | ...                               |
/// |(7 bits)|(9 bits)|   (1 octet)   |     (1 octet)       |(1-31 octets)| ...                               |
/// +--------+--------+---------------+---------------------+-------------+-----------------------------------|
///              len  <--------------------------------------------------------------------------------------->
///                     mgmt addr len <----------------------------------->
///                                            IPv4  4-byte <------------->
///                                            IPv6 16-byte <------------->
///
/// NOTE: this specifically ignores the interface data that comes after the address data because we currently
/// do not need it.
///
#[derive(Debug)]
pub struct LLDPTLVMgmtAddr {
    /// len is the length of the subtype and address bytes together
    pub len: u8,
    /// will be a IANA address family number. For details see: https://www.iana.org/assignments/address-family-numbers/address-family-numbers.xhtml
    pub subtype: u8,
    /// the raw bytes of the management address
    pub bytes: Vec<u8>,
    /// will contain an IPv4 or IPv6 address as long as subtype is 1 or 2
    pub addr: Option<IpAddr>,
}

impl TryFrom<&LLDPTLV> for LLDPTLVMgmtAddr {
    type Error = String;

    fn try_from(tlv: &LLDPTLV) -> Result<Self, Self::Error> {
        if tlv.typ != LLDP_TLV_TYPE_MGMT_ADDRESS {
            return Err("not a management address TLV".to_string());
        }

        if tlv.length < 6 {
            return Err("management address TLV too short".to_string());
        }

        let mut cursor = Cursor::new(&tlv.value);
        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let mgmt_addr_len = buf[0];
        if mgmt_addr_len < 5 {
            return Err("management address TLV has invalid length".to_string());
        }
        cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let mgmt_addr_subtype = buf[0];

        let mut mgmt_addr_bytes = vec![0; (mgmt_addr_len - 1) as usize];
        cursor.read_exact(&mut mgmt_addr_bytes).unwrap();

        let mgmt_addr = match mgmt_addr_subtype {
            1 => {
                if (mgmt_addr_len - 1) != 4 {
                    return Err(format!(
                        "IPv4 management address has wrong length: {mgmt_addr_len}"
                    ));
                }
                match <[u8; 4]>::try_from(mgmt_addr_bytes.as_slice()) {
                    Ok(addr_bytes) => Some(IpAddr::V4(addr_bytes.into())),
                    Err(e) => {
                        return Err(format!(
                            "IPv4 management address has invalid bytes: {}",
                            e.to_string()
                        ))
                    }
                }
            }
            2 => {
                if (mgmt_addr_len - 1) != 16 {
                    return Err("IPv6 management address has wrong length".to_string());
                }
                match <[u8; 16]>::try_from(mgmt_addr_bytes.as_slice()) {
                    Ok(addr_bytes) => Some(IpAddr::V6(addr_bytes.into())),
                    Err(e) => {
                        return Err(format!(
                            "IPv6 management address has invalid bytes: {}",
                            e.to_string()
                        ))
                    }
                }
            }
            _ => None,
        };

        Ok(LLDPTLVMgmtAddr {
            len: mgmt_addr_len,
            subtype: mgmt_addr_subtype,
            bytes: mgmt_addr_bytes,
            addr: mgmt_addr,
        })
    }
}

/// +--------+--------+----------+---------+---------------+
/// |TLV Type|  len   |   OUI    | subtype |     data     |
/// |  =127  |        |          |         |              |
/// |(7 bits)|(9 bits)|(3 octets)|(1 octet)|(0-507 octets)|
/// +--------+--------+----------+---------+---------------+
pub struct LLDPTLVOrgSpecific {
    pub oui: [u8; 3],
    pub subtype: u8,
    pub bytes: Vec<u8>,
}

impl TryFrom<&LLDPTLV> for LLDPTLVOrgSpecific {
    type Error = String;

    fn try_from(tlv: &LLDPTLV) -> Result<Self, Self::Error> {
        if tlv.typ != LLDP_TLV_TYPE_VENDOR_SPECIFIC {
            return Err("not a vendor-specific TLV".to_string());
        }

        if tlv.length < 4 {
            return Err("vendor-specific TLV too short".to_string());
        }

        let mut cursor = Cursor::new(&tlv.value);
        let mut buf = [0u8; 3];
        cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let oui = buf;

        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let subtype = buf[0];

        let mut bytes = vec![0; (tlv.length - 4) as usize];
        cursor.read_exact(&mut bytes).unwrap();

        Ok(LLDPTLVOrgSpecific {
            oui,
            subtype,
            bytes,
        })
    }
}

/// See section 12 of RFC 8520.
/// +--------+--------+----------+---------+---------------+
/// |TLV Type|  len   |   OUI    |subtype  | MUDString     |
/// |  =127  |        |= 00 00 5E|  = 1    |               |
/// |(7 bits)|(9 bits)|(3 octets)|(1 octet)|(1-255 octets) |
/// +--------+--------+----------+---------+---------------+
/// where:
///
/// o  TLV Type = 127 indicates a vendor-specific TLV
/// o  len = indicates the TLV string length
/// o  OUI = 00 00 5E is the organizationally unique identifier of IANA
/// o  subtype = 1 (as assigned by IANA for the MUDstring)
/// o  MUDstring = the length MUST NOT exceed 255 octets
///
/// This parsing information is shamelessly stolen from systemd at:
/// https://github.com/systemd/systemd/blob/main/src/libsystemd-network/sd-lldp-tx.c#L415
///
pub struct LLDPTLVMUDString {
    pub mud_string: String,
}

impl TryFrom<&LLDPTLVOrgSpecific> for LLDPTLVMUDString {
    type Error = String;

    fn try_from(tlv: &LLDPTLVOrgSpecific) -> Result<Self, Self::Error> {
        if tlv.oui != [0, 0, 0x5e] {
            return Err("vendor-specific TLV is not the IANA OUI".to_string());
        }

        if tlv.subtype != 1 {
            return Err("vendor-specific TLV is not the MUDString subtype".to_string());
        }

        let mud_string = String::from_utf8_lossy(&tlv.bytes).to_string();

        Ok(LLDPTLVMUDString { mud_string })
    }
}

impl TryFrom<LLDPTLVOrgSpecific> for LLDPTLVMUDString {
    type Error = String;

    fn try_from(tlv: LLDPTLVOrgSpecific) -> Result<Self, Self::Error> {
        TryFrom::try_from(&tlv)
    }
}

impl TryFrom<&LLDPTLV> for LLDPTLVMUDString {
    type Error = String;

    fn try_from(tlv: &LLDPTLV) -> Result<Self, Self::Error> {
        if tlv.typ != LLDP_TLV_TYPE_VENDOR_SPECIFIC {
            return Err("not a vendor-specific TLV".to_string());
        }

        if tlv.length < 5 {
            return Err("vendor-specific TLV too short".to_string());
        }

        let mut cursor = Cursor::new(&tlv.value);
        let mut buf = [0u8; 3];
        cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let oui = buf;
        if oui != [0, 0, 0x5e] {
            return Err("vendor-specific TLV is not the IANA OUI".to_string());
        }

        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let subtype = buf[0];
        if subtype != 1 {
            return Err("vendor-specific TLV is not the MUDString subtype".to_string());
        }

        let mut bytes = vec![0; (tlv.length - 4) as usize];
        cursor.read_exact(&mut bytes).unwrap();
        let mud_string = String::from_utf8_lossy(&bytes).to_string();

        Ok(LLDPTLVMUDString { mud_string })
    }
}

/// All LLDP TLVs follow the same format:
/// - 7 bits for the type
/// - 9 bits for the length
/// - the value as "length" number of bytes
#[derive(Debug)]
pub struct LLDPTLV {
    pub typ: u8,
    pub length: u16,
    pub value: Vec<u8>,
}

impl LLDPTLV {
    fn parse(cursor: &mut Cursor<&[u8]>) -> Option<LLDPTLV> {
        // read 2 bytes as an unsigned 16-bit integer in network byte order (big endian)
        let mut buf = [0u8; 2];
        cursor.read_exact(&mut buf).ok()?;
        let typ_and_length = u16::from_be_bytes(buf);

        // the first 7 bits are the type, the last 9 bits are the length
        let typ = ((typ_and_length >> 9) & 0b0111_1111) as u8;
        let length = (typ_and_length & 0b0000_0001_1111_1111) as u16;

        let mut value = vec![0; length as usize];
        cursor.read_exact(&mut value).ok()?;

        Some(LLDPTLV { typ, length, value })
    }
}

pub fn parse_lldp(data: &[u8]) -> Vec<LLDPTLV> {
    let mut cursor = Cursor::new(data);
    let mut tlvs = Vec::new();

    while let Some(tlv) = LLDPTLV::parse(&mut cursor) {
        tlvs.push(tlv);
    }
    tlvs
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACKET_SWITCH_1: [u8; 405] = [
        0x01, 0x80, 0xc2, 0x00, 0x00, 0x0e, 0x54, 0xbf, 0x64, 0xba, 0x3d, 0xc2, 0x88, 0xcc, 0x02,
        0x07, 0x04, 0x54, 0xbf, 0x64, 0xba, 0x3d, 0xc0, 0x04, 0x0b, 0x07, 0x45, 0x74, 0x68, 0x65,
        0x72, 0x6e, 0x65, 0x74, 0x35, 0x31, 0x06, 0x02, 0x00, 0x78, 0x0a, 0x08, 0x73, 0x77, 0x69,
        0x74, 0x63, 0x68, 0x2d, 0x31, 0x0c, 0x8d, 0x53, 0x4f, 0x4e, 0x69, 0x43, 0x20, 0x53, 0x6f,
        0x66, 0x74, 0x77, 0x61, 0x72, 0x65, 0x20, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x3a,
        0x20, 0x53, 0x4f, 0x4e, 0x69, 0x43, 0x2e, 0x34, 0x2e, 0x31, 0x2e, 0x31, 0x2d, 0x45, 0x6e,
        0x74, 0x65, 0x72, 0x70, 0x72, 0x69, 0x73, 0x65, 0x5f, 0x42, 0x61, 0x73, 0x65, 0x20, 0x2d,
        0x20, 0x48, 0x77, 0x53, 0x6b, 0x75, 0x3a, 0x20, 0x44, 0x65, 0x6c, 0x6c, 0x45, 0x4d, 0x43,
        0x2d, 0x53, 0x35, 0x32, 0x34, 0x38, 0x66, 0x2d, 0x50, 0x2d, 0x32, 0x35, 0x47, 0x2d, 0x44,
        0x50, 0x42, 0x20, 0x2d, 0x20, 0x44, 0x69, 0x73, 0x74, 0x72, 0x69, 0x62, 0x75, 0x74, 0x69,
        0x6f, 0x6e, 0x3a, 0x20, 0x44, 0x65, 0x62, 0x69, 0x61, 0x6e, 0x20, 0x31, 0x30, 0x2e, 0x31,
        0x33, 0x20, 0x2d, 0x20, 0x4b, 0x65, 0x72, 0x6e, 0x65, 0x6c, 0x3a, 0x20, 0x35, 0x2e, 0x31,
        0x30, 0x2e, 0x30, 0x2d, 0x38, 0x2d, 0x32, 0x2d, 0x61, 0x6d, 0x64, 0x36, 0x34, 0x0e, 0x04,
        0x00, 0x9c, 0x00, 0x10, 0x10, 0x0c, 0x05, 0x01, 0xc0, 0xa8, 0x65, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x08, 0x21, 0x4d, 0x43, 0x4c, 0x41, 0x47, 0x20, 0x73, 0x65, 0x73, 0x73,
        0x69, 0x6f, 0x6e, 0x20, 0x6c, 0x69, 0x6e, 0x6b, 0x20, 0x50, 0x6f, 0x72, 0x74, 0x43, 0x68,
        0x61, 0x6e, 0x6e, 0x65, 0x6c, 0x32, 0x35, 0x31, 0xfe, 0x09, 0x00, 0x12, 0x0f, 0x03, 0x01,
        0x00, 0x00, 0x00, 0x00, 0xfe, 0x09, 0x00, 0x12, 0x0f, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
        0xfe, 0x07, 0x00, 0x12, 0xbb, 0x01, 0x00, 0x3f, 0x04, 0xfe, 0x0f, 0x00, 0x12, 0xbb, 0x05,
        0x33, 0x2e, 0x34, 0x30, 0x2e, 0x30, 0x2e, 0x39, 0x2d, 0x31, 0x31, 0xfe, 0x0f, 0x00, 0x12,
        0xbb, 0x06, 0x33, 0x2e, 0x34, 0x30, 0x2e, 0x30, 0x2e, 0x39, 0x2d, 0x31, 0x31, 0xfe, 0x14,
        0x00, 0x12, 0xbb, 0x07, 0x35, 0x2e, 0x31, 0x30, 0x2e, 0x30, 0x2d, 0x38, 0x2d, 0x32, 0x2d,
        0x61, 0x6d, 0x64, 0x36, 0x34, 0xfe, 0x18, 0x00, 0x12, 0xbb, 0x08, 0x43, 0x4e, 0x30, 0x34,
        0x36, 0x4d, 0x52, 0x4a, 0x43, 0x45, 0x53, 0x30, 0x30, 0x38, 0x35, 0x45, 0x30, 0x30, 0x31,
        0x35, 0xfe, 0x0c, 0x00, 0x12, 0xbb, 0x09, 0x44, 0x65, 0x6c, 0x6c, 0x20, 0x45, 0x4d, 0x43,
        0xfe, 0x0d, 0x00, 0x12, 0xbb, 0x0a, 0x53, 0x35, 0x32, 0x34, 0x38, 0x46, 0x2d, 0x4f, 0x4e,
        0xfe, 0x0b, 0x00, 0x12, 0xbb, 0x0b, 0x47, 0x43, 0x4e, 0x53, 0x47, 0x30, 0x32, 0x00, 0x00,
    ];

    const PACKET_SWITCH_2: [u8; 300] = [
        0x01, 0x80, 0xc2, 0x00, 0x00, 0x0e, 0x3c, 0x2c, 0x30, 0x66, 0xf0, 0x02, 0x88, 0xcc, 0x02,
        0x07, 0x04, 0x3c, 0x2c, 0x30, 0x66, 0xf0, 0x00, 0x04, 0x0b, 0x07, 0x45, 0x74, 0x68, 0x65,
        0x72, 0x6e, 0x65, 0x74, 0x35, 0x31, 0x06, 0x02, 0x00, 0x14, 0x0a, 0x08, 0x73, 0x77, 0x69,
        0x74, 0x63, 0x68, 0x2d, 0x32, 0x0c, 0x24, 0x68, 0x65, 0x64, 0x67, 0x65, 0x68, 0x6f, 0x67,
        0x2d, 0x66, 0x61, 0x62, 0x72, 0x69, 0x63, 0x2d, 0x6d, 0x61, 0x67, 0x69, 0x63, 0x2d, 0x66,
        0x2a, 0x63, 0x6b, 0x69, 0x6e, 0x67, 0x2d, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x0e, 0x04,
        0x00, 0x9c, 0x00, 0x10, 0x10, 0x0c, 0x05, 0x01, 0xc0, 0xa8, 0x66, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x08, 0x21, 0x4d, 0x43, 0x4c, 0x41, 0x47, 0x20, 0x73, 0x65, 0x73, 0x73,
        0x69, 0x6f, 0x6e, 0x20, 0x6c, 0x69, 0x6e, 0x6b, 0x20, 0x50, 0x6f, 0x72, 0x74, 0x43, 0x68,
        0x61, 0x6e, 0x6e, 0x65, 0x6c, 0x32, 0x35, 0x31, 0xfe, 0x09, 0x00, 0x12, 0x0f, 0x03, 0x03,
        0x00, 0x00, 0x00, 0x43, 0xfe, 0x09, 0x00, 0x12, 0x0f, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
        0xfe, 0x07, 0x00, 0x12, 0xbb, 0x01, 0x00, 0x3f, 0x04, 0xfe, 0x0f, 0x00, 0x12, 0xbb, 0x05,
        0x33, 0x2e, 0x34, 0x30, 0x2e, 0x30, 0x2e, 0x39, 0x2d, 0x31, 0x35, 0xfe, 0x0f, 0x00, 0x12,
        0xbb, 0x06, 0x33, 0x2e, 0x34, 0x30, 0x2e, 0x30, 0x2e, 0x39, 0x2d, 0x31, 0x35, 0xfe, 0x14,
        0x00, 0x12, 0xbb, 0x07, 0x35, 0x2e, 0x31, 0x30, 0x2e, 0x30, 0x2d, 0x38, 0x2d, 0x32, 0x2d,
        0x61, 0x6d, 0x64, 0x36, 0x34, 0xfe, 0x18, 0x00, 0x12, 0xbb, 0x08, 0x43, 0x4e, 0x30, 0x4e,
        0x57, 0x34, 0x32, 0x4d, 0x43, 0x45, 0x53, 0x30, 0x30, 0x39, 0x33, 0x50, 0x30, 0x30, 0x32,
        0x35, 0xfe, 0x0c, 0x00, 0x12, 0xbb, 0x09, 0x44, 0x65, 0x6c, 0x6c, 0x20, 0x45, 0x4d, 0x43,
        0xfe, 0x0d, 0x00, 0x12, 0xbb, 0x0a, 0x53, 0x35, 0x32, 0x34, 0x38, 0x46, 0x2d, 0x4f, 0x4e,
        0xfe, 0x0b, 0x00, 0x12, 0xbb, 0x0b, 0x47, 0x47, 0x57, 0x59, 0x5a, 0x50, 0x32, 0x00, 0x00,
    ];

    const PACKET_CONTROL_1: [u8; 156] = [
        0x01, 0x80, 0xc2, 0x00, 0x00, 0x0e, 0x0c, 0x20, 0x12, 0xfe, 0x00, 0x01, 0x88, 0xcc, 0x02,
        0x21, 0x07, 0x39, 0x33, 0x61, 0x65, 0x38, 0x36, 0x64, 0x39, 0x30, 0x38, 0x61, 0x36, 0x34,
        0x64, 0x62, 0x34, 0x38, 0x34, 0x66, 0x61, 0x38, 0x64, 0x62, 0x34, 0x36, 0x35, 0x66, 0x32,
        0x61, 0x63, 0x64, 0x36, 0x04, 0x07, 0x05, 0x65, 0x6e, 0x70, 0x30, 0x73, 0x33, 0x06, 0x02,
        0x00, 0x78, 0x0a, 0x09, 0x63, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c, 0x2d, 0x31, 0xfe, 0x49,
        0x00, 0x00, 0x5e, 0x01, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x31, 0x39, 0x32, 0x2e,
        0x31, 0x36, 0x38, 0x2e, 0x34, 0x32, 0x2e, 0x31, 0x2f, 0x3f, 0x6d, 0x79, 0x3d, 0x31, 0x39,
        0x32, 0x25, 0x32, 0x45, 0x31, 0x36, 0x38, 0x25, 0x32, 0x45, 0x31, 0x30, 0x31, 0x25, 0x32,
        0x45, 0x30, 0x26, 0x79, 0x6f, 0x75, 0x72, 0x73, 0x3d, 0x31, 0x39, 0x32, 0x25, 0x32, 0x45,
        0x31, 0x36, 0x38, 0x25, 0x32, 0x45, 0x31, 0x30, 0x31, 0x25, 0x32, 0x45, 0x31, 0x0e, 0x04,
        0x00, 0x94, 0x00, 0x10, 0x00, 0x00,
    ];

    #[test]
    fn test_packet_switch_1() {
        // parse ethernet frame for switch-2 LLDP packet
        let a = etherparse::SlicedPacket::from_ethernet(&PACKET_SWITCH_1).unwrap();
        // the parsing worked, so we know that link must be Some, so unwrap is safe to call here
        let b = a.link.unwrap().to_header();
        assert_eq!(b.ether_type, ETH_P_LLDP);
        println!("{:x}", b.ether_type);

        // parse LLDP frame as LLDP TLVs
        let ret = parse_lldp(a.payload);
        println!("{:?}", ret);

        let mut found_mgmt_address = false;
        let mut found_system_name = false;
        let mut found_system_descr = false;
        for tlv in ret {
            // ensure the system name is what we expect
            if tlv.typ == LLDP_TLV_TYPE_SYSTEM_NAME {
                let system_name = LLDPTLVSystemName::try_from(&tlv).unwrap();
                println!("System Name: {}", system_name.name);
                assert_eq!(system_name.name, "switch-1".to_string());
                found_system_name = true;
            }

            // ensure the system description is what we expect
            if tlv.typ == LLDP_TLV_TYPE_SYSTEM_DESCRIPTION {
                let system_descr = LLDPTLVSystemDescription::try_from(&tlv).unwrap();
                println!("System Description: {}", system_descr.description);
                assert_eq!(
                    system_descr.description,
                    "SONiC Software Version: SONiC.4.1.1-Enterprise_Base - HwSku: DellEMC-S5248f-P-25G-DPB - Distribution: Debian 10.13 - Kernel: 5.10.0-8-2-amd64".to_string()
                );
                found_system_descr = true;
            }

            // ensure the mgmt address is what we expect
            if tlv.typ == LLDP_TLV_TYPE_MGMT_ADDRESS {
                let mgmt_addr = LLDPTLVMgmtAddr::try_from(&tlv).unwrap();
                let addr = mgmt_addr.addr.unwrap();
                println!("Management Address: {}", addr);
                assert_eq!(addr, IpAddr::V4([192, 168, 101, 0].into()));
                found_mgmt_address = true;
            }
        }
        assert!(found_mgmt_address);
        assert!(found_system_name);
        assert!(found_system_descr);
    }

    #[test]
    fn test_packet_switch_2() {
        // parse ethernet frame for switch-2 LLDP packet
        let a = etherparse::SlicedPacket::from_ethernet(&PACKET_SWITCH_2).unwrap();
        // the parsing worked, so we know that link must be Some, so unwrap is safe to call here
        let b = a.link.unwrap().to_header();
        assert_eq!(b.ether_type, ETH_P_LLDP);
        println!("{:x}", b.ether_type);

        // parse LLDP frame as LLDP TLVs
        let ret = parse_lldp(a.payload);
        println!("{:?}", ret);

        let mut found_mgmt_address = false;
        let mut found_system_name = false;
        let mut found_system_descr = false;
        for tlv in ret {
            // ensure the system name is what we expect
            if tlv.typ == LLDP_TLV_TYPE_SYSTEM_NAME {
                let system_name = LLDPTLVSystemName::try_from(&tlv).unwrap();
                println!("System Name: {}", system_name.name);
                assert_eq!(system_name.name, "switch-2".to_string());
                found_system_name = true;
            }

            // ensure the system description is what we expect
            if tlv.typ == LLDP_TLV_TYPE_SYSTEM_DESCRIPTION {
                let system_descr = LLDPTLVSystemDescription::try_from(&tlv).unwrap();
                println!("System Description: {}", system_descr.description);
                assert_eq!(
                    system_descr.description,
                    "hedgehog-fabric-magic-f*cking-string".to_string()
                );
                found_system_descr = true;
            }

            // ensure the mgmt address is what we expect
            if tlv.typ == LLDP_TLV_TYPE_MGMT_ADDRESS {
                let mgmt_addr = LLDPTLVMgmtAddr::try_from(&tlv).unwrap();
                let addr = mgmt_addr.addr.unwrap();
                println!("Management Address: {}", addr);
                assert_eq!(addr, IpAddr::V4([192, 168, 102, 0].into()));
                found_mgmt_address = true;
            }
        }
        assert!(found_mgmt_address);
        assert!(found_system_name);
        assert!(found_system_descr);
    }

    #[test]
    fn test_packet_control_1() {
        // parse ethernet frame for switch-2 LLDP packet
        let a = etherparse::SlicedPacket::from_ethernet(&PACKET_CONTROL_1).unwrap();
        // the parsing worked, so we know that link must be Some, so unwrap is safe to call here
        let b = a.link.unwrap().to_header();
        assert_eq!(b.ether_type, ETH_P_LLDP);
        println!("{:x}", b.ether_type);

        // parse LLDP frame as LLDP TLVs
        let ret = parse_lldp(a.payload);
        println!("{:?}", ret);

        let mut found_system_name = false;
        let mut found_mud_url = false;
        for tlv in ret {
            // ensure the system name is what we expect
            if tlv.typ == LLDP_TLV_TYPE_SYSTEM_NAME {
                let system_name = LLDPTLVSystemName::try_from(&tlv).unwrap();
                println!("System Name: {}", system_name.name);
                assert_eq!(system_name.name, "control-1".to_string());
                found_system_name = true;
            }

            // ensure our MUD URL is what we expect
            if tlv.typ == LLDP_TLV_TYPE_VENDOR_SPECIFIC {
                let vendor_tlv = LLDPTLVOrgSpecific::try_from(&tlv).unwrap();
                if let Ok(mud_string) = LLDPTLVMUDString::try_from(&vendor_tlv) {
                    println!("MUD String: {}", mud_string.mud_string);
                    assert_eq!(
                        mud_string.mud_string,
                        "http://192.168.42.1/?my=192%2E168%2E101%2E0&yours=192%2E168%2E101%2E1"
                            .to_string()
                    );
                    found_mud_url = true;
                }
            }
        }
        assert!(found_system_name);
        assert!(found_mud_url);
    }
}
