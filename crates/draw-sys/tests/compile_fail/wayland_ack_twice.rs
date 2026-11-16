// expect: use of moved value
// Acking the first configure consumes the window: it cannot be acked again.
use draw_sys::wayland::{Connection, Created, Window};

pub fn twice(conn: &Connection) {
    let window: Window<Created> = conn.create_window("t", "a").unwrap();
    let _configured = window.ack_configure(1);
    let _again = window.ack_configure(1);
}
