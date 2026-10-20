#![allow(non_upper_case_globals, non_snake_case)]

use super::{
    libx11::{self, Display, Window, _XPrivDisplay},
    xi_input,
};

pub const XIAllDevices: libc::c_int = 0 as libc::c_int;
pub const XI_RawMotion: libc::c_int = 17 as libc::c_int;
pub const XI_RawMotionMask: libc::c_int = (1 as libc::c_int) << XI_RawMotion;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct XIEventMask {
    pub deviceid: libc::c_int,
    pub mask_len: libc::c_int,
    pub mask: *mut libc::c_uchar,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct XIValuatorState {
    pub mask_len: libc::c_int,
    pub mask: *mut libc::c_uchar,
    pub values: *mut libc::c_double,
}

pub type Time = libc::c_ulong;

// draw patch (ADR-T35-1): device queries for the pen's pressure valuator.
pub const XIValuatorClass: libc::c_int = 2;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct XIAnyClassInfo {
    pub type_0: libc::c_int,
    pub sourceid: libc::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct XIValuatorClassInfo {
    pub type_0: libc::c_int,
    pub sourceid: libc::c_int,
    pub number: libc::c_int,
    pub label: libx11::Atom,
    pub min: libc::c_double,
    pub max: libc::c_double,
    pub value: libc::c_double,
    pub resolution: libc::c_int,
    pub mode: libc::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct XIDeviceInfo {
    pub deviceid: libc::c_int,
    pub name: *mut libc::c_char,
    pub use_0: libc::c_int,
    pub attachment: libc::c_int,
    pub enabled: libc::c_int,
    pub num_classes: libc::c_int,
    pub classes: *mut *mut XIAnyClassInfo,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct XIRawEvent {
    pub type_0: libc::c_int,
    pub serial: libc::c_ulong,
    pub send_event: libc::c_int,
    pub display: *mut Display,
    pub extension: libc::c_int,
    pub evtype: libc::c_int,
    pub time: Time,
    pub deviceid: libc::c_int,
    pub sourceid: libc::c_int,
    pub detail: libc::c_int,
    pub flags: libc::c_int,
    pub valuators: XIValuatorState,
    pub raw_values: *mut libc::c_double,
}

use core::ffi::{c_char, c_int};
use std::collections::HashMap;
crate::declare_module!(
    LibXi,
    "libXi.so",
    "libXi.so.6",
    ...
    ...
    pub fn XQueryExtension(*mut Display, *const c_char, *mut c_int, *mut c_int, *mut c_int) -> c_int,
    pub fn XIQueryVersion(*mut Display, *mut c_int, *mut c_int) -> c_int,
    pub fn XISelectEvents(*mut Display, Window, *mut XIEventMask, c_int),
    pub fn XGetEventData(*mut Display, *mut libx11::XGenericEventCookie) -> c_int,
    pub fn XFreeEventData(*mut Display, *mut libx11::XGenericEventCookie),
    pub fn XIQueryDevice(*mut Display, c_int, *mut c_int) -> *mut XIDeviceInfo,
    pub fn XIFreeDeviceInfo(*mut XIDeviceInfo),
    ...
    ...
    pub xi_extension_opcode: Option<i32>,
);

impl LibXi {
    pub unsafe fn query_xi_extension(
        &mut self,
        libx11: &mut libx11::LibX11,
        display: *mut Display,
    ) {
        let mut ev = 0;
        let mut err = 0;
        let mut xi_opcode = 0;

        if (libx11.XQueryExtension)(
            display,
            b"XInputExtension\x00" as *const u8 as *const libc::c_char,
            &mut xi_opcode,
            &mut ev,
            &mut err,
        ) == 0
        {
            return;
        }

        // check the version of XInput
        let mut major = 2;
        let mut minor = 3;
        if (self.XIQueryVersion)(display, &mut major, &mut minor) != 0 {
            return;
        }

        // select events to listen
        let mut mask = XI_RawMotionMask;
        let mut masks = XIEventMask {
            deviceid: XIAllDevices,
            mask_len: ::std::mem::size_of::<libc::c_int>() as _,
            mask: &mut mask as *mut _ as *mut _,
        };

        (self.XISelectEvents)(
            display,
            // this weird pointers is macro expansion of DefaultRootWindow(display)
            (*(*(display as _XPrivDisplay))
                .screens
                .offset((*(display as _XPrivDisplay)).default_screen as isize))
            .root,
            &mut masks,
            1 as libc::c_int,
        );
        self.xi_extension_opcode = Some(xi_opcode);
    }

    /// Get mouse delta from XI_RawMotion's event XGenericEventCookie data,
    /// and feed the pen pressure tracker (draw patch, ADR-T35-1).
    pub unsafe fn read_cookie(
        &mut self,
        xcookie: &mut libx11::XGenericEventCookie,
        display: *mut Display,
        pen: &mut PenTracker,
        libx11: &mut libx11::LibX11,
    ) -> (f64, f64) {
        assert!(xcookie.evtype == xi_input::XI_RawMotion);

        (self.XGetEventData)(display, xcookie);

        let raw_event = xcookie.data as *mut xi_input::XIRawEvent;

        // Data returned from Xlib is not guaranteed to be aligned
        let ptr = (*raw_event).raw_values as *const f64;
        let dx = std::ptr::read_unaligned(ptr);
        let dy = std::ptr::read_unaligned(ptr.add(1));

        pen.raw_event(self, libx11, display, &*raw_event);

        (self.XFreeEventData)(display, &mut (*xcookie) as *mut _);

        (dx, dy)
    }
}

/// The pressure valuator of a pen device.
#[derive(Copy, Clone, Debug)]
struct PressureAxis {
    number: i32,
    min: f64,
    max: f64,
}

/// Follows the pressure of the pen that sent the last raw motion
/// (draw patch, ADR-T35-1).
///
/// Every `XI_RawMotion` names its source device. A device with a valuator
/// labelled `Abs Pressure` is a pen: its pressure, normalised to `[0, 1]`,
/// is kept until the next raw event. Raw events only carry the valuators
/// that changed, so a missing pressure keeps the last one. A raw event
/// from any other device clears it.
#[derive(Default)]
pub struct PenTracker {
    /// Pressure axis of each source device seen, `None` for devices
    /// without one.
    axes: HashMap<i32, Option<PressureAxis>>,
    /// The `Abs Pressure` atom, once looked up.
    label: Option<libx11::Atom>,
    /// Source device and pressure of the pen that moved last.
    current: Option<(i32, f32)>,
}

impl PenTracker {
    /// The pressure of the pen that sent the last raw event, if it was one.
    pub fn pressure(&self) -> Option<f32> {
        self.current.map(|(_, pressure)| pressure)
    }

    unsafe fn raw_event(
        &mut self,
        libxi: &mut LibXi,
        libx11: &mut libx11::LibX11,
        display: *mut Display,
        event: &XIRawEvent,
    ) {
        let device = event.sourceid;
        let axis = match self.axes.get(&device) {
            Some(axis) => *axis,
            None => {
                let axis = self.query_axis(libxi, libx11, display, device);
                self.axes.insert(device, axis);
                axis
            }
        };
        let Some(axis) = axis else {
            self.current = None;
            return;
        };
        let previous = match self.current {
            Some((pen, pressure)) if pen == device => pressure,
            _ => 0.0,
        };
        let pressure = valuator(&event.valuators, axis.number)
            .map(|value| {
                let range = axis.max - axis.min;
                if range > 0.0 {
                    ((value - axis.min) / range).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                }
            })
            .filter(|p| p.is_finite())
            .unwrap_or(previous);
        self.current = Some((device, pressure));
    }

    /// The `Abs Pressure` valuator of `device`, if it has one.
    unsafe fn query_axis(
        &mut self,
        libxi: &mut LibXi,
        libx11: &mut libx11::LibX11,
        display: *mut Display,
        device: i32,
    ) -> Option<PressureAxis> {
        let label = *self.label.get_or_insert_with(|| {
            (libx11.XInternAtom)(
                display,
                b"Abs Pressure\x00".as_ptr() as *const libc::c_char,
                true as _,
            )
        });
        if label == 0 {
            // No device on this server has ever had the label.
            return None;
        }
        let mut count = 0;
        let info = (libxi.XIQueryDevice)(display, device, &mut count);
        if info.is_null() {
            return None;
        }
        let mut found = None;
        for d in 0..count.max(0) as usize {
            let device_info = &*info.add(d);
            for c in 0..device_info.num_classes.max(0) as usize {
                let class = *device_info.classes.add(c);
                if class.is_null() || (*class).type_0 != XIValuatorClass {
                    continue;
                }
                let valuator = &*(class as *const XIValuatorClassInfo);
                if valuator.label == label {
                    found = Some(PressureAxis {
                        number: valuator.number,
                        min: valuator.min,
                        max: valuator.max,
                    });
                }
            }
        }
        (libxi.XIFreeDeviceInfo)(info);
        found
    }
}

/// The value of valuator `number` in a raw event, if it changed.
///
/// Values are packed in the order of the set bits of the mask.
unsafe fn valuator(state: &XIValuatorState, number: i32) -> Option<f64> {
    if number < 0 || number >= state.mask_len * 8 || state.mask.is_null() {
        return None;
    }
    let is_set = |bit: i32| *state.mask.add((bit / 8) as usize) & (1 << (bit % 8)) != 0;
    if !is_set(number) {
        return None;
    }
    let index = (0..number).filter(|&bit| is_set(bit)).count();
    Some(std::ptr::read_unaligned(
        state.values.add(index) as *const f64
    ))
}
