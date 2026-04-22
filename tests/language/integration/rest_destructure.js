// Rest params with object destructuring
function test() {
    function process(...objs) {
        return objs.reduce(function(acc, {val}) {
            return acc + val;
        }, 0);
    }
    return process({val: 1}, {val: 2}, {val: 3});
}
test()
