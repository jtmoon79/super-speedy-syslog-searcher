// An error produced by an operation on LZXpress data
#[derive(Debug)]
pub enum Error {
    // Memory limit would be violated
    MemLimit,
    // Corrupt data
    CorruptedData,
    // An unknown error
    Other,
}
