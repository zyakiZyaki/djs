// Rest with no extra args
function test() {
    function gather(a, b, ...rest) {
        return rest;
    }
    return gather(1, 2);
}
test()
