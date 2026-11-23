use handl_plugin_codegen_backend_webidl::WebIdlBackend;
use std::path::Path;

#[test]
fn parses_console_webidl() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/console.webidl");
    let backend = WebIdlBackend::new(None);
    let module = backend.parse_header(&path).expect("parse fixture");

    assert!(module.diagnostics.is_empty(), "{:?}", module.diagnostics);

    assert_eq!(module.client_package, "js");
    assert_eq!(module.backend_id, "webidl");
    assert_eq!(module.target, "browser");

    let event = module
        .records
        .iter()
        .find(|r| r.name == "Event")
        .expect("Event record");
    assert_eq!(event.fields.len(), 2);
    assert_eq!(
        event.fields.iter().map(|f| &f.name).collect::<Vec<_>>(),
        vec!["type", "bubbles"]
    );
    assert_eq!(event.fields[0].descriptor, "js_string");
    assert!(event.fields[0].is_const);
    assert_eq!(event.fields[1].descriptor, "bool");

    // In-source docs: the interface comment becomes the record doc and
    // the member comment lands on its field.
    assert!(event.doc.as_deref().unwrap().contains("basis for the DOM"));
    assert!(
        event.fields[1]
            .doc
            .as_deref()
            .unwrap()
            .contains("bubbles through")
    );

    let event_init = module
        .records
        .iter()
        .find(|r| r.name == "EventInit")
        .expect("EventInit record");
    assert_eq!(event_init.fields.len(), 2);
    assert_eq!(event_init.fields[0].descriptor, "bool");
    assert_eq!(event_init.fields[1].descriptor, "bool");

    // Partial interface merges in the isTrusted operation.
    let stop = module
        .functions
        .iter()
        .find(|f| f.name == "event_stop_propagation")
        .expect("stopPropagation");
    assert_eq!(stop.port_name, "EventStopPropagation");
    assert_eq!(stop.return_port, "()");
    assert_eq!(stop.ret_descriptor, "void");

    let is_trusted = module
        .functions
        .iter()
        .find(|f| f.name == "event_is_trusted")
        .expect("isTrusted");
    assert_eq!(is_trusted.port_name, "EventIsTrusted");
    assert_eq!(is_trusted.return_port, "Bool");
    assert_eq!(is_trusted.ret_descriptor, "bool");

    let console_log = module
        .functions
        .iter()
        .find(|f| f.name == "console_log")
        .expect("console.log");
    assert_eq!(console_log.port_name, "ConsoleLog");
    assert_eq!(console_log.arg_descriptors, &["js_string"]);
    assert_eq!(console_log.ret_descriptor, "void");

    let console_warn = module
        .functions
        .iter()
        .find(|f| f.name == "console_warn")
        .expect("console.warn");
    assert!(console_warn.variadic);
    assert_eq!(console_warn.arg_descriptors, &["js_object"]);

    let callback = module
        .functions
        .iter()
        .find(|f| f.name == "frame_request_callback")
        .expect("FrameRequestCallback");
    assert_eq!(callback.port_name, "FrameRequestCallback");
    assert_eq!(callback.ret_descriptor, "void");

    let mixed = module
        .typedefs
        .iter()
        .find(|t| t.name == "MixedBytes")
        .expect("MixedBytes");
    assert_eq!(mixed.descriptor, "js_object");

    let open = module
        .enum_constants
        .iter()
        .find(|c| c.name == "ShadowRootMode_Open")
        .expect("Open");
    assert_eq!(open.value, 0);
    let closed = module
        .enum_constants
        .iter()
        .find(|c| c.name == "ShadowRootMode_Closed")
        .expect("Closed");
    assert_eq!(closed.value, 1);
}

/// `static` ops, `stringifier attribute`, bare `stringifier;`, and
/// `iterable<K,V>` members must parse without diagnostics — statics map to
/// functions, the stringifier attribute to a readonly field, and the
/// container/stringifier markers drop cleanly.
#[test]
fn parses_extended_member_forms() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/members.webidl");
    let backend = WebIdlBackend::new(None);
    let module = backend.parse_header(&path).expect("parse fixture");

    assert!(module.diagnostics.is_empty(), "{:?}", module.diagnostics);

    let widget = module
        .records
        .iter()
        .find(|r| r.name == "Widget")
        .expect("Widget record");
    assert_eq!(
        widget.fields.iter().map(|f| &f.name).collect::<Vec<_>>(),
        vec!["href", "origin"]
    );

    for port in ["WidgetFind", "WidgetCanFind"] {
        assert!(
            module.functions.iter().any(|f| f.port_name == port),
            "missing {port}"
        );
    }
    let find = module
        .functions
        .iter()
        .find(|f| f.port_name == "WidgetFind")
        .unwrap();
    assert_eq!(find.arg_descriptors, &["js_string"]);
}

/// `A includes B;` grafts the mixin's members onto the target interface:
/// `Request`/`Response` gain `Body`'s ops as their own ports while `Body`
/// keeps its own. Missing targets/mixins surface as diagnostics.
#[test]
fn includes_grafts_mixin_members() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/includes.webidl");
    let backend = WebIdlBackend::new(None);
    let module = backend.parse_header(&path).expect("parse fixture");

    for port in [
        "BodyText",
        "RequestText",
        "RequestBlob",
        "ResponseText",
        "ResponseBlob",
    ] {
        assert!(
            module.functions.iter().any(|f| f.port_name == port),
            "missing {port}"
        );
    }
    let req = module
        .records
        .iter()
        .find(|r| r.name == "Request")
        .expect("Request record");
    assert!(
        req.fields.iter().any(|f| f.name == "bodyUsed"),
        "grafted attribute missing"
    );
    assert_eq!(
        module.diagnostics,
        vec![
            "includes: Ghost is not an interface in this spec",
            "includes: mixin MissingMixin not found (on Request)",
        ]
    );
}

/// A supplementary spec (`add_include_spec`) supplies mixin definitions
/// the primary spec references but does not declare — the graft resolves
/// and the diagnostic clears. Supplementary interfaces are not emitted.
#[test]
fn includes_resolve_across_specs() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/includes.webidl");
    let extra = manifest.join("tests/fixtures/includes-extra.webidl");
    let mut backend = WebIdlBackend::new(None);
    backend.add_include_spec(&extra);
    let module = backend.parse_header(&path).expect("parse fixture");

    let req = module
        .records
        .iter()
        .find(|r| r.name == "Request")
        .expect("Request record");
    assert!(
        req.fields.iter().any(|f| f.name == "extra"),
        "cross-spec grafted attribute missing"
    );
    assert!(
        !module.records.iter().any(|r| r.name == "SupplementaryOnly"),
        "supplementary spec emitted a record"
    );
    assert_eq!(
        module.diagnostics,
        vec!["includes: Ghost is not an interface in this spec"]
    );
}

/// An `includes` whose *target* is foreign (storage.idl's
/// `Navigator includes NavigatorStorage`, where `Navigator` is html.idl's)
/// grafts onto the supplementary interface and emits the augmented record
/// in the primary package.
#[test]
fn includes_graft_onto_foreign_interface() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/includes-foreign.webidl");
    let extra = manifest.join("tests/fixtures/includes-extra.webidl");
    let mut backend = WebIdlBackend::new(None);
    backend.add_include_spec(&extra);
    let module = backend.parse_header(&path).expect("parse fixture");

    let foreign = module
        .records
        .iter()
        .find(|r| r.name == "Foreign")
        .expect("Foreign record emitted in primary package");
    for field in ["ownMember", "bodyUsed"] {
        assert!(
            foreign.fields.iter().any(|f| f.name == field),
            "grafted/own field {field} missing"
        );
    }
    assert!(
        module.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        module.diagnostics
    );
}

/// Worker-class bindings: `constructor` returns the instance
/// (`Handle`/`js_object`), instance ops gain a `recv:`-marked `self`
/// arg, `static` ops and namespace members stay receiver-less, and
/// `on<event>` attributes become `handler:<event>`-marked listener
/// registrations. Plain data attributes stay record fields only.
#[test]
fn worker_members_carry_js_dispatch_markers() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/worker.webidl");
    let backend = WebIdlBackend::new(None);
    let module = backend.parse_header(&path).expect("parse fixture");

    assert!(module.diagnostics.is_empty(), "{:?}", module.diagnostics);

    let find = |name: &str| {
        module
            .functions
            .iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("missing function {name}"))
    };

    // Constructor: returns the constructed interface, no receiver.
    let ctor = find("worker_constructor");
    assert_eq!(ctor.c_decl, "Worker.constructor");
    assert_eq!(ctor.return_port, "Handle");
    assert_eq!(ctor.ret_descriptor, "js_object");
    assert_eq!(ctor.port_args.len(), 2);
    assert!(
        !ctor.arg_descriptors.iter().any(|d| d.starts_with("recv:")),
        "constructor must not carry a receiver: {:?}",
        ctor.arg_descriptors
    );

    // Instance op: leading `self` receiver with the `recv:` marker.
    let terminate = find("worker_terminate");
    assert_eq!(terminate.c_decl, "Worker.terminate");
    assert_eq!(terminate.port_args[0].name, "self");
    assert_eq!(terminate.port_args[0].port, "Handle");
    assert_eq!(terminate.arg_descriptors, &["recv:js_object"]);

    // Overloaded op: the later overload wins the shared name; the
    // receiver is still peeled first.
    let post = find("worker_post_message");
    assert_eq!(post.port_args.len(), 3);
    assert_eq!(post.arg_descriptors[0], "recv:js_object");
    assert_eq!(post.arg_descriptors.len(), 3);

    // Static op: no receiver — the emitter dispatches `call` on the
    // `Worker` global.
    let register = find("worker_register");
    assert!(
        !register
            .arg_descriptors
            .iter()
            .any(|d| d.starts_with("recv:")),
        "static op must not carry a receiver: {:?}",
        register.arg_descriptors
    );
    assert_eq!(register.arg_descriptors, &["js_string"]);

    // `on<event>` attributes (own + mixin-grafted): `handler:<event>`
    // marks the arrow arg for `js.on`.
    for (fname, event) in [
        ("worker_onmessage", "message"),
        ("worker_onmessageerror", "messageerror"),
        ("worker_onerror", "error"),
        ("message_event_target_onmessage", "message"),
        ("abstract_worker_onerror", "error"),
    ] {
        let f = find(fname);
        assert_eq!(f.port_args.len(), 2, "{fname}");
        assert_eq!(f.port_args[0].name, "self");
        assert_eq!(f.port_args[1].name, "handler");
        assert_eq!(f.arg_descriptors[0], "recv:js_object");
        assert_eq!(f.arg_descriptors[1], format!("handler:{event}"), "{fname}");
        assert_eq!(f.return_port, "Handle");
        assert_eq!(f.ret_descriptor, "js_object");
    }

    // The data attribute stays a record field and gets no accessor.
    let worker = module
        .records
        .iter()
        .find(|r| r.name == "Worker")
        .expect("Worker record");
    assert!(worker.fields.iter().any(|f| f.name == "label"));
    assert!(worker.fields.iter().any(|f| f.name == "onmessage"));
    assert!(
        !module.functions.iter().any(|f| f.name == "worker_label"),
        "data attributes must not emit accessor functions"
    );
}

/// `?` follows the whole type production in WebIDL, so nullable
/// generics/arrays (`FrozenArray<T>?`, `sequence<T>?`, `T[]?`) must not
/// strand the `?` — an unconsumed `?` used to leave the attribute name
/// empty and emit a nameless `: Handle` record field (html.idl's
/// `NotRestoredReasons`). Every field must carry its declared name.
#[test]
fn parses_nullable_generic_attribute_types() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/nullable-generic.webidl");
    let backend = WebIdlBackend::new(None);
    let module = backend.parse_header(&path).expect("parse fixture");

    assert!(module.diagnostics.is_empty(), "{:?}", module.diagnostics);

    let reasons = module
        .records
        .iter()
        .find(|r| r.name == "NotRestoredReasons")
        .expect("NotRestoredReasons record");
    assert_eq!(
        reasons.fields.iter().map(|f| &f.name).collect::<Vec<_>>(),
        vec!["src", "id", "reasons", "children", "tags", "aliases"]
    );
    assert!(
        reasons.fields.iter().all(|f| !f.name.is_empty()),
        "nameless field emitted: {:?}",
        reasons.fields
    );
    // Foreign/generic types collapse to `Handle`/`js_object` per
    // `map_type` — the nullable wrapper doesn't change that.
    for f in &reasons.fields[2..] {
        assert_eq!(f.port, "Handle", "{}", f.name);
        assert_eq!(f.descriptor, "js_object", "{}", f.name);
    }
}

/// Enum constants are gathered through a `HashMap` merge; without a sort the
/// emitted order varies per run (fresh `RandomState`). Parsing twice in one
/// process must yield identical output.
#[test]
fn enum_constants_are_deterministic() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("tests/fixtures/enums.webidl");
    let backend = WebIdlBackend::new(None);

    let names = |m: &handl_plugin_codegen::SdkgenModule| {
        m.enum_constants
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>()
    };
    let first = backend.parse_header(&path).expect("parse fixture");
    for _ in 0..16 {
        let again = backend.parse_header(&path).expect("parse fixture");
        assert_eq!(names(&first), names(&again));
    }
    assert_eq!(
        names(&first),
        vec![
            "AlphaMode_X",
            "AlphaMode_Y",
            "MidMode_M",
            "MidMode_N",
            "ZetaMode_A",
            "ZetaMode_B"
        ]
    );
}
