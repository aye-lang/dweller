console.log(/^\p{Lu}+$/u.test("ABCÀΩ"), /\p{Lu}/u.test("a"), /^\P{Lu}+$/u.test("abc"), /^[\p{Nd}\p{Script=Greek}_]+$/u.test("12αβ_"), /\p{L}/.test("p{L}"));
console.log(/^\p{scx=Hira}+$/u.test("あー"), /^\p{sc=Hira}+$/u.test("あー"), /^\p{Emoji}$/u.test("😀"), /^\p{RI}{2}$/u.test("🇫🇷"));
for (const bad of ["\\p{Foo}", "\\p{Script=Foo}", "\\p{L", "\\p{lu}", "\\p{General_Category}", "\\p{Latin}", "\\P{sc=}"]) { try { new RegExp(bad, "u"); console.log("no error", bad) } catch (e) { console.log(e.name) } }
console.log(/[a-cx-zb-e]/.test("d"), /[^a-cx-z]/.test("b"), /[\d\w]/i.test("K"), /[k]/i.test("K"), /[A-F]/i.test("e"));
console.log("a1٣٤".replace(/\p{Nd}/gu, "#"), "ǅ Ω x".match(/\p{Lt}|\p{Script_Extensions=Grek}/gu).join("|"), /^\p{Any}$/u.test("\u{10FFFF}"), /^\P{Assigned}$/u.test("͸"));
for (const src of ["/\\p{Foo}/u", "/a/gg", "/a/x", "/(/", "/[b-a]/"]) { try { eval(src); console.log("no error", src) } catch (e) { console.log(e.name) } }
try { eval("if (false) { /\\p{Foo}/u }"); } catch (e) { console.log("early", e.name) }
