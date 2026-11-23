// Trimmed Worker surface from w3c/webref html.idl — enough to exercise
// constructors, instance/static operations, `on<event>` handler
// attributes, data attributes, and `includes` mixin grafting.

callback EventHandlerNonNull = any (Event event);
typedef EventHandlerNonNull? EventHandler;

callback OnErrorEventHandlerNonNull = any ((Event or DOMString) event, optional DOMString source, optional unsigned long lineno, optional unsigned long colno, optional any error);
typedef OnErrorEventHandlerNonNull? OnErrorEventHandler;

interface EventTarget {
  undefined addEventListener(DOMString type, EventListener? callback, optional AddEventListenerOptions options = {});
  undefined removeEventListener(DOMString type, EventListener? callback, optional EventListenerOptions options = {});
  boolean dispatchEvent(Event event);
};

interface mixin MessageEventTarget {
  attribute EventHandler onmessage;
  attribute EventHandler onmessageerror;
};

interface mixin AbstractWorker {
  attribute EventHandler onerror;
};

[Exposed=(Window,Worker)]
interface Worker : EventTarget {
  constructor((TrustedScriptURL or USVString) scriptURL, optional WorkerOptions options = {});

  undefined terminate();

  undefined postMessage(any message, sequence<object> transfer);
  undefined postMessage(any message, optional StructuredSerializeOptions options = {});

  // A plain data attribute — stays a record field (no js.get/js.set yet).
  attribute DOMString label;

  // A static operation — dispatches on the Worker global, no receiver.
  static undefined register(USVString name);
};

Worker includes AbstractWorker;
Worker includes MessageEventTarget;

dictionary WorkerOptions {
  DOMString name = "";
  WorkerType type = "classic";
};

enum WorkerType { "classic", "module" };

dictionary StructuredSerializeOptions {
  sequence<object> transfer = [];
};
