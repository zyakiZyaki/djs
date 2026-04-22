// Debug: check each variable
function counter(eff) {
    function step(count) {
        return {
            inc: function () {
                console.log("eff:", eff);
                console.log("count:", count);
                console.log("step:", step);
                eff(count);
                return step(count);
            }
        };
    }
    return step;
}

function test() {
    function logEffect(value) {
        console.log("effect called:", value);
        return value;
    }
    return counter(logEffect)(42).inc()
}
test()
