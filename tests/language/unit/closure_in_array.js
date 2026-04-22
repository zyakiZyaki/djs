// Closures stored in array
function test() {
    function makeAdder(x) {
        return function(y) { return x + y; };
    }
    return [makeAdder(10), makeAdder(100), makeAdder(1000)].length;
}
test()
