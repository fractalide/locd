use {
    anyhow::{Result, anyhow},
    bincode,
    copernica_common::{
        bloom_filter_index as bfi,
        NarrowWaistPacket, HBFI, PrivateIdentityInterface, PublicIdentity,
        Operations, RequestPublicIdentity
    },
    copernica_protocols::{Protocol, TxRx},
    log::debug,
    rand::{distributions::Alphanumeric, thread_rng, Rng},
};
static APP_NAME: &str = "locd";
static MOD_HTLC: &str = "htlc";
static FUN_PEER: &str = "peer";
static FUN_PING: &str = "ping";
static ARG_PING: &str = "ping";
#[derive(Clone)]
pub struct LOCD {
    protocol_sid: PrivateIdentityInterface,
    txrx: TxRx,
    ops: Operations,
}
impl<'a> LOCD {
    pub fn cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        let hbfi = HBFI::new(RequestPublicIdentity::new(Some(self.txrx.protocol_public_id()?)), response_pid, APP_NAME, MOD_HTLC, FUN_PING, ARG_PING)?;
        let echo: Vec<Vec<u8>> = self.txrx.unreliable_unordered_request(hbfi.clone(), 0, 0)?;
        let mut result: String = "".into();
        for s in &echo {
            let data: &str = bincode::deserialize(&s)?;
            result.push_str(data);
        }
        Ok(result)
    }
}
impl<'a> Protocol<'a> for LOCD {
    fn new(protocol_sid: PrivateIdentityInterface, (label, ops): (String, Operations)) -> Self {
        ops.register_protocol(protocol_sid.public_id(), label);
        Self {
            protocol_sid,
            txrx: TxRx::Inert,
            ops,
        }
    }
    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let txrx = self.txrx.clone();
        std::thread::spawn(move || {
            match txrx {
                TxRx::Initialized {
                    ref unreliable_unordered_response_tx, .. } => {
                    let res_check = bfi(&format!("{}", txrx.protocol_public_id()?))?;
                    let app_check = bfi(APP_NAME)?;
                    let m0d_check = bfi(MOD_HTLC)?;
                    loop {
                        match txrx.clone().next() {
                            Ok(ilp) => {
                                debug!("\t\t\t|  link-to-protocol");
                                let nw: NarrowWaistPacket = ilp.narrow_waist();
                                match nw.clone() {
                                    NarrowWaistPacket::Request { hbfi, .. } => match hbfi {
                                        HBFI { res, app, m0d, .. }
                                            if (res == res_check)
                                                && (app == app_check)
                                                && (m0d == m0d_check) =>
                                        {
                                            match hbfi {
                                                HBFI { fun, arg, .. }
                                                    if (fun == bfi(FUN_PING)?)
                                                        && (arg == bfi(ARG_PING)?) =>
                                                {
                                                    let  echo: Vec<u8> = bincode::serialize(&"ping")?;
                                                    txrx.clone().respond(hbfi.clone(), echo)?;
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    },
                                    NarrowWaistPacket::Response { hbfi, .. } => match hbfi {
                                        HBFI { app, m0d, fun, arg, .. }
                                            if (app == app_check)
                                                && (m0d == m0d_check)
                                                && (fun == bfi(FUN_PING)?)
                                            => {
                                                match arg {
                                                    arg if arg == bfi(ARG_PING)? => {
                                                        debug!("\t\t\t|  RESPONSE PACKET ARRIVED");
                                                        unreliable_unordered_response_tx.send(ilp)?;
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        _ => {}
                                    }
                                }
                            },
                            Err(_e) => {}
                        }
                    }
                },
                TxRx::Inert => panic!("{}", anyhow!("You must peer with a link first")),
            };
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
    fn set_txrx(&mut self, txrx: TxRx) {
        self.txrx = txrx;
    }
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface {
        self.protocol_sid.clone()
    }
    fn get_ops(&self) -> Operations {
        self.ops.clone()
    }
}
fn secret_generate() -> (String, [u8; 32]) {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    use cryptoxide::digest::Digest as _;
    let mut hash = [0; 32];
    let mut b = cryptoxide::blake2b::Blake2b::new(32);
    b.input(&rand_string.as_bytes());
    b.result(&mut hash);
    (rand_string, hash)
}
fn amount() -> (String, [u8; 32]) {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    use cryptoxide::digest::Digest as _;
    let mut hash = [0; 32];
    let mut b = cryptoxide::blake2b::Blake2b::new(32);
    b.input(&rand_string.as_bytes());
    b.result(&mut hash);
    (rand_string, hash)
}
