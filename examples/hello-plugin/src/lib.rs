use winforge_plugin::prelude::*;

pub struct HelloPlugin {
    greeting: String,
}

impl HelloPlugin {
    pub fn new() -> Self {
        Self { greeting: "Hello from hello-plugin!".to_string() }
    }
}

#[async_trait]
impl Plugin for HelloPlugin {
    fn name(&self) -> &str {
        "hello-plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn on_load(&mut self, host: &PluginHost) -> CoreResult<()> {
        host.require_capability(&Capability::EventsPublish)?;
        info!("{}", self.greeting);
        Ok(())
    }

    async fn on_unload(&mut self) -> CoreResult<()> {
        info!("hello-plugin unloaded — goodbye!");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Generate the `wf_plugin_create` C export that WinForge's loader looks for.
winforge_plugin::export_plugin!(HelloPlugin::new());
