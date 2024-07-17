use std::str::FromStr;

use crate::kinode::process::poke::{Request as PokeRequest, Response as ChatResponse, SendRequest};
use kinode_process_lib::{
    await_next_message_body, call_init, println, Address, Message, Request,
};

wit_bindgen::generate!({
    path: "target/wit",
    world: "poke-tantum-ergo-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize],
});

call_init!(init);
fn init(our: Address) {
    let Ok(body) = await_next_message_body() else {
        println!("failed to get args!");
        return;
    };

    let args = String::from_utf8(body).unwrap_or_default();

    let Ok(target) = Address::from_str(&args) else {
        println!("usage:\nsend:poke:tantum-ergo.os target");
        return;
    };

    let Ok(Ok(Message::Response { body, .. })) =
        Request::to((our.node(), ("poke", "poke", "tantum-ergo.os")))
            .body(
                serde_json::to_vec(&PokeRequest::Send(SendRequest {
                    target: target.to_string(),
                }))
                .unwrap(),
            )
            .send_and_await_response(5)
    else {
        println!("did not receive expected Response from poke:poke:tantum-ergo.os");
        return;
    };

    let Ok(ChatResponse::Send) = serde_json::from_slice(&body) else {
        println!("did not receive expected Ack from poke:poke:tantum-ergo.os");
        return;
    };
}
