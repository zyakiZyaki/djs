// Debug: check capture slots
function outer(eff) {
    function inner(count) {
        return {
            method: function () {
                console.log("eff:", eff);
                console.log("count:", count);
                return eff;
            }
        };
    }
    return inner;
}

function test() {
    function identity(x) {
        return x;
    }
    return outer(identity)(42).method()
}
test()
