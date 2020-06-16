use libsamplerate::*;

pub fn version() -> &'static str {
    let cstr = unsafe {
        std::ffi::CStr::from_ptr(samplerate::src_get_version())
    };
    cstr.to_str().unwrap() // assume libsamplerate uses valid utf-8
}

#[derive(Debug)]
pub struct SampleRate<A> {
    state: *mut SRC_STATE,
    // we have the same variance as the "process" method, A -> A
    _marker: std::marker::PhantomData<fn(A) -> A>,
}

impl<A> Clone for SampleRate<A> {
    fn clone(&self) -> Self {
        self.try_clone().unwrap() // this will *usually* work, barring OOM
    }
}

// implement this manually -- it's not automatic, because of the *mut SRC_STATE
// however, the way we use it, this is safe
unsafe impl<A> Send for SampleRate<A> {}

// states that Self is memory-identical to [f32; channels]
pub unsafe trait Resample: Clone {
    fn channels() -> usize;
}

impl<A> SampleRate<A> where A: Resample {
    pub fn new(typ: ConverterType) -> Result<Self> {
        let mut errcode = 0;
        let state = unsafe {
            let staten = samplerate::src_new(
                typ.to_c(),
                A::channels() as i32,
                &mut errcode,
            );
            Error::result(staten, errcode)
        }?;
        Ok(SampleRate { state, _marker: std::marker::PhantomData })
    }

    pub fn process(&mut self, ratio: f64, input: &[A], output: &mut Vec<A>)
                   -> Result<usize>
    {
        let mut cmd = samplerate::SRC_DATA {
            data_in: input.as_ptr() as *const f32,
            data_out: output.as_mut_ptr() as *mut f32,
            input_frames: input.len() as i32,
            output_frames: output.capacity() as i32,
            input_frames_used: 0,
            output_frames_gen: 0,
            end_of_input: if input.len() == 0 { 1 } else { 0 },
            src_ratio: ratio,
        };

        unsafe {
            let errcode = samplerate::src_process(self.state, &mut cmd);
            Error::result((), errcode)?;
            output.set_len(cmd.output_frames_gen as usize);
        }

        Ok(cmd.input_frames_used as usize)
    }
}

impl<A> SampleRate<A> {
    pub fn reset(&mut self) -> Result<()> {
        unsafe {
            let errcode = samplerate::src_reset(self.state);
            Error::result((), errcode)
        }
    }

    pub fn try_clone(&self) ->  Result<Self> {
        let state = unsafe {
            let mut errcode = 0;
            let staten = samplerate::src_clone(self.state, &mut errcode);
            Error::result(staten, errcode)
        }?;
        Ok(SampleRate { state, _marker: std::marker::PhantomData })
    }

    pub fn channels(&self) -> usize {
        unsafe {
            samplerate::src_get_channels(self.state) as usize
        }
    }

    pub fn set_ratio(&mut self, ratio: f64) -> Result<()> {
        unsafe {
            let errcode = samplerate::src_set_ratio(self.state, ratio);
            Error::result((), errcode)
        }
    }
}

impl<A> Drop for SampleRate<A> {
    fn drop(&mut self) {
        if !self.state.is_null() {
            unsafe {
                self.state = samplerate::src_delete(self.state);
            }
        }

    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConverterType {
    SincBestQuality,
    SincMediumQuality,
    SincFastest,
    ZeroOrderHold,
    Linear,
}

impl ConverterType {
    pub fn name(&self) -> &'static str {
        let code = self.to_c() as i32;
        let cstr = unsafe {
            std::ffi::CStr::from_ptr(samplerate::src_get_name(code))
        };
        cstr.to_str().unwrap() // assume libsamplerate uses valid utf-8
    }

    pub fn description(&self) -> &'static str {
        let code = self.to_c() as i32;
        let cstr = unsafe {
            std::ffi::CStr::from_ptr(samplerate::src_get_description(code))
        };
        cstr.to_str().unwrap() // assume libsamplerate uses valid utf-8
    }

    fn to_c(&self) -> i32 {
        use ConverterType::*;
        let code = match self {
            SincBestQuality => src_sinc::SRC_SINC_BEST_QUALITY,
            SincMediumQuality => src_sinc::SRC_SINC_MEDIUM_QUALITY,
            SincFastest => src_sinc::SRC_SINC_FASTEST,
            ZeroOrderHold => src_sinc::SRC_ZERO_ORDER_HOLD,
            Linear => src_sinc::SRC_LINEAR,
        };
        code as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    BadCallback,
    BadChannelCount,
    BadConverter,
    BadData,
    BadDataPtr,
    BadInternalState,
    BadMode,
    BadPrivPtr,
    BadProcPtr,
    BadSincState,
    BadSrcRatio,
    BadState,
    DataOverlap,
    FilterLen,
    MallocFailed,
    NoPrivate,
    NoVariableRatio,
    NullCallback,
    ShiftBits,
    SincBadBufferLen,
    SincPrepareDataBadLen,
    SizeIncompatibility,
    Unknown(u32),
}

type Result<A> = std::result::Result<A, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.description()
    }
}

impl Error {
    pub fn description(&self) -> &'static str {
        let code = self.to_c() as i32;
        let cstr = unsafe {
            std::ffi::CStr::from_ptr(samplerate::src_strerror(code))
        };
        cstr.to_str().unwrap() // assume libsamplerate uses valid utf-8
    }

    fn result<A>(result: A, code: i32) -> Result<A> {
        if let Some(err) = Self::from_c(code) {
            Err(err)
        } else {
            Ok(result)
        }
    }

    fn to_c(&self) -> i32 {
        use Error::*;
        let code = match self {
            BadCallback => samplerate::SRC_ERR_BAD_CALLBACK,
            BadChannelCount => samplerate::SRC_ERR_BAD_CHANNEL_COUNT,
            BadConverter => samplerate::SRC_ERR_BAD_CONVERTER,
            BadData => samplerate::SRC_ERR_BAD_DATA,
            BadDataPtr => samplerate::SRC_ERR_BAD_DATA_PTR,
            BadInternalState => samplerate::SRC_ERR_BAD_INTERNAL_STATE,
            BadMode => samplerate::SRC_ERR_BAD_MODE,
            BadPrivPtr => samplerate::SRC_ERR_BAD_PRIV_PTR,
            BadProcPtr => samplerate::SRC_ERR_BAD_PROC_PTR,
            BadSincState => samplerate::SRC_ERR_BAD_SINC_STATE,
            BadSrcRatio => samplerate::SRC_ERR_BAD_SRC_RATIO,
            BadState => samplerate::SRC_ERR_BAD_STATE,
            DataOverlap => samplerate::SRC_ERR_DATA_OVERLAP,
            FilterLen => samplerate::SRC_ERR_FILTER_LEN,
            MallocFailed => samplerate::SRC_ERR_MALLOC_FAILED,
            NoPrivate => samplerate::SRC_ERR_NO_PRIVATE,
            NoVariableRatio => samplerate::SRC_ERR_NO_VARIABLE_RATIO,
            NullCallback => samplerate::SRC_ERR_NULL_CALLBACK,
            ShiftBits => samplerate::SRC_ERR_SHIFT_BITS,
            SincBadBufferLen => samplerate::SRC_ERR_SINC_BAD_BUFFER_LEN,
            SincPrepareDataBadLen => samplerate::SRC_ERR_SINC_PREPARE_DATA_BAD_LEN,
            SizeIncompatibility => samplerate::SRC_ERR_SIZE_INCOMPATIBILITY,
            Unknown(err) => *err,
        };
        code as i32
    }

    fn from_c(err: i32) -> Option<Self> {
        use Error::*;
        match err as u32 {
            0 => None,
            samplerate::SRC_ERR_BAD_CALLBACK => Some(BadCallback),
            samplerate::SRC_ERR_BAD_CHANNEL_COUNT => Some(BadChannelCount),
            samplerate::SRC_ERR_BAD_CONVERTER => Some(BadConverter),
            samplerate::SRC_ERR_BAD_DATA => Some(BadData),
            samplerate::SRC_ERR_BAD_DATA_PTR => Some(BadDataPtr),
            samplerate::SRC_ERR_BAD_INTERNAL_STATE => Some(BadInternalState),
            samplerate::SRC_ERR_BAD_MODE => Some(BadMode),
            samplerate::SRC_ERR_BAD_PRIV_PTR => Some(BadPrivPtr),
            samplerate::SRC_ERR_BAD_PROC_PTR => Some(BadProcPtr),
            samplerate::SRC_ERR_BAD_SINC_STATE => Some(BadSincState),
            samplerate::SRC_ERR_BAD_SRC_RATIO => Some(BadSrcRatio),
            samplerate::SRC_ERR_BAD_STATE => Some(BadState),
            samplerate::SRC_ERR_DATA_OVERLAP => Some(DataOverlap),
            samplerate::SRC_ERR_FILTER_LEN => Some(FilterLen),
            samplerate::SRC_ERR_MALLOC_FAILED => Some(MallocFailed),
            samplerate::SRC_ERR_NO_PRIVATE => Some(NoPrivate),
            samplerate::SRC_ERR_NO_VARIABLE_RATIO => Some(NoVariableRatio),
            samplerate::SRC_ERR_NULL_CALLBACK => Some(NullCallback),
            samplerate::SRC_ERR_SHIFT_BITS => Some(ShiftBits),
            samplerate::SRC_ERR_SINC_BAD_BUFFER_LEN => Some(SincBadBufferLen),
            samplerate::SRC_ERR_SINC_PREPARE_DATA_BAD_LEN => Some(SincPrepareDataBadLen),
            samplerate::SRC_ERR_SIZE_INCOMPATIBILITY => Some(SizeIncompatibility),
            other => Some(Unknown(other)),
        }
    }
}

unsafe impl Resample for f32 {
    fn channels() -> usize { 1 }
}

unsafe impl<F> Resample for num::Complex<F> where F: Resample {
    fn channels() -> usize { 2 * F::channels() }
}
