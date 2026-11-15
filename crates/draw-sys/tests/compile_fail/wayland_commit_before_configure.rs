// expect: no method named `commit` found
// A surface that has not acked its first configure cannot commit a buffer.
use draw_sys::wayland::{Connection, Created, Window};

pub fn early(conn: &Connection) {
    let window: Window<Created> = conn.create_window("t", "a").unwrap();
    window.commit();
}
