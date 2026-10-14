use std::sync::Arc;

use anyhow::Context;
use codex_app_server_client::DEFAULT_IN_PROCESS_CHANNEL_CAPACITY;
use codex_app_server_client::EmbeddedNetworkPolicy;
use codex_app_server_client::EnvironmentManager;
use codex_app_server_client::ExecServerRuntimeOptions;
use codex_app_server_client::InProcessAppServerClient;
use codex_app_server_client::InProcessClientStartArgs;
use codex_app_server_protocol::ConfigWarningNotification;
use codex_cloud_config::cloud_config_bundle_loader_for_storage;
use codex_config::CloudConfigBundleLoader;
use codex_config::ConfigLoadOptions;
use codex_config::LoaderOverrides;
use codex_core::config::Config;
use codex_core::config::ConfigBuilder;
use codex_core::config::ConfigOverrides;
use codex_core::config::bootstrap_auth_config;
use codex_core::config::find_codex_home;
use codex_core::config::load_config_toml_with_layer_stack;
use codex_feedback::CodexFeedback;
use codex_protocol::protocol::SessionSource;
use codex_utils_absolute_path::AbsolutePathBuf;

use crate::launch::LaunchOptions;

const CLIENT_NAME: &str = "codex-gui";

struct Bootstrap {
    codex_home: AbsolutePathBuf,
    loader_overrides: LoaderOverrides,
    network_policy: EmbeddedNetworkPolicy,
    cloud_config_bundle: CloudConfigBundleLoader,
}

pub async fn start_client(launch: &LaunchOptions) -> anyhow::Result<InProcessAppServerClient> {
    let bootstrap = prepare_bootstrap(launch).await?;
    let config = load_config(launch, &bootstrap).await?;
    let start_args = client_start_args(launch, bootstrap, config).await?;

    InProcessAppServerClient::start(start_args)
        .await
        .context("failed to start the embedded app-server")
}

async fn prepare_bootstrap(launch: &LaunchOptions) -> anyhow::Result<Bootstrap> {
    let codex_home = find_codex_home().context("failed to locate CODEX_HOME")?;
    let loader_overrides = LoaderOverrides::default();
    let network_policy = EmbeddedNetworkPolicy::load(&loader_overrides).await;

    let cloud_config_bundle =
        load_cloud_config_bundle(launch, &codex_home, &loader_overrides, &network_policy).await?;

    Ok(Bootstrap {
        codex_home,
        loader_overrides,
        network_policy,
        cloud_config_bundle,
    })
}

async fn load_cloud_config_bundle(
    launch: &LaunchOptions,
    codex_home: &AbsolutePathBuf,
    loader_overrides: &LoaderOverrides,
    network_policy: &EmbeddedNetworkPolicy,
) -> anyhow::Result<CloudConfigBundleLoader> {
    let cwd = AbsolutePathBuf::from_absolute_path(&launch.cwd)?;
    let options = ConfigLoadOptions {
        loader_overrides: loader_overrides.clone(),
        strict_config: false,
        cloud_config_bundle: CloudConfigBundleLoader::default(),
    };

    let bootstrap_config =
        load_config_toml_with_layer_stack(codex_home, Some(&cwd), Vec::new(), options)
            .await
            .context("failed to load config.toml")?;
    let auth_config =
        network_policy.bind_bootstrap_auth(bootstrap_auth_config(codex_home, &bootstrap_config)?);

    Ok(
        cloud_config_bundle_loader_for_storage(
            auth_config,
            /*enable_codex_api_key_env*/ false,
        )
        .await?,
    )
}

async fn load_config(launch: &LaunchOptions, bootstrap: &Bootstrap) -> anyhow::Result<Config> {
    let overrides = ConfigOverrides {
        cwd: Some(launch.cwd.clone()),
        codex_self_exe: launch.codex_self_exe.clone(),
        codex_linux_sandbox_exe: launch.codex_linux_sandbox_exe.clone(),
        ..ConfigOverrides::default()
    };

    let mut config = ConfigBuilder::default()
        .codex_home(bootstrap.codex_home.to_path_buf())
        .harness_overrides(overrides)
        .loader_overrides(bootstrap.loader_overrides.clone())
        .cloud_config_bundle(bootstrap.cloud_config_bundle.clone())
        .build()
        .await
        .context("failed to resolve Codex configuration")?;

    bootstrap.network_policy.activate(&mut config);
    Ok(config)
}

async fn client_start_args(
    launch: &LaunchOptions,
    bootstrap: Bootstrap,
    config: Config,
) -> anyhow::Result<InProcessClientStartArgs> {
    let environment_manager = environment_manager(launch, &bootstrap, &config).await?;
    let state_db = codex_core::init_state_db(&config).await;
    let config_warnings = config_warnings(&config);

    Ok(InProcessClientStartArgs {
        arg0_paths: launch.dispatch_paths(),
        config: Arc::new(config),
        cli_overrides: Vec::new(),
        loader_overrides: bootstrap.loader_overrides,
        strict_config: false,
        cloud_config_bundle: bootstrap.cloud_config_bundle,
        embedded_network_policy: bootstrap.network_policy,
        feedback: CodexFeedback::new(),
        log_db: None,
        state_db,
        environment_manager: Arc::new(environment_manager),
        config_warnings,
        session_source: SessionSource::VSCode,
        enable_codex_api_key_env: false,
        client_name: CLIENT_NAME.to_string(),
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        experimental_api: true,
        mcp_server_openai_form_elicitation: false,
        opt_out_notification_methods: Vec::new(),
        channel_capacity: DEFAULT_IN_PROCESS_CHANNEL_CAPACITY,
    })
}

async fn environment_manager(
    launch: &LaunchOptions,
    bootstrap: &Bootstrap,
    config: &Config,
) -> anyhow::Result<EnvironmentManager> {
    let runtime_options = ExecServerRuntimeOptions::from_optional_paths(
        launch.codex_self_exe.clone(),
        launch.codex_linux_sandbox_exe.clone(),
    )?;
    let http_client_factory = bootstrap.network_policy.bind(config.http_client_factory());

    let manager = EnvironmentManager::from_codex_home(
        config.codex_home.clone(),
        Some(runtime_options),
        http_client_factory,
    )
    .await?;
    Ok(manager)
}

fn config_warnings(config: &Config) -> Vec<ConfigWarningNotification> {
    config
        .startup_warnings
        .iter()
        .map(|summary| ConfigWarningNotification {
            summary: summary.clone(),
            details: None,
            path: None,
            range: None,
        })
        .collect()
}
