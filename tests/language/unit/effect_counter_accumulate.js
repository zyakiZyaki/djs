// Counter effect pattern - effect with accumulator via nested calls
// The effect callback tracks all state transitions through function composition
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

// Test: chain inc/dec starting from 10, result should be 12
function test() {
    function acc(value) {
        return value;
    }
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
    makeCounter(acc)(10).inc().inc().dec()
}
test()
