use std::cell::OnceCell;
use std::sync::{OnceLock, Mutex, Arc};
use std::collections::HashMap;
use std::rc::Rc;

use sai_sys::*;
use std::os::raw::{c_char, c_int};
use std::ptr::null;

static BLAH: OnceLock<Box<dyn Fn(i32) + Send + Sync>> = OnceLock::new();

static PROFILE_GET_NEXT_VALUE_CALLBACK: OnceLock<Box<dyn Fn(sai_switch_profile_id_t, *mut *const c_char, *mut *const c_char) -> c_int + Send + Sync>> = OnceLock::new();

extern "C" fn profile_get_next_value_cb(profile_id: sai_switch_profile_id_t,
    variable: *mut *const c_char,
    value: *mut *const c_char,
) -> c_int {
        // Check if the callback is set
        if let Some(ref mut callback) = PROFILE_GET_NEXT_VALUE_CALLBACK.get() {
            callback(profile_id, variable, value)
        } else {
            0
        }
}

    static PROFILE_GET_VALUE_CALLBACK: OnceLock<Box<dyn Fn(sai_switch_profile_id_t, *const c_char) -> *const c_char + Send + Sync>> = OnceLock::new();

    extern "C" fn profile_get_value_cb(
    profile_id: sai_switch_profile_id_t,
    variable: *const c_char,
) -> *const c_char {
        if let Some(ref mut callback) = PROFILE_GET_VALUE_CALLBACK.get() {
            callback(profile_id, variable)
        } else {
            null()
        }
}

static PROFILE_SMT: sai_service_method_table_t = sai_service_method_table_t {
    profile_get_next_value: Some(profile_get_next_value_cb),
    profile_get_value: Some(profile_get_value_cb),
};

#[derive(Clone, Debug)]
struct SAIProfile {
    profile_idx: u32,
    profile: HashMap<String, String>,
}

impl SAIProfile {
    fn profile_get_next_value(&self, profile_id: sai_switch_profile_id_t, variable: *mut *const c_char, value: *mut *const c_char) -> c_int {
        0
    }

    fn profile_get_value(&self, profile_id: sai_switch_profile_id_t, variable: *const c_char) -> *const c_char {
        null()
    }
}

pub struct SAI {
}

impl SAI {

    pub fn new(profile: HashMap<String, String>) -> SAI {
        let ret = SAI {};
        let p1 = Arc::new(SAIProfile {
            profile_idx: 0,
            profile: profile,
        });
        let p2 = Arc::clone(&p1);
        PROFILE_GET_NEXT_VALUE_CALLBACK.set(Box::new(move |profile_id: sai_switch_profile_id_t, variable: *mut *const c_char, value: *mut *const c_char| { p1.profile_get_next_value(profile_id, variable, value) }));
        PROFILE_GET_VALUE_CALLBACK.set(Box::new(move |profile_id: sai_switch_profile_id_t, variable: *const c_char| { p2.profile_get_value(profile_id, variable) }));
        ret
    }

    fn init(&self) {
        unsafe {
            sai_api_initialize(0, &PROFILE_SMT);
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
