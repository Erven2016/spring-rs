#[cfg(feature = "file")]
pub mod file;
#[cfg(feature = "kafka")]
pub mod kafka;
#[cfg(feature = "redis")]
pub mod redis;
#[cfg(feature = "stdio")]
pub mod stdio;

use schemars::JsonSchema;
use sea_streamer::{
    ConsumerGroup, ConsumerMode, ConsumerOptions, SeaConnectOptions, SeaConsumerOptions,
    SeaProducerOptions,
};
use serde::Deserialize;

use crate::consumer::ConsumerOpts;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct StreamConfig {
    /// streamer uri
    /// https://docs.rs/sea-streamer-types/latest/sea_streamer_types/struct.StreamerUri.html
    pub(crate) uri: String,

    #[cfg(feature = "kafka")]
    pub(crate) kafka: Option<kafka::KafkaOptions>,
    #[cfg(feature = "redis")]
    pub(crate) redis: Option<redis::RedisOptions>,
    #[cfg(feature = "stdio")]
    pub(crate) stdio: Option<stdio::StdioOptions>,
    #[cfg(feature = "file")]
    pub(crate) file: Option<file::FileOptions>,
}

impl StreamConfig {
    pub fn connect_options(&self) -> SeaConnectOptions {
        let mut connect_options = SeaConnectOptions::default();

        #[cfg(feature = "kafka")]
        if let Some(kafka) = &self.kafka {
            connect_options.set_kafka_connect_options(|opts| kafka.fill_connect_options(opts));
        }
        #[cfg(feature = "redis")]
        if let Some(redis) = &self.redis {
            connect_options.set_redis_connect_options(|opts| redis.fill_connect_options(opts));
        }
        #[cfg(feature = "stdio")]
        if let Some(stdio) = &self.stdio {
            connect_options.set_stdio_connect_options(|opts| stdio.fill_connect_options(opts));
        }
        #[cfg(feature = "file")]
        if let Some(file) = &self.file {
            connect_options.set_file_connect_options(|opts| file.fill_connect_options(opts));
        }
        connect_options
    }

    pub fn new_consumer_options(&self, ConsumerOpts(opts): ConsumerOpts) -> SeaConsumerOptions {
        let mode = opts.mode().ok();
        let group = match opts.consumer_group() {
            Ok(group) => Some(group),
            _ => None,
        };
        #[cfg(feature = "kafka")]
        if let Some(kafka) = &self.kafka {
            let mut consumer_options = kafka.new_consumer_options(mode, group);
            consumer_options.set_kafka_consumer_options(|opts| kafka.fill_consumer_options(opts));
            return consumer_options;
        }
        #[cfg(feature = "redis")]
        if let Some(redis) = &self.redis {
            let mut consumer_options = redis.new_consumer_options(mode, group);
            consumer_options.set_redis_consumer_options(|opts| redis.fill_consumer_options(opts));
            return consumer_options;
        }
        #[cfg(feature = "stdio")]
        if let Some(stdio) = &self.stdio {
            let mut consumer_options = stdio.new_consumer_options(mode, group);
            consumer_options.set_stdio_consumer_options(|opts| stdio.fill_consumer_options(opts));
            return consumer_options;
        }
        #[cfg(feature = "file")]
        if let Some(file) = &self.file {
            let mut consumer_options = file.new_consumer_options(mode, group);
            consumer_options.set_file_consumer_options(|opts| file.fill_consumer_options(opts));
            return consumer_options;
        }
        opts
    }

    pub fn new_producer_options(&self) -> SeaProducerOptions {
        let mut producer_options = SeaProducerOptions::default();
        #[cfg(feature = "kafka")]
        if let Some(kafka) = &self.kafka {
            producer_options.set_kafka_producer_options(|opts| kafka.fill_producer_options(opts));
        }
        #[cfg(feature = "redis")]
        if let Some(redis) = &self.redis {
            producer_options.set_redis_producer_options(|opts| redis.fill_producer_options(opts));
        }
        #[cfg(feature = "stdio")]
        if let Some(stdio) = &self.stdio {
            producer_options.set_stdio_producer_options(|opts| stdio.fill_producer_options(opts));
        }
        #[cfg(feature = "file")]
        if let Some(file) = &self.file {
            producer_options.set_file_producer_options(|opts| file.fill_producer_options(opts));
        }
        producer_options
    }
}

pub(crate) trait OptionsFiller {
    type ConnectOptsType;
    type ConsumerOptsType;
    type ProducerOptsType;
    fn fill_connect_options(&self, opts: &mut Self::ConnectOptsType);
    fn fill_consumer_options(&self, opts: &mut Self::ConsumerOptsType);
    fn fill_producer_options(&self, opts: &mut Self::ProducerOptsType);

    fn new_consumer_options(
        &self,
        mode: Option<&ConsumerMode>,
        group_id: Option<&ConsumerGroup>,
    ) -> SeaConsumerOptions {
        let mode = match mode.or_else(|| self.default_consumer_mode()) {
            Some(mode) => mode.to_owned(),
            None => ConsumerMode::default(),
        };
        let mut opts = SeaConsumerOptions::new(mode);
        let group_id = match group_id {
            Some(group) => Some(group.name().to_string()),
            None => self.default_consumer_group_id(),
        };
        if let Some(group_id) = group_id {
            let _ = opts.set_consumer_group(ConsumerGroup::new(group_id));
        }
        opts
    }
    fn default_consumer_mode(&self) -> Option<&ConsumerMode>;
    fn default_consumer_group_id(&self) -> Option<String>;
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "ConsumerMode")]
pub(crate) enum ConsumerModeRef {
    /// This is the 'vanilla' stream consumer. It does not auto-commit, and thus only consumes messages from now on.
    RealTime,
    /// When the process restarts, it will resume the stream from the previous committed sequence.
    Resumable,
    /// You should assign a consumer group manually. The load-balancing mechanism is implementation-specific.
    LoadBalanced,
}