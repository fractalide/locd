use {
    copernica_common::{LinkId, NarrowWaistPacket, LinkPacket, InterLinkPacket, HBFI, serialization::*, bloom_filter_index as bfi},
    copernica_protocols::{Protocol},
    copernica_identity::{PrivateIdentity},
    crossbeam_channel::{Sender, Receiver },
    std::{thread},
    sled::{Db},
    anyhow::{Result},
    log::{debug},
    rand::{
        distributions::Alphanumeric,
        thread_rng, Rng}
};
#[derive(Clone)]
pub struct LOCD {
    link_id: Option<LinkId>,
    rs: Db,
    l2p_rx: Option<Receiver<InterLinkPacket>>,
    p2l_tx: Option<Sender<InterLinkPacket>>,
    sid: PrivateIdentity,
}
impl<'a> LOCD {
    pub fn hashed_secret(&mut self, hbfi: HBFI) -> Result<Vec<u8>> {
        let hbfi = hbfi.clone().offset(0);
        let secret = self.get(hbfi.clone(), 0, 0)?;
        Ok(secret)
    }
}
impl<'a> Protocol<'a> for LOCD {
    fn new(rs: Db, sid: PrivateIdentity) -> LOCD {
        LOCD {
            sid,
            link_id: None,
            l2p_rx: None,
            p2l_tx: None,
            rs,
        }
    }
    fn run(&mut self) -> Result<()> {
        let rs = self.response_store();
        let l2p_rx = self.get_l2p_rx();
        let p2l_tx = self.get_p2l_tx();
        let link_id = self.get_link_id();
        let sid = self.get_sid();
        thread::spawn(move || {
            if let (Some(l2p_rx), Some(p2l_tx), Some(link_id)) = (l2p_rx, p2l_tx, link_id) {
                let res_me = bfi(&format!("{}", sid.public_id()))?;
                let app_me = bfi("locd")?;
                loop {
                    if let Ok(ilp) = l2p_rx.recv() {
                        debug!("\t\t\t|  link-to-protocol");
                        let nw: NarrowWaistPacket = ilp.narrow_waist();
                        match nw.clone() {
                            NarrowWaistPacket::Request { hbfi, .. } => {
                                match hbfi {
                                    HBFI { res, app, m0d, fun, arg, .. } if
                                    (res == res_me) &&
                                    (app == app_me) &&
                                    (m0d == bfi("htlc")?) &&
                                    (fun == bfi("generate_secret")?) &&
                                    (arg == bfi("shh")?) => {
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
                                        debug!("\t\t{:?}", hash);
                                        debug!("\t\t\t|  RESPONSE PACKET FOUND ENCRYPT IT");
                                        let nw = NarrowWaistPacket::response(sid.clone(), hbfi.clone(), hash.to_vec(), 0, 0)?;
                                        let lp = LinkPacket::new(link_id.reply_to()?, nw);
                                        let ilp = InterLinkPacket::new(link_id.clone(), lp);
                                        debug!("\t\t\t|  protocol-to-link");
                                        p2l_tx.send(ilp.clone())?;
                                    },
                                    _ => {}
                                }
                            },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                debug!("\t\t\t|  RESPONSE PACKET ARRIVED");
                                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                let (_, nw_s) = serialize_narrow_waist_packet(&nw)?;
                                rs.insert(hbfi_s, nw_s)?;
                            },
                        }
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
    fn response_store(&self) -> Db {
        self.rs.clone()
    }
    fn set_l2p_rx(&mut self, r: Receiver<InterLinkPacket>) {
        self.l2p_rx = Some(r);
    }
    fn get_l2p_rx(&mut self) -> Option<Receiver<InterLinkPacket>> {
        self.l2p_rx.clone()
    }
    fn set_p2l_tx(&mut self, s: Sender<InterLinkPacket>) {
        self.p2l_tx = Some(s);
    }
    fn get_p2l_tx(&mut self) -> Option<Sender<InterLinkPacket>> {
        self.p2l_tx.clone()
    }
    fn set_link_id(&mut self, link_id: LinkId) {
        self.link_id = Some(link_id);
    }
    fn get_link_id(&mut self) -> Option<LinkId> {
        self.link_id.clone()
    }
    fn get_sid(&mut self) -> PrivateIdentity {
        self.sid.clone()
    }
}

