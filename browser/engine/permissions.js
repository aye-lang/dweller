// spec: https://w3c.github.io/permissions/ https://notifications.spec.whatwg.org/ https://w3c.github.io/geolocation/ https://w3c.github.io/mediacapture-main/
// chromium: third_party/blink/renderer/modules/permissions/permissions.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const KINDS = { notifications: 0, geolocation: 1, camera: 2, microphone: 3 };
const STATES = ['prompt', 'granted', 'denied'];
const GRANTED = 1;
const statuses = [];
const KEY = {};
const dispatchTrusted = EventTarget.prototype.dispatchEvent;

function fire(target, type) {
  const ev = new Event(type);
  ev.isTrusted = true;
  dispatchTrusted.call(target, ev);
}

function handlers(proto, types) {
  for (const type of types) {
    Object.defineProperty(proto, 'on' + type, {
      configurable: true,
      enumerable: true,
      get() { return (this._handlers && this._handlers[type]) || null; },
      set(f) {
        if (!this._handlers) hidden(this, '_handlers', {});
        if (!(type in this._handlers)) this.addEventListener(type, e => typeof this._handlers[type] === 'function' && this._handlers[type].call(this, e));
        this._handlers[type] = typeof f === 'function' ? f : null;
      },
    });
  }
}

function ask(kind) {
  const before = ak.permissionState(kind);

  return new Promise(resolve => ak.permissionAsk(kind, answered => {
    const now = ak.permissionState(kind);
    if (now !== before) statuses.filter(s => s._kind === kind).forEach(s => fire(s, 'change'));
    resolve(answered);
  }));
}

class PermissionStatus extends EventTarget {
  constructor(key) {
    if (key !== KEY) throw new TypeError('Illegal constructor');
    super();
  }
  get name() { return this._name; }
  get state() { return STATES[ak.permissionState(this._kind)]; }
}
handlers(PermissionStatus.prototype, ['change']);

class Permissions {
  constructor() { throw new TypeError('Illegal constructor'); }
  query(descriptor) {
    if (descriptor === null || typeof descriptor !== 'object') return Promise.reject(new TypeError("Failed to execute 'query' on 'Permissions': parameter 1 is not of type 'object'."));

    const name = String(descriptor.name);
    if (!(name in KINDS)) return Promise.reject(new TypeError("Failed to execute 'query' on 'Permissions': Failed to read the 'name' property from 'PermissionDescriptor': The provided value '" + name + "' is not a valid enum value of type PermissionName."));

    const status = new PermissionStatus(KEY);
    hidden(status, '_name', name);
    hidden(status, '_kind', KINDS[name]);
    statuses.push(status);

    return Promise.resolve(status);
  }
}

class Notification extends EventTarget {
  constructor(title, options) {
    super();
    if (arguments.length < 1) throw new TypeError("Failed to construct 'Notification': 1 argument required, but only 0 present.");

    const o = options === undefined || options === null ? {} : options;
    hidden(this, '_title', String(title));
    for (const [k, d] of [['dir', 'auto'], ['lang', ''], ['body', ''], ['tag', ''], ['icon', ''], ['badge', '']]) hidden(this, '_' + k, o[k] === undefined ? d : String(o[k]));
    hidden(this, '_data', o.data === undefined ? null : o.data);
    hidden(this, '_silent', o.silent === undefined ? null : o.silent);
    hidden(this, '_requireInteraction', !!o.requireInteraction);
    ak.setTimer(() => fire(this, ak.permissionState(KINDS.notifications) === GRANTED ? 'show' : 'error'), 0, false, [], false);
  }

  static get permission() { return ['default', 'granted', 'denied'][ak.permissionState(KINDS.notifications)]; }
  static get maxActions() { return 2; }
  static requestPermission(callback) {
    return ask(KINDS.notifications).then(() => {
      const answer = Notification.permission;
      if (typeof callback === 'function') callback(answer);

      return answer;
    });
  }

  close() { ak.setTimer(() => fire(this, 'close'), 0, false, [], false); }
}
for (const k of ['title', 'dir', 'lang', 'body', 'tag', 'icon', 'badge', 'data', 'silent', 'requireInteraction']) {
  Object.defineProperty(Notification.prototype, k, { configurable: true, enumerable: true, get() { return this['_' + k]; } });
}
handlers(Notification.prototype, ['click', 'show', 'error', 'close']);

class GeolocationPositionError {
  constructor() { throw new TypeError('Illegal constructor'); }
  get code() { return this._code; }
  get message() { return this._message; }
}
for (const [k, v] of [['PERMISSION_DENIED', 1], ['POSITION_UNAVAILABLE', 2], ['TIMEOUT', 3]]) {
  Object.defineProperty(GeolocationPositionError, k, { value: v, enumerable: true });
  Object.defineProperty(GeolocationPositionError.prototype, k, { value: v, enumerable: true });
}

function positionError(code, message) {
  const e = Object.create(GeolocationPositionError.prototype);
  hidden(e, '_code', code);
  hidden(e, '_message', message);

  return e;
}

let watches = 0;
function locate(what, success, error) {
  if (typeof success !== 'function') throw new TypeError("Failed to execute '" + what + "' on 'Geolocation': parameter 1 is not of type 'Function'.");

  ask(KINDS.geolocation).then(() => {
    if (typeof error !== 'function') return;
    if (ak.permissionState(KINDS.geolocation) === GRANTED) error(positionError(2, 'Position unavailable'));
    else error(positionError(1, 'User denied Geolocation'));
  });
}

class Geolocation {
  constructor() { throw new TypeError('Illegal constructor'); }
  getCurrentPosition(success, error) { locate('getCurrentPosition', success, error); }
  watchPosition(success, error) {
    locate('watchPosition', success, error);

    return ++watches;
  }
  clearWatch() {}
}

async function userMedia(constraints) {
  const wanted = constraints === undefined || constraints === null ? {} : constraints;
  const kinds = [];
  if (wanted.audio) kinds.push(KINDS.microphone);
  if (wanted.video) kinds.push(KINDS.camera);
  if (kinds.length === 0) throw new TypeError("Failed to execute 'getUserMedia' on 'MediaDevices': At least one of audio and video must be requested");

  for (const kind of kinds) {
    await ask(kind);

    const state = ak.permissionState(kind);
    if (state !== GRANTED) throw new DOMException(state === 0 ? 'Permission dismissed' : 'Permission denied', 'NotAllowedError');
  }

  throw new DOMException('Requested device not found', 'NotFoundError');
}

class MediaDevices extends EventTarget {
  constructor(key) {
    if (key !== KEY) throw new TypeError('Illegal constructor');
    super();
  }
  getUserMedia(constraints) { return userMedia(constraints); }
  enumerateDevices() { return Promise.resolve([]); }
  getSupportedConstraints() { return {}; }
}
handlers(MediaDevices.prototype, ['devicechange']);

const made = {};
function lazily(name, make) {
  Object.defineProperty(Navigator.prototype, name, {
    configurable: true,
    enumerable: true,
    get() {
      if (!made[name]) made[name] = make();

      return made[name];
    },
  });
}
lazily('permissions', () => Object.create(Permissions.prototype));
lazily('geolocation', () => Object.create(Geolocation.prototype));
lazily('mediaDevices', () => new MediaDevices(KEY));

for (const c of [PermissionStatus, Permissions, Notification, GeolocationPositionError, Geolocation, MediaDevices]) hidden(G, c.name, c);
})();
