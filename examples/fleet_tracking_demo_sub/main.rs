use ctrlc;
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Deserializers, Root},
    encode::pattern::PatternEncoder,
    init_config, init_file,
};
use mio_extras::channel as mio_channel;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use rand::SeedableRng;
use speedy::Writable;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use umber_dds::dds::{qos::*, sample::InstanceHandle, DataReaderStatusChanged, DomainParticipant};
use umber_dds::{DdsData, DdsDeserialize, KeyHash};

#[derive(Clone, Debug, DdsData, DdsDeserialize)]
struct VehiclePosition {
    #[key]
    vehicle_id: String,
    latitude: f64,
    longitude: f64,
}

fn main() {
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
    const DATAREADER: Token = Token(0);
    const STOP: Token = Token(4);

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
        println!("SIGINT received");
        stop_sender
            .send(())
            .expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    let subscriber = participant.create_subscriber(SubscriberQos::Default);
    let dr_qos = DataReaderQosBuilder::new()
        .reliability(policy::Reliability::default_besteffort())
        .deadline(policy::Deadline::new(Duration::from_millis(2000)))
        .build();
    let datareader = subscriber
        .create_datareader::<VehiclePosition>(DataReaderQos::Policies(Box::new(dr_qos)), topic);
    poll.register(&datareader, DATAREADER, Ready::readable(), PollOpt::edge())
        .unwrap();
    poll.register(&stop_receiver, STOP, Ready::readable(), PollOpt::edge())
        .unwrap();

    let mut ih2vid: HashMap<InstanceHandle, String> = HashMap::new();

    'sub_loop: loop {
        let mut events = Events::with_capacity(64);
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                DATAREADER => {
                    while let Ok(drc) = datareader.try_recv() {
                        match drc {
                            DataReaderStatusChanged::DataAvailable => {
                                let received_vehicles = datareader.take();
                                for vehicle in received_vehicles {
                                    let si = vehicle.sample_info();
                                    if si.valid_data {
                                        let data = vehicle.data().unwrap();
                                        if !ih2vid.contains_key(&si.instance_handle) {
                                            ih2vid.insert(
                                                si.instance_handle,
                                                data.vehicle_id.clone(),
                                            );
                                        }
                                        println!("received: {:?}, {}", data, si.instance_handle);
                                    } else {
                                        let ih = si.instance_handle;
                                        if let Some(vid) = ih2vid.get(&ih) {
                                            println!("vehicle: {} is deleted", vid);
                                        } else {
                                            println!("received unknown instance: {}", ih);
                                        }
                                    }
                                }
                            }
                            DataReaderStatusChanged::SubscriptionMatched(state) => {
                                match state.current_count_change {
                                    1 => {
                                        println!("SubscriptionMatched, {}", state.guid);
                                    }
                                    -1 => {
                                        println!("SubscriptionUnmatched, {}", state.guid);
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            DataReaderStatusChanged::RequestedIncompatibleQos(e) => {
                                println!("RequestedIncompatibleQos:\n{}", e);
                            }
                            DataReaderStatusChanged::LivelinessChanged(l) => {
                                println!(
                                    "LivelinessChanged: alive:{}, not_alive: {}, guid: {}",
                                    l.alive_count_change, l.not_alive_count_change, l.guid
                                );
                            }
                            DataReaderStatusChanged::RequestedDeadlineMissed(w, ih) => {
                                println!("Deadline missed: {}, {}", w, ih);
                            }
                            _ => (), // TODO
                        }
                    }
                }
                STOP => {
                    break 'sub_loop;
                }
                _ => (), // unreachable
            }
        }
    }
}
