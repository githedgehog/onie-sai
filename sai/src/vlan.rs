pub mod member;

use super::*;
use member::VLANMember;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct VLANID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for VLANID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for VLANID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<VLANID> for sai_object_id_t {
    fn from(value: VLANID) -> Self {
        value.id
    }
}

impl From<VLAN<'_>> for VLANID {
    fn from(value: VLAN) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct VLAN<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for VLAN<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VLAN(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for VLAN<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> VLAN<'a> {
    pub fn get_members(&self) -> Result<Vec<VLANMember<'a>>, Error> {
        // check that API is available/callable
        let vlan_api = self.sai.vlan_api().ok_or(Error::APIUnavailable)?;
        let get_vlan_attribute = vlan_api
            .get_vlan_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut members: Vec<sai_object_id_t> = vec![0u64; 128];
        let mut attr = sai_attribute_t {
            id: _sai_vlan_attr_t_SAI_VLAN_ATTR_MEMBER_LIST,
            value: sai_attribute_value_t {
                objlist: sai_object_list_t {
                    count: 128,
                    list: members.as_mut_ptr(),
                },
            },
        };

        let st = unsafe { get_vlan_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // iterate over the returned list and build the vector for return
        let count = unsafe { attr.value.objlist.count };
        let list = unsafe { attr.value.objlist.list };
        let mut ret: Vec<VLANMember> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let oid: sai_object_id_t = unsafe { *list.offset(i as isize) };
            ret.push(VLANMember {
                id: oid,
                sai: self.sai,
            });
        }
        Ok(ret)
    }
}
