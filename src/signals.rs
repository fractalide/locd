#[derive(Debug)]
pub enum Request {
    Secret,
}

#[derive(Debug)]
pub enum Response {
    Secret(Vec<u8>),
}

