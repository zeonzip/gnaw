use std::ffi::{OsString};
use std::mem;
use crate::os::platform::{from_units, to_units};

#[repr(C)]
pub struct OsStrSlice {
    unit: *mut platform::StrUnit,
    len: usize,
    cap: usize
}

impl OsStrSlice {
    pub fn new(mut data: Vec<platform::StrUnit>) -> Self {
        let slice = OsStrSlice {
            unit: data.as_mut_ptr(),
            len: data.len(),
            cap: data.capacity(),
        };

        mem::forget(data);

        slice
    }

    pub fn into_os_string(self) -> OsString {
        from_units(self.into_vec())
    }

    pub(crate) fn into_vec(self) -> Vec<platform::StrUnit> {
        let this = mem::ManuallyDrop::new(self);
        unsafe { Vec::from_raw_parts(this.unit, this.len, this.cap) }
    }
}

impl Drop for OsStrSlice {
    fn drop(&mut self) {
        let data = unsafe { Vec::from_raw_parts(self.unit, self.len, self.cap) };

        drop(data);
    }
}

#[repr(C)]
pub struct OsArgsData {
    argv: *mut OsStrSlice,
    argc: usize,
    cap: usize
}

impl OsArgsData {
    pub fn args(&mut self) -> &mut [OsStrSlice] {
        unsafe { std::slice::from_raw_parts_mut(self.argv, self.argc) }
    }

    pub fn from_os_strings(args: Vec<OsString>) -> Self {
        let mut argv =  args.iter().map(|s| to_units(s)).map(|s| OsStrSlice::new(s)).collect::<Vec<_>>();

        let (ptr, len, cap) = (argv.as_mut_ptr(), argv.len(), argv.capacity());

        mem::forget(argv);

        OsArgsData {
            argc: len,
            argv: ptr,
            cap
        }
    }

    pub fn into_os_strings(self) -> Vec<OsString> {
        self.into_vec().into_iter().map(|s| s.into_os_string()).collect()
    }

    pub(crate) fn into_vec(self) -> Vec<OsStrSlice> {
        let this = mem::ManuallyDrop::new(self);
        unsafe { Vec::from_raw_parts(this.argv, this.argc, this.cap) }
    }
}

impl Drop for OsArgsData {
    fn drop(&mut self) {
        let data = unsafe { Vec::from_raw_parts(self.argv, self.argc, self.cap) };
        data.iter().for_each(drop);

        drop(data);
    }
}

#[cfg(unix)]
mod platform {
    use std::ffi::{OsStr, OsString};
    use std::os::unix::ffi::{OsStringExt, OsStrExt};

    pub type StrUnit = u8;

    pub(super) fn to_units(str: &OsStr) -> Vec<StrUnit> {
        str.as_bytes().iter().cloned().collect()
    }
    pub(super) fn from_units(units: Vec<StrUnit>) -> OsString {
        OsString::from_vec(units)
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::{OsStr, OsString};

    pub type StrUnit = u16;

    pub(super) fn to_units(str: &OsStr) -> Vec<StrUnit> {
        str.encode_wide().collect()
    }
    pub(super) fn from_units(units: Vec<StrUnit>) -> OsString {
        OsString::from_wide(&units)
    }
}

#[cfg(all(not(unix), not(windows)))]
compile_error!("Gnaw only supports unix and windows targets.");