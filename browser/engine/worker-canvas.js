(function () {
const ak = globalThis.__canvasHost;
class HTMLCanvasElement {
  get width() {
    return this._width;
  }
  set width(v) {
    this._width = v >>> 0;
    ak.workerResize(this, this._width, this._height);
  }
  get height() {
    return this._height;
  }
  set height(v) {
    this._height = v >>> 0;
    ak.workerResize(this, this._width, this._height);
  }
}
class HTMLImageElement {}
class HTMLVideoElement {}
const document = {
  createElement() {
    const node = ak.workerCanvas(HTMLCanvasElement.prototype);
    node._width = 300;
    node._height = 150;
    ak.workerResize(node, 300, 150);
    return node;
  },
};
Object.defineProperty(globalThis, '__workerCanvas', {value: {HTMLCanvasElement, HTMLImageElement, HTMLVideoElement, document, frame: (node, id) => ak.workerFrame(node, id)}, configurable: true});
})();
