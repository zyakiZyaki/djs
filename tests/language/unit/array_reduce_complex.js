// Complex reduce with object accumulator
function test() {
    return [1, 2, 3, 4, 5].reduce(function(acc, x) {
        return { sum: acc.sum + x, count: acc.count + 1 };
    }, { sum: 0, count: 0 });
}
test()
