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
    let A_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let B_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let C_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let brkstr = sled::open(generate_random_dir_name())?;
    let storeA = sled::open(generate_random_dir_name())?;
    let storeB = sled::open(generate_random_dir_name())?;
    let storeC = sled::open(generate_random_dir_name())?;
    let mut broker = Broker::new(brkstr);
    let mut locdsA = LOCDService::new(storeA, A_sid.clone());
    let mut locdsB = LOCDService::new(storeB, B_sid.clone());
    let mut locdsC = LOCDService::new(storeC, C_sid.clone());
    let av_br_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let a_vbr_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let bv_br_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let b_vbr_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let cv_br_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let c_vbr_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let av_br_link_id = LinkId::listen(av_br_link_sid.clone(), Some(a_vbr_link_sid.public_id()), ReplyTo::Mpsc);
    let a_vbr_link_id = LinkId::listen(a_vbr_link_sid.clone(), Some(av_br_link_sid.public_id()), ReplyTo::Mpsc);
    let mut av_br_link: MpscChannel = Link::new("lv_b".into(), av_br_link_id.clone(), locdsA.peer_with_link(av_br_link_id)?)?;
    let mut a_vbr_link: MpscChannel = Link::new("l_vb".into(), a_vbr_link_id.clone(), broker.peer_with_link(a_vbr_link_id)?)?;
    av_br_link.female(a_vbr_link.male());
    a_vbr_link.female(av_br_link.male());
    let bv_br_link_id = LinkId::listen(bv_br_link_sid.clone(), Some(b_vbr_link_sid.public_id()), ReplyTo::Mpsc);
    let b_vbr_link_id = LinkId::listen(b_vbr_link_sid.clone(), Some(bv_br_link_sid.public_id()), ReplyTo::Mpsc);
    let mut bv_br_link: MpscChannel = Link::new("lv_b".into(), bv_br_link_id.clone(), locdsB.peer_with_link(bv_br_link_id)?)?;
    let mut b_vbr_link: MpscChannel = Link::new("l_vb".into(), b_vbr_link_id.clone(), broker.peer_with_link(b_vbr_link_id)?)?;
    bv_br_link.female(b_vbr_link.male());
    b_vbr_link.female(bv_br_link.male());
    let cv_br_link_id = LinkId::listen(cv_br_link_sid.clone(), Some(c_vbr_link_sid.public_id()), ReplyTo::Mpsc);
    let c_vbr_link_id = LinkId::listen(c_vbr_link_sid.clone(), Some(cv_br_link_sid.public_id()), ReplyTo::Mpsc);
    let mut cv_br_link: MpscChannel = Link::new("lv_b".into(), cv_br_link_id.clone(), locdsC.peer_with_link(cv_br_link_id)?)?;
    let mut c_vbr_link: MpscChannel = Link::new("l_vb".into(), c_vbr_link_id.clone(), broker.peer_with_link(c_vbr_link_id)?)?;
    cv_br_link.female(c_vbr_link.male());
    c_vbr_link.female(cv_br_link.male());
    let links: Vec<Box<dyn Link>> = vec![
        Box::new(av_br_link),
        Box::new(a_vbr_link),
        Box::new(bv_br_link),
        Box::new(b_vbr_link),
        Box::new(cv_br_link),
        Box::new(c_vbr_link),
    ];
    for link in links {
        link.run()?;
    }
    let (locdsA_c2p_tx, locdsA_p2c_rx) = locdsA.peer_with_client()?;
    broker.run()?;
    locdsA.run()?;
    locdsB.run()?;
    locdsC.run()?;
    let hbfi0: HBFI = HBFI::new(Some(A_sid.public_id()), C_sid.public_id(), "locd", "htlc", "generate_secret", "shh")?;
    //let hbfi0: HBFI = HBFI::new(None, response_sid.public_id(), "app", "m0d", "fun", &name)?;
    debug!("\t\t\t\t\tclient-to-protocol");
    locdsA_c2p_tx.send((hbfi0.clone(), Signals::RequestSecret))?;
    let (hbfi, secret_hash) = locdsA_p2c_rx.recv()?;
    debug!("\t\t\t\t\tprotocol-to-client");
    debug!("\t\t\t\t{:?}", secret_hash);
    let (hbfi, secret_hash) = locdsA_p2c_rx.recv()?;
    Ok(())
}
