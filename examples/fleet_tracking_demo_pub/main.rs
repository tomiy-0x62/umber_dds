use ctrlc;
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Deserializers, Root},
    encode::pattern::PatternEncoder,
    init_config, init_file,
};
use mio_extras::{channel as mio_channel, timer::Timer};
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use rand::{rngs::ThreadRng, Rng, SeedableRng};
use speedy::Writable;
use std::time::{Duration, SystemTime};
use umber_dds::dds::{qos::*, DataWriterStatusChanged, DomainParticipant};
use umber_dds::{DdsData, DdsSerialize, KeyHash};

#[path = "../fleet_tracking_tui/mod.rs"]
mod fleet_tracking_tui;

#[derive(Clone, Debug, DdsData, DdsSerialize)]
struct VehiclePosition {
    #[key]
    vehicle_id: String,
    latitude: f64,
    longitude: f64,
}

impl VehiclePosition {
    fn new(rng: &mut ThreadRng) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let time_str = now.as_millis().to_string();
        let digest = md5::compute(time_str.as_bytes());
        Self {
            vehicle_id: format!("{:x}", digest),
            latitude: rng.gen(),
            longitude: rng.gen(),
        }
    }

    fn update(&mut self, rng: &mut ThreadRng) {
        self.latitude += rng.gen::<f64>();
        self.longitude += rng.gen::<f64>();
    }
}

fn main() {
    let mut tui = fleet_tracking_tui::Tui::new();
    if let Err(_e) = init_file("shapes_logging.yml", Deserializers::default()) {
        let stderr = ConsoleAppender::builder()
            .target(log4rs::append::console::Target::Stderr)
            .encoder(Box::new(PatternEncoder::new(
                "[{l}] [{d(%s%.f)}] [{t}]: {m}{n}",
            )))
            .build();

        let config = Config::builder()
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .build(Root::builder().appender("stderr").build(LevelFilter::Trace))
            .unwrap();

        init_config(config).unwrap();
    }

    let poll = Poll::new().unwrap();
    const DATAWRITER: Token = Token(0);
    const WRITER_TIMER: Token = Token(1);
    const GEN_TIMER: Token = Token(2);
    const DEL_TIMER: Token = Token(3);
    const STOP: Token = Token(5);
    const GEN_PERIOD: Duration = Duration::from_secs(5);
    const DEL_PERIOD: Duration = Duration::from_secs(8);
    const UPDATE_PERIOD: Duration = Duration::from_secs(1);
    const PROB: f64 = 0.3;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let mut small_rng = rand::rngs::SmallRng::seed_from_u64(now.as_nanos() as u64);

    let domain_id = 0;
    // You should specify some Network Interface that DDS use.
    // let nic = vec![std::net::Ipv4Addr::new(127, 0, 0, 1)];
    let nic = Vec::new();
    let participant = DomainParticipant::new(domain_id, nic, None, &mut small_rng);
    let topic_qos = TopicQosBuilder::new()
        .reliability(policy::Reliability::default_besteffort())
        // .lifespan(policy::Lifespan::from_secs(1))
        // .history(policy::History::new(policy::HistoryQosKind::KeepLast, 10))
        .build();
    let topic = participant.create_topic::<VehiclePosition>(
        "Vehicle".to_string(),
        TopicQos::Policies(Box::new(topic_qos)),
    );

    let (stop_sender, stop_receiver) = mio_channel::sync_channel::<()>(1);
    ctrlc::set_handler(move || {
        stop_sender
            .send(())
            .expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    poll.register(&stop_receiver, STOP, Ready::readable(), PollOpt::edge())
        .unwrap();

    let publisher = participant.create_publisher(PublisherQos::Default);
    let dw_qos = DataWriterQosBuilder::new()
        .reliability(policy::Reliability::default_besteffort())
        .deadline(policy::Deadline::new(Duration::from_millis(2000)))
        .build();
    let mut datawriter = publisher
        .create_datawriter::<VehiclePosition>(DataWriterQos::Policies(Box::new(dw_qos)), topic);

    poll.register(&datawriter, DATAWRITER, Ready::readable(), PollOpt::edge())
        .unwrap();
    let mut w_timer = Timer::default();
    poll.register(&w_timer, WRITER_TIMER, Ready::readable(), PollOpt::edge())
        .unwrap();
    w_timer.set_timeout(UPDATE_PERIOD, ());
    let mut gen_timer = Timer::default();
    poll.register(&gen_timer, GEN_TIMER, Ready::readable(), PollOpt::edge())
        .unwrap();
    gen_timer.set_timeout(GEN_PERIOD, ());
    let mut del_timer = Timer::default();
    poll.register(&del_timer, DEL_TIMER, Ready::readable(), PollOpt::edge())
        .unwrap();
    del_timer.set_timeout(DEL_PERIOD, ());

    let mut vehicles: Vec<VehiclePosition> = Vec::new();
    let mut rng = rand::thread_rng();
    // let is_true = rng.gen_bool(PROB);
    let v = VehiclePosition::new(&mut rng);
    vehicles.push(v);

    'pub_loop: loop {
        let mut events = Events::with_capacity(64);
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                DATAWRITER => {
                    while let Ok(dwc) = datawriter.try_recv() {
                        match dwc {
                            DataWriterStatusChanged::PublicationMatched(state) => {
                                match state.current_count_change {
                                    1 => {
                                        tui.log(format!("PublicationMatched, {}", state.guid));
                                    }
                                    -1 => {
                                        tui.log(format!("PublicationUnmatched, {}", state.guid));
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            DataWriterStatusChanged::OfferedIncompatibleQos(e) => {
                                tui.log(format!("OfferedIncompatibleQos:\n{}", e));
                            }
                            DataWriterStatusChanged::LivelinessLost(_) => {
                                tui.log("LivelinessLost");
                            }
                            DataWriterStatusChanged::OfferedDeadlineMissed => {
                                tui.log("Deadline missed");
                            }
                        }
                    }
                }
                WRITER_TIMER => {
                    for vp in &mut vehicles {
                        vp.update(&mut rng);
                        datawriter.write(&vp);
                        tui.update_vehicle(vp.vehicle_id.clone(), vp.latitude, vp.longitude);
                    }
                    w_timer.set_timeout(UPDATE_PERIOD, ());
                }
                GEN_TIMER => {
                    let v = VehiclePosition::new(&mut rng);
                    vehicles.push(v);
                    gen_timer.set_timeout(GEN_PERIOD, ());
                }
                DEL_TIMER => {
                    for i in (0..vehicles.len()).rev() {
                        if rng.gen_bool(PROB) {
                            let vp = vehicles.swap_remove(i);
                            tui.remove_vehicle(&vp.vehicle_id);
                            tui.log(format!("remove: {:?}", vp.vehicle_id));
                            datawriter.dispose(vp);
                        }
                    }
                    del_timer.set_timeout(DEL_PERIOD, ());
                }
                STOP => {
                    for vp in &vehicles {
                        datawriter.dispose(vp.clone());
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    break 'pub_loop;
                }
                _ => (), // unreachable
            }
        }
    }
}
