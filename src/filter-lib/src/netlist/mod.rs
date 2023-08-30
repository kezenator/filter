mod device;
mod netlist;
mod nodename;
mod parser;
mod value;

pub type Scalar = f64;

pub use device::Device;
pub use netlist::Netlist;
pub use nodename::NodeName;
pub use parser::NetlistParseError;
pub use value::Value;