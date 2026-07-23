use data_encoding::BASE32_NOPAD;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::tor::control;

pub struct CreatedOnion {
    pub service_id: String,
    pub client_credential: Option<String>,
}

fn value<'a>(reply: &'a control::ControlReply, key: &str) -> Option<&'a str> {
    reply
        .lines
        .iter()
        .find_map(|line| line.body.strip_prefix(key))
}

pub async fn add(
    local_port: u16,
    virtual_port: u16,
    private: bool,
) -> Result<CreatedOnion, String> {
    let (auth_flag, public_key, credential) = if private {
        let secret = StaticSecret::random();
        let public = PublicKey::from(&secret);
        (
            ",V3Auth",
            format!(" ClientAuthV3={}", BASE32_NOPAD.encode(public.as_bytes())),
            Some(format!(
                "descriptor:x25519:{}",
                BASE32_NOPAD.encode(secret.as_bytes())
            )),
        )
    } else {
        ("", String::new(), None)
    };
    let command = format!(
        "ADD_ONION NEW:ED25519-V3 Flags=DiscardPK,Detach{auth_flag} Port={virtual_port},127.0.0.1:{local_port}{public_key}"
    );
    let reply = control::run_authenticated(&[&command]).await?;
    let service_id = value(&reply, "ServiceID=")
        .ok_or_else(|| format!("ADD_ONION did not return a ServiceID: {}", reply.raw))?
        .to_ascii_lowercase();
    if service_id.len() != 56 {
        return Err("Tor returned an invalid v3 onion service ID".into());
    }
    Ok(CreatedOnion {
        service_id,
        client_credential: credential,
    })
}

pub async fn authorize_client(service_id: &str, credential: &str) -> Result<(), String> {
    let command = format!("ONION_CLIENT_AUTH_ADD {service_id} {credential}");
    control::run_authenticated(&[&command]).await?;
    Ok(())
}

pub async fn remove_client_authorization(service_id: &str) {
    let command = format!("ONION_CLIENT_AUTH_REMOVE {service_id}");
    let _ = control::run_authenticated(&[&command]).await;
}

pub async fn delete(service_id: &str) -> Result<(), String> {
    let command = format!("DEL_ONION {service_id}");
    control::run_authenticated(&[&command]).await?;
    Ok(())
}
