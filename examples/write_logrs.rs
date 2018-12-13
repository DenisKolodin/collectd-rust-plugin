#![cfg(feature = "serde")]

#[macro_use]
extern crate collectd_plugin;
extern crate chrono;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use chrono::Duration;
use collectd_plugin::{
    collectd_log, CollectdLoggerBuilder, ConfigItem, LogLevel, Plugin, PluginCapabilities,
    PluginManager, PluginRegistration, ValueList,
};
use itertools::Itertools;
use log::LevelFilter;
use std::error;

fn true_default() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct LogWritePlugin {
    #[serde(default = "true_default", rename = "StoreRates")]
    store_rates: bool,
}

impl Drop for LogWritePlugin {
    fn drop(&mut self) {
        info!("yes drop is called");
    }
}

impl PluginManager for LogWritePlugin {
    fn name() -> &'static str {
        "write_logrs"
    }

    fn plugins(config: Option<&[ConfigItem]>) -> Result<PluginRegistration, Box<error::Error>> {
        // Register a logging hook so that any usage of the `log` crate will be forwarded to
        // collectd's logging facilities
        CollectdLoggerBuilder::new()
            .prefix_plugin::<Self>()
            .filter_level(LevelFilter::Info)
            .try_init()
            .expect("really the only thing that should create a logger");

        collectd_log_raw!(LogLevel::Info, b"A raw log with argument: %d\0", 10);
        let line = format!("collectd logging configuration: {:?}", config);
        collectd_log(LogLevel::Info, &line);
        info!("rust logging configuration: {:?}", config);
        let plugin: LogWritePlugin =
            collectd_plugin::de::from_collectd(config.unwrap_or_else(Default::default))?;
        Ok(PluginRegistration::Single(Box::new(plugin)))
    }
}

impl Plugin for LogWritePlugin {
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::WRITE | PluginCapabilities::FLUSH
    }

    fn write_values(&self, list: ValueList) -> Result<(), Box<error::Error>> {
        let values = if self.store_rates {
            list.rates()
        } else {
            Ok(::std::borrow::Cow::Borrowed(&list.values))
        }?;

        let values = values
            .iter()
            .map(|v| format!("{} - {}", v.name, v.value))
            .join(", ");

        info!(
            "plugin_instance: {}, plugin: {}, type: {}, type_instance: {}, host: {}, time: {}, interval: {} seconds, values: {}",
            list.plugin_instance.unwrap_or("<none>"),
            list.plugin,
            list.type_,
            list.type_instance.unwrap_or("<none>"),
            list.host,
            list.time,
            list.interval.num_seconds(),
            values,
        );

        Ok(())
    }

    fn flush(
        &self,
        timeout: Option<Duration>,
        identifier: Option<&str>,
    ) -> Result<(), Box<error::Error>> {
        info!(
            "flushing: timeout: {}, identifier: {}",
            timeout
                .map(|x| format!("{}", x))
                .unwrap_or_else(|| String::from("no timeout")),
            identifier
                .map(|x| x.to_string())
                .unwrap_or_else(|| String::from("no identifier"))
        );
        Ok(())
    }
}

collectd_plugin!(LogWritePlugin);