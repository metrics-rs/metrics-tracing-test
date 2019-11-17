use metrics::{Key, Recorder};
use tracing::{debug, error, info, span, trace, warn, Level};
use tracing_subscriber::{layer::Layer, registry::Registry};

mod thingy;
use self::thingy::Thingy;

mod layer;
use self::layer::Metrics;

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
        println!("metrics -> histogram(name={}, value={})", key, value);
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
            let thingy = Thingy::default();
            thingy.handle_unshaved(yak);
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

    let subscriber = Metrics::default().with_subscriber(Registry::default());

    tracing::subscriber::with_default(subscriber, || {
        let number_of_yaks = 3;
        debug!("preparing to shave {} yaks", number_of_yaks);

        let number_shaved = shave_all(number_of_yaks);

        debug!(
            message = "yak shaving completed.",
            all_yaks_shaved = number_shaved == number_of_yaks,
        );
    });
}
