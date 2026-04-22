function test() {
    return JSON.parse(JSON.stringify({a: 1, b: [2, 3]}));
}
test()
