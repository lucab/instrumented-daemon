use actix::prelude::*;
use tokio::net::{UnixListener, UnixStream};
use tokio::prelude::*;
use prometheus::{IntCounter, register_int_counter, register_counter, opts};

static SOCKET_PATH: &str = "/run/daemon2/public/metrics.promsock";

static FILE_PATH: &str = "/run/daemon1/public/metrics.promfile";


lazy_static::lazy_static! {
    static ref SIMPLE_COUNTER: IntCounter = register_int_counter!(
        "example_counter",
        "Dummy counter"
    ).unwrap();
}

fn main() {
    env_logger::init();
    let sys = actix::System::new("unix-socket");

    std::fs::File::create(FILE_PATH).expect("file create failed");
    let d1 = Daemon1 { };
    d1.start();
    println!("Started text-file producer: {}", FILE_PATH);

    let listener = UnixListener::bind(SOCKET_PATH).expect("socket bind failed");
    let d2 = Daemon2 {
        listener: Some(listener),
    };
    d2.start();
    println!("Started unix-socket producer: {}", SOCKET_PATH);

    sys.run().expect("actix exited");
}

fn encode() -> Vec<u8> {
    use prometheus::Encoder;
    let mut buffer = Vec::new();
    let encoder = prometheus::TextEncoder::new();

    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    buffer
}

// ---- Text file ----

struct Daemon1 {
}

impl Actor for Daemon1 {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(std::time::Duration::from_secs(5), move |_act, _ctx| {
            SIMPLE_COUNTER.inc();

            //NOTE: this can block/panic in a place where it should not.
            let mut fp = std::fs::File::create(FILE_PATH).expect("file create failed");
            fp.write_all(&encode()).unwrap_or_default();
        });
    }
}

// ---- Unix socket ----

struct Daemon2 {
    listener: Option<UnixListener>,
}

#[derive(Message)]
struct Connection {
    stream: UnixStream,
}

impl actix::io::WriteHandler<std::io::Error> for Daemon2 {
    fn error(&mut self, _err: std::io::Error, _ctx: &mut Self::Context) -> Running {
        actix::Running::Continue
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {}
}

impl StreamHandler<Connection, std::io::Error> for Daemon2 {
    fn handle(&mut self, item: Connection, ctx: &mut Context<Daemon2>) {
        let (_, sink) = item.stream.split();
        let mut wr = actix::io::Writer::new(sink, ctx);
        wr.write(&encode());
        wr.close();
    }
}

impl Actor for Daemon2 {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let connections = self
            .listener
            .take()
            .unwrap()
            .incoming()
            .map(|stream| Connection { stream });
        ctx.add_stream(connections);
    }
}
