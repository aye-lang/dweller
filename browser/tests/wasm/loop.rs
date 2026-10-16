// A loop to compare Dweller's WebAssembly (wasmtime, Cranelift) with
// node's (V8) on the same module (P11.17): integer hashing and float
// math over a small table in linear memory. Built with
//
//     rustc --target wasm32-unknown-unknown -C opt-level=3 --crate-type cdylib loop.rs -o loop.wasm
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

static mut TABLE: [f64; 1024] = [0.0; 1024];

/// `n` steps; the result is a checksum that does not depend on the
/// runtime.
#[no_mangle]
pub extern "C" fn run(n: u32) -> f64 {
    let mut s: u32 = 1;
    let mut acc = 0.0f64;
    unsafe {
        let t = &mut *core::ptr::addr_of_mut!(TABLE);
        for i in 0..1024 {
            t[i] = i as f64 * 0.5;
        }
        for i in 0..n {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            let k = (s >> 22) as usize;
            let v = t[k] * 0.999 + (i & 7) as f64;
            t[k] = v;
            acc += v / (1.0 + (k as f64));
        }
    }
    acc
}
