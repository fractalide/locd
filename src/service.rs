use {
    copernica_common::{LinkId, InterLinkPacket, HBFI},
    copernica_protocols::{Protocol},
    crate::{
        protocol::{LOCD},
        signals::{Signals},
    },
    std::{thread},
    crossbeam_channel::{Sender, Receiver, unbounded},
    copernica_identity::{PrivateIdentity},
    anyhow::{Result},
    //log::{debug},
};
pub struct LOCDService {
    link_id: Option<LinkId>,
    p2c_tx: Option<Sender<(HBFI, Signals)>>,
    c2p_rx: Option<Receiver<(HBFI, Signals)>>,
    db: sled::Db,
    protocol: LOCD,
    response_sid: PrivateIdentity,
}
impl LOCDService {
    pub fn new(db: sled::Db, response_sid: PrivateIdentity) -> Self {
        let protocol: LOCD = Protocol::new(db.clone(), response_sid.clone());
        Self {
            link_id: None,
            p2c_tx: None,
            c2p_rx: None,
            db,
            protocol,
            response_sid,
        }
    }
    pub fn peer_with_link(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        self.link_id = Some(link_id.clone());
        Ok(self.protocol.peer_with_link(link_id)?)
    }
    pub fn peer_with_client(&mut self)
    -> Result<(Sender<(HBFI, Signals)>, Receiver<(HBFI, Signals)>)> {
        let (c2p_tx, c2p_rx) = unbounded::<(HBFI, Signals)>();
        let (p2c_tx, p2c_rx) = unbounded::<(HBFI, Signals)>();
        self.p2c_tx = Some(p2c_tx);
        self.c2p_rx = Some(c2p_rx);
        Ok((c2p_tx, p2c_rx))
    }
    pub fn run(&mut self) -> Result<()>{
        //use HTLC::{Variant::*, *};
        let p2c_tx = self.p2c_tx.clone();
        let c2p_rx = self.c2p_rx.clone();
        let link_id = self.link_id.clone();
        let mut protocol = self.protocol.clone();
        protocol.run()?;
        thread::spawn(move || {
            if let (Some(c2p_rx), Some(p2c_tx), Some(_link_id)) = (c2p_rx, p2c_tx, link_id) {
                loop {
                    if let Ok((hbfi, command)) = c2p_rx.recv() {
                        match command {
                            Signals::RequestSecret => {
                                let hashed_secret = protocol.hashed_secret(hbfi.clone())?;
                                p2c_tx.send((hbfi, Signals::ResponseSecret(hashed_secret)))?;
                            },
                            _ => {},
                        }

                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
}
