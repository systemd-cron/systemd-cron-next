extern crate libc;

use std::ptr;
use std::path::Path;
use std::mem::uninitialized;
use libc::{uid_t, gid_t, c_int, FILE, c_char, size_t, fopen, fclose};
use std::ffi::CStr;

#[repr(C)]
pub struct PwEnt {
    pub pw_name: *const c_char,
    pw_passwd: *const c_char,
    pub pw_uid: uid_t,
    pub pw_gid: gid_t,
    pw_gecos: *const c_char,
    pw_dir: *const c_char,
    pw_shell: *const c_char,
}

extern "C" {
    fn fgetpwent_r(stream: *mut FILE,
                   pwbuf: *mut PwEnt,
                   buf: *mut c_char,
                   buflen: size_t,
                   pwbufp: *mut *mut PwEnt)
                   -> c_int;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct User {
    pub name: String,
    pub passwd: String,
    pub uid: u32,
    pub gid: u32,
    pub gecos: String,
    pub home_dir: String,
    pub shell: String,
}

impl User {
    pub unsafe fn from_ptr(pwent: *const PwEnt) -> User {
        User {
            name: CStr::from_ptr((*pwent).pw_name).to_string_lossy().into_owned(),
            passwd: CStr::from_ptr((*pwent).pw_passwd).to_string_lossy().into_owned(),
            uid: (*pwent).pw_uid,
            gid: (*pwent).pw_gid,
            gecos: CStr::from_ptr((*pwent).pw_gecos).to_string_lossy().into_owned(),
            home_dir: CStr::from_ptr((*pwent).pw_dir).to_string_lossy().into_owned(),
            shell: CStr::from_ptr((*pwent).pw_shell).to_string_lossy().into_owned(),
        }
    }
}

const BUFLEN: size_t = 4096;
pub struct PwEntIter {
    stream: *mut FILE,
    pwbuf: PwEnt,
    pwbufp: *mut PwEnt,
    buf: [u8; BUFLEN],
}

impl Drop for PwEntIter {
    fn drop(&mut self) {
        if !self.stream.is_null() {
            unsafe {
                fclose(self.stream);
            }
        }
    }
}

impl PwEntIter {
    unsafe fn from_ptr(name: *const c_char) -> Option<PwEntIter> {
        let stream = fopen(name, b"r" as *const _ as *const c_char);
        if stream.is_null() {
            None
        } else {
            Some(PwEntIter {
                stream: stream,
                pwbuf: uninitialized(),
                pwbufp: ptr::null_mut(),
                buf: [0u8; BUFLEN],
            })
        }
    }

    pub fn new() -> Option<PwEntIter> {
        unsafe { PwEntIter::from_ptr(b"/etc/passwd\0" as *const _ as *const c_char) }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<PwEntIter> {
        unsafe {
            path.as_ref().to_str().and_then(|p| {
                PwEntIter::from_ptr(CStr::from_ptr(p.as_ptr() as *const c_char).as_ptr())
            })
        }
    }
}

impl Iterator for PwEntIter {
    type Item = *const PwEnt;
    fn next(&mut self) -> Option<*const PwEnt> {
        if unsafe {
            fgetpwent_r(self.stream,
                        &mut self.pwbuf,
                        &mut self.buf as *mut _ as *mut c_char,
                        BUFLEN,
                        &mut self.pwbufp)
        } != 0 || self.pwbufp.is_null() {
            None
        } else {
            Some(self.pwbufp)
        }
    }
}

#[test]
fn find_root() {
    let root = PwEntIter::new()
                   .and_then(|mut iter| {
                       iter.find(|&pw| unsafe {
                           (*pw).pw_uid == 0 || CStr::from_ptr((*pw).pw_name).to_bytes() == b"root"
                       })
                   })
                   .map(|pw| unsafe { User::from_ptr(pw) });
    assert_eq!(root,
               Some(User {
                   name: String::from("root"),
                   gecos: String::from("root"),
                   uid: 0,
                   gid: 0,
                   passwd: String::from("x"),
                   home_dir: String::from("/root"),
                   shell: String::from("/bin/bash"),
               }));
}
