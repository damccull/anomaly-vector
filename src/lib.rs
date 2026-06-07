// TODO: Single top-level struct offering simplified API for the library
// 1. Create a top level struct in the library
// 2. Impl all the functions on it privately or make them private
// 3. Provide public API of 2 fns:
//    - initialize() that does all the work and store offset+len of encryption key
//    - get_key() that returns a slice from the offset+len stored by initialize
//
//
//
pub mod error;
pub mod telemetry;

mod nms_key_extractor;

pub use nms_key_extractor::NmsKeyExtractor;
