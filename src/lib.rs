//! This crate provides a high-level interface to the RTL-SDR that separates controlling
//! the device and reading samples, for integration into multithreaded applications.
//!
//! ## Example
//!
//! This example reads incoming samples, printing the first I/Q pair, in the main thread
//! while incrementing the receive frequency by 1kHz every second in a subthread.
//!
//! ```rust,no_run
//! let (mut ctl, mut reader) = rtlsdr_mt::open(0).unwrap();
//!
//! ctl.enable_agc().unwrap();
//! ctl.set_ppm(-2).unwrap();
//! ctl.set_center_freq(774_781_250).unwrap();
//!
//! std::thread::spawn(move || {
//!     loop {
//!         let next = ctl.center_freq() + 1000;
//!         ctl.set_center_freq(next).unwrap();
//!
//!         std::thread::sleep(std::time::Duration::from_secs(1));
//!     }
//! });
//!
//! reader.read_async(4, 32768, |bytes| {
//!     println!("i[0] = {}", bytes[0]);
//!     println!("q[0] = {}", bytes[1]);
//! }).unwrap();
//! ```

extern crate libc;
extern crate rtlsdr_sys as ffi;

use std::ffi::CStr;
use std::sync::Arc;

use libc::{c_char, c_uchar, c_void};

/// Holds a list of valid gain values.
pub type TunerGains = [i32; 32];

/// Error type for this crate.
pub type Error = ();

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Create an iterator over available RTL-SDR devices.
///
/// The iterator yields device names in index order, so the device with the first yielded
/// name can be opened at index 0, and so on.
pub fn devices() -> impl Iterator<Item = &'static CStr> {
    let count = unsafe { ffi::rtlsdr_get_device_count() };

    (0..count).map(|idx| unsafe { CStr::from_ptr(ffi::rtlsdr_get_device_name(idx)) })
}

#[derive(Debug)]
pub struct UsbInfo {
    pub manufact: String,
    pub product: String,
    pub serial: String,
    pub index: u32,
    pub error: i32,
}

/// Create an iterator over available RTL-SDR devices.
///
/// The iterator yields device info in index order, so the device with the first yielded
/// name can be opened at index 0, and so on.
pub fn devices_by_usbinfo() -> impl Iterator<Item = UsbInfo> {
    let count = unsafe { ffi::rtlsdr_get_device_count() };

    (0..count).map(|idx| unsafe {
        let manufact = (&mut [0 as c_char; 256]).as_mut_ptr();
        let product = (&mut [0 as c_char; 256]).as_mut_ptr();
        let serial = (&mut [0 as c_char; 256]).as_mut_ptr();
        let info = ffi::rtlsdr_get_device_usb_strings(idx, manufact, product, serial);
        if info < 0 {
            return UsbInfo {
                error: info,
                index: idx,
                manufact: String::from("Error"),
                product: String::from("Error"),
                serial: String::from("Error"),
            };
        }

        let m = CStr::from_ptr(manufact).to_str().unwrap().to_owned();
        let p = CStr::from_ptr(product).to_str().unwrap().to_owned();
        let s = CStr::from_ptr(serial).to_str().unwrap().to_owned();

        print!("man {:?}, {:?}, {:?}", m, p, s);

        return UsbInfo {
            error: 0,
            index: idx,
            manufact: m,
            product: p,
            serial: s,
        };
    })
}

/// Try to open the RTL-SDR device at the given index.
///
/// Return a controller and reader for the device on success.
pub fn open(idx: u32) -> Result<(Controller, Reader)> {
    Device::open(idx)
        .map(|dev| Arc::new(dev))
        .map(|arc| (Controller::new(arc.clone()), Reader::new(arc)))
}

/// Wraps a raw device pointer.
struct Device(ffi::rtlsdr_dev_t);

impl Device {
    /// Try to open and initialize the device at the given index.
    fn open(idx: u32) -> Result<Self> {
        let mut dev = Device(std::ptr::null_mut());

        if unsafe { ffi::rtlsdr_open(&mut dev.0, idx) } == 0
            && unsafe { ffi::rtlsdr_reset_buffer(dev.0) } == 0
        {
            Ok(dev)
        } else {
            Err(())
        }
    }

    /// Close the device.
    fn close(&self) {
        unsafe {
            ffi::rtlsdr_close(self.0);
        }
    }
}

impl std::ops::Drop for Device {
    fn drop(&mut self) {
        self.close();
    }
}

impl std::ops::Deref for Device {
    type Target = ffi::rtlsdr_dev_t;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Controls hardware parameters.
pub struct Controller(Arc<Device>);

impl Controller {
    /// Create a new `Controller` for controlling the given device.
    fn new(dev: Arc<Device>) -> Self {
        Controller(dev)
    }

    /// Get the current sample rate (megasamples/sec).
    pub fn sample_rate(&self) -> u32 {
        unsafe { ffi::rtlsdr_get_sample_rate(**self.0) }
    }

    /// Set the sample rate (megasamples/sec).
    pub fn set_sample_rate(&mut self, rate: u32) -> Result<()> {
        if unsafe { ffi::rtlsdr_set_sample_rate(**self.0, rate) } == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Get the current center frequency (Hz).
    pub fn center_freq(&self) -> u32 {
        unsafe { ffi::rtlsdr_get_center_freq(**self.0) }
    }

    /// Set the center frequency (Hz).
    pub fn set_center_freq(&mut self, freq: u32) -> Result<()> {
        if unsafe { ffi::rtlsdr_set_center_freq(**self.0, freq) } == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Set tuner bandwidth (Hz).
    ///
    /// Note that this is not bit DEPTH which is fixed at 8 in hardware.
    pub fn set_bandwidth(&mut self, bw: u32) -> Result<()> {
        if unsafe { ffi::rtlsdr_set_tuner_bandwidth(**self.0, bw) } == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Get the current frequency correction (ppm).
    pub fn ppm(&self) -> i32 {
        unsafe { ffi::rtlsdr_get_freq_correction(**self.0) }
    }

    /// Set the frequency correction (ppm).
    pub fn set_ppm(&mut self, ppm: i32) -> Result<()> {
        let ret = unsafe { ffi::rtlsdr_set_freq_correction(**self.0, ppm) };

        // librtlsdr returns -2 if the ppm is already set to the given value.
        if ret == 0 || ret == -2 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Enable the hardware AGC.
    ///
    /// Note that this also disables manual tuner gain.
    pub fn enable_agc(&mut self) -> Result<()> {
        if unsafe { ffi::rtlsdr_set_tuner_gain_mode(**self.0, 0) } == 0
            && unsafe { ffi::rtlsdr_set_agc_mode(**self.0, 1) } == 0
        {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Disable the hardware AGC.
    ///
    /// Note that this also enables manual tuner gain.
    pub fn disable_agc(&mut self) -> Result<()> {
        if unsafe { ffi::rtlsdr_set_tuner_gain_mode(**self.0, 1) } == 0
            && unsafe { ffi::rtlsdr_set_agc_mode(**self.0, 0) } == 0
        {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Get the list of valid tuner gain values.
    ///
    /// Each value represents a dB gain with the decimal place shifted right. For example,
    /// the value 496 represents 49.6dB.
    pub fn tuner_gains<'a>(&self, gains: &'a mut TunerGains) -> &'a [i32] {
        let ret = unsafe { ffi::rtlsdr_get_tuner_gains(**self.0, gains.as_mut_ptr()) };

        assert!(ret > 0 && ret as usize <= gains.len());

        &gains[..ret as usize]
    }

    /// Get the current tuner gain in the same format as that returned by `tuner_gains()`.
    pub fn tuner_gain(&self) -> i32 {
        unsafe { ffi::rtlsdr_get_tuner_gain(**self.0) }
    }

    /// Set the tuner gain in the same format as that returned by `tuner_gains()`.
    ///
    /// Note that this also disables the hardware AGC.
    pub fn set_tuner_gain(&mut self, gain: i32) -> Result<()> {
        if unsafe { ffi::rtlsdr_set_tuner_gain_mode(**self.0, 1) } == 0
            && unsafe { ffi::rtlsdr_set_tuner_gain(**self.0, gain) } == 0
        {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Cancel an asynchronous read if one is running.
    pub fn cancel_async_read(&mut self) {
        unsafe {
            ffi::rtlsdr_cancel_async(**self.0);
        }
    }

    /// Reset device's buffer of incoming samples.
    ///
    /// This will clear any samples that have been received by the device but not yet read
    /// by an async_read.
    pub fn reset_buffer(&mut self) -> Result<()> {
        if unsafe { ffi::rtlsdr_reset_buffer(**self.0) } == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

unsafe impl Send for Controller {}

/// Reads I/Q samples.
pub struct Reader(Arc<Device>);

impl Reader {
    /// Create a new `Reader` for reading from the given device.
    fn new(dev: Arc<Device>) -> Self {
        Reader(dev)
    }

    /// Begin reading I/Q samples, buffering into the given number of chunks, with each
    /// chunk holding the given number of bytes. The given callback is called whenever new
    /// samples are available, receiving a chunk at a time.
    ///
    /// This function blocks until the read is cancelled or otherwise terminated. Hardware
    /// parameters can be changed in a separate thread while this function is running.
    pub fn read_async<F>(&mut self, bufs: u32, len: u32, cb: F) -> Result<()>
    where
        F: FnMut(&[u8]),
    {
        let ctx = &cb as *const _ as *mut c_void;

        let ret = unsafe { ffi::rtlsdr_read_async(**self.0, async_wrapper::<F>, ctx, bufs, len) };

        if ret == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Wraps a callback for use as a librtlsdr async callback.
extern "C" fn async_wrapper<F>(buf: *mut c_uchar, len: u32, ctx: *mut c_void)
where
    F: FnMut(&[u8]),
{
    let closure = ctx as *mut F;
    unsafe {
        (*closure)(std::slice::from_raw_parts(buf, len as usize));
    }
}

unsafe impl Send for Reader {}

#[cfg(test)]
mod tests {
    use crate::devices;
    use crate::devices_by_usbinfo;

    #[test]
    fn test_device_by_usbinfo_count() {
        let iter = devices_by_usbinfo();

        assert!(
            iter.count() > 0,
            "Expected number of devices to be greater than 0"
        );
    }

    #[test]
    fn test_device_by_usbinfo() {
        //assert!(false, "Duck");
        let iter = devices_by_usbinfo();

        for val in iter {
            assert!(val.error == 0, "Expected error to be 0");
            //assert!("00000003".eq(val.serial.), "Expected error to be 00000003");
            match val.serial == String::from("00000003") {
                false => assert!(
                    false,
                    "Expected error to be 00000003, {}",
                    val.serial.trim()
                ),
                _ => print!("Value = {:?}", val),
            }
        }
    }
}
