use {
    copernica_common::{PublicIdentity},
};
#[derive(Debug)]
pub enum Request {
    Secret(PublicIdentity),
    //Amount
}

#[derive(Debug)]
pub enum Response {
    Secret(Vec<u8>),
    //Amount(f64),
}
