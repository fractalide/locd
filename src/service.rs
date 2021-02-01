use {
    copernica_common::{LinkId, InterLinkPacket, HBFI},
    copernica_protocols::{Protocol},
    crate::{
        protocol::{LOCD},
        signals::{Request, Response},
    },
    std::{thread},
    crossbeam_channel::{Sender, Receiver, unbounded},
    copernica_identity::{PrivateIdentity},
    anyhow::{Result},
    //log::{debug},
};
pub struct LOCDService {
    link_id: Option<LinkId>,
    c2p_rx: Option<Receiver<(HBFI, Request)>>,
    p2c_tx: Option<Sender<(HBFI, Response)>>,
    db: sled::Db,
    protocol: LOCD,
    sid: PrivateIdentity,
}
impl LOCDService {
    pub fn new(db: sled::Db, sid: PrivateIdentity) -> Self {
        let protocol: LOCD = Protocol::new(db.clone(), sid.clone());
        Self {
            link_id: None,
            p2c_tx: None,
            c2p_rx: None,
            db,
            protocol,
            sid,
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
    -> Result<(Sender<(HBFI, Request)>, Receiver<(HBFI, Response)>)> {
        let (c2p_tx, c2p_rx) = unbounded::<(HBFI, Request)>();
        let (p2c_tx, p2c_rx) = unbounded::<(HBFI, Response)>();
        self.p2c_tx = Some(p2c_tx);
        self.c2p_rx = Some(c2p_rx);
        Ok((c2p_tx, p2c_rx))
    }
    pub fn run(&mut self) -> Result<()>{
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
                            Request::Secret => {
                                let hashed_secret = protocol.hashed_secret(hbfi.clone())?;
                                p2c_tx.send((hbfi, Response::Secret(hashed_secret)))?;
                            },
                        }

                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
}
