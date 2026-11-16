// The legal order: create, configure, open the GL context, commit.
use draw_sys::wayland::{Connection, WaylandError};

pub fn sequence(conn: &Connection) -> Result<(), WaylandError> {
    let window = conn.create_window("t", "a")?;
    let window = window.wait_configure()?;
    let window = window.open_gl()?;
    window.commit();
    window.swap().map_err(WaylandError::Context)?;
    Ok(())
}
