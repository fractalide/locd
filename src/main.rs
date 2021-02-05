#![allow(dead_code)]
#![feature(bindings_after_at)]
mod service;
mod protocol;
mod signals;
use {
    anyhow::{Result},
    sled,
    crate::{
        service::{LOCDService},
    },
    copernica_broker::{Broker},
    copernica_common::{LinkId, ReplyTo},
    copernica_links::{Link, MpscChannel},
    copernica_identity::{PrivateIdentity, Seed},
    copernica_tests::{generate_random_dir_name},
    log::{debug},
};

pub fn main() -> Result<()> {
    copernica_common::setup_logging(3, None).unwrap();
    let mut rng = rand::thread_rng();
    let a_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let b_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let c_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let brkstr = sled::open(generate_random_dir_name())?;
    let store_a = sled::open(generate_random_dir_name())?;
    let store_b = sled::open(generate_random_dir_name())?;
    let store_c = sled::open(generate_random_dir_name())?;
    let mut broker = Broker::new(brkstr);
    let mut locds_a = LOCDService::new(store_a, a_sid.clone());
    let mut locds_b = LOCDService::new(store_b, b_sid.clone());
    let mut locds_c = LOCDService::new(store_c, c_sid.clone());
    let av_br_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let a_vbr_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let bv_br_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let b_vbr_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let cv_br_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let c_vbr_link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let av_br_link_id = LinkId::listen(av_br_link_sid.clone(), Some(a_vbr_link_sid.public_id()), ReplyTo::Mpsc);
    let a_vbr_link_id = LinkId::listen(a_vbr_link_sid.clone(), Some(av_br_link_sid.public_id()), ReplyTo::Mpsc);
    let mut av_br_link: MpscChannel = Link::new("lv_b".into(), av_br_link_id.clone(), locds_a.peer_with_link(av_br_link_id)?)?;
    let mut a_vbr_link: MpscChannel = Link::new("l_vb".into(), a_vbr_link_id.clone(), broker.peer_with_link(a_vbr_link_id)?)?;
    av_br_link.female(a_vbr_link.male());
    a_vbr_link.female(av_br_link.male());
    let bv_br_link_id = LinkId::listen(bv_br_link_sid.clone(), Some(b_vbr_link_sid.public_id()), ReplyTo::Mpsc);
    let b_vbr_link_id = LinkId::listen(b_vbr_link_sid.clone(), Some(bv_br_link_sid.public_id()), ReplyTo::Mpsc);
    let mut bv_br_link: MpscChannel = Link::new("lv_b".into(), bv_br_link_id.clone(), locds_b.peer_with_link(bv_br_link_id)?)?;
    let mut b_vbr_link: MpscChannel = Link::new("l_vb".into(), b_vbr_link_id.clone(), broker.peer_with_link(b_vbr_link_id)?)?;
    bv_br_link.female(b_vbr_link.male());
    b_vbr_link.female(bv_br_link.male());
    let cv_br_link_id = LinkId::listen(cv_br_link_sid.clone(), Some(c_vbr_link_sid.public_id()), ReplyTo::Mpsc);
    let c_vbr_link_id = LinkId::listen(c_vbr_link_sid.clone(), Some(cv_br_link_sid.public_id()), ReplyTo::Mpsc);
    let mut cv_br_link: MpscChannel = Link::new("lv_b".into(), cv_br_link_id.clone(), locds_c.peer_with_link(cv_br_link_id)?)?;
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
    broker.run()?;
    locds_a.run()?;
    locds_b.run()?;
    locds_c.run()?;
    debug!("\t\t\t\t\tclient-to-protocol");
    let i_dont_know  = locds_a.peer_with_node(b_sid.public_id(), 1000, 800);
    //let i_dont_know2 = locds_b.peer_with_node(c_sid.public_id(), 1000, 800);
    debug!("\t\t\t\t\tprotocol-to-client");
    //debug!("\t\t\t\tamount {:?}, secret {:?}", amount, secret_hash);
    //debug!("\t\t\t\t{:?}", i_dont_know);
    Ok(())
}
