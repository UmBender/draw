//! Bootstrap checks: the module skeleton is complete and the window is configured.

#[test]
#[allow(
    unused_imports,
    reason = "the imports are the test: compilation fails if a module is missing"
)]
fn all_modules_are_reachable() {
    use draw::core::camera as _;
    use draw::core::clipboard as _;
    use draw::core::command as _;
    use draw::core::document as _;
    use draw::core::editor as _;
    use draw::core::geom as _;
    use draw::core::history as _;
    use draw::core::input as _;
    use draw::core::keymap as _;
    use draw::core::palette as _;
    use draw::core::shape as _;
    use draw::core::smoothing as _;
    use draw::core::tools::bucket as _;
    use draw::core::tools::eraser as _;
    use draw::core::tools::navigate as _;
    use draw::core::tools::pen as _;
    use draw::core::tools::select as _;
    use draw::core::tools::shape_tool as _;
    use draw::shell::app as _;
    use draw::shell::input_map as _;
    use draw::shell::render as _;
    use draw::shell::toolbar as _;
}

#[test]
fn window_conf_has_expected_title_and_size() {
    let conf = draw::shell::app::window_conf();

    assert_eq!(conf.window_title, "draw");
    assert_eq!((conf.window_width, conf.window_height), (1280, 800));
    assert!(conf.window_resizable);
    assert!(conf.high_dpi);
}
