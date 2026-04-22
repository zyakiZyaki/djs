// Counter effect pattern - working version using function composition
// Named methods instead of inline anonymous functions
function counter(eff) {
    return function step(count) {
        function doInc() {
            eff(count + 1);
            return step(count + 1);
        }
        function doDec() {
            eff(count - 1);
            return step(count - 1);
        }
        return {inc: doInc, dec: doDec};
    };
}

// Test: counter effect chain
function test() {
    function logEffect(value) {
        return value;
    }
    counter(logEffect)(0).inc().inc()
}
test()
