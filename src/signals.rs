#[derive(Debug)]
pub enum Signals {
    RequestSecret,
    ResponseSecret(Vec<u8>),
}
