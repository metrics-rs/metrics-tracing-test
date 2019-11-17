use metrics::{counter, timing};
use quanta::Clock;
use std::fmt::Debug;
use tracing_core::{
    span::{Attributes, Id, Record},
    Event, Metadata, Subscriber,
};
use tracing_subscriber::{
    layer::{Context, Layer},
    registry::LookupSpan,
};

#[derive(Default)]
pub struct Metrics {
    clock: Clock,
}

#[derive(Default)]
struct MetricData {
    enter_count: u64,
    entered: Option<u64>,
    exited: Option<u64>,
}

impl MetricData {
    pub fn mark_entered(&mut self, now: u64) {
        self.enter_count += 1;
        if self.entered.is_none() {
            self.entered.replace(now);
        }
    }

    pub fn mark_exited(&mut self, now: u64) {
        self.exited.replace(now);
    }

    pub fn flush(&mut self, metadata: &'static Metadata<'static>) {
        let target = metadata.target().replace("::", "_");
        if self.enter_count > 0 {
            counter!(format!("{}_{}", target, metadata.name()), self.enter_count);
            timing!(
                format!("{}_{}_ns", target, metadata.name()),
                self.entered.take().unwrap(),
                self.exited.take().unwrap()
            );
        }
    }
}

impl<S> Layer<S> for Metrics
where
    S: Subscriber + for<'span> LookupSpan<'span> + Debug,
{
    fn new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
        let data = MetricData::default();
        let span = ctx.span(id).expect("in new_span but span does not exist");
        span.extensions_mut().insert(data);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, _ctx: Context<S>) {}

    fn on_event(&self, event: &Event<'_>, _ctx: Context<S>) {}

    fn on_enter(&self, id: &Id, ctx: Context<S>) {
        let span = ctx.span(id).expect("in on_enter but span does not exist");
        let mut ext = span.extensions_mut();
        let data = ext
            .get_mut::<MetricData>()
            .expect("span does not have metric data");
        data.mark_entered(self.clock.now());
    }

    fn on_exit(&self, id: &Id, ctx: Context<S>) {
        let now = self.clock.now();
        let span = ctx.span(id).expect("in on_exit but span does not exist");
        let mut ext = span.extensions_mut();
        let data = ext
            .get_mut::<MetricData>()
            .expect("span does not have metric data");
        data.mark_exited(now);
    }

    fn on_close(&self, id: Id, ctx: Context<S>) {
        let span = ctx.span(&id).expect("in on_close but span does not exist");
        let mut ext = span.extensions_mut();
        let data = ext
            .get_mut::<MetricData>()
            .expect("span does not have metric data");
        data.flush(span.metadata());
    }
}
