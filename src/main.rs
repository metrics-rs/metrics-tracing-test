use tracing_core::{
    Subscriber, Metadata, Event,
    span::{Attributes, Record, Id},
};
use tracing::{debug, error, info, span, trace, warn, Level};
use std::sync::Arc;
use parking_lot::Mutex;
use slab::Slab;
use core::convert::TryInto;
use metrics::{Recorder, Key, counter, timing};
use quanta::Clock;

#[derive(Default)]
pub struct MetricsSubscriber {
    spans: Arc<Mutex<Slab<Span>>>,
    clock: Clock,
}

struct Span {
    name: &'static str,
    last_enter: u64,
}

impl Subscriber for MetricsSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, attrs: &Attributes<'_>) -> Id {
        let span = Span {
            name: attrs.metadata().name(),
            last_enter: 0,
        };
        let idx = self.spans.lock().insert(span);
        Id::from_u64((idx + 1).try_into().unwrap())
    }

    fn record(&self, _id: &Id, _values: &Record<'_>) {
    }

    fn record_follows_from(&self, _child_id: &Id, _parent_id: &Id) {
    }

    fn event(&self, _event: &Event<'_>) {
    }

    fn enter(&self, id: &Id) {
        let idx = id.into_u64() as usize - 1;
        if let Some(span) = self.spans.lock().get_mut(idx) {
            span.last_enter = self.clock.now();
        }
    }

    fn exit(&self, id: &Id) {
        let idx = id.into_u64() as usize - 1;
        if let Some(span) = self.spans.lock().get(idx) {
            let last = span.last_enter;
            let now = self.clock.now();
            assert!(now >= last);
            let delta = now - last;

            counter!(span.name, 1);
            timing!(span.name, delta);
        }
    }
}

#[derive(Default)]
struct PrintRecorder;

impl Recorder for PrintRecorder {
    fn increment_counter(&self, key: Key, value: u64) {
        println!("metrics -> counter(name={}, value={})", key, value);
    }

    fn update_gauge(&self, key: Key, value: i64) {
        println!("metrics -> gauge(name={}, value={})", key, value);
    }

    fn record_histogram(&self, key: Key, value: u64) {
        println!("metrics -> histogram(name={}, value={}ns)", key, value);
    }
}

#[tracing::instrument]
fn shave(yak: usize) -> bool {
    debug!(
        message = "hello! I'm gonna shave a yak.",
        excitement = "yay!"
    );
    if yak == 3 {
        warn!(target: "yak_events", "could not locate yak!");
        false
    } else {
        trace!(target: "yak_events", "yak shaved successfully");
        true
    }
}

fn shave_all(yaks: usize) -> usize {
    let span = span!(Level::TRACE, "shaving_yaks", yaks_to_shave = yaks);
    let _enter = span.enter();

    info!("shaving yaks");

    let mut num_shaved = 0;
    for yak in 1..=yaks {
        let shaved = shave(yak);
        trace!(target: "yak_events", yak, shaved);

        if !shaved {
            error!(message = "failed to shave yak!", yak);
        } else {
            num_shaved += 1;
        }

        trace!(target: "yak_events", yaks_shaved = num_shaved);
    }

    num_shaved
}

fn main() {
    let recorder = PrintRecorder::default();
    metrics::set_boxed_recorder(Box::new(recorder)).unwrap();

    let metrics_sub = MetricsSubscriber::default();
    tracing::subscriber::with_default(metrics_sub, || {
        let number_of_yaks = 3;
        debug!("preparing to shave {} yaks", number_of_yaks);

        let number_shaved = shave_all(number_of_yaks);

        debug!(
            message = "yak shaving completed.",
            all_yaks_shaved = number_shaved == number_of_yaks,
        );
    });
}
