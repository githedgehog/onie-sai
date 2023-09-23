use crate::*;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct TrapGroupID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for TrapGroupID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "trapgroup:oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for TrapGroupID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<TrapGroupID> for sai_object_id_t {
    fn from(value: TrapGroupID) -> Self {
        value.id
    }
}

impl From<TrapGroup<'_>> for TrapGroupID {
    fn from(value: TrapGroup) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct TrapGroup<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for TrapGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrapGroup(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for TrapGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> TrapGroup<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let remove_hostif_trap_group = hostif_api
            .remove_hostif_trap_group
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_hostif_trap_group(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

impl ObjectID<TrapGroupID> for TrapGroup<'_> {
    fn to_id(&self) -> TrapGroupID {
        TrapGroupID { id: self.id }
    }
}
