// Complex JSON with nested objects and arrays
function test() {
    return JSON.parse('{"users":[{"name":"max","scores":[1,2,3]},{"name":"bob","scores":[4,5]}]}');
}
test()
