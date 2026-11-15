// expect: no method named `attach` found
// Attaching a buffer is possible only once the first configure is acked.
use draw_sys::wayland::{Connection, Created, Window};

pub fn early(conn: &Connection) {
    let window: Window<Created> = conn.create_window("t", "a").unwrap();
    unsafe { window.attach(std::ptr::null_mut(), 0, 0) };
}
