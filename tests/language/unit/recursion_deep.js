// Deep recursion (not tail-recursive)
function test() {
    function sum(n) {
        return n <= 0 ? 0 : n + sum(n - 1);
    }
    return sum(100);
}
test()
