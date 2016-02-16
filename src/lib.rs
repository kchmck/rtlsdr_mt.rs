extern crate libc;
extern crate rtlsdr_sys as ffi;

use libc::{c_uchar, uint32_t, c_int, c_void};

pub fn device_count() -> u32 {
    unsafe { ffi::rtlsdr_get_device_count() }
}

pub struct RtlSdr(ffi::rtlsdr_dev_t);

impl RtlSdr {
    pub fn open(idx: u32) -> Option<RtlSdr> {
        let mut sdr = RtlSdr(0 as *mut c_void);

        let ret = unsafe {
            ffi::rtlsdr_open(&mut sdr.0 as *mut ffi::rtlsdr_dev_t, idx)
        };

        if ret == 0 {
            Some(sdr)
        } else {
            None
        }
    }

    pub fn close(&mut self) {
        unsafe { ffi::rtlsdr_close(self.0); }
    }

    pub fn get_sample_rate(&self) -> u32 {
        unsafe { ffi::rtlsdr_get_sample_rate(self.0) }
    }

    pub fn set_sample_rate(&mut self, rate: u32) -> bool {
        unsafe { ffi::rtlsdr_set_sample_rate(self.0, rate) == 0 }
    }

    pub fn get_center_freq(&self) -> u32 {
        unsafe { ffi::rtlsdr_get_center_freq(self.0) }
    }

    pub fn set_center_freq(&mut self, freq: u32) -> bool {
        unsafe { ffi::rtlsdr_set_center_freq(self.0, freq) == 0 }
    }

    pub fn get_ppm(&self) -> i32 {
        unsafe { ffi::rtlsdr_get_freq_correction(self.0) }
    }

    pub fn set_ppm(&mut self, ppm: i32) -> bool {
        unsafe { ffi::rtlsdr_set_freq_correction(self.0, ppm) == 0 }
    }

    pub fn enable_agc(&mut self) -> bool {
        unsafe {
            ffi::rtlsdr_set_tuner_gain_mode(self.0, 0) == 0 &&
            ffi::rtlsdr_set_agc_mode(self.0, 1) == 0
        }
    }

    pub fn disable_agc(&mut self) -> bool {
        unsafe {
            ffi::rtlsdr_set_tuner_gain_mode(self.0, 1) == 0 &&
            ffi::rtlsdr_set_agc_mode(self.0, 0) == 0
        }
    }

    pub fn get_tuner_gains(&self) -> [i32; 32] {
        let mut gains = [0; 32];

        let ret = unsafe {
            ffi::rtlsdr_get_tuner_gains(self.0, gains.as_mut_ptr())
        };

        assert!(ret > 0 && ret as usize <= gains.len());

        gains
    }

    pub fn get_tuner_gain(&self) -> i32 {
        unsafe { ffi::rtlsdr_get_tuner_gain(self.0) }
    }

    pub fn set_tuner_gain(&mut self, gain: i32) -> bool {
        unsafe {
            ffi::rtlsdr_set_tuner_gain_mode(self.0, 1) == 0 &&
            ffi::rtlsdr_set_tuner_gain(self.0, gain) == 0
        }
    }

    pub fn read_sync(&self, buf: &mut [u8]) -> Option<u32> {
        let len = buf.len() as i32;
        let mut read = 0;

        let ret = unsafe {
            ffi::rtlsdr_read_sync(self.0, buf.as_mut_ptr() as *mut c_void,
                                  len, &mut read as *mut c_int)
        };

        if ret == 0 {
            Some(read as u32)
        } else {
            None
        }
    }

    pub fn read_async<F>(&self, bufs: u32, len: u32, cb: F) -> bool
        where F: FnMut(&[u8])
    {
        let ctx = &cb as *const _ as *mut c_void;

        unsafe {
            ffi::rtlsdr_read_async(self.0, async_wrapper::<F>, ctx, bufs, len) == 0
        }
    }

    pub fn cancel_async(&mut self) {
        unsafe { ffi::rtlsdr_cancel_async(self.0); }
    }

    pub fn reset_buf(&mut self) -> bool {
        unsafe { ffi::rtlsdr_reset_buffer(self.0) == 0 }
    }
}

impl std::ops::Drop for RtlSdr {
    fn drop(&mut self) {
        self.close();
    }
}

extern fn async_wrapper<F>(buf: *mut c_uchar, len: uint32_t, ctx: *mut c_void)
    where F: FnMut(&[u8])
{
    let closure = ctx as *mut F;

    unsafe {
        let slice = std::slice::from_raw_parts(buf, len as usize);
        (*closure)(slice);
    }
}
