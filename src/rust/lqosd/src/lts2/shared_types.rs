//! Keep this synchronized with the server-side version.
#![allow(dead_code)]

use serde::Deserialize;

pub type ControlSender = std::sync::mpsc::Sender<LtsCommand>;
pub type ControlReceiver = std::sync::mpsc::Receiver<LtsCommand>;
pub type GetConfigFn = fn(&mut Lts2Config);
pub type SendStatusFn = fn(bool, i32, i32);
pub type StartLts2Fn = fn(GetConfigFn, SendStatusFn, ControlReceiver);

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Lts2Config {
    /// The path to the root certificate for the LTS server
    pub path_to_certificate: Option<String>,
    /// The domain name of the LTS server
    pub domain: Option<String>,
    /// The license key for the LTS server
    pub license_key: Option<String>,
    /// The ID of the node
    pub node_id: String,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub enum LtsCommand {
    Placeholder,
    RequestFreeTrial(FreeTrialDetails),
}


#[repr(C)]
#[derive(Debug, Clone, Deserialize)]
pub struct FreeTrialDetails {
    pub name: String,
    pub email: String,
    pub business_name: String,
    pub address1: String,
    pub address2: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub phone: String,
    pub website: String,
}