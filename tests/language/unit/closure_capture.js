function factory(x) {
    return function(y) {
        return x + y;
    };
}
factory(100)(50)
