// Nullable generic/array types: WebIDL puts `?` *after* the whole type
// production (`FrozenArray<T>?`, `sequence<T>?`, `T[]?`), so the parser
// must accept a trailing `?` following `<...>`/`[]` suffixes. The
// `NotRestoredReasons` shape comes from html.idl — before the fix the
// `?` was left unconsumed, the attribute name parsed as empty, and the
// emitter wrote a field with no name (`: Handle`).
[Exposed=Window]
interface NotRestoredReasonDetails {
  readonly attribute DOMString reason;
};

[Exposed=Window]
interface NotRestoredReasons {
  readonly attribute USVString? src;
  readonly attribute DOMString? id;
  readonly attribute FrozenArray<NotRestoredReasonDetails>? reasons;
  readonly attribute FrozenArray<NotRestoredReasons>? children;
  readonly attribute sequence<USVString>? tags;
  readonly attribute DOMString[]? aliases;
};
