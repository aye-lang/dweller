// A named class expression binds its name in a class scope; methods,
// accessors and field initializers are functions of their own.
let C = class Point {
  x = 1;
  static origin = new Point();
  constructor(y) {
    this.y = y;
  }
  dist() {
    let d = this.x * this.y;
    return () => d + Point.origin.x;
  }
};
