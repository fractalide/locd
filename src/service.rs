use {
    copernica_common::{LinkId, InterLinkPacket, PrivateIdentityInterface, PublicIdentity, Operations},
    copernica_protocols::{Protocol},
    crate::{
        protocol::{LOCD},
    },
    std::sync::mpsc::{Receiver, SyncSender as Sender},
    anyhow::{Result},
    //log::{debug, error},
};
pub struct LOCDService {
    link_id: Option<LinkId>,
    protocol: LOCD,
    sid: PrivateIdentityInterface,
}
impl LOCDService {
    pub fn new(sid: PrivateIdentityInterface, ops: (String, Operations)) -> Self {
        let protocol: LOCD = Protocol::new(sid.clone(), ops);
        Self {
            link_id: None,
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
    pub fn ping(&mut self, identity: PublicIdentity) -> Result<String> {
        self.protocol.cyphertext_ping(identity)
    }
    pub fn run(&mut self) -> Result<()> {
        self.protocol.run()?;
        Ok(())
    }
}
