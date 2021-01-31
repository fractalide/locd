#![allow(dead_code)]
mod service;
mod protocol;
mod signals;
mod lut;
use {
    anyhow::{Result},
    sled,
    std::{
        io::prelude::*,
        fs,
    },
    crate::{
        service::{LOCDService},
    },
    copernica_broker::{Broker},
    copernica_common::{HBFI, LinkId, ReplyTo},
    copernica_links::{Link, MpscChannel,  UdpIp},
    copernica_identity::{PrivateIdentity, Seed},
    copernica_tests::{populate_tmp_dir, TestData, generate_random_dir_name},
    log::{debug},
    signals::{Signals},
};

pub fn main() -> Result<()> {
    copernica_common::setup_logging(3, None).unwrap();

    let mut rng = rand::thread_rng();
    let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));

    let rs0 = sled::open(generate_random_dir_name())?;
    let rs1 = sled::open(generate_random_dir_name())?;
    let brs = sled::open(generate_random_dir_name())?;

    let mut b = Broker::new(brs);
    let mut locd0 = LOCDService::new(rs0, response_sid.clone());
    let mut locd1 = LOCDService::new(rs1, request_sid.clone());

    let lnk_sid0 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid1 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid2 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid3 = PrivateIdentity::from_seed(Seed::generate(&mut rng));

    let mpscv_b_id = LinkId::listen(lnk_sid0.clone(), Some(lnk_sid1.public_id()), ReplyTo::Mpsc);
    let mpsc_vb_id = LinkId::listen(lnk_sid1.clone(), Some(lnk_sid0.public_id()), ReplyTo::Mpsc);
    //let mpscv_b_id = LinkId::listen(lnk_sid0.clone(), None, ReplyTo::Mpsc);
    //let mpsc_vb_id = LinkId::listen(lnk_sid1.clone(), None, ReplyTo::Mpsc);
    let mut mpscv_b_link: MpscChannel = Link::new("lv_b".into(), mpscv_b_id.clone(), locd0.peer_with_link(mpscv_b_id)?)?;
    let mut mpsc_vb_link: MpscChannel = Link::new("l_vb".into(), mpsc_vb_id.clone(), b.peer(mpsc_vb_id)?)?;
    mpscv_b_link.female(mpsc_vb_link.male());
    mpsc_vb_link.female(mpscv_b_link.male());

    let locd_vb_address = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let locdv_b_address = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let locd_vb_id = LinkId::listen(lnk_sid2.clone(), Some(lnk_sid3.public_id()), locd_vb_address.clone());
    let locdv_b_id = LinkId::listen(lnk_sid3.clone(), Some(lnk_sid2.public_id()), locdv_b_address.clone());
    //let locd_vb_id = LinkId::listen(lnk_sid2.clone(), None, locd_vb_address.clone());
    //let locdv_b_id = LinkId::listen(lnk_sid3.clone(), None, locdv_b_address.clone());
    let locd_vb_link: UdpIp = Link::new("l_vb".into(), locd_vb_id.clone(), b.peer(locd_vb_id.remote(locdv_b_address)?)?)?;
    let locdv_b_link: UdpIp = Link::new("lv_b".into(), locdv_b_id.clone(), locd1.peer_with_link(locdv_b_id.remote(locd_vb_address)?)?)?;

    let links: Vec<Box<dyn Link>> = vec![
        Box::new(mpsc_vb_link),
        Box::new(mpscv_b_link),
        Box::new(locd_vb_link),
        Box::new(locdv_b_link)
    ];
    for link in links {
        link.run()?;
    }
    let (locd1_c2p_tx, locd1_p2c_rx) = locd1.peer_with_client()?;
    b.run()?;
    locd0.run()?;
    locd1.run()?;


    let hbfi0: HBFI = HBFI::new(Some(request_sid.public_id()), response_sid.public_id(), "locd", "htlc", "generate_secret", "shh")?;
    //let hbfi0: HBFI = HBFI::new(None, response_sid.public_id(), "app", "m0d", "fun", &name)?;

    debug!("\t\t\t\t\tclient-to-protocol");
    locd1_c2p_tx.send((hbfi0.clone(), Signals::RequestSecret))?;
    let (hbfi, secret_hash) = locd1_p2c_rx.recv()?;
    debug!("\t\t\t\t\tprotocol-to-client");
    debug!("\t\t\t\t{:?}", secret_hash);
    Ok(())
}
#[cfg(test)]
mod copernicafs {
    use super::*;
    use async_std::{ task, };

    #[test]
    fn test_value_transfer_two_hops() {
        task::block_on(async {
            let _r = value_transfer_two_hops();
        })
    }
}
