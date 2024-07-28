use crate::imports::*;
use spectre_daemon::SpectredConfig;
use workflow_core::task::sleep;
use workflow_node::process;
pub use workflow_node::process::Event;
use workflow_store::fs;

#[derive(Describe, Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum SpectredSettings {
    #[describe("Specify the binary location")]
    Location,
    #[describe("Toggle logging output")]
    Mute,
}

#[async_trait]
impl DefaultSettings for SpectredSettings {
    async fn defaults() -> Vec<(Self, Value)> {
        let mut settings = vec![(Self::Mute, to_value(true).unwrap())];

        let root = nw_sys::app::folder();
        if let Ok(binaries) = spectre_daemon::locate_binaries(&root, "spectred").await {
            if let Some(path) = binaries.first() {
                settings.push((Self::Location, to_value(path.to_string_lossy().to_string()).unwrap()));
            }
        }

        settings
    }
}

pub struct Node {
    settings: SettingsStore<SpectredSettings>,
    mute: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            settings: SettingsStore::try_new("spectred").expect("Failed to create node settings store"),
            mute: Arc::new(AtomicBool::new(true)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl Handler for Node {
    fn verb(&self, ctx: &Arc<dyn Context>) -> Option<&'static str> {
        if let Ok(ctx) = ctx.clone().downcast_arc::<SpectreCli>() {
            ctx.daemons().clone().spectred.as_ref().map(|_| "node")
        } else {
            None
        }
    }

    fn help(&self, _ctx: &Arc<dyn Context>) -> &'static str {
        "Manage the local Spectre node instance."
    }

    async fn start(self: Arc<Self>, _ctx: &Arc<dyn Context>) -> cli::Result<()> {
        self.settings.try_load().await.ok();
        if let Some(mute) = self.settings.get(SpectredSettings::Mute) {
            self.mute.store(mute, Ordering::Relaxed);
        }
        Ok(())
    }

    async fn handle(self: Arc<Self>, ctx: &Arc<dyn Context>, argv: Vec<String>, cmd: &str) -> cli::Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;
        self.main(ctx, argv, cmd).await.map_err(|e| e.into())
    }
}

impl Node {
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    async fn create_config(&self, ctx: &Arc<SpectreCli>) -> Result<SpectredConfig> {
        let location: String = self
            .settings
            .get(SpectredSettings::Location)
            .ok_or_else(|| Error::Custom("No miner binary specified, please use `node select` to select a binary.".into()))?;
        let network_id = ctx.wallet().network_id()?;
        // Disabled for prompt update (until progress events are implemented)
        // let mute = self.mute.load(Ordering::SeqCst);
        let mute = false;
        let config = SpectredConfig::new(location.as_str(), network_id, mute);
        Ok(config)
    }

    async fn main(self: Arc<Self>, ctx: Arc<SpectreCli>, mut argv: Vec<String>, cmd: &str) -> Result<()> {
        if argv.is_empty() {
            return self.display_help(ctx, argv).await;
        }
        let spectred = ctx.daemons().spectred();
        match argv.remove(0).as_str() {
            "start" => {
                let mute = self.mute.load(Ordering::SeqCst);
                if mute {
                    tprintln!(ctx, "Starting Spectre node... {}", style("(logs are muted, use 'node mute' to toggle)").dim());
                } else {
                    tprintln!(ctx, "Starting Spectre node... {}", style("(use 'node mute' to mute logging)").dim());
                }

                let wrpc_client = ctx.wallet().try_wrpc_client().ok_or(Error::custom("Unable to start node with non-wRPC client"))?;

                spectred.configure(self.create_config(&ctx).await?).await?;
                spectred.start().await?;

                // Temporary setup for auto-connect
                let url = ctx.wallet().settings().get(WalletSettings::Server);
                let network_type = ctx.wallet().network_id()?;
                if let Some(url) = url
                    .map(|url| wrpc_client.parse_url_with_network_type(url, network_type.into()).map_err(|e| e.to_string()))
                    .transpose()?
                {
                    // log_info!("connecting to url: {}", url);
                    if url.contains("127.0.0.1") || url.contains("localhost") {
                        spawn(async move {
                            let options = ConnectOptions {
                                block_async_connect: true,
                                strategy: ConnectStrategy::Fallback,
                                url: Some(url),
                                ..Default::default()
                            };
                            for _ in 0..5 {
                                sleep(Duration::from_millis(1000)).await;
                                if wrpc_client.connect(Some(options.clone())).await.is_ok() {
                                    break;
                                }
                            }
                        });
                    }
                }
            }
            "stop" => {
                spectred.stop().await?;
            }
            "restart" => {
                spectred.configure(self.create_config(&ctx).await?).await?;
                spectred.restart().await?;
            }
            "kill" => {
                spectred.kill().await?;
            }
            "mute" | "logs" => {
                let mute = !self.mute.load(Ordering::SeqCst);
                self.mute.store(mute, Ordering::SeqCst);
                if mute {
                    tprintln!(ctx, "{}", style("Node logging is now muted").dim());
                } else {
                    tprintln!(ctx, "{}", style("Node logging is now unmuted").dim());
                }
                self.settings.set(SpectredSettings::Mute, mute).await?;
            }
            "status" => {
                let status = spectred.status().await?;
                tprintln!(ctx, "{}", status);
            }
            "select" => {
                let regex = Regex::new(r"(?i)^\s*node\s+select\s+").unwrap();
                let path = regex.replace(cmd, "").trim().to_string();
                self.select(ctx, path.is_not_empty().then_some(path)).await?;
            }
            "version" => {
                spectred.configure(self.create_config(&ctx).await?).await?;
                let version = spectred.version().await?;
                tprintln!(ctx, "{}", version);
            }
            v => {
                tprintln!(ctx, "Unknown command: '{v}'\r\n");

                return self.display_help(ctx, argv).await;
            }
        }

        Ok(())
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<SpectreCli>, _argv: Vec<String>) -> Result<()> {
        ctx.term().help(
            &[
                ("select", "Select the Spectred executable (binary) location"),
                ("version", "Display the Spectred executable version"),
                ("start", "Start the local Spectre node instance"),
                ("stop", "Stop the local Spectre node instance"),
                ("restart", "Restart the local Spectre node instance"),
                ("kill", "Kill the local Spectre node instance"),
                ("status", "Get the status of the local Spectre node instance"),
                ("mute", "Toggle logging output"),
            ],
            None,
        )?;

        Ok(())
    }

    async fn select(self: Arc<Self>, ctx: Arc<SpectreCli>, path: Option<String>) -> Result<()> {
        let root = nw_sys::app::folder();

        match path {
            None => {
                let binaries = spectre_daemon::locate_binaries(root.as_str(), "spectred").await?;

                if binaries.is_empty() {
                    tprintln!(ctx, "No Spectred binaries found.");
                } else {
                    let binaries = binaries.iter().map(|p| p.display().to_string()).collect::<Vec<_>>();
                    if let Some(selection) = ctx.term().select("Please select a Spectred binary", &binaries).await? {
                        tprintln!(ctx, "Selecting: {}", selection);
                        self.settings.set(SpectredSettings::Location, selection.as_str()).await?;
                    } else {
                        tprintln!(ctx, "No selection made.");
                    }
                }
            }
            Some(path) => {
                if fs::exists(&path).await? {
                    let version = process::version(&path).await?;
                    tprintln!(ctx, "Detected binary version: {}", version);
                    tprintln!(ctx, "Selecting: {path}");
                    self.settings.set(SpectredSettings::Location, path.as_str()).await?;
                } else {
                    twarnln!(ctx, "Destination binary not found, please specify the full path including the binary name.");
                    twarnln!(ctx, "Example: 'node select /home/user/testnet/spectred'");
                    tprintln!(ctx, "No selection made.");
                }
            }
        }

        Ok(())
    }

    pub async fn handle_event(&self, ctx: &Arc<SpectreCli>, event: Event) -> Result<()> {
        let term = ctx.term();

        match event {
            Event::Start => {
                self.is_running.store(true, Ordering::SeqCst);
                term.refresh_prompt();
            }
            Event::Exit(_code) => {
                tprintln!(ctx, "Spectred has exited");
                self.is_running.store(false, Ordering::SeqCst);
                term.refresh_prompt();
            }
            Event::Error(error) => {
                tprintln!(ctx, "{}", style(format!("Spectred error: {error}")).red());
                self.is_running.store(false, Ordering::SeqCst);
                term.refresh_prompt();
            }
            Event::Stdout(text) | Event::Stderr(text) => {
                if !ctx.wallet().utxo_processor().is_synced() {
                    ctx.wallet().utxo_processor().sync_proc().handle_stdout(&text).await?;
                }

                if !self.mute.load(Ordering::SeqCst) {
                    let sanitize = true;
                    if sanitize {
                        let lines = text.split('\n').collect::<Vec<_>>();
                        lines.into_iter().for_each(|line| {
                            let line = line.trim();
                            if !line.is_empty() {
                                if line.len() < 38 || &line[30..31] != "[" {
                                    term.writeln(line);
                                } else {
                                    let time = &line[11..23];
                                    let kind = &line[31..36];
                                    let text = &line[38..];

                                    match kind {
                                        "WARN " => {
                                            term.writeln(format!("{time} {}", style(text).yellow()));
                                        }
                                        "ERROR" => {
                                            term.writeln(format!("{time} {}", style(text).red()));
                                        }
                                        _ => {
                                            if text.starts_with("Processed") {
                                                term.writeln(format!("{time} {}", style(text).blue()));
                                            } else {
                                                term.writeln(format!("{time} {text}"));
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    } else {
                        term.writeln(text.trim().crlf());
                    }
                }
            }
        }
        Ok(())
    }
}
