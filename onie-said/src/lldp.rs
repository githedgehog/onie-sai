use std::io::Cursor;
use std::io::Read;
use std::net::IpAddr;
use std::str::FromStr;

use etherparse::ReadError;
use ipnet::IpNet;

#[derive(Debug)]
pub enum BindError {
    Socket(std::io::Error),
    Setsockopt(libc::c_int, std::io::Error),
    Bind(std::io::Error),
}

#[derive(Debug)]
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

    #[allow(dead_code)]
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

    /// this should only be used when consuming the socket is not possible
    /// which can be the case when this value is sitting in an Arc for
    pub fn ref_close(&self) {
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

const IANA_ADDRESS_FAMILY_NUMBER_IP: u8 = 1; // IP (IP version 4)
#[allow(dead_code)]
const IANA_ADDRESS_FAMILY_NUMBER_IP6: u8 = 2; // IP6 (IP version 6)

const LLDP_TLV_TYPE_SYSTEM_NAME: u8 = 5;
const LLDP_TLV_TYPE_SYSTEM_DESCRIPTION: u8 = 6;
const LLDP_TLV_TYPE_MGMT_ADDRESS: u8 = 8;
const LLDP_TLV_TYPE_ORGANIZATION_SPECIFIC: u8 = 127;

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
    pub hh_kv_pairs: Option<Vec<(String, String)>>,
}

impl TryFrom<&LLDPTLV> for LLDPTLVSystemDescription {
    type Error = String;

    fn try_from(tlv: &LLDPTLV) -> Result<Self, Self::Error> {
        if tlv.typ != LLDP_TLV_TYPE_SYSTEM_DESCRIPTION {
            return Err("not a system description TLV".to_string());
        }

        let description = String::from_utf8_lossy(&tlv.value).to_string();

        Ok(Self::new(description))
    }
}

impl LLDPTLVSystemDescription {
    pub fn new(description: String) -> Self {
        let hh_kv_pairs = Self::get_hh_key_value_pairs(&description);
        Self {
            description,
            hh_kv_pairs: hh_kv_pairs,
        }
    }

    fn get_hh_key_value_pairs(descr: &str) -> Option<Vec<(String, String)>> {
        if descr.starts_with("Hedgehog:") {
            // strip the Hedgehog: prefix, and remove all whitespaces around it
            let a = descr.strip_prefix("Hedgehog:").unwrap().trim().to_string();

            // strip the [] around the string if it exists
            let a = if let Some(a) = a.strip_prefix('[') {
                a.to_string()
            } else {
                a
            };
            let a = if let Some(a) = a.strip_suffix(']') {
                a.to_string()
            } else {
                a
            };

            // now let's split by "," and trim whitespaces again
            let a = a
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            // and last but not least, split all entries into key value pairs by splitting on the first "="
            // and trim whitespaces around it again
            // we'll ignore all entries which are not key value pairs
            let a = a
                .iter()
                .flat_map(|s| {
                    s.split_once('=')
                        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                })
                .collect::<Vec<(String, String)>>();

            return Some(a);
        }
        None
    }

    pub fn get_hh_control_vip(&self) -> Option<IpNet> {
        if let Some(hh_kv_pairs) = &self.hh_kv_pairs {
            for (k, v) in hh_kv_pairs {
                if k == "control_vip" {
                    return IpNet::from_str(v).ok();
                }
            }
        }
        None
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

impl LLDPTLVMgmtAddr {
    /// As long as this is a Hedgehog IP discovery
    /// we assume that the gateway address is the management IP address
    /// NOTE: We only know if it is, if we have a system description TLV
    /// which indicates that it is
    pub fn get_hh_gateway(&self) -> Option<IpAddr> {
        self.addr
    }

    /// If this is a Hedgehog IP discovery, then we assume that the management IP address is a /31
    /// and we assume that the IP that we want to have is "the other" IP of the /31 subnet.
    /// NOTE: We only know if it is, if we have a system description TLV
    /// which indicates that it is
    pub fn get_hh_ip(&self) -> Option<IpNet> {
        if let Some(addr) = self.addr {
            // we are assuming that the management IP address is a /31
            if let Ok(remote_ip) = IpNet::new(addr, 31) {
                // iterate over all possible IPs
                for host in remote_ip.hosts() {
                    // if this is the management IP, then we skip it
                    if host == addr {
                        continue;
                    }
                    // otherwise, it's "the other" IP, which is the one we are looking for
                    return IpNet::new(host, 31).ok();
                }
            }
        }
        None
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
        if tlv.typ != LLDP_TLV_TYPE_ORGANIZATION_SPECIFIC {
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
        if tlv.typ != LLDP_TLV_TYPE_ORGANIZATION_SPECIFIC {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkConfig {
    pub ip: IpNet,
    pub routes: Vec<Route>,
    pub is_hh: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub destinations: Vec<IpNet>,
    pub gateway: IpAddr,
}

// just a convenience conversion method for our RPC
impl From<NetworkConfig> for onie_sai_rpc::onie_sai::NetworkConfig {
    fn from(value: NetworkConfig) -> Self {
        let mut ret = onie_sai_rpc::onie_sai::NetworkConfig::new();
        ret.ip = value.ip.to_string();
        ret.routes = value
            .routes
            .iter()
            .map(|r| {
                let mut ret = onie_sai_rpc::onie_sai::Route::new();
                ret.destinations = r.destinations.iter().map(|d| d.to_string()).collect();
                ret.gateway = r.gateway.to_string();
                ret
            })
            .collect();
        ret.is_hh = value.is_hh;
        ret
    }
}

impl LLDPTLVMUDString {
    pub fn get_hh_network_config(&self) -> Option<NetworkConfig> {
        let mud_url = url::Url::parse(&self.mud_string).ok()?;
        match mud_url.host() {
            None => return None,
            Some(host) => {
                let host = host.to_string();
                if host.as_str() != "hedgehog" {
                    return None;
                }
            }
        };
        let mut found_my_ipnet = false;
        let mut found_your_ipnet = false;
        let mut found_control_vip = false;
        let mut my_ipnet = IpNet::default();
        let mut your_ipnet = IpNet::default();
        let mut control_vip = IpNet::default();
        for (k, v) in mud_url.query_pairs() {
            match k.as_ref() {
                "my_ipnet" => {
                    my_ipnet = IpNet::from_str(v.as_ref()).ok()?;
                    found_my_ipnet = true;
                }
                "your_ipnet" => {
                    your_ipnet = IpNet::from_str(v.as_ref()).ok()?;
                    found_your_ipnet = true;
                }
                "control_vip" => {
                    control_vip = IpNet::from_str(v.as_ref()).ok()?;
                    found_control_vip = true;
                }
                _ => continue,
            }
        }
        if !found_my_ipnet || !found_your_ipnet || !found_control_vip {
            return None;
        }
        Some(NetworkConfig {
            ip: your_ipnet,
            routes: vec![Route {
                destinations: vec![control_vip],
                gateway: my_ipnet.addr(),
            }],
            is_hh: true,
        })
    }
}

/// All LLDP TLVs follow the same format:
/// - 7 bits for the type
/// - 9 bits for the length
/// - the value as "length" number of bytes
#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub enum LLDPPacketReadError {
    ReadError(ReadError),
    NotAnLLDPPacket(u16),
}

#[derive(Debug, Clone)]
pub struct LLDPTLVs(pub Vec<LLDPTLV>);

impl TryFrom<&[u8]> for LLDPTLVs {
    type Error = LLDPPacketReadError;
    fn try_from(raw_pkt: &[u8]) -> Result<Self, Self::Error> {
        let pkt = etherparse::SlicedPacket::from_ethernet(raw_pkt)
            .map_err(|e| LLDPPacketReadError::ReadError(e))?;
        // it is safe to call unwrap() here as link will be set if from_ethernet() does not fail
        let ll_hdr = pkt.link.unwrap().to_header();
        if ll_hdr.ether_type != ETH_P_LLDP {
            return Err(LLDPPacketReadError::NotAnLLDPPacket(ll_hdr.ether_type));
        }
        Ok(Self::parse_lldp(pkt.payload))
    }
}

impl LLDPTLVs {
    pub fn parse_lldp(data: &[u8]) -> Self {
        Self(parse_lldp(data))
    }

    pub fn to_strings(&self) -> Vec<String> {
        let mut ret = Vec::new();
        for tlv in self.0.iter() {
            match tlv.typ {
                LLDP_TLV_TYPE_SYSTEM_NAME => {
                    if let Ok(tlv_name) = LLDPTLVSystemName::try_from(tlv) {
                        ret.push(format!("System Name: {}", tlv_name.name));
                    }
                }
                LLDP_TLV_TYPE_SYSTEM_DESCRIPTION => {
                    if let Ok(tlv_desc) = LLDPTLVSystemDescription::try_from(tlv) {
                        ret.push(format!("System Description: {}", tlv_desc.description));
                    }
                }
                LLDP_TLV_TYPE_MGMT_ADDRESS => {
                    if let Ok(tlv_mgmt) = LLDPTLVMgmtAddr::try_from(tlv) {
                        if let Some(addr) = tlv_mgmt.addr {
                            ret.push(format!("Management Address: {}", addr));
                        }
                    }
                }
                LLDP_TLV_TYPE_ORGANIZATION_SPECIFIC => {
                    if let Ok(tlv_org) = LLDPTLVOrgSpecific::try_from(tlv) {
                        // we could test here if the OUI and subtype match, however, that's already done in the TryFrom
                        // implemenation as well
                        if let Ok(tlv_mud) = LLDPTLVMUDString::try_from(tlv_org) {
                            ret.push(format!(
                                "ICANN, IANA Department - Manufacturer Usage Description URL: {}",
                                tlv_mud.mud_string
                            ));
                        }
                    }
                }
                _ => continue,
            }
        }
        ret
    }

    pub fn get_hh_network_config(&self) -> Option<NetworkConfig> {
        let mut control_vip: Option<IpNet> = None;
        let mut my_ipnet: Option<IpNet> = None;
        let mut gateway: Option<IpAddr> = None;
        for tlv in self.0.iter() {
            match tlv.typ {
                LLDP_TLV_TYPE_SYSTEM_DESCRIPTION => {
                    if let Ok(tlv_desc) = LLDPTLVSystemDescription::try_from(tlv) {
                        control_vip = tlv_desc.get_hh_control_vip();
                    }
                }
                LLDP_TLV_TYPE_MGMT_ADDRESS => {
                    if let Ok(tlv_mgmt) = LLDPTLVMgmtAddr::try_from(tlv) {
                        // our solution only works with IPv4
                        // and we expect this to be the first IPv4 management address that shows up
                        if tlv_mgmt.subtype == IANA_ADDRESS_FAMILY_NUMBER_IP
                            && my_ipnet.is_none()
                            && gateway.is_none()
                        {
                            my_ipnet = tlv_mgmt.get_hh_ip();
                            gateway = tlv_mgmt.get_hh_gateway();
                        }
                    }
                }
                LLDP_TLV_TYPE_ORGANIZATION_SPECIFIC => {
                    if let Ok(tlv_org) = LLDPTLVOrgSpecific::try_from(tlv) {
                        // we could test here if the OUI and subtype match, however, that's already done in the TryFrom
                        // implemenation as well
                        if let Ok(tlv_mud) = LLDPTLVMUDString::try_from(tlv_org) {
                            if let Some(network_config) = tlv_mud.get_hh_network_config() {
                                // if we are able to get a network config from the MUD URL of the TLVs
                                // then we will just take it completely from there, and ignore everything else
                                // this essentially short-circuits everything
                                //
                                // Hedgehog: this is the case for control nodes where we are using systemd-networkd
                                // and we are encoding the network configuration in the MUD URL
                                return Some(network_config);
                            }
                        }
                    }
                }
                _ => continue,
            }
        }

        // if the other side is a SONiC switch, then we are relying on two TLVs to gather the network
        // configuration:
        // - the system description TLV which provides the control VIP
        // - the management address TLV which provides the gateway IP address, and from which we deduce our own IP address and subnet
        if control_vip.is_some() && my_ipnet.is_some() && gateway.is_some() {
            Some(NetworkConfig {
                ip: my_ipnet.unwrap(),
                routes: vec![Route {
                    destinations: vec![control_vip.unwrap()],
                    gateway: gateway.unwrap(),
                }],
                is_hh: true,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use ipnet::Ipv4Net;

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
    fn test_mudurl_parsing() {
        // this reference one must work
        let a = LLDPTLVMUDString { mud_string: "http://hedgehog/?my_ipnet=192.168.101.0/31&your_ipnet=192.168.101.1/31&control_vip=192.168.42.1/32".to_string() };
        let b = a.get_hh_network_config().unwrap();
        println!("{b:?}");
        assert_eq!(
            b,
            NetworkConfig {
                ip: IpNet::V4(Ipv4Net::new(Ipv4Addr::new(192, 168, 101, 1), 31).unwrap()),
                routes: vec![Route {
                    gateway: IpAddr::V4(Ipv4Addr::new(192, 168, 101, 0)),
                    destinations: vec![IpNet::V4(
                        Ipv4Net::new(Ipv4Addr::new(192, 168, 42, 1), 32).unwrap()
                    )],
                }],
                is_hh: true,
            }
        );

        // additional query fields should not hurt
        let a = LLDPTLVMUDString { mud_string: "http://hedgehog/?haha=hehe&my_ipnet=192.168.101.0/31&your_ipnet=192.168.101.1/31&control_vip=192.168.42.1/32&a=b".to_string() };
        let b = a.get_hh_network_config();
        assert!(b.is_some());

        // additional path should not hurt
        let a = LLDPTLVMUDString { mud_string: "http://hedgehog/blah/blah/blah?haha=hehe&my_ipnet=192.168.101.0/31&your_ipnet=192.168.101.1/31&control_vip=192.168.42.1/32&a=b".to_string() };
        let b = a.get_hh_network_config();
        assert!(b.is_some());

        // now let's try some negative cases
        let a = LLDPTLVMUDString { mud_string: "http://localhost/?my_ipnet=192.168.101.0/31&your_ipnet=192.168.101.1/31&control_vip=192.168.42.1/32".to_string() };
        let b = a.get_hh_network_config();
        assert!(b.is_none());
        let a = LLDPTLVMUDString { mud_string: "http://hedgehog/?myyyy_ipnet=192.168.101.0/31&your_ipnet=192.168.101.1/31&control_vip=192.168.42.1/32".to_string() };
        let b = a.get_hh_network_config();
        assert!(b.is_none());
        let a = LLDPTLVMUDString {
            mud_string: "http://hedgehog/?control_vip=192.168.42.1/32".to_string(),
        };
        let b = a.get_hh_network_config();
        assert!(b.is_none());
        let a = LLDPTLVMUDString { mud_string: "http://hedgehog/?my_ipnet=192.168.1001.0/31&your_ipnet=192.168.101.1/31&control_vip=192.168.42.1/32".to_string() };
        let b = a.get_hh_network_config();
        assert!(b.is_none());
        let a = LLDPTLVMUDString { mud_string: "http://hedgehog/?my_ipnet=192.168.101.0/31&your_ipnet=192.168.101..1/31&control_vip=192.168.42.1/32".to_string() };
        let b = a.get_hh_network_config();
        assert!(b.is_none());
        let a = LLDPTLVMUDString { mud_string: "http://hedgehog/?my_ipnet=192.168.101.0/31&your_ipnet=192.168.101.1/31&control_vip=300.168.42.1".to_string() };
        let b = a.get_hh_network_config();
        assert!(b.is_none());
    }

    #[test]
    fn test_hh_mgmt_ip_extraction() {
        let a = LLDPTLVMgmtAddr {
            len: 5,
            subtype: 1,
            bytes: vec![192, 168, 101, 0],
            addr: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 101, 0))),
        };

        // this test is straight forward
        let b = a.get_hh_gateway();
        assert!(b.is_some());
        assert_eq!(a.addr, b);

        // now check tha we get "the other" IP as well
        let b = a.get_hh_ip();
        assert!(b.is_some());
        assert_eq!(
            b,
            Some(IpNet::V4(
                Ipv4Net::new(Ipv4Addr::new(192, 168, 101, 1), 31).unwrap()
            ))
        );

        // check it for the other way around as well
        let a = LLDPTLVMgmtAddr {
            len: 5,
            subtype: 1,
            bytes: vec![192, 168, 101, 1],
            addr: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 101, 1))),
        };

        // this test is straight forward
        let b = a.get_hh_gateway();
        assert!(b.is_some());
        assert_eq!(a.addr, b);

        // now check tha we get "the other" IP as well
        let b = a.get_hh_ip();
        assert!(b.is_some());
        assert_eq!(
            b,
            Some(IpNet::V4(
                Ipv4Net::new(Ipv4Addr::new(192, 168, 101, 0), 31).unwrap()
            ))
        );
    }

    #[test]
    fn test_tlv_sys_descr() {
        let a =
            "Hedgehog: [control_vip=192.168.42.1/32, a=b, c=d ,  e=f, blah, tra=la=la]".to_string();
        if a.starts_with("Hedgehog:") {
            // strip the Hedgehog: prefix, and remove all whitespaces around it
            let a = a.strip_prefix("Hedgehog:").unwrap().trim().to_string();

            // strip the [] around the string if it exists
            let a = if let Some(a) = a.strip_prefix('[') {
                a.to_string()
            } else {
                a
            };
            let a = if let Some(a) = a.strip_suffix(']') {
                a.to_string()
            } else {
                a
            };

            // now let's split by "," and trim whitespaces again
            let a = a
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            // and last but not least, split all entries into key value pairs by splitting on "="
            // and trim whitespaces around it again
            // we'll ignore all entries which are not key value pairs
            let a = a
                .iter()
                .flat_map(|s| {
                    s.split_once('=')
                        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                })
                .collect::<Vec<(String, String)>>();

            println!("{a:?}");
        }
    }

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
            if tlv.typ == LLDP_TLV_TYPE_ORGANIZATION_SPECIFIC {
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
