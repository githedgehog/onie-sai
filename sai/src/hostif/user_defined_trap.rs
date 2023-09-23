use crate::*;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct UserDefinedTrapID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for UserDefinedTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for UserDefinedTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<UserDefinedTrapID> for sai_object_id_t {
    fn from(value: UserDefinedTrapID) -> Self {
        value.id
    }
}

impl From<UserDefinedTrap<'_>> for UserDefinedTrapID {
    fn from(value: UserDefinedTrap<'_>) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct UserDefinedTrap<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for UserDefinedTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfUserDefinedTrap(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for UserDefinedTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> UserDefinedTrap<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let remove_hostif_user_defined_trap = hostif_api
            .remove_hostif_user_defined_trap
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_hostif_user_defined_trap(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

impl ObjectID<UserDefinedTrapID> for UserDefinedTrap<'_> {
    fn to_id(&self) -> UserDefinedTrapID {
        UserDefinedTrapID { id: self.id }
    }
}
