// Simplest test: nested function with expression statement
function test() {
    function outer(x) {
        return function() {
            x;
            return 99;
        };
    }
    outer(42)()
}
test()
