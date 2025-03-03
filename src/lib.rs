use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::Args;
use serde::{Deserialize, Serialize};
use spin_app::AppComponent;
use spin_core::Engine;
use spin_trigger::TriggerInstancePre;
use spin_trigger::{TriggerAppEngine, TriggerExecutor};
use std::env;
use std::path::Path;
use wasmtime_wasi::bindings::Command;

mod github;

use github::{static_env_vars, static_vol_mounts, GITHUB_ENV};
type RuntimeData = ();
type Store = spin_core::Store<RuntimeData>;

pub struct CommandTrigger {
    engine: TriggerAppEngine<Self>,
    components: Vec<Component>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    pub env: Vec<String>,
}

#[derive(Args, Debug)]
#[clap(trailing_var_arg(true))]
pub struct CliArgs {
    #[clap(multiple_values(true), allow_hyphen_values(true))]
    pub guest_args: Vec<String>,
}

impl CliArgs {
    fn apply_args_to_store(
        &self,
        component_id: &str,
        store_builder: &mut spin_core::StoreBuilder,
    ) -> Result<()> {
        // Insert the component id as the first argument as the command name
        let args = vec![component_id]
            .into_iter()
            .chain(self.guest_args.iter().map(|arg| &**arg))
            .collect::<Vec<&str>>();

        store_builder.args(args)?;

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandTriggerConfig {
    pub component: String,
    pub env: Vec<String>,
}

pub enum CommandInstancePre {
    Component(spin_core::InstancePre<RuntimeData>),
    Module(spin_core::ModuleInstancePre<RuntimeData>),
}

pub enum CommandInstance {
    Component(spin_core::Instance),
    Module(spin_core::ModuleInstance),
}

#[async_trait]
impl TriggerExecutor for CommandTrigger {
    const TRIGGER_TYPE: &'static str = "ghaction";
    type RuntimeData = RuntimeData;
    type TriggerConfig = CommandTriggerConfig;
    type RunConfig = CliArgs;
    type InstancePre = CommandInstancePre;

    async fn new(engine: TriggerAppEngine<Self>) -> Result<Self> {
        let components = engine
            .trigger_configs()
            .map(|(_, config)| Component {
                id: config.component.clone(),
                env: config.env.clone(),
            })
            .collect();
        Ok(Self { engine, components })
    }

    async fn run(self, config: Self::RunConfig) -> Result<()> {
        self.handle(config).await
    }
}

#[async_trait]
impl TriggerInstancePre<RuntimeData, CommandTriggerConfig> for CommandInstancePre {
    type Instance = CommandInstance;

    async fn instantiate_pre(
        engine: &Engine<RuntimeData>,
        component: &AppComponent,
        _config: &CommandTriggerConfig,
    ) -> Result<CommandInstancePre> {
        // Attempt to load as a module and fallback to loading a component
        match component.load_module(engine).await {
            Ok(m) => Ok(CommandInstancePre::Module(
                engine
                    .module_instantiate_pre(&m)
                    .context("Preview1 modules supports only preview1 imports")?,
            )),
            Err(module_load_err) => match component.load_component(engine).await {
                Ok(c) => Ok(CommandInstancePre::Component(engine.instantiate_pre(&c)?)),
                Err(component_load_err) => Err(anyhow!("{component_load_err}")
                    .context(module_load_err)
                    .context("failed to load component or module")),
            },
        }
    }

    async fn instantiate(&self, store: &mut Store) -> Result<CommandInstance> {
        match self {
            CommandInstancePre::Component(pre) => pre
                .instantiate_async(store)
                .await
                .map(CommandInstance::Component),
            CommandInstancePre::Module(pre) => pre
                .instantiate_async(store)
                .await
                .map(CommandInstance::Module),
        }
    }
}

impl CommandTrigger {
    pub async fn handle(&self, args: CliArgs) -> Result<()> {
        let component = &self.components[0];
        let mut store_builder = self
            .engine
            .store_builder(&component.id, spin_core::WasiVersion::Preview2)?;

        // inject env variables
        let component_env_vars = get_env_for_component(component.env.clone())?;
        store_builder.env(component_env_vars)?;

        for mount in static_vol_mounts().iter() {
            if !Path::new(mount.0).exists() {
                return Err(anyhow::anyhow!("dir {} does not exist", mount.0));
            }

            store_builder.read_write_preopened_dir(mount.0, mount.1.into())?;
        }

        args.apply_args_to_store(&component.id, &mut store_builder)?;

        let (instance, mut store) = self
            .engine
            .prepare_instance_with_store(&component.id, store_builder)
            .await?;
        match instance {
            CommandInstance::Component(instance) => {
                let handler = Command::new(&mut store, &instance)
                    .context("Wasi preview 2 components need to target the wasi:cli world")?;
                let _ = handler.wasi_cli_run().call_run(store).await?;
            }
            CommandInstance::Module(_) => {
                // Toss the commandInstance we have and create a new one as the
                // associated store will be a preview2 store
                let mut store_builder = self
                    .engine
                    .store_builder(&component.id, spin_core::WasiVersion::Preview1)?;

                args.apply_args_to_store(&component.id, &mut store_builder)?;

                let (instance, mut store) = self
                    .engine
                    .prepare_instance_with_store(&component.id, store_builder)
                    .await?;
                let CommandInstance::Module(instance) = instance else {
                    unreachable!();
                };

                let start = instance
                    .get_func(&mut store, "_start")
                    .context("Expected component to export _start function")?;

                start.call_async(&mut store, &[], &mut []).await?;
            }
        }

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
