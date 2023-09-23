use crate::*;
use sai_sys::*;

#[derive(Clone)]
pub struct VLANMember<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for VLANMember<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VLANMember(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for VLANMember<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> VLANMember<'a> {
    pub fn remove(self) -> Result<(), Error> {
        // check that API is available/callable
        let vlan_api = self.sai.vlan_api().ok_or(Error::APIUnavailable)?;
        let remove_vlan_member = vlan_api
            .remove_vlan_member
            .ok_or(Error::APIFunctionUnavailable)?;

        match unsafe { remove_vlan_member(self.id) } {
            0 => Ok(()),
            v => Err(Error::SAI(Status::from(v))),
        }
    }
}
