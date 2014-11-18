mod to_program;
mod to_wasm;

#[derive(Copy, Clone, Debug, Default)]
pub struct WasmView {
    pub magic_head: [u8; 4],
}
