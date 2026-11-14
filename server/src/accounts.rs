use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use p256::pkcs8::DecodePrivateKey;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{unix_ms, Hub};

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const PENDING_MS: i64 = 10 * 60 * 1000;
pub const CLOUD_MAX: usize = 96 * 1024;

pub fn b64url(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, b)| n | (*b as u32) << (16 - 8 * i));
        for k in 0..chunk.len() + 1 {
            out.push(B64[((n >> (18 - 6 * k)) & 63) as usize] as char);
        }
    }
    out
}

pub fn b64url_decode(text: &str) -> Option<Vec<u8>> {
    let mut bits = 0u32;
    let mut have = 0;
    let mut out = Vec::new();
    for c in text.trim().trim_end_matches('=').bytes() {
        let c = match c {
            b'+' => b'-',
            b'/' => b'_',
            c => c,
        };
        let v = B64.iter().position(|x| *x == c)? as u32;
        bits = (bits << 6) | v;
        have += 6;
        if have >= 8 {
            have -= 8;
            out.push((bits >> have) as u8);
            bits &= (1 << have) - 1;
        }
    }
    Some(out)
}

pub fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    if getrandom::getrandom(&mut buf).is_err() {
        let t = unix_ms() as u64;
        for (i, b) in buf.iter_mut().enumerate() {
            *b = (t.rotate_left(i as u32 * 7) ^ (i as u64 * 0x9E37)) as u8;
        }
    }
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

fn enc(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn dec(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                match u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("zz"), 16) {
                    Ok(v) => {
                        out.push(v);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn query(path: &str) -> (String, Vec<(String, String)>) {
    let (route, q) = path.split_once('?').unwrap_or((path, ""));
    let pairs = q.split('&').filter(|p| !p.is_empty()).map(|p| {
        let (k, v) = p.split_once('=').unwrap_or((p, ""));
        (dec(k), dec(v))
    });
    (route.to_string(), pairs.collect())
}

fn get<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

pub fn jwt_claims(token: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    serde_json::from_slice(&b64url_decode(payload)?).ok()
}

pub struct Apple {
    pub client: String,
    pub team: String,
    pub key_id: String,
    pub key_pem: String,
}

pub fn apple_secret(a: &Apple, now_s: i64) -> Option<String> {
    let key = SigningKey::from_pkcs8_pem(&a.key_pem).ok()?;
    let header = b64url(format!(r#"{{"alg":"ES256","kid":"{}"}}"#, a.key_id).as_bytes());
    let claims = b64url(format!(r#"{{"iss":"{}","iat":{},"exp":{},"aud":"https://appleid.apple.com","sub":"{}"}}"#, a.team, now_s, now_s + 300, a.client).as_bytes());
    let body = format!("{header}.{claims}");
    let sig: Signature = key.sign(body.as_bytes());
    Some(format!("{body}.{}", b64url(&sig.to_bytes())))
}

pub fn steam_id(claimed: &str) -> Option<String> {
    let id = claimed.strip_prefix("https://steamcommunity.com/openid/id/")?;
    (!id.is_empty() && id.len() <= 20 && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.to_string())
}

pub struct Session {
    pub account: String,
    pub provider: String,
    pub label: String,
}

#[derive(Default)]
pub struct Accounts {
    pub public: String,
    pub dir: Option<PathBuf>,
    pub google: Option<(String, String)>,
    pub apple: Option<Apple>,
    pub sessions: HashMap<String, Session>,
    pub pending: HashMap<String, (u32, String, i64)>,
}

impl Accounts {
    pub fn providers(&self) -> Vec<&'static str> {
        if self.public.is_empty() {
            return Vec::new();
        }
        let mut v = Vec::new();
        if self.google.is_some() {
            v.push("google");
        }
        v.push("steam");
        if self.apple.is_some() {
            v.push("apple");
        }
        v
    }

    pub fn load(&mut self) {
        let Some(dir) = &self.dir else {
            return;
        };
        let _ = std::fs::create_dir_all(dir.join("saves"));
        if let Ok(text) = std::fs::read_to_string(dir.join("sessions.txt")) {
            for line in text.lines() {
                let mut parts = line.splitn(4, ' ');
                if let (Some(s), Some(a), Some(p), Some(l)) = (parts.next(), parts.next(), parts.next(), parts.next()) {
                    self.sessions.insert(s.to_string(), Session { account: a.to_string(), provider: p.to_string(), label: l.to_string() });
                }
            }
        }
    }

    fn save_sessions(&self) {
        let Some(dir) = &self.dir else {
            return;
        };
        let text: String = self.sessions.iter().map(|(s, x)| format!("{s} {} {} {}\n", x.account, x.provider, x.label)).collect();
        let tmp = dir.join("sessions.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(tmp, dir.join("sessions.txt"));
        }
    }

    fn save_file(&self, account: &str) -> Option<PathBuf> {
        let name: String = account.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        self.dir.as_ref().map(|d| d.join("saves").join(format!("{name}.txt")))
    }

    pub fn begin(&mut self, client: u32, provider: &str, now: i64) -> Option<String> {
        if !self.providers().contains(&provider) {
            return None;
        }
        self.pending.retain(|_, (_, _, at)| now - *at < PENDING_MS);
        let nonce = random_hex(16);
        self.pending.insert(nonce.clone(), (client, provider.to_string(), now));
        Some(format!("{}/login/{provider}?n={nonce}", self.public))
    }

    pub fn finish(&mut self, nonce: &str, provider: &str, account: String, label: String, now: i64) -> Option<(u32, String)> {
        let (client, wanted, at) = self.pending.remove(nonce)?;
        if wanted != provider || now - at > PENDING_MS {
            return None;
        }
        let session = random_hex(24);
        let label: String = label.chars().filter(|c| !c.is_whitespace() && !c.is_control()).take(48).collect();
        let label = if label.is_empty() { provider.to_string() } else { label };
        self.sessions.insert(session.clone(), Session { account, provider: provider.to_string(), label });
        self.save_sessions();
        Some((client, session))
    }

    pub fn forget(&mut self, session: &str) {
        if self.sessions.remove(session).is_some() {
            self.save_sessions();
        }
    }

    pub fn cloud_get(&self, account: &str) -> Option<String> {
        std::fs::read_to_string(self.save_file(account)?).ok().filter(|t| !t.is_empty())
    }

    pub fn cloud_put(&self, account: &str, data: &str) -> bool {
        if data.len() > CLOUD_MAX || b64url_decode(data).is_none() {
            return false;
        }
        let Some(path) = self.save_file(account) else {
            return false;
        };
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, data).is_ok() && std::fs::rename(tmp, path).is_ok()
    }

    pub fn pending_provider(&self, nonce: &str) -> Option<String> {
        self.pending.get(nonce).map(|(_, p, _)| p.clone())
    }

    pub fn auth_url(&self, provider: &str, nonce: &str) -> Option<String> {
        let back = format!("{}/auth/{provider}", self.public);
        match provider {
            "google" => {
                let (id, _) = self.google.as_ref()?;
                Some(format!("https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={nonce}&prompt=select_account", enc(id), enc(&back), enc("openid email")))
            }
            "apple" => {
                let a = self.apple.as_ref()?;
                Some(format!("https://appleid.apple.com/auth/authorize?client_id={}&redirect_uri={}&response_type=code&response_mode=query&state={nonce}", enc(&a.client), enc(&back)))
            }
            "steam" => {
                let ret = format!("{back}?n={nonce}");
                let ident = "http://specs.openid.net/auth/2.0/identifier_select";
                Some(format!(
                    "https://steamcommunity.com/openid/login?openid.ns={}&openid.mode=checkid_setup&openid.return_to={}&openid.realm={}&openid.identity={}&openid.claimed_id={}",
                    enc("http://specs.openid.net/auth/2.0"),
                    enc(&ret),
                    enc(&self.public),
                    enc(ident),
                    enc(ident)
                ))
            }
            _ => None,
        }
    }
}

async fn exchange(hub: &Arc<Mutex<Hub>>, provider: &str, pairs: &[(String, String)]) -> Result<(String, String, String), String> {
    let (public, google, apple) = {
        let h = hub.lock().unwrap();
        let a = &h.accounts;
        (a.public.clone(), a.google.clone(), a.apple.as_ref().map(|x| Apple { client: x.client.clone(), team: x.team.clone(), key_id: x.key_id.clone(), key_pem: x.key_pem.clone() }))
    };
    let http = reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).build().map_err(|e| e.to_string())?;
    let back = format!("{public}/auth/{provider}");
    match provider {
        "google" | "apple" => {
            let nonce = get(pairs, "state").ok_or("missing state")?.to_string();
            let code = get(pairs, "code").ok_or("the sign-in was cancelled")?.to_string();
            let (url, id, secret) = if provider == "google" {
                let (id, secret) = google.ok_or("google is not set up")?;
                ("https://oauth2.googleapis.com/token", id, secret)
            } else {
                let a = apple.ok_or("apple is not set up")?;
                let secret = apple_secret(&a, unix_ms() / 1000).ok_or("the apple key could not be read")?;
                ("https://appleid.apple.com/auth/token", a.client, secret)
            };
            let form = [("grant_type", "authorization_code"), ("code", code.as_str()), ("redirect_uri", back.as_str()), ("client_id", id.as_str()), ("client_secret", secret.as_str())];
            let body = form.iter().map(|(k, v)| format!("{k}={}", enc(v))).collect::<Vec<_>>().join("&");
            let reply = http.post(url).header("content-type", "application/x-www-form-urlencoded").body(body).send().await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = serde_json::from_str(&reply.text().await.map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            let token = json.get("id_token").and_then(|t| t.as_str()).ok_or("no id token in the reply")?;
            let claims = jwt_claims(token).ok_or("unreadable id token")?;
            let aud_ok = claims.get("aud").and_then(|a| a.as_str()) == Some(id.as_str());
            let iss = claims.get("iss").and_then(|a| a.as_str()).unwrap_or("");
            let iss_ok = if provider == "google" { iss == "https://accounts.google.com" || iss == "accounts.google.com" } else { iss == "https://appleid.apple.com" };
            let sub = claims.get("sub").and_then(|s| s.as_str()).ok_or("no subject")?;
            if !aud_ok || !iss_ok {
                return Err("the token was not for this game".to_string());
            }
            let label = claims.get("email").and_then(|e| e.as_str()).unwrap_or(if provider == "google" { "google" } else { "apple-id" }).to_string();
            Ok((nonce, format!("{provider}:{sub}"), label))
        }
        "steam" => {
            let nonce = get(pairs, "n").ok_or("missing state")?.to_string();
            let ret = get(pairs, "openid.return_to").unwrap_or("");
            if ret != format!("{back}?n={nonce}") {
                return Err("the steam reply was for another page".to_string());
            }
            let id = steam_id(get(pairs, "openid.claimed_id").unwrap_or("")).ok_or("no steam id")?;
            let mut check: Vec<(String, String)> = pairs.iter().filter(|(k, _)| k.starts_with("openid.")).cloned().collect();
            for p in check.iter_mut() {
                if p.0 == "openid.mode" {
                    p.1 = "check_authentication".to_string();
                }
            }
            let body = check.iter().map(|(k, v)| format!("{}={}", enc(k), enc(v))).collect::<Vec<_>>().join("&");
            let reply = http.post("https://steamcommunity.com/openid/login").header("content-type", "application/x-www-form-urlencoded").body(body).send().await.map_err(|e| e.to_string())?;
            let text = reply.text().await.map_err(|e| e.to_string())?;
            if !text.lines().any(|l| l.trim() == "is_valid:true") {
                return Err("steam did not confirm the sign-in".to_string());
            }
            Ok((nonce, format!("steam:{id}"), format!("steam-{id}")))
        }
        _ => Err("unknown provider".to_string()),
    }
}

fn page(title: &str, text: &str) -> String {
    format!(
        "<!doctype html><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><title>melody quack</title><body style='margin:0;min-height:100vh;display:grid;place-items:center;background:#241a47;color:#fff7e0;font:16px/1.5 ui-monospace,monospace;text-align:center'><div style='padding:24px'><h1 style='color:#ffd66e;font-size:28px'>{title}</h1><p>{text}</p></div>"
    )
}

async fn respond(stream: &mut TcpStream, status: &str, extra: &str, body: &str) {
    let head = format!("HTTP/1.1 {status}\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\ncache-control: no-store\r\nreferrer-policy: no-referrer\r\n{extra}connection: close\r\n\r\n", body.len());
    let _ = stream.write_all(head.as_bytes()).await;
    let _ = stream.write_all(body.as_bytes()).await;
    let _ = stream.shutdown().await;
}

pub async fn http(mut stream: TcpStream, hub: Arc<Mutex<Hub>>) {
    let mut buf = vec![0u8; 8192];
    let mut have = 0;
    while have < buf.len() {
        match stream.read(&mut buf[have..]).await {
            Ok(0) | Err(_) => break,
            Ok(n) => have += n,
        }
        if buf[..have].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let head = String::from_utf8_lossy(&buf[..have]).to_string();
    let target = head.lines().next().and_then(|l| l.split_whitespace().nth(1)).unwrap_or("/").to_string();
    let (route, pairs) = query(&target);
    if let Some(provider) = route.strip_prefix("/login/") {
        let nonce = get(&pairs, "n").unwrap_or("");
        let url = {
            let h = hub.lock().unwrap();
            (h.accounts.pending_provider(nonce).as_deref() == Some(provider)).then(|| h.accounts.auth_url(provider, nonce)).flatten()
        };
        match url {
            Some(url) => respond(&mut stream, "302 Found", &format!("location: {url}\r\n"), "").await,
            None => respond(&mut stream, "400 Bad Request", "", &page("link expired", "go back to melody quack and press sign in again.")).await,
        }
        return;
    }
    if let Some(provider) = route.strip_prefix("/auth/") {
        let provider = provider.to_string();
        let result = exchange(&hub, &provider, &pairs).await;
        let body = match result {
            Ok((nonce, account, label)) => {
                if hub.lock().unwrap().signed_in(&nonce, &provider, account, label, unix_ms()) {
                    page("you're signed in", "go back to melody quack. your game is saved to your account from now on.")
                } else {
                    page("link expired", "go back to melody quack and press sign in again.")
                }
            }
            Err(e) => page("that didn't work", &e.replace('<', "")),
        };
        respond(&mut stream, "200 OK", "", &body).await;
        return;
    }
    if let Some(name) = route.strip_prefix("/live/").and_then(|r| r.strip_suffix(".mp3")) {
        let joined = {
            let h = hub.lock().unwrap();
            h.casts.get(name).map(|c| (c.tx.subscribe(), c.recent.iter().copied().collect::<Vec<u8>>()))
        };
        let Some((mut rx, recent)) = joined else {
            respond(&mut stream, "404 Not Found", "", &page("nobody is live", "this room isn't streaming right now.")).await;
            return;
        };
        let head = "HTTP/1.1 200 OK\r\ncontent-type: audio/mpeg\r\ncache-control: no-store\r\naccess-control-allow-origin: *\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n";
        if stream.write_all(head.as_bytes()).await.is_err() {
            return;
        }
        let chunk = |data: &[u8]| {
            let mut out = format!("{:x}\r\n", data.len()).into_bytes();
            out.extend_from_slice(data);
            out.extend_from_slice(b"\r\n");
            out
        };
        if !recent.is_empty() && stream.write_all(&chunk(&recent)).await.is_err() {
            return;
        }
        loop {
            match rx.recv().await {
                Ok(data) => {
                    if stream.write_all(&chunk(&data)).await.is_err() {
                        return;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
        let _ = stream.write_all(b"0\r\n\r\n").await;
        let _ = stream.shutdown().await;
        return;
    }
    respond(&mut stream, "200 OK", "", &page("melody quack", "the rave server is running.")).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::signature::Verifier;
    use p256::ecdsa::VerifyingKey;
    use p256::pkcs8::EncodePrivateKey;

    fn temp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mq-accounts-{}", random_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn base64url_round_trips_and_query_decodes() {
        for text in ["", "a", "ab", "abc", r#"{"alg":"none"}"#] {
            assert_eq!(b64url_decode(&b64url(text.as_bytes())).unwrap(), text.as_bytes());
        }
        let (route, pairs) = query("/auth/steam?n=abc&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F7656&x=a+b");
        assert_eq!(route, "/auth/steam");
        assert_eq!(get(&pairs, "openid.claimed_id"), Some("https://steamcommunity.com/openid/id/7656"));
        assert_eq!(get(&pairs, "x"), Some("a b"));
        assert_eq!(steam_id("https://steamcommunity.com/openid/id/76561197960287930").as_deref(), Some("76561197960287930"));
        assert!(steam_id("https://evil.example/openid/id/1").is_none());
        assert!(steam_id("https://steamcommunity.com/openid/id/12a").is_none());
    }

    #[test]
    fn the_apple_secret_is_a_valid_es256_token() {
        let key = SigningKey::from_bytes(&[7u8; 32].into()).unwrap();
        let pem = key.to_pkcs8_pem(Default::default()).unwrap().to_string();
        let a = Apple { client: "com.example.melody".to_string(), team: "TEAM123456".to_string(), key_id: "KEY1234567".to_string(), key_pem: pem };
        let token = apple_secret(&a, 1_700_000_000).unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        let claims = jwt_claims(&token).unwrap();
        assert_eq!(claims["sub"], "com.example.melody");
        assert_eq!(claims["iss"], "TEAM123456");
        assert_eq!(claims["exp"], 1_700_000_300);
        let sig = Signature::from_slice(&b64url_decode(parts[2]).unwrap()).unwrap();
        VerifyingKey::from(&key).verify(format!("{}.{}", parts[0], parts[1]).as_bytes(), &sig).unwrap();
    }

    #[test]
    fn a_login_is_one_use_and_bound_to_its_provider() {
        let mut a = Accounts { public: "https://rave.example".to_string(), dir: Some(temp()), ..Default::default() };
        a.load();
        assert_eq!(a.providers(), vec!["steam"]);
        assert!(a.begin(1, "google", 0).is_none());
        let url = a.begin(1, "steam", 0).unwrap();
        let nonce = url.split("n=").nth(1).unwrap().to_string();
        assert!(a.auth_url("steam", &nonce).unwrap().contains(&enc(&format!("https://rave.example/auth/steam?n={nonce}"))));
        assert!(a.finish(&nonce, "google", "google:1".to_string(), "x".to_string(), 1).is_none());
        let url = a.begin(1, "steam", 0).unwrap();
        let nonce = url.split("n=").nth(1).unwrap().to_string();
        assert!(a.finish(&nonce, "steam", "steam:9".to_string(), "steam-9".to_string(), PENDING_MS + 1).is_none());
        let url = a.begin(4, "steam", 0).unwrap();
        let nonce = url.split("n=").nth(1).unwrap().to_string();
        let (client, session) = a.finish(&nonce, "steam", "steam:9".to_string(), "steam-9".to_string(), 5).unwrap();
        assert_eq!(client, 4);
        assert!(a.finish(&nonce, "steam", "steam:9".to_string(), "steam-9".to_string(), 6).is_none());
        assert!(a.cloud_get("steam:9").is_none());
        assert!(a.cloud_put("steam:9", &b64url(b"melody-quack-save 1\nname ky\n")));
        assert!(!a.cloud_put("steam:9", "not base64 !!"));
        assert!(!a.cloud_put("steam:9", &"A".repeat(CLOUD_MAX + 1)));
        let mut again = Accounts { public: a.public.clone(), dir: a.dir.clone(), ..Default::default() };
        again.load();
        assert_eq!(again.sessions.get(&session).unwrap().account, "steam:9");
        assert_eq!(b64url_decode(&again.cloud_get("steam:9").unwrap()).unwrap(), b"melody-quack-save 1\nname ky\n");
    }
}
