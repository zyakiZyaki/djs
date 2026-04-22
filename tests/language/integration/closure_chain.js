// Chain of closures passing data through
function test() {
    function pipe(val) {
        return function step1() {
            return function step2() {
                return val * 2 + 10;
            }();
        }();
    }
    return pipe(5);
}
test()
