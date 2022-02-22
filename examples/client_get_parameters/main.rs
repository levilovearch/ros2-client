use std::env;

use mio::{Events, Poll, PollOpt, Ready, Token};
use ros2_client::{
    interfaces::{GetParametersRequest, GetParametersService},
    Context, Node, NodeOptions, ServiceMappings,
};
use rustdds::{policy, Duration, QosPolicies, QosPolicyBuilder};

fn main() {
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("There is no args");
        return;
    }

    println!(">>> ros2_service starting...");
    let mut node = create_node();
    let service_qos = create_qos();

    println!(">>> ros2_service node started");

    let mut client = node
        .create_client::<GetParametersService>(
            ServiceMappings::Enhanced,
            // "/ros2_param_node/list_parameters",
            &args[1],
            service_qos.clone(),
        )
        .unwrap();

    println!(">>> ros2_service client created");

    let poll = Poll::new().unwrap();

    poll.register(&client, Token(7), Ready::readable(), PollOpt::edge())
        .unwrap();

    //std::thread::sleep(std::time::Duration::from_secs(4));

    println!(">>> request sending...");
    let request = GetParametersRequest {
        names: args[2..].to_vec(),
    };

    match client.send_request(request) {
        Ok(id) => {
            println!(">>> request sent {:?}", id);
        }
        Err(e) => {
            println!(">>> request sending error {:?}", e);
        }
    }

    'e_loop: loop {
        println!(">>> event loop iter");
        let mut events = Events::with_capacity(100);
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            println!(">>> New event");
            match event.token() {
                Token(7) => {
                    while let Ok(Some((id, response))) = client.receive_response() {
                        println!(">>> Response received -  response: {:?}, id: {:?}", response, id,);
                        break 'e_loop;
                    }
                }
                _ => println!(">>> Unknown poll token {:?}", event.token()),
            }
        }
    }
}

fn create_qos() -> QosPolicies {
    let service_qos: QosPolicies = {
        QosPolicyBuilder::new()
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: Duration::from_millis(100),
            })
            .history(policy::History::KeepLast { depth: 1 })
            .build()
    };
    service_qos
}

fn create_node() -> Node {
    let context = Context::new().unwrap();
    let node = context
        .new_node(
            "rustdds_client",
            "/rustdds",
            NodeOptions::new().enable_rosout(true),
        )
        .unwrap();
    node
}
