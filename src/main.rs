#![allow(dead_code)]
#![feature(bindings_after_at)]
mod service;
mod protocol;
mod signals;
use {
    anyhow::{Result},
    crate::{
        service::{LOCDService},
    },
    copernica_broker::{Broker},
    copernica_common::{LinkId, ReplyTo, PrivateIdentityInterface, Operations},
    copernica_links::{Link, MpscChannel, MpscCorruptor, UdpIp},
    log::{debug},
};

pub fn main() -> Result<()> {
    copernica_common::setup_logging(3, None).unwrap();
    let ops = Operations::turned_off();
    let mut broker0 = Broker::new(ops.clone());
    let mut broker1 = Broker::new(ops.clone());
    let locd_service_sid0 = PrivateIdentityInterface::new_key();
    let locd_service_sid1 = PrivateIdentityInterface::new_key();
    let locd_service_sid2 = PrivateIdentityInterface::new_key();
    let mut locd_service0: LOCDService = LOCDService::new(locd_service_sid0.clone(), ops.label("service0"));
    let mut locd_service1: LOCDService = LOCDService::new(locd_service_sid1.clone(), ops.label("service1"));
    let mut locd_service2: LOCDService = LOCDService::new(locd_service_sid2.clone(), ops.label("service2"));

    // locd_service0 to broker0
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), None, ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), None, ReplyTo::Mpsc);
    let mut link0: MpscChannel = Link::new(link_id0.clone(), ops.label("link0"), broker0.peer_with_link(link_id0.clone())?)?;
    let mut link1: MpscChannel = Link::new(link_id1.clone(), ops.label("link1"), locd_service0.peer_with_link(link_id0.clone())?)?;
    link0.female(link1.male());
    link1.female(link0.male());

    // broker0 to broker1
    let link_sid2 = PrivateIdentityInterface::new_key();
    let link_sid3 = PrivateIdentityInterface::new_key();
    let link_id2 = LinkId::link_with_type(link_sid2.clone(), Some(link_sid3.public_id()), ReplyTo::Mpsc);
    let link_id3 = LinkId::link_with_type(link_sid3.clone(), Some(link_sid2.public_id()), ReplyTo::Mpsc);
    let mut link2: MpscCorruptor = Link::new(link_id2.clone(), ops.label("link2"), broker0.peer_with_link(link_id2.clone())?)?;
    let mut link3: MpscCorruptor = Link::new(link_id3.clone(), ops.label("link3"), broker1.peer_with_link(link_id3.clone())?)?;
    link2.female(link3.male());
    link3.female(link2.male());

    // broker1 to locd_service1
    let link_sid4 = PrivateIdentityInterface::new_key();
    let link_sid5 = PrivateIdentityInterface::new_key();
    let address4 = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let address5 = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let link_id4 = LinkId::link_with_type(link_sid4.clone(), Some(link_sid5.public_id()), address4.clone());
    let link_id5 = LinkId::link_with_type(link_sid5.clone(), Some(link_sid4.public_id()), address5.clone());
    let link4: UdpIp = Link::new(link_id4.clone(), ops.label("link4"), broker1.peer_with_link(link_id4.remote(address5)?)?)?;
    let link5: UdpIp = Link::new(link_id5.clone(), ops.label("link5"), locd_service1.peer_with_link(link_id5.remote(address4)?)?)?;

    // broker1 to locd_service2
    let link_sid6 = PrivateIdentityInterface::new_key();
    let link_sid7 = PrivateIdentityInterface::new_key();
    let address6 = ReplyTo::UdpIp("127.0.0.1:50004".parse()?);
    let address7 = ReplyTo::UdpIp("127.0.0.1:50005".parse()?);
    let link_id6 = LinkId::link_with_type(link_sid6.clone(), Some(link_sid7.public_id()), address6.clone());
    let link_id7 = LinkId::link_with_type(link_sid7.clone(), Some(link_sid6.public_id()), address7.clone());
    let link6: UdpIp = Link::new(link_id6.clone(), ops.label("link6"), broker1.peer_with_link(link_id6.remote(address7)?)?)?;
    let link7: UdpIp = Link::new(link_id7.clone(), ops.label("link7"), locd_service2.peer_with_link(link_id7.remote(address6)?)?)?;

    locd_service0.run()?;
    link0.run()?;
    link1.run()?;
    broker0.run()?;
    link2.run()?;
    link3.run()?;
    broker1.run()?;
    link4.run()?;
    link5.run()?;
    link6.run()?;
    link7.run()?;
    locd_service1.run()?;
    locd_service2.run()?;
/*
    debug!("unreliable unordered cleartext ping");
    let pong: String = locd_service1.ping(locd_service_sid0.public_id())?;
    debug!("unreliable unordered cleartext {:?}", pong);
*/
    debug!("unreliable unordered cyphertext ping");
    let pong: String = locd_service0.ping(locd_service_sid1.public_id())?;
    debug!("unreliable unordered cyphertext {:?}", pong);

    Ok(())
}
