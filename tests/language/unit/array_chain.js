function test() {
    return [1, 2, 3, 4, 5].filter(function(x) { return x > 2; }).map(function(x) { return x * 10; });
}
test()
