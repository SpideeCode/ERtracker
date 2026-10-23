//! Oodle Kraken decompression through the game's own oo2core_6_win64.dll.
//! The DLL is never shipped with ERtracker: it is loaded from the player's install.

use super::{ExtractError, Result};
use std::path::Path;
use std::sync::OnceLock;

type DecompressFn = unsafe extern "C" fn(
    src: *const u8,
    src_len: isize,
    dst: *mut u8,
    dst_len: isize,
    fuzz_safe: i32,
    check_crc: i32,
    verbosity: i32,
    dec_buf_base: *mut u8,
    dec_buf_size: isize,
    callback: *const u8,
    callback_user: *const u8,
    decoder_memory: *mut u8,
    decoder_memory_size: isize,
    thread_phase: i32,
) -> isize;

static LIB: OnceLock<std::result::Result<libloading::Library, String>> = OnceLock::new();

/// Must be called once, with the game's `Game` folder, before any Kraken DCX is read.
pub fn init(game_dir: &Path) -> Result<()> {
    let lib = LIB.get_or_init(|| {
        let path = game_dir.join("oo2core_6_win64.dll");
        // SAFETY: loading the game's own Oodle DLL; it has no initialisation side effects.
        unsafe { libloading::Library::new(&path) }.map_err(|e| format!("cannot load {}: {e}", path.display()))
    });
    lib.as_ref().map(|_| ()).map_err(|e| ExtractError::new(e.clone()))
}

pub fn decompress(src: &[u8], size: usize) -> Result<Vec<u8>> {
    let lib = LIB
        .get()
        .and_then(|l| l.as_ref().ok())
        .ok_or_else(|| ExtractError::new("Oodle is not loaded"))?;
    // SAFETY: the symbol has the OodleLZ_Decompress signature in every Oodle 2.6 release.
    let decompress: libloading::Symbol<DecompressFn> =
        unsafe { lib.get(b"OodleLZ_Decompress\0") }.map_err(|e| ExtractError::new(e.to_string()))?;
    let mut out = vec![0u8; size];
    // SAFETY: both buffers are valid for the lengths passed; no callbacks or decoder memory.
    let written = unsafe {
        decompress(
            src.as_ptr(),
            src.len() as isize,
            out.as_mut_ptr(),
            size as isize,
            1,
            0,
            0,
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
            0,
            3,
        )
    };
    if written as usize != size {
        return Err(ExtractError::new(format!("Oodle returned {written}, expected {size}")));
    }
    Ok(out)
}
