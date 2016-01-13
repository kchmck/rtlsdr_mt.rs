#![allow(non_camel_case_types)]

extern crate libc;

use libc::{uint32_t, uint16_t, uint8_t, c_char, c_uchar, c_int, c_void};

pub type rtlsdr_dev_t = *mut c_void;

pub type rtlsdr_read_async_cb_t = extern fn(buf: *mut c_uchar, len: uint32_t,
                                            ctx: *mut c_void);

#[repr(C)]
pub enum rtlsdr_tuner {
	RTLSDR_TUNER_UNKNOWN,
	RTLSDR_TUNER_E4000,
	RTLSDR_TUNER_FC0012,
	RTLSDR_TUNER_FC0013,
	RTLSDR_TUNER_FC2580,
	RTLSDR_TUNER_R820T,
	RTLSDR_TUNER_R828D
}

#[link(name = "rtlsdr")]
extern {
    pub fn rtlsdr_get_device_count() -> uint32_t;
    pub fn rtlsdr_get_device_name(idx: uint32_t) -> *const c_char;
    pub fn rtlsdr_get_device_usb_strings(idx: uint32_t, mfg: *mut c_char,
                                         prod: *mut c_char, serial: *mut c_char) -> c_int;
    pub fn rtlsdr_get_index_by_serial(serial: *const c_char) -> c_int;

    pub fn rtlsdr_open(dev: *mut rtlsdr_dev_t, idx: uint32_t) -> c_int;
    pub fn rtlsdr_close(dev: rtlsdr_dev_t) -> c_int;

    pub fn rtlsdr_set_xtal_freq(dev: rtlsdr_dev_t, rtl_freq: uint32_t,
                                tuner_freq: uint32_t) -> c_int;
    pub fn rtlsdr_get_xtal_freq(dev: rtlsdr_dev_t, rtl_freq: *mut uint32_t,
                                tuner_freq: *mut uint32_t) -> c_int;
    pub fn rtlsdr_get_usb_strings(dev: rtlsdr_dev_t, mfg: *mut c_char, prod: *mut c_char,
                                  serial: *mut c_char) -> c_int;

    pub fn rtlsdr_write_eeprom(dev: rtlsdr_dev_t, data: *const uint8_t, offset: uint8_t,
                               len: uint16_t) -> c_int;
    pub fn rtlsdr_read_eeprom(dev: rtlsdr_dev_t, data: *mut uint8_t, offset: uint8_t,
                              len: uint16_t) -> c_int;

    pub fn rtlsdr_set_center_freq(dev: rtlsdr_dev_t, freq: uint32_t) -> c_int;
    pub fn rtlsdr_get_center_freq(dev: rtlsdr_dev_t) -> uint32_t;

    pub fn rtlsdr_set_freq_correction(dev: rtlsdr_dev_t, ppm: c_int) -> c_int;
    pub fn rtlsdr_get_freq_correction(dev: rtlsdr_dev_t) -> c_int;

    pub fn rtlsdr_get_tuner_type(dev: rtlsdr_dev_t) -> rtlsdr_tuner;

    pub fn rtlsdr_get_tuner_gains(dev: rtlsdr_dev_t, gains: *mut c_int) -> c_int;
    pub fn rtlsdr_set_tuner_gain(dev: rtlsdr_dev_t, gain: c_int) -> c_int;
    pub fn rtlsdr_get_tuner_gain(dev: rtlsdr_dev_t) -> c_int;
    pub fn rtlsdr_set_tuner_if_gain(dev: rtlsdr_dev_t, stage: c_int, gain: c_int) ->c_int;
    pub fn rtlsdr_set_tuner_gain_mode(dev: rtlsdr_dev_t, manual: c_int) -> c_int;

    pub fn rtlsdr_set_sample_rate(dev: rtlsdr_dev_t, rate: uint32_t) -> c_int;
    pub fn rtlsdr_get_sample_rate(dev: rtlsdr_dev_t) -> uint32_t;

    pub fn rtlsdr_set_testmode(dev: rtlsdr_dev_t, on: c_int) -> c_int;
    pub fn rtlsdr_set_agc_mode(dev: rtlsdr_dev_t, on: c_int) -> c_int;

    pub fn rtlsdr_set_direct_sampling(dev: rtlsdr_dev_t, on: c_int) -> c_int;
    pub fn rtlsdr_get_direct_sampling(dev: rtlsdr_dev_t) -> c_int;

    pub fn rtlsdr_set_offset_tuning(dev: rtlsdr_dev_t, on: c_int) -> c_int;
    pub fn rtlsdr_get_offset_tuning(dev: rtlsdr_dev_t) -> c_int;

    pub fn rtlsdr_reset_buffer(dev: rtlsdr_dev_t) -> c_int;
    pub fn rtlsdr_read_sync(dev: rtlsdr_dev_t, buf: *mut c_void, len: c_int,
                            read: *mut c_int) -> c_int;
    pub fn rtlsdr_read_async(dev: rtlsdr_dev_t, cb: rtlsdr_read_async_cb_t,
                             ctx: *mut c_void, num: uint32_t, len: uint32_t) -> c_int;
    pub fn rtlsdr_cancel_async(dev: rtlsdr_dev_t) -> c_int;
}
