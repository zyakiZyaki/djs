// Counter effect pattern - debug version
function counter(eff) {
    function step(count) {
        return {
            inc: function () {
                eff(count + 1);
                return step(count + 1);
            },
            dec: function () {
                eff(count - 1);
                return step(count - 1);
            }
        };
    }
    return step;
}

// Test: simple version - just call once
function test() {
    function logEffect(value) {
        console.log("effect:", value);
        return value;
    }
    counter(logEffect)(0).inc()
}
test()
