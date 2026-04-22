// Negative test: counter effect with non-function eff should fail
function counter(eff) {
    return function step(count) {
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
    };
}

// Test: passing a number as eff should fail when inc() is called
function test() {
    function makeCounter(eff) {
        return function step(count) {
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
        };
    }
    makeCounter(42)(0).inc()
}
test()
