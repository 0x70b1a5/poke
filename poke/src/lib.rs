use crate::kinode::process::poke::{Request as PokeRequest, Response as PokeResponse, SendRequest};
use crate::kinode::process::notify;
use kinode_process_lib::homepage::add_to_homepage;
use kinode_process_lib::{await_message, call_init, println, Address, Message, Request, Response};

wit_bindgen::generate!({
    path: "target/wit",
    world: "poke-tantum-ergo-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

fn handle_message(
    our: &Address,
    message: &Message,
) -> anyhow::Result<()> {
    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let body = message.body();
    let source = message.source();
    match body.try_into()? {
        PokeRequest::Send(SendRequest {
            ref target,
        }) => {
            if target == &our.node {
                if source.node == *target {
                    println!("you poked yourself");
                } else {
                    println!("{} poked you", source.node);
                }
                let Ok(_) = Request::new()
                    .target(Address {
                        node: our.node,
                        process: "notify:notify:tantum-ergo.os".parse()?,
                    })
                    .body(notify::Request::Push(notify::Notification {
                        title: "You got poked!",
                        body: format!("{} poked you", source.node),
                    }))
                    .send_and_await_response(5) 
                else {
                    return Err(anyhow::anyhow!("{} is offline or doesn't have notify", target));
                };
            } else {
                let Ok(Ok(Message::Response { body, .. })) = Request::new()
                    .target(Address {
                        node: target.clone(),
                        process: "poke:poke:tantum-ergo.os".parse()?,
                    }) 
                    .body(body)
                    .send_and_await_response(5)
                else {
                    return Err(anyhow::anyhow!("{} is offline or doesn't have poke", target));
                };
            }
            Response::new()
                .body(PokeResponse::Send)
                .send()
                .unwrap();
        }
    }
    Ok(())
}

const ICON: &str = include_str!("ICON");

call_init!(init);
fn init(our: Address) {
    println!("begin");

    add_to_homepage(
        "Poke",
        Some(ICON),
        None,
        None,
    );

    loop {
        match await_message() {
            Err(send_error) => println!("got SendError: {send_error}"),
            Ok(ref message) => match handle_message(&our, message) {
                Ok(_) => {}
                Err(e) => println!("got error while handling message: {e:?}"),
            }
        }
    }
}
