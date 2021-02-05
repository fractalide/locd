use {
    anyhow::{Result, anyhow},
    bincode,
    copernica_common::{
        bloom_filter_index as bfi, serialization::*, InterLinkPacket, LinkId, LinkPacket,
        NarrowWaistPacket, HBFI,
    },
    copernica_identity::{PrivateIdentity, PublicIdentity, Signature},
    copernica_protocols::{Protocol, TxRx},
    crossbeam_channel::{Receiver, Sender},
    log::debug,
    rand::{distributions::Alphanumeric, thread_rng, Rng},
    sled::Db,
    sm::sm,
    std::collections::HashMap,
    std::thread,
    serde::{Deserialize, Serialize},
};
sm! {
    LocdStateMachine {
        InitialStates { AZero, BZero}
        SendSecret {
            AZero => AOne
        }
        SendAmount {
            BZero => BOne
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ContractDetails {
    address: u64,
    funding_amount: u64,
    donation_request: u64,
    signature: Signature,
    initiator_address: Option<u64>,
    initiator_funding_amount: Option<u64>,
    initiator_donation_amount: Option<u64>,
    initiator_signature: Option<Signature>,
}
impl ContractDetails {
    pub fn new(address: u64, funding_amount: u64, donation_request: u64, sid: PrivateIdentity) -> Self {
        let manifest = [u64_to_u8(address).to_vec(), u64_to_u8(funding_amount).to_vec(), u64_to_u8(donation_request).to_vec()].concat();
        let signing_key = sid.signing_key();
        let signature = signing_key.sign(manifest);
        ContractDetails {
            address,
            funding_amount,
            donation_request,
            signature,
            initiator_address: None,
            initiator_funding_amount: None,
            initiator_donation_amount: None, initiator_signature: None,
        }
    }
    pub fn donation_request(&self) -> u64 {
        self.donation_request
    }
    pub fn address(&self) -> u64 {
        self.address
    }
}
#[derive(Clone)]
pub struct LOCD {
    txrx: Option<TxRx>,
}
impl<'a> LOCD {
    pub fn contract_details(&mut self, response_pid: PublicIdentity) -> Result<ContractDetails> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.sid.public_id()), response_pid, "locd", "htlc", "peer", "request_contract_details")?;
            let details = txrx.request(hbfi.clone(), 0, 0)?;
            let details: ContractDetails = bincode::deserialize(&details)?;
            Ok(details)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn hashed_secret(&mut self, response_pid: PublicIdentity) -> Result<Vec<u8>> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.sid.public_id()), response_pid, "locd", "htlc", "secret", "generate")?;
            let secret = txrx.request(hbfi.clone(), 0, 0)?;
            Ok(secret)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn contract_counter_offer(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.sid.public_id()), response_pid, "locd", "htlc", "peer", "contract_counter_offer")?;
            let address = txrx.request(hbfi.clone(), 0, 0)?;
            let address: String = bincode::deserialize(&address)?;
            Ok(address)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
}
impl<'a> Protocol<'a> for LOCD {
    fn new() -> LOCD {
        LOCD {
            txrx: None,
        }
    }
    fn run(&mut self) -> Result<()> {
        let txrx = self.get_txrx();
        thread::spawn(move || {
            if let Some(txrx) = txrx {
                use LocdStateMachine::*;
                let mut sms = HashMap::new();
                let res_check = bfi(&format!("{}", txrx.sid.clone().public_id()))?;
                let app_check = bfi("locd")?;
                let m0d_check = bfi("htlc")?;
                loop {
                    if let Ok(ilp) = txrx.l2p_rx.recv() {
                        debug!("\t\t\t|  link-to-protocol");
                        let nw: NarrowWaistPacket = ilp.narrow_waist();
                        match nw.clone() {
                            NarrowWaistPacket::Request { hbfi, .. } => match hbfi {
                                HBFI { res, app, m0d, .. }
                                    if (res == res_check)
                                        && (app == app_check)
                                        && (m0d == m0d_check) =>
                                {
                                    if let Some(request_pid) = hbfi.request_pid.clone() {
                                        match hbfi {
                                            HBFI { fun, arg, .. }
                                                if (fun == bfi("peer")?)
                                                    && (arg == bfi("request_contract_details")?) =>
                                            {
                                                let mut rng = rand::thread_rng();
                                                let address: u64 = rng.gen::<u64>();
                                                let funding_amount: u64 = rng.gen_range(10_000, 1_000_000);
                                                let donation_request: u64 = rng.gen_range(0, 1000);
                                                let details = ContractDetails::new(address, funding_amount, donation_request, txrx.sid.clone());
                                                let details: Vec<u8> = bincode::serialize(&details)?;
                                                txrx.respond(hbfi.clone(), details)?;
                                            }
                                            HBFI { fun, arg, .. }
                                                if (fun == bfi("address")?)
                                                    && (arg == bfi("get")?) =>
                                            {
                                                let sm = LocdStateMachine::Machine::new(BZero);
                                                sms.insert(request_pid.clone(), sm.as_enum());
                                                let state = sms.get(&request_pid);
                                                let mut rng = rand::thread_rng();
                                                let number: u64 = rng.gen::<u64>();
                                                let number: Vec<u8> = bincode::serialize(&number)?;
                                                txrx.respond(hbfi.clone(), number)?;
                                            }
                                            HBFI { fun, arg, .. }
                                                if (fun == bfi("peer")?)
                                                    && (arg == bfi("contract_counter_offer")?) =>
                                            {
                                                let ack = String::from("ack");
                                                txrx.respond(hbfi.clone(), bincode::serialize(&ack)?)?;
                                                let hbfi1 = HBFI::new(Some(txrx.sid.public_id()), request_pid, "locd", "htlc", "peer", "request_for_request")?;
                                                let response = txrx.request(hbfi1, 0, 0)?;
                                                println!("Hello 1 {:?}", response);
                                                let baby: String = bincode::deserialize(&response)?;
                                                println!("Hello {:?}", baby);
                                            }
                                            HBFI { fun, arg, .. }
                                                if (fun == bfi("peer")?)
                                                    && (arg == bfi("request_for_request")?) =>
                                            {
                                                println!("I'm here");
                                                let baby = String::from("baby");
                                                println!("I'm here");
                                                let baby: Vec<u8> = bincode::serialize(&baby)?;
                                                println!("I'm here {:?}", baby);
                                                txrx.respond(hbfi, baby)?;
                                                println!("I'm here");
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                debug!("\t\t\t|  RESPONSE PACKET ARRIVED");
                                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                let (_, nw_s) = serialize_narrow_waist_packet(&nw)?;
                                txrx.db.insert(hbfi_s, nw_s)?;
                            }
                        }
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
    fn set_txrx(&mut self, txrx: TxRx) {
        self.txrx = Some(txrx);
    }
    fn get_txrx(&mut self) -> Option<TxRx> {
        self.txrx.clone()
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
