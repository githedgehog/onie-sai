use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use sai_sys::*;
use std::os::raw::{c_char, c_int};
use std::ptr::null;

thread_local!(
    static PROFILE_GET_NEXT_VALUE_CALLBACK: RefCell<Option<Box<dyn FnMut(sai_switch_profile_id_t, *mut *const c_char, *mut *const c_char) -> c_int>>> = RefCell::new(None);
);
extern "C" fn profile_get_next_value_cb(profile_id: sai_switch_profile_id_t,
    variable: *mut *const c_char,
    value: *mut *const c_char,
) -> c_int {
    PROFILE_GET_NEXT_VALUE_CALLBACK.with(|cb| {
        // Check if the callback is set
        if let Some(ref mut callback) = *cb.borrow_mut() {
            callback(profile_id, variable, value)
        } else {
            0
        }
    })
}

thread_local!(
    static PROFILE_GET_VALUE_CALLBACK: RefCell<Option<Box<dyn FnMut(sai_switch_profile_id_t, *const c_char) -> *const c_char>>> = RefCell::new(None);
);
extern "C" fn profile_get_value_cb(
    profile_id: sai_switch_profile_id_t,
    variable: *const c_char,
) -> *const c_char {
    PROFILE_GET_VALUE_CALLBACK.with(|cb| {
        if let Some(ref mut callback) = *cb.borrow_mut() {
            callback(profile_id, variable)
        } else {
            null()
        }
    })
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
        let p1 = Rc::new(SAIProfile {
            profile_idx: 0,
            profile: profile,
        });
        let p2 = Rc::clone(&p1);
        PROFILE_GET_NEXT_VALUE_CALLBACK.with(|cb| {
            *cb.borrow_mut() = Some(Box::new(move |profile_id: sai_switch_profile_id_t, variable: *mut *const c_char, value: *mut *const c_char| { p1.profile_get_next_value(profile_id, variable, value) }));
        });
        PROFILE_GET_VALUE_CALLBACK.with(|cb| {
            *cb.borrow_mut() = Some(Box::new(move |profile_id: sai_switch_profile_id_t, variable: *const c_char| { p2.profile_get_value(profile_id, variable) }));
        });
        ret.init();
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
