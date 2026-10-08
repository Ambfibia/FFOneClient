//! Server-scoped accounts and asynchronous ofapi authentication. Secrets live in the OS vault.
use super::*;
use serde::{Deserialize, Serialize};
use std::{fs, net::{TcpStream, ToSocketAddrs}, path::PathBuf, sync::{Mutex, mpsc}, time::{Duration, Instant}};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginServer {
    pub address: String,
    /// HTTPS ofapi origin, or None for an explicitly chosen direct game endpoint.
    pub api: Option<String>,
}

impl LoginServer {
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() || input.len() > 240 { return Err("Enter a server address".into()); }
        if input.starts_with("https://") || (!input.contains(':') && input != "localhost") {
            let origin = if input.starts_with("https://") { input.to_owned() } else { format!("https://{input}") };
            let url = reqwest::Url::parse(&origin).map_err(|_| "Invalid API address")?;
            if url.scheme() != "https" || url.host_str().is_none() || !url.username().is_empty()
                || url.password().is_some() || url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
                return Err("Use an HTTPS API host or host:port".into());
            }
            let api = url.as_str().trim_end_matches('/').to_owned();
            Ok(Self { address: url.host_str().unwrap().to_owned(), api: Some(api) })
        } else {
            let address = if input == "localhost" { "localhost:23000".into() } else { input.to_owned() };
            let (host, port) = address.rsplit_once(':').ok_or("Use host:port for direct login")?;
            if host.is_empty() || host.contains('/') || port.parse::<u16>().ok().filter(|p| *p > 0).is_none() {
                return Err("Invalid game server address".into());
            }
            Ok(Self { address, api: None })
        }
    }
    pub(super) fn key(&self) -> &str { self.api.as_deref().unwrap_or(&self.address) }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedAccount { pub server: String, pub username: String }

#[derive(Serialize, Deserialize)]
struct Profiles { schema: String, servers: Vec<LoginServer>, selected: usize, accounts: Vec<SavedAccount> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerHealth { Checking, Online(Option<usize>), Offline }

struct LoginReady {
    server: LoginServer, username: String, password: String, address: String,
    secret: String, remember: bool,
}
enum WorkerResult { Status(String, ServerHealth), Login(Result<LoginReady, String>) }

#[derive(Resource)]
pub struct LoginBrowser {
    pub servers: Vec<LoginServer>,
    pub selected: usize,
    pub accounts: Vec<SavedAccount>,
    pub health: Vec<ServerHealth>,
    pub remember: bool,
    pub server_input: String,
    pub editing_server: bool,
    pub server_edit: TextEdit,
    pub server_page: usize,
    pub account_page: usize,
    pub message: String,
    sender: mpsc::Sender<WorkerResult>,
    receiver: Mutex<mpsc::Receiver<WorkerResult>>,
    refresh_at: Instant,
    ready: Option<Result<LoginReady, String>>,
    awaiting_acceptance: Option<LoginReady>,
    path: Option<PathBuf>,
}

impl Default for LoginBrowser {
    fn default() -> Self {
        let path = crate::user_settings::default_settings_path().ok().map(|p| p.with_file_name("login-profiles.json"));
        let loaded = path.as_ref().and_then(|p| fs::read(p).ok()).and_then(|b| serde_json::from_slice::<Profiles>(&b).ok())
            .filter(|p| p.schema == "ffone.login-profiles.v1" && !p.servers.is_empty()
                && p.servers.iter().all(|s| LoginServer::parse(s.key()).as_ref() == Ok(s)));
        let (servers, selected, accounts) = loaded.map(|p| (p.servers, p.selected, p.accounts)).unwrap_or_else(|| (
            vec![LoginServer::parse("localhost").unwrap(), LoginServer::parse("api.slavicfall.ru").unwrap()], 0, vec![]));
        let (sender, receiver) = mpsc::channel();
        Self { selected: selected.min(servers.len()-1), health: vec![ServerHealth::Checking; servers.len()],
            servers, accounts, remember: true, server_input: String::new(), editing_server: false,
            server_edit: TextEdit::default(), server_page: 0, account_page: 0, message: String::new(), sender,
            receiver: Mutex::new(receiver), refresh_at: Instant::now(), ready: None, awaiting_acceptance: None, path }
    }
}

impl LoginBrowser {
    pub(super) fn refresh(&mut self) {
        self.health.fill(ServerHealth::Checking);
        self.refresh_at = Instant::now();
        self.poll();
    }
    pub fn selected_address(&self) -> &str { &self.servers[self.selected].address }
    pub fn selected_accounts(&self) -> Vec<&SavedAccount> {
        let key = self.servers[self.selected].key();
        self.accounts.iter().filter(|a| a.server == key).collect()
    }
    fn save(&self) -> Result<(), String> {
        let path = self.path.as_ref().ok_or("User settings path is unavailable")?;
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let profiles = Profiles { schema: "ffone.login-profiles.v1".into(), servers: self.servers.clone(), selected: self.selected, accounts: self.accounts.clone() };
        let temp = tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&temp, &profiles).map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        temp.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }
    pub(super) fn select(&mut self, index: usize) {
        if index >= self.servers.len() { return; }
        self.selected = index; self.account_page = 0; self.editing_server = false;
        self.message = self.save().err().unwrap_or_default();
    }
    pub(super) fn add_server(&mut self) {
        match LoginServer::parse(&self.server_input) {
            Ok(server) => {
                let index = self.servers.iter().position(|s| s.key() == server.key()).unwrap_or_else(|| {
                    self.servers.push(server); self.health.push(ServerHealth::Checking); self.servers.len()-1
                });
                self.select(index); self.server_page = index / 3; self.server_input.clear(); self.refresh_at = Instant::now();
            }
            Err(error) => self.message = error,
        }
    }
    pub fn use_saved(&mut self, username: &str) -> Result<String, String> {
        vault(&self.servers[self.selected], username)?.get_password().map_err(|_| "Saved account expired or is unavailable; enter the password again".into())
    }
    pub(super) fn forget(&mut self, username: &str) {
        let server = &self.servers[self.selected];
        match vault(server, username).and_then(|e| e.delete_credential().or_else(|err| if matches!(err, keyring::Error::NoEntry) { Ok(()) } else { Err(err) }).map_err(|e| e.to_string())) {
            Ok(()) => { let key = server.key().to_owned(); self.accounts.retain(|a| a.server != key || a.username != username); self.message = self.save().err().unwrap_or_default(); }
            Err(error) => self.message = error,
        }
    }
    /// Password may be blank only for an explicitly selected saved account.
    pub fn start_login(&mut self, username: String, password: String) {
        let server = self.servers[self.selected].clone();
        let saved = password.is_empty();
        let secret = if saved { self.use_saved(&username) } else { Ok(password) };
        let sender = self.sender.clone(); let remember = self.remember;
        self.awaiting_acceptance = None;
        std::thread::spawn(move || { let result = secret.and_then(|secret| authenticate(server, username, secret, saved, remember)); let _ = sender.send(WorkerResult::Login(result)); });
    }
    pub fn take_login(&mut self) -> Option<Result<(String, String, String), String>> {
        self.poll();
        self.ready.take().map(|result| result.map(|login| {
            let wire = (login.address.clone(), login.username.clone(), login.password.clone());
            self.awaiting_acceptance = Some(login); wire
        }))
    }
    pub fn finish_login(&mut self, accepted: bool) {
        let Some(login) = self.awaiting_acceptance.take() else { return; };
        if !accepted || !login.remember { return; }
        let result = vault(&login.server, &login.username).and_then(|entry| entry.set_password(&login.secret).map_err(|e| e.to_string())).and_then(|()| {
            if !self.accounts.iter().any(|a| a.server == login.server.key() && a.username == login.username) {
                self.accounts.push(SavedAccount { server: login.server.key().into(), username: login.username });
            }
            self.save()
        });
        self.message = result.err().unwrap_or_default();
    }
    pub(super) fn poll(&mut self) {
        let results: Vec<_> = self.receiver.lock().unwrap().try_iter().collect();
        for result in results {
            match result {
                WorkerResult::Login(login) => self.ready = Some(login),
                WorkerResult::Status(key, health) => if let Some(index) = self.servers.iter().position(|s| s.key() == key) { self.health[index] = health; },
            }
        }
        if Instant::now() < self.refresh_at { return; }
        self.refresh_at = Instant::now() + Duration::from_secs(20);
        for server in self.servers.clone() {
            let sender = self.sender.clone();
            std::thread::spawn(move || {
                let health = probe(&server).unwrap_or(ServerHealth::Offline);
                let _ = sender.send(WorkerResult::Status(server.key().into(), health));
            });
        }
    }
}

fn vault(server: &LoginServer, username: &str) -> Result<keyring::Entry, String> {
    let identity = blake3::hash(format!("{}\n{}", server.key(), username).as_bytes());
    keyring::Entry::new("FFOneClient", &format!("v1:{identity}")).map_err(|e| e.to_string())
}
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder().timeout(Duration::from_secs(8)).redirect(reqwest::redirect::Policy::none()).build().map_err(|e| e.to_string())
}
fn response(response: reqwest::blocking::Response) -> Result<reqwest::blocking::Response, String> {
    if response.status().is_success() { Ok(response) } else { Err(format!("API request failed ({})", response.status())) }
}
fn probe(server: &LoginServer) -> Result<ServerHealth, String> {
    if let Some(api) = &server.api {
        let client = client()?;
        response(client.get(api).send().map_err(|e| e.to_string())?)?;
        let value: serde_json::Value = response(client.get(format!("{api}/status")).send().map_err(|e| e.to_string())?)?.json().map_err(|e| e.to_string())?;
        Ok(ServerHealth::Online(value.get("player_count").and_then(|v| v.as_u64()).map(|n| n as usize)))
    } else {
        let addresses = server.address.to_socket_addrs().map_err(|e| e.to_string())?;
        for address in addresses { if TcpStream::connect_timeout(&address, Duration::from_secs(2)).is_ok() { return Ok(ServerHealth::Online(None)); } }
        Ok(ServerHealth::Offline)
    }
}
fn authenticate(server: LoginServer, username: String, secret: String, saved: bool, remember: bool) -> Result<LoginReady, String> {
    let Some(api) = &server.api else { return Ok(LoginReady { address: server.address.clone(), server, username, password: secret.clone(), secret, remember }); };
    let client = client()?;
    let info: serde_json::Value = response(client.get(api).send().map_err(|e| e.to_string())?)?.json().map_err(|e| e.to_string())?;
    let address = info.get("login_address").and_then(|v| v.as_str()).ok_or("API did not provide a login address")?.to_owned();
    let secret = if saved { secret } else {
        response(client.post(format!("{api}/auth")).json(&serde_json::json!({"username": username, "password": secret})).send().map_err(|e| e.to_string())?)?.text().map_err(|e| e.to_string())?
    };
    let session: serde_json::Value = response(client.post(format!("{api}/auth/session")).bearer_auth(secret.trim()).send().map_err(|e| e.to_string())?)?.json().map_err(|e| e.to_string())?;
    let token = session.get("session_token").and_then(|v| v.as_str()).ok_or("API did not provide a session")?;
    let cookie: serde_json::Value = response(client.post(format!("{api}/cookie")).bearer_auth(token).send().map_err(|e| e.to_string())?)?.json().map_err(|e| e.to_string())?;
    let username = cookie.get("username").and_then(|v| v.as_str()).ok_or("API did not provide a username")?.to_owned();
    let password = cookie.get("cookie").and_then(|v| v.as_str()).ok_or("API did not provide a login cookie")?.to_owned();
    Ok(LoginReady { server, username, password, address, secret, remember })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    #[test]
    fn ofapi_uses_refresh_session_and_cookie_without_sending_password_to_game() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let worker = std::thread::spawn(move || {
            for (path, bearer, body) in [
                ("/", "", r#"{"login_address":"127.0.0.1:23000"}"#),
                ("/auth", "", "refresh-secret"),
                ("/auth/session", "refresh-secret", r#"{"session_token":"session-secret"}"#),
                ("/cookie", "session-secret", r#"{"username":"canonical","cookie":"one-use-cookie"}"#),
            ] {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new(); reader.read_line(&mut line).unwrap();
                assert_eq!(line.split_whitespace().nth(1), Some(path));
                let mut headers = String::new();
                loop { line.clear(); reader.read_line(&mut line).unwrap(); if line == "\r\n" { break; } headers.push_str(&line.to_lowercase()); }
                if !bearer.is_empty() { assert!(headers.contains(&format!("authorization: bearer {bearer}"))); }
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let login = authenticate(LoginServer { address: "mock".into(), api: Some(origin) }, "input".into(), "password-secret".into(), false, true).unwrap();
        assert_eq!(login.username, "canonical"); assert_eq!(login.password, "one-use-cookie"); assert_eq!(login.secret, "refresh-secret");
        assert_eq!(login.address, "127.0.0.1:23000"); worker.join().unwrap();
    }
    #[test]
    fn endpoints_are_canonical_and_do_not_accept_embedded_credentials() {
        assert_eq!(LoginServer::parse("api.slavicfall.ru").unwrap(), LoginServer::parse("https://api.slavicfall.ru/").unwrap());
        assert_eq!(LoginServer::parse("localhost").unwrap().address, "localhost:23000");
        for bad in ["", "https://user:pass@example.org", "https://example.org/auth", "host:0", "http://example.org"] { assert!(LoginServer::parse(bad).is_err(), "{bad}"); }
    }
    #[test]
    fn accounts_are_scoped_and_failed_login_is_not_persisted() {
        let mut browser = LoginBrowser::default();
        browser.servers = vec![LoginServer::parse("localhost").unwrap(), LoginServer::parse("api.slavicfall.ru").unwrap()];
        browser.selected = 0; browser.accounts = vec![SavedAccount { server: "https://api.slavicfall.ru".into(), username: "same".into() }];
        assert!(browser.selected_accounts().is_empty());
        browser.selected = 1; assert_eq!(browser.selected_accounts().len(), 1);
        browser.awaiting_acceptance = Some(LoginReady { server: browser.servers[1].clone(), username: "rejected".into(), password: "cookie".into(), address: "host:23000".into(), secret: "token".into(), remember: true });
        browser.finish_login(false); assert_eq!(browser.accounts.len(), 1);
        let json = serde_json::to_string(&Profiles { schema: "ffone.login-profiles.v1".into(), servers: browser.servers.clone(), selected: 1, accounts: browser.accounts.clone() }).unwrap();
        assert!(!json.contains("password") && !json.contains("token"));
    }
}
