// Debug: with recursive call
function outer(eff) {
    function inner(count) {
        return {
            method: function () {
                console.log("before eff");
                eff(count);
                console.log("after eff");
                return inner(count + 1);
            }
        };
    }
    return inner;
}

function test() {
    function identity(x) {
        console.log("identity:", x);
        return x;
    }
    return outer(identity)(42).method()
}
test()
