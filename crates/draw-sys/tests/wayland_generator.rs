//! The generator of the Wayland protocol code (T168, ADR-T168-1): a minimal
//! XML reader for the protocol subset and an emitter of `wl_interface`
//! tables, listener structs and request wrappers. The test
//! `generated_code_matches_the_checked_in_files` fails when
//! `src/wayland/generated/` differs from the output; regenerate with
//!
//! ```text
//! cargo test -p draw-sys --test wayland_generator -- --ignored write_generated_protocol
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// One argument of a request or event.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Arg {
    name: String,
    kind: String,
    interface: Option<String>,
    nullable: bool,
}

/// One request or event.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    name: String,
    since: u32,
    destructor: bool,
    args: Vec<Arg>,
}

/// One interface.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Interface {
    name: String,
    version: u32,
    requests: Vec<Message>,
    events: Vec<Message>,
}

/// The `key="value"` pairs of a tag body.
fn attributes(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(eq) = rest.find('=') {
        let key = rest[..eq]
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_string();
        let after = &rest[eq + 1..];
        let Some(open) = after.find('"') else { break };
        let Some(len) = after[open + 1..].find('"') else {
            break;
        };
        let value = &after[open + 1..open + 1 + len];
        out.push((key, value.replace("&quot;", "\"").replace("&amp;", "&")));
        rest = &after[open + 1 + len + 1..];
    }
    out
}

fn attr<'a>(attrs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// The tags of an XML text, comments and declarations skipped: the tag
/// text between `<` and `>` (quotes respected).
fn tags(xml: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find('<') {
        rest = &rest[start..];
        if let Some(tail) = rest.strip_prefix("<!--") {
            rest = tail.find("-->").map_or("", |end| &tail[end + 3..]);
            continue;
        }
        let mut quoted = false;
        let mut end = None;
        for (i, c) in rest.char_indices().skip(1) {
            match c {
                '"' => quoted = !quoted,
                '>' if !quoted => {
                    end = Some(i);
                    break;
                }
                _ => {}
            }
        }
        let Some(end) = end else { break };
        let tag = &rest[1..end];
        if !tag.starts_with('?') && !tag.starts_with('!') {
            out.push(tag);
        }
        rest = &rest[end + 1..];
    }
    out
}

/// The interfaces of a protocol XML text.
fn parse(xml: &str) -> Vec<Interface> {
    let mut interfaces = Vec::new();
    let mut current: Option<Interface> = None;
    let mut message: Option<(bool, Message)> = None;
    for tag in tags(xml) {
        let closing = tag.starts_with('/');
        let self_closing = tag.ends_with('/');
        let body = tag.trim_start_matches('/').trim_end_matches('/');
        let name = body.split_whitespace().next().unwrap_or("");
        let attrs = attributes(body);
        match (name, closing) {
            ("interface", false) => {
                current = Some(Interface {
                    name: attr(&attrs, "name").unwrap_or("").to_string(),
                    version: attr(&attrs, "version").map_or(1, |v| v.parse().unwrap()),
                    requests: Vec::new(),
                    events: Vec::new(),
                });
                if self_closing {
                    interfaces.extend(current.take());
                }
            }
            ("interface", true) => interfaces.extend(current.take()),
            ("request" | "event", false) => {
                let m = Message {
                    name: attr(&attrs, "name").unwrap_or("").to_string(),
                    since: attr(&attrs, "since").map_or(1, |v| v.parse().unwrap()),
                    destructor: attr(&attrs, "type") == Some("destructor"),
                    args: Vec::new(),
                };
                message = Some((name == "request", m));
                if self_closing {
                    push_message(&mut current, &mut message);
                }
            }
            ("request" | "event", true) => push_message(&mut current, &mut message),
            ("arg", false) => {
                if let Some((_, m)) = message.as_mut() {
                    m.args.push(Arg {
                        name: attr(&attrs, "name").unwrap_or("").to_string(),
                        kind: attr(&attrs, "type").unwrap_or("").to_string(),
                        interface: attr(&attrs, "interface").map(str::to_string),
                        nullable: attr(&attrs, "allow-null") == Some("true"),
                    });
                }
            }
            _ => {}
        }
    }
    interfaces
}

fn push_message(current: &mut Option<Interface>, message: &mut Option<(bool, Message)>) {
    if let (Some(iface), Some((is_request, m))) = (current.as_mut(), message.take()) {
        if is_request {
            iface.requests.push(m);
        } else {
            iface.events.push(m);
        }
    }
}

/// The wire signature of a message (`wl_message.signature`).
fn signature(m: &Message) -> String {
    let mut s = String::new();
    if m.since > 1 {
        s.push_str(&m.since.to_string());
    }
    for a in &m.args {
        if a.nullable {
            s.push('?');
        }
        let letter = match a.kind.as_str() {
            "int" => 'i',
            "uint" => 'u',
            "fixed" => 'f',
            "string" => 's',
            "object" => 'o',
            "new_id" => {
                if a.interface.is_none() {
                    s.push_str("su");
                }
                'n'
            }
            "array" => 'a',
            "fd" => 'h',
            other => panic!("unknown argument type {other}"),
        };
        s.push(letter);
    }
    s
}

/// The `wl_message.types` entries, one per signature letter: the interface
/// path or `None` for null.
fn type_entries<'a>(
    m: &'a Message,
    known: &'a HashMap<String, String>,
) -> Vec<Option<(&'a str, &'a str)>> {
    let mut out = Vec::new();
    for a in &m.args {
        if a.kind == "new_id" && a.interface.is_none() {
            out.extend([None, None]);
        }
        let target = a
            .interface
            .as_deref()
            .and_then(|i| known.get(i).map(|file| (file.as_str(), i)));
        out.push(target);
    }
    out
}

const KEYWORDS: [&str; 40] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];

fn ident(name: &str) -> String {
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

fn rust_type(kind: &str) -> &'static str {
    match kind {
        "int" | "fixed" | "fd" => "i32",
        "uint" => "u32",
        "string" => "*const c_char",
        "array" => "*mut c_void",
        _ => "*mut Proxy",
    }
}

const MARKER: &str = "wayland_window_configures_with_a_size";

/// The Rust module of one interface.
fn emit_interface(out: &mut String, iface: &Interface, known: &HashMap<String, String>) {
    let name = &iface.name;
    let path_of =
        |(file, i): (&str, &str)| format!("crate::wayland::generated::{file}::{i}::INTERFACE");
    let _ = writeln!(out, "\n/// The `{name}` interface.\npub mod {name} {{");
    out.push_str("    use crate::wayland::client::{Client, Proxy};\n");
    out.push_str("    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};\n");
    out.push_str("    use std::ffi::{c_char, c_void};\n");
    out.push_str("    use std::ptr::{null, null_mut};\n\n");
    let _ = writeln!(
        out,
        "    /// Highest version this code knows.\n    pub const VERSION: u32 = {};\n",
        iface.version
    );

    for (kind, list) in [("REQUEST", &iface.requests), ("EVENT", &iface.events)] {
        for (i, m) in list.iter().enumerate() {
            let entries = type_entries(m, known);
            let cells: Vec<String> = entries
                .iter()
                .map(|e| {
                    e.map_or_else(
                        || "null()".to_string(),
                        |t| format!("&raw const {}", path_of(t)),
                    )
                })
                .collect();
            let _ = writeln!(
                out,
                "    static {kind}_TYPES_{i}: TypeTable<{}> = TypeTable([{}]);",
                cells.len(),
                cells.join(", ")
            );
        }
        let rows: Vec<String> = list
            .iter()
            .enumerate()
            .map(|(i, m)| {
                format!(
                    "Message {{ name: c\"{}\".as_ptr(), signature: c\"{}\".as_ptr(), types: {kind}_TYPES_{i}.0.as_ptr() }}",
                    m.name,
                    signature(m)
                )
            })
            .collect();
        let _ = writeln!(
            out,
            "    static {kind}S: MessageTable<{}> = MessageTable([{}]);\n",
            list.len(),
            rows.join(", ")
        );
    }
    let _ = writeln!(
        out,
        "    /// The `wl_interface` table.\n    pub static INTERFACE: Interface = Interface {{\n        name: c\"{name}\".as_ptr(),\n        version: {},\n        method_count: {},\n        methods: REQUESTS.0.as_ptr(),\n        event_count: {},\n        events: EVENTS.0.as_ptr(),\n    }};\n",
        iface.version,
        iface.requests.len(),
        iface.events.len()
    );

    emit_listener(out, iface);
    for (opcode, m) in iface.requests.iter().enumerate() {
        emit_request(out, iface, m, opcode, known);
    }
    out.push_str("}\n");
}

fn event_params(m: &Message) -> String {
    let mut p = "*mut c_void, *mut Proxy".to_string();
    for a in &m.args {
        p.push_str(", ");
        p.push_str(rust_type(&a.kind));
    }
    p
}

fn emit_listener(out: &mut String, iface: &Interface) {
    if iface.events.is_empty() {
        return;
    }
    out.push_str("    /// The event handlers, in protocol order.\n    #[repr(C)]\n    pub struct Listener {\n");
    for m in &iface.events {
        let _ = writeln!(
            out,
            "        /// Event `{0}`.\n        pub {1}: unsafe extern \"C\" fn({2}), // TEST: {MARKER}",
            m.name,
            ident(&m.name),
            event_params(m)
        );
    }
    out.push_str("    }\n\n");
    for m in &iface.events {
        let blanks: Vec<String> = event_params(m)
            .split(", ")
            .map(|t| format!("_: {t}"))
            .collect();
        let _ = writeln!(
            out,
            "    extern \"C\" fn ignore_{}({}) {{}}",
            m.name,
            blanks.join(", ")
        );
    }
    out.push_str("\n    /// Handlers that do nothing, to update with `..IGNORE`.\n    pub const IGNORE: Listener = Listener {\n");
    for m in &iface.events {
        let _ = writeln!(out, "        {0}: ignore_{1},", ident(&m.name), m.name);
    }
    out.push_str("    };\n\n");
    let _ = writeln!(
        out,
        "    // TEST: {MARKER}\n    /// Installs `listener` on `proxy`; `0` on success.\n    ///\n    /// # Safety\n    /// `proxy` is a live `{0}` proxy of `client`, and `data` stays valid\n    /// (whatever the handlers do with it) until the proxy is destroyed.\n    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {{\n        // TEST: {MARKER}\n        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a\n        // `repr(C)` table of function pointers in event order.\n        unsafe {{ (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }}\n    }}\n",
        iface.name
    );
}

fn emit_request(
    out: &mut String,
    iface: &Interface,
    m: &Message,
    opcode: usize,
    known: &HashMap<String, String>,
) {
    let unknown_new = m.args.iter().find(|a| {
        a.kind == "new_id" && a.interface.as_ref().is_some_and(|i| !known.contains_key(i))
    });
    if let Some(a) = unknown_new {
        let _ = writeln!(
            out,
            "    // `{}` is not wrapped: it creates `{}`, which is not generated.\n",
            m.name,
            a.interface.as_deref().unwrap_or("")
        );
        return;
    }
    let mut params = String::from("client: &Client, proxy: *mut Proxy");
    let mut call = String::new();
    let mut returns = false;
    let mut new_interface = "null()".to_string();
    let mut version = "(client.get_version)(proxy)".to_string();
    for a in &m.args {
        let n = ident(&a.name);
        if a.kind == "new_id" {
            returns = true;
            if let Some(i) = &a.interface {
                new_interface = format!(
                    "&raw const {}",
                    format_args!("crate::wayland::generated::{}::{i}::INTERFACE", known[i])
                );
                call.push_str(", null_mut::<Proxy>()");
            } else {
                params.push_str(", interface: *const Interface, version: u32");
                new_interface = "interface".to_string();
                version = "version".to_string();
                call.push_str(", (*interface).name, version, null_mut::<Proxy>()");
            }
        } else {
            let _ = write!(params, ", {n}: {}", rust_type(&a.kind));
            let _ = write!(call, ", {n}");
        }
    }
    let flags = u32::from(m.destructor);
    let ret = if returns { " -> *mut Proxy" } else { "" };
    let body = format!(
        "(client.marshal_flags)(proxy, {opcode}, {new_interface}, {version}, {flags}{call})"
    );
    let _ = writeln!(
        out,
        "    // TEST: {MARKER}\n    /// Request `{0}` of `{1}`.\n    ///\n    /// # Safety\n    /// `proxy` is a live `{1}` proxy of `client`; object arguments are live\n    /// proxies of the same connection (null where the protocol allows it).\n    pub unsafe fn {2}({params}){ret} {{\n        // TEST: {MARKER}\n        // SAFETY: the caller vouches for the proxies; the opcode, signature and\n        // argument types follow the protocol XML this file was generated from.\n        unsafe {{ {body} }}{3}\n    }}\n",
        m.name,
        iface.name,
        ident(&m.name),
        if returns { "" } else { ";" },
    );
}

/// A protocol file and the interfaces generated from it.
const PROTOCOLS: [(&str, &str, &[&str]); 2] = [
    (
        "wayland.xml",
        "wayland",
        &[
            "wl_display",
            "wl_registry",
            "wl_compositor",
            "wl_surface",
            "wl_callback",
            "wl_output",
            "wl_seat",
            "wl_pointer",
            "wl_keyboard",
        ],
    ),
    (
        "xdg-shell.xml",
        "xdg_shell",
        &["xdg_wm_base", "xdg_surface", "xdg_toplevel"],
    ),
];

fn emit_file(xml_name: &str, interfaces: &[&Interface], known: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! Generated from `protocols/{xml_name}` by the generator of\n//! `tests/wayland_generator.rs` (ADR-T168-1). Do not edit: regenerate with\n//! `cargo test -p draw-sys --test wayland_generator -- --ignored write_generated_protocol`.\n\n#![allow(missing_docs, unsafe_code, unused_imports, clippy::all, clippy::pedantic)]"
    );
    for iface in interfaces {
        emit_interface(&mut out, iface, known);
    }
    out
}

/// Every generated file: path relative to `src/wayland/generated/`, text.
fn generate(protocols: &Path) -> Vec<(String, String)> {
    let parsed: Vec<(&str, &str, Vec<Interface>)> = PROTOCOLS
        .iter()
        .map(|(xml, module, names)| {
            let text = fs::read_to_string(protocols.join(xml)).unwrap();
            let all = parse(&text);
            let picked = names
                .iter()
                .map(|n| all.iter().find(|i| i.name == *n).unwrap().clone())
                .collect();
            (*xml, *module, picked)
        })
        .collect();
    let mut known = HashMap::new();
    for (_, module, list) in &parsed {
        for i in list {
            known.insert(i.name.clone(), (*module).to_string());
        }
    }
    let mut files = vec![(
        "mod.rs".to_string(),
        "//! Generated protocol code (ADR-T168-1), one module per protocol file.\n\npub mod wayland;\npub mod xdg_shell;\n".to_string(),
    )];
    for (xml, module, list) in &parsed {
        let refs: Vec<&Interface> = list.iter().collect();
        files.push((format!("{module}.rs"), emit_file(xml, &refs, &known)));
    }
    files
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn generated_code_matches_the_checked_in_files() {
    let dir = crate_dir().join("src/wayland/generated");

    for (name, text) in generate(&crate_dir().join("protocols")) {
        let on_disk = fs::read_to_string(dir.join(&name)).unwrap_or_default();
        assert!(
            on_disk == text,
            "{name} differs from the generator's output; run the ignored test write_generated_protocol"
        );
    }
}

#[test]
#[ignore = "writes src/wayland/generated/"]
fn write_generated_protocol() {
    let dir = crate_dir().join("src/wayland/generated");
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in generate(&crate_dir().join("protocols")) {
        fs::write(dir.join(name), text).unwrap();
    }
}

const SAMPLE: &str = r#"<?xml version="1.0"?>
<protocol name="t">
  <!-- a comment with <tags> inside -->
  <interface name="a" version="3">
    <description summary="x > y">text</description>
    <request name="make">
      <arg name="id" type="new_id" interface="b"/>
      <arg name="x" type="int"/>
    </request>
    <request name="bind">
      <arg name="name" type="uint"/>
      <arg name="id" type="new_id"/>
    </request>
    <request name="gone" type="destructor"/>
    <event name="ping" since="2">
      <arg name="serial" type="uint"/>
      <arg name="who" type="object" interface="b" allow-null="true"/>
    </event>
  </interface>
  <interface name="b" version="1"/>
</protocol>"#;

#[test]
fn parser_reads_interfaces_requests_and_events() {
    let list = parse(SAMPLE);

    assert_eq!(list.len(), 2);
    assert_eq!(list[0].name, "a");
    assert_eq!(list[0].version, 3);
    assert_eq!(list[0].requests.len(), 3);
    assert!(list[0].requests[2].destructor);
    assert_eq!(list[0].events[0].since, 2);
    assert_eq!(list[0].events[0].args[1].interface.as_deref(), Some("b"));
    assert!(list[0].events[0].args[1].nullable);
    assert_eq!(list[1].name, "b");
}

#[test]
fn generator_emits_signature_for_new_id_and_object_args() {
    let list = parse(SAMPLE);

    assert_eq!(signature(&list[0].requests[0]), "ni");
    assert_eq!(signature(&list[0].requests[1]), "usun");
    assert_eq!(signature(&list[0].events[0]), "2u?o");
}

#[test]
fn generator_skips_wrappers_for_unknown_interfaces() {
    let list = parse(SAMPLE);
    let mut known = HashMap::new();
    known.insert("a".to_string(), "t".to_string());

    let mut out = String::new();
    emit_interface(&mut out, &list[0], &known);

    assert!(out.contains("`make` is not wrapped"), "{out}");
    assert!(!out.contains("pub unsafe fn make"));
    assert!(out.contains("pub unsafe fn bind"));
    assert!(out.contains("interface: *const Interface, version: u32"));
    assert!(out.contains("pub struct Listener"));
}

#[test]
fn generator_escapes_keywords_and_marks_destructors() {
    let list = parse(
        r#"<protocol name="t"><interface name="m" version="1">
        <request name="move"><arg name="type" type="uint"/></request>
        <request name="destroy" type="destructor"/></interface></protocol>"#,
    );
    let known = HashMap::new();

    let mut out = String::new();
    emit_interface(&mut out, &list[0], &known);

    assert!(out.contains("pub unsafe fn r#move(client: &Client, proxy: *mut Proxy, r#type: u32)"));
    assert!(
        out.contains("null(), (client.get_version)(proxy), 1)"),
        "{out}"
    );
}
