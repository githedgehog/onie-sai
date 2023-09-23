use crate::{
    route::{RouteEntry, RouteEntryAttribute},
    router_interface::{RouterInterface, RouterInterfaceAttribute},
};

use super::*;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct VirtualRouterID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for VirtualRouterID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for VirtualRouterID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<VirtualRouterID> for sai_object_id_t {
    fn from(value: VirtualRouterID) -> Self {
        value.id
    }
}

impl From<VirtualRouter<'_>> for VirtualRouterID {
    fn from(value: VirtualRouter) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct VirtualRouter<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) switch_id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for VirtualRouter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VirtualRouter(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for VirtualRouter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> VirtualRouter<'a> {
    pub fn create_router_interface(
        &self,
        attrs: Vec<RouterInterfaceAttribute>,
    ) -> Result<RouterInterface, Error> {
        let router_interface_api = self
            .sai
            .router_interface_api()
            .ok_or(Error::APIUnavailable)?;
        let create_router_interface = router_interface_api
            .create_router_interface
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut args: Vec<sai_attribute_t> = Vec::with_capacity(attrs.len() + 1);
        args.push(sai_attribute_t {
            id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_VIRTUAL_ROUTER_ID,
            value: sai_attribute_value_t { oid: self.id },
        });
        for attr in attrs.into_iter() {
            let sai_attr: sai_attribute_t = attr.into();
            args.push(sai_attr);
        }

        let mut oid: sai_object_id_t = 0;
        let st = unsafe {
            create_router_interface(&mut oid, self.switch_id, args.len() as u32, args.as_ptr())
        };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(RouterInterface {
            id: oid,
            sai: self.sai,
        })
    }

    pub fn create_route_entry(
        &self,
        destination: IpNet,
        attrs: Vec<RouteEntryAttribute>,
    ) -> Result<RouteEntry, Error> {
        let route_api = self.sai.route_api().ok_or(Error::APIUnavailable)?;
        let create_route_entry = route_api
            .create_route_entry
            .ok_or(Error::APIFunctionUnavailable)?;

        let args: Vec<sai_attribute_t> = attrs.into_iter().map(|v| v.into()).collect();

        let entry = sai_route_entry_t {
            switch_id: self.switch_id,
            vr_id: self.id,
            destination: destination.into(),
        };
        let st = unsafe { create_route_entry(&entry, args.len() as u32, args.as_ptr()) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(RouteEntry {
            entry: entry,
            sai: self.sai,
        })
    }
}
