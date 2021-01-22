use {
    copernica_common::{LinkId, InterLinkPacket, HBFI},
    copernica_protocols::{Protocol},
    crate::protocol::{LOCD},
    std::{thread},
    crossbeam_channel::{Sender, Receiver, unbounded},
    copernica_identity::{PrivateIdentity},
    anyhow::{Result},
    //log::{debug},
};

#[derive(Clone, Debug)]
pub enum LOCDCommands {
    RequestFileList(HBFI),
    ResponseFileList(Option<Vec<String>>),
    RequestFile(HBFI, String),
    ResponseFile(Option<Vec<u8>>),
}

pub struct LOCDService {
    link_id: Option<LinkId>,
    p2c_tx: Option<Sender<LOCDCommands>>,
    c2p_rx: Option<Receiver<LOCDCommands>>,
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
        Ok(self.protocol.peer(link_id)?)
    }

    pub fn peer_with_client(&mut self)
    -> Result<(Sender<LOCDCommands>, Receiver<LOCDCommands>)> {
        let (c2p_tx, c2p_rx) = unbounded::<LOCDCommands>();
        let (p2c_tx, p2c_rx) = unbounded::<LOCDCommands>();
        self.p2c_tx = Some(p2c_tx);
        self.c2p_rx = Some(c2p_rx);
        Ok((c2p_tx, p2c_rx))
    }

    pub fn run(&mut self) -> Result<()>{
        let _rs = self.db.clone();
        let p2c_tx = self.p2c_tx.clone();
        let c2p_rx = self.c2p_rx.clone();
        let link_id = self.link_id.clone();
        let mut protocol = self.protocol.clone();
        protocol.run()?;
        thread::spawn(move || {
            if let (Some(c2p_rx), Some(p2c_tx), Some(_link_id)) = (c2p_rx, p2c_tx, link_id) {
                loop {
                    if let Ok(command) = c2p_rx.recv() {
                        match command {
                            LOCDCommands::RequestFileList(hbfi) => {
                                let files: Vec<String> = protocol.file_names(hbfi)?;
                                p2c_tx.send(LOCDCommands::ResponseFileList(Some(files.clone())))?;
                            },
                            LOCDCommands::RequestFile(hbfi, name) => {
                                let file: Vec<u8> = protocol.file(hbfi, name.clone())?;
                                p2c_tx.send(LOCDCommands::ResponseFile(Some(file)))?;
                            },
                            _ => {}
                        }
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }

}
