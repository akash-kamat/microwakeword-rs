use std::ffi::c_int;

use crate::{AUDIO_BLOCK_SAMPLES, Error, Result};

pub(crate) const FEATURE_COUNT: usize = 40;
pub(crate) const FEATURE_SCALE: f32 = 0.039_062_5;

#[repr(C)]
struct NativeFrontend {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn mww_frontend_create() -> *mut NativeFrontend;
    fn mww_frontend_destroy(frontend: *mut NativeFrontend);
    fn mww_frontend_process(
        frontend: *mut NativeFrontend,
        samples: *const i16,
        sample_count: usize,
        output: *mut u16,
        output_capacity: usize,
    ) -> c_int;
}

pub(crate) struct Frontend(*mut NativeFrontend);

impl Frontend {
    pub(crate) fn new() -> Result<Self> {
        // SAFETY: The constructor returns a new uniquely owned handle.
        let handle = unsafe { mww_frontend_create() };
        if handle.is_null() {
            return Err(Error::Audio(
                "failed to initialize the microWakeWord frontend".into(),
            ));
        }
        Ok(Self(handle))
    }

    pub(crate) fn process(
        &mut self,
        samples: &[i16; AUDIO_BLOCK_SAMPLES],
    ) -> Result<Option<[u16; FEATURE_COUNT]>> {
        let mut output = [0; FEATURE_COUNT];
        // SAFETY: The buffers have the advertised sizes and the handle is exclusively borrowed.
        let count = unsafe {
            mww_frontend_process(
                self.0,
                samples.as_ptr(),
                samples.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };
        match count {
            0 => Ok(None),
            n if n == FEATURE_COUNT as c_int => Ok(Some(output)),
            n => Err(Error::Audio(format!(
                "microfrontend returned {n} features; expected {FEATURE_COUNT}"
            ))),
        }
    }
}

impl Drop for Frontend {
    fn drop(&mut self) {
        // SAFETY: This is the matching destructor for the uniquely owned handle.
        unsafe { mww_frontend_destroy(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_reference_features() {
        let expected = [
            548, 546, 604, 582, 665, 610, 662, 583, 583, 493, 0, 0, 0, 0, 419, 464, 425, 436, 492,
            473, 400, 0, 0, 0, 448, 371, 0, 446, 478, 406, 377, 436, 425, 411, 0, 464, 500, 497,
            485, 476,
        ];
        let mut frontend = Frontend::new().unwrap();
        let mut output = None;
        for block in 0..3 {
            let samples = std::array::from_fn(|index| {
                let position = (block * AUDIO_BLOCK_SAMPLES + index) as f64;
                (10_000.0 * (2.0 * std::f64::consts::PI * 440.0 * position / 16_000.0).sin())
                    .round() as i16
            });
            output = frontend.process(&samples).unwrap();
        }
        assert_eq!(output.unwrap(), expected);
    }
}
