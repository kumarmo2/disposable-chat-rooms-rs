use lapin::{self, Connection, ConnectionProperties};

pub(crate) async fn create_connection() -> lapin::Result<Connection> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f"; // this decodes to "amqp://127.0.0.1:5672//"
    Connection::connect(addr, ConnectionProperties::default()).await
}
