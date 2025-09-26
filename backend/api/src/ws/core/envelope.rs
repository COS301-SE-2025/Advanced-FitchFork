use crate::ws::core::event::Event;
use util::ws::{WebSocketManager, emit as emit_enveloped};

pub async fn emit<E>(ws: &WebSocketManager, ev: &E)
where
    E: Event,
{
    let topic = ev.topic_path();
    emit_enveloped(ws, &topic, E::NAME, ev).await;
}
