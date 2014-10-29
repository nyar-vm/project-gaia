/// 平台适配器模块
/// 
/// 该模块包含各个目标平台的具体适配器实现

pub mod pe_adapter;
pub mod dotnet_adapter;
pub mod jvm_adapter;
pub mod wasi_adapter;

pub use pe_adapter::PeAdapter;
pub use dotnet_adapter::DotNetAdapter;
pub use jvm_adapter::JvmAdapter;
pub use wasi_adapter::WasiAdapter;