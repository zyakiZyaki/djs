function test() {
    function outer(x) {
        function inner(...vals) {
            return vals.reduce(function(acc, v) {
                return acc + v * x;
            }, 0);
        }
        return inner(1, 2, 3);
    }
    return outer(10);
}
test()
