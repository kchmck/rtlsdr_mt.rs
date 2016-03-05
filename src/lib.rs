extern crate libc;
extern crate rtlsdr_sys as ffi;

use std::sync::Arc;

use libc::{c_uchar, uint32_t, c_void};

pub fn open(idx: u32) -> Option<(Control, Reader)> {
    Device::open(idx).map(|dev| Arc::new(dev)).map(|arc| {
        (Control::new(arc.clone()), Reader::new(arc.clone()))
    })
}

struct Device(ffi::rtlsdr_dev_t);

impl Device {
    pub fn open(idx: u32) -> Option<Device> {
        let mut dev = Device(0 as *mut c_void);

        let ret = unsafe {
            ffi::rtlsdr_open(&mut dev.0 as *mut ffi::rtlsdr_dev_t, idx)
        };

        if ret == 0 {
            Some(dev)
        } else {
            None
        }
    }

    pub fn close(&self) {
        unsafe { ffi::rtlsdr_close(self.0); }
    }
}

impl std::ops::Drop for Device {
    fn drop(&mut self) {
        self.close();
    }
}

impl std::ops::Deref for Device {
    type Target = ffi::rtlsdr_dev_t;
    fn deref(&self) -> &Self::Target { &self.0 }
}

pub struct Control(Arc<Device>);

impl Control {
    fn new(dev: Arc<Device>) -> Control {
        Control(dev)
    }

    pub fn get_sample_rate(&mut self) -> u32 {
        unsafe { ffi::rtlsdr_get_sample_rate(**self.0) }
    }

    pub fn set_sample_rate(&mut self, rate: u32) -> bool {
        unsafe { ffi::rtlsdr_set_sample_rate(**self.0, rate) == 0 }
    }

    pub fn get_center_freq(&self) -> u32 {
        unsafe { ffi::rtlsdr_get_center_freq(**self.0) }
    }

    pub fn set_center_freq(&mut self, freq: u32) -> bool {
        unsafe { ffi::rtlsdr_set_center_freq(**self.0, freq) == 0 }
    }

    pub fn get_ppm(&self) -> i32 {
        unsafe { ffi::rtlsdr_get_freq_correction(**self.0) }
    }

    pub fn set_ppm(&mut self, ppm: i32) -> bool {
        unsafe { ffi::rtlsdr_set_freq_correction(**self.0, ppm) == 0 }
    }

    pub fn enable_agc(&mut self) -> bool {
        unsafe {
            ffi::rtlsdr_set_tuner_gain_mode(**self.0, 0) == 0 &&
            ffi::rtlsdr_set_agc_mode(**self.0, 1) == 0
        }
    }

    pub fn disable_agc(&mut self) -> bool {
        unsafe {
            ffi::rtlsdr_set_tuner_gain_mode(**self.0, 1) == 0 &&
            ffi::rtlsdr_set_agc_mode(**self.0, 0) == 0
        }
    }

    pub fn get_tuner_gains(&self, gains: &mut [i32; 32]) -> u32 {
        let ret = unsafe {
            ffi::rtlsdr_get_tuner_gains(**self.0, gains.as_mut_ptr())
        };

        assert!(ret > 0 && ret as usize <= gains.len());

        ret as u32
    }

    pub fn get_tuner_gain(&self) -> i32 {
        unsafe { ffi::rtlsdr_get_tuner_gain(**self.0) }
    }

    pub fn set_tuner_gain(&mut self, gain: i32) -> bool {
        unsafe {
            ffi::rtlsdr_set_tuner_gain_mode(**self.0, 1) == 0 &&
            ffi::rtlsdr_set_tuner_gain(**self.0, gain) == 0
        }
    }

    pub fn reset_buf(&mut self) -> bool {
        unsafe { ffi::rtlsdr_reset_buffer(**self.0) == 0 }
    }

    pub fn cancel_async_read(&mut self) {
        unsafe { ffi::rtlsdr_cancel_async(**self.0); }
    }
}

unsafe impl Send for Control {}

pub struct Reader(Arc<Device>);

impl Reader {
    fn new(dev: Arc<Device>) -> Reader {
        Reader(dev)
    }

    pub fn read_async<F>(&mut self, bufs: u32, len: u32, cb: F) -> bool
        where F: FnMut(&[u8])
    {
        let ctx = &cb as *const _ as *mut c_void;

        unsafe {
            ffi::rtlsdr_read_async(**self.0, async_wrapper::<F>, ctx, bufs, len) == 0
        }
    }
}

extern fn async_wrapper<F>(buf: *mut c_uchar, len: uint32_t, ctx: *mut c_void)
    where F: FnMut(&[u8])
{
    let closure = ctx as *mut F;
    unsafe { (*closure)(std::slice::from_raw_parts(buf, len as usize)); }
}

unsafe impl Send for Reader {}
