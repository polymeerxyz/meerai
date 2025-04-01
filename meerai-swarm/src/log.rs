use std::io::{IsTerminal, stderr};

use tracing_glog::{Glog, GlogFields};
use tracing_subscriber::{Registry, filter::EnvFilter, layer::SubscriberExt};

pub fn init_logging() {
    let directives = tracing_subscriber::filter::Directive::from(tracing::Level::DEBUG);

    let fmt = tracing_subscriber::fmt::Layer::default()
        .with_ansi(stderr().is_terminal())
        .with_writer(std::io::stderr)
        .event_format(Glog::default().with_timer(tracing_glog::LocalTime::default()))
        .fmt_fields(GlogFields::default().compact());

    let filter = vec![directives]
        .into_iter()
        .fold(EnvFilter::from_default_env(), |filter, directive| {
            filter.add_directive(directive)
        });

    let subscriber = Registry::default().with(filter).with(fmt);
    tracing::subscriber::set_global_default(subscriber).expect("to set global subscriber");
}
