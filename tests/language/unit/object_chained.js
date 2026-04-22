// Chained object access
function test() {
    return {a: {b: {c: 42}}}.a.b.c;
}
test()
