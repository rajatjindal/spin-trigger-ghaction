use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use spin_factor_wasi::WasiFactor;
use spin_factors::RuntimeFactors;
use spin_trigger::{Trigger, TriggerApp};
use std::env;
use std::path::Path;

mod github;

use github::{static_env_vars, static_vol_mounts, GITHUB_ENV};

pub struct GitHubActionsTrigger {
    components: Vec<Component>,
    config: CliArgs,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    pub env: Vec<String>,
    pub mount_rw_dir: bool,
}

#[derive(Args, Debug, Clone)]
#[clap(trailing_var_arg(true))]
pub struct CliArgs {
    #[clap(multiple_values(true), allow_hyphen_values(true))]
    pub guest_args: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubActionsTriggerConfig {
    pub component: String,
}

impl<F: RuntimeFactors> Trigger<F> for GitHubActionsTrigger {
    const TYPE: &'static str = "ghaction";

    type CliArgs = CliArgs;

    type InstanceState = ();

    fn new(cli_args: Self::CliArgs, app: &spin_trigger::App) -> anyhow::Result<Self> {
        let components: Vec<Component> = app
            .trigger_configs::<GitHubActionsTriggerConfig>(<Self as Trigger<F>>::TYPE)?
            .into_iter()
            .map(|(_, config)| Component {
                id: config.component.clone(),
                env: vec![],        //TODO: fix me,
                mount_rw_dir: true, // TODO: fix me,
            })
            .collect();
        if components.len() > 1 {
            tracing::warn!(
                "Multiple components found for ghaction trigger, only the first one will be used"
            );
        }

        if components.is_empty() {
            return Err(anyhow::anyhow!(
                "No components found for ghaction trigger, exiting"
            ));
        }

        Ok(Self {
            components,
            config: cli_args,
        })
    }

    async fn run(self, trigger_app: spin_trigger::TriggerApp<Self, F>) -> anyhow::Result<()> {
        Self::handle(
            self.components
                .first()
                .context("Failed to get the component for the ghaction trigger")?
                .to_owned(),
            trigger_app.into(),
            self.config.clone(),
        )
        .await
    }
}

impl GitHubActionsTrigger {
    pub async fn handle<F: RuntimeFactors>(
        component: Component,
        trigger_app: Arc<TriggerApp<Self, F>>,
        args: CliArgs,
    ) -> Result<()> {
        let mut instance_builder = trigger_app.prepare(&component.id)?;
        if let Some(wasi) = instance_builder.factor_builder::<WasiFactor>() {
            let args = std::iter::once(component.id).chain(args.guest_args);
            wasi.args(args);

            // inject env variables
            let component_env_vars = get_env_for_component(component.env.clone())?;
            wasi.env(component_env_vars);

            // TODO(rajatjindal): make the dir configurable
            if component.mount_rw_dir {
                for mount in static_vol_mounts().iter() {
                    if !Path::new(mount.0).exists() {
                        return Err(anyhow::anyhow!("dir {} does not exist", mount.0));
                    }

                    wasi.preopened_dir(mount.0, mount.1, true)?;
                }
            }
        }

        let (instance, mut store) = instance_builder.instantiate(()).await?;
        let func = wasmtime_wasi::bindings::Command::new(&mut store, &instance)?;
        let func = func.wasi_cli_run();
        let _ = func.call_run(store).await?;

        Ok(())
    }
}

fn get_env_for_component(requested_by_user: Vec<String>) -> Result<Vec<(String, String)>> {
    // setup env variables for component
    let mut component_env_vars: Vec<(String, String)> = vec![];

    // add env variables injected by GitHub
    for var in GITHUB_ENV {
        match env::var(var) {
            Ok(val) => component_env_vars.push((var.to_string(), val)),
            Err(_) => tracing::trace!("env variable {} not found", var),
        }
    }

    // add env variables with static values
    component_env_vars.extend(
        static_env_vars()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    );

    // add env variables requested by user
    for var in requested_by_user.iter() {
        match env::var(var) {
            Ok(val) => component_env_vars.push((var.clone(), val)),
            Err(_) => tracing::trace!("env variable {} not found", var),
        }
    }

    Ok(component_env_vars)
}
