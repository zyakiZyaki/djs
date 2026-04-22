function test() {
    function createMerger(...arrays) {
        return function() {
            return arrays.reduce(function(acc, arr) {
                return acc.concat(arr);
            }, []);
        };
    }
    return createMerger([1, 2], [3, 4])();
}
test()
