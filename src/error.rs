#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    StackLocked,
    StackUnlocked,
    InvalidAnsi,
    TempError,
}
