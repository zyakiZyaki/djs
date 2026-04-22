// Rest params with object destructuring
function test() {
    function process(...objs) {
        return objs.reduce(function(acc, {val}) {
            return acc + val;
        }, 0);
    }
    return process({val: 10}, {val: 20}, {val: 30});
}
test()
