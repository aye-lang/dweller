function f(x) {
  switch (x) {
    case 1:
      let a = 10
      return a + 1
    case 2: {
      const b = 20
      return b
    }
    case 3:
      const c = () => x * 3
      return c()
    default:
      let d = [x]
      return d.length
  }
}
console.log(f(1), f(2), f(3), f(4))
function g(x) {
  const fs = []
  for (let i = 0; i < 3; i++) {
    switch (x + i) {
      case 0: case 1: case 2: case 3: case 4:
        let v = i * 2
        fs.push(() => v + x)
        if (i == 1) continue
        break
      default:
        let w = 'w'
        fs.push(() => w)
    }
  }
  return fs.map(h => h()).join(',')
}
console.log(g(0), g(3))
function h(x) {
  switch (x) {
    case 'a':
      let y = 1
    case 'b':
      try { return y } catch (e) { return e instanceof ReferenceError }
  }
}
console.log(h('a'), h('b'))
function k(x) {
  switch (x) {
    case 0:
      class C { m() { return 'c' } }
      return new C().m()
  }
  return 'none'
}
console.log(k(0), k(1))
function a(x) { switch (x) { case 0: let y = 1; case 1: try { y = 2; return y } catch (e) { return e.constructor.name } } }
function b(x) { switch (x) { case 0: return z; case 1: let z = 3; return z } }
function c(x) { let o = 5; switch (x) { case 1: o += 1; let q = o; case 2: return [o, typeof q === 'number' ? q : 'x'] } }
console.log(a(0), a(1), (()=>{try{return b(0)}catch(e){return e.name}})(), b(1))
try { console.log(c(1)) } catch (e) { console.log('c1', e.name) }
try { console.log(c(2)) } catch (e) { console.log('c2', e.name) }
