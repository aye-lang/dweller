(function (G) {
  G.__makeWorkerUrl = function (h) {
    const PARTS = /^([a-zA-Z][a-zA-Z0-9+.-]*:)(?:\/\/([^\/?#]*))?([^?#]*)(\?[^#]*)?(#.*)?$/;
    const SPECIAL = ['http:', 'https:', 'ws:', 'wss:', 'ftp:'];
    function absolute(input, base) {
      if (PARTS.test(input) && /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(input)) return input;
      if (base === undefined) {
        const r = h.resolve(input);
        if (r === null) throw new TypeError("Failed to construct 'URL': Invalid URL");
        return r;
      }
      const b = new URL(String(base));
      if (input.startsWith('//')) return b.protocol + input;
      if (input.startsWith('/')) return b.protocol + '//' + b.host + input;
      if (input.startsWith('?')) return b.protocol + '//' + b.host + b.pathname + input;
      if (input.startsWith('#')) return b.href.split('#')[0] + input;
      return b.protocol + '//' + b.host + b.pathname.replace(/[^\/]*$/, '') + input;
    }
    class URL {
      constructor(input, base) {
        const href = absolute(String(input), base);
        const m = PARTS.exec(href);
        if (!m) throw new TypeError("Failed to construct 'URL': Invalid URL");
        this.protocol = m[1].toLowerCase();
        this.host = (m[2] || '').toLowerCase();
        const at = this.host.lastIndexOf(':');
        this.hostname = at > 0 && !this.host.endsWith(']') ? this.host.slice(0, at) : this.host;
        this.port = at > 0 && !this.host.endsWith(']') ? this.host.slice(at + 1) : '';
        this.pathname = m[3] || (SPECIAL.includes(this.protocol) ? '/' : '');
        this.search = m[4] && m[4].length > 1 ? m[4] : '';
        this.hash = m[5] && m[5].length > 1 ? m[5] : '';
      }
      get origin() { return SPECIAL.includes(this.protocol) && this.protocol !== 'ftp:' ? this.protocol + '//' + this.host : 'null'; }
      get href() { return this.protocol + (this.host || SPECIAL.includes(this.protocol) ? '//' + this.host : '') + this.pathname + this.search + this.hash; }
      toString() { return this.href; }
      toJSON() { return this.href; }
    }
    return URL;
  };
})(globalThis);
