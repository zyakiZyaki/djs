function test(...arrays) {
    return arrays.reduce(function(acc, arr) {
        return acc.concat(arr);
    }, []);
}
test([1, 2], [3, 4], [5, 6])
