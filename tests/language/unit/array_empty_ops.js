// Array operations on empty arrays
function test() {
    return [[].length, [].push(1), [].concat([]).length, [].join(",")];
}
test()
